use chrono::{DateTime, Utc};
use crossterm::event::KeyCode;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::{
  core::beliefs::belief::Belief,
  error::AppResult,
  ipc::{
    client::IPCClient,
    protocol::{IPCRequest, IPCResponse},
  },
};
use ratatui::{
  DefaultTerminal, Frame,
  layout::{Constraint, Direction, Layout},
  style::{Modifier, Style, Stylize},
  symbols::border,
  text::Line,
  widgets::{Block, Borders, List, ListItem, ListState, Padding, Paragraph, Wrap},
};

struct ConsoleState {
  beliefs: Option<Vec<Belief>>,
  belief_list_ui: ListState,
  error: String,
}

pub struct ConsoleTUI {
  terminal: DefaultTerminal,
  cmd_tx: Sender<IPCRequest>,
  msg_rx: Receiver<IPCResponse>,
  state: ConsoleState,
}

impl ConsoleTUI {
  pub fn initialize(terminal: DefaultTerminal) -> Self {
    let (cmd_tx, cmd_rx) = mpsc::channel::<IPCRequest>(32);
    let (msg_tx, msg_rx) = mpsc::channel::<IPCResponse>(32);

    tokio::spawn(Self::ipc_worker(cmd_rx, msg_tx));

    Self {
      terminal,
      state: ConsoleState {
        beliefs: None,
        belief_list_ui: ListState::default().with_selected(Some(0)),
        error: String::new(),
      },
      cmd_tx,
      msg_rx,
    }
  }

  async fn ipc_worker(
    mut cmd_rx: Receiver<IPCRequest>,
    msg_tx: Sender<IPCResponse>,
  ) -> AppResult<()> {
    let mut ipc_client = IPCClient::connect().await?;

    while let Some(ipc_request) = cmd_rx.recv().await {
      let response = match ipc_client.request(&ipc_request).await {
        Ok(response) => response,
        Err(err) => IPCResponse::Error {
          message: err.to_string(),
        },
      };

      let _ = msg_tx.send(response).await;
    }

    Ok(())
  }

  pub async fn run(&mut self) -> AppResult<()> {
    self
      .cmd_tx
      .send(IPCRequest::ListBeliefs {
        search: None,
        limit: None,
        after: None,
      })
      .await?;

    loop {
      while let Ok(ipc_response) = self.msg_rx.try_recv() {
        match ipc_response {
          IPCResponse::ListBeliefs { beliefs } => {
            self.state.beliefs = Some(beliefs);
          }
          IPCResponse::Error { message } => {
            self.state.error = message;
          }
          _ => (),
        }
      }

      self
        .terminal
        .draw(|frame| Self::render(frame, &mut self.state))?;

      if self.handle_key_press().await? {
        break Ok(());
      }
    }
  }

  fn render(frame: &mut Frame, state: &mut ConsoleState) {
    let chunks = Layout::default()
      .direction(Direction::Horizontal)
      .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
      .split(frame.area());

    let belief_blurbs = match &state.beliefs {
      Some(beliefs) => {
        if beliefs.is_empty() {
          vec![ListItem::new("No beliefs recorded yet.")]
        } else {
          beliefs
            .iter()
            .map(|belief| {
              let blurb = truncate(&belief.content, 40);

              ListItem::new(Line::from(blurb))
            })
            .collect()
        }
      }
      None => vec![ListItem::new("Press any key to load beliefs")],
    };

    let beliefs_list = List::new(belief_blurbs)
      .highlight_style(Modifier::REVERSED)
      .block(
        Block::bordered()
          .title(Line::from(" Beliefs ".blue().bold()).centered())
          .border_set(border::THICK)
          .border_style(Style::new().blue()),
      );

    frame.render_stateful_widget(beliefs_list, chunks[0], &mut state.belief_list_ui);

    let mut detail_title = Line::from(" Belief Details ".blue().bold());
    let mut detail_text =
      vec!["More information about the selected belief will be visible here".into()];
    let mut detail_status_left = String::from("");
    let mut detail_status_right = String::from("");

    if state.error.is_empty()
      && let Some(beliefs) = &state.beliefs
    {
      let belief_idx = state.belief_list_ui.selected().unwrap_or(0);

      let belief = beliefs.get(belief_idx).unwrap();

      detail_title = Line::from_iter([
        " Belief ".blue().bold(),
        belief.id.clone().green().bold(),
        " ".into(),
      ]);

      detail_text = vec![
        belief.content.clone().cyan().into(),
        Line::from(""),
        " Possible Queries ".on_magenta().bold().into(),
        Line::from_iter(belief.possible_queries.iter().map(|q| format!("'{}'", q))),
      ];

      let created_at: DateTime<Utc> =
        (SystemTime::UNIX_EPOCH + Duration::from_secs(belief.created_at)).into();

      let updated_at: DateTime<Utc> =
        (SystemTime::UNIX_EPOCH + Duration::from_secs(belief.updated_at)).into();

      detail_status_left = format!(" {}", belief.tags.join(", "));
      detail_status_right = format!(
        "Created: {}   Last Updated: {} ",
        created_at.format("%m-%d-%Y %H:%M:%S"),
        updated_at.format("%m-%d-%Y %H:%M:%S")
      );
    } else {
      detail_text = vec![state.error.clone().into()];
    };

    let detail_block = Block::bordered()
      .title(detail_title.centered())
      .border_set(border::THICK)
      .border_style(Style::new().blue());

    let detail_inner = detail_block.inner(chunks[1]);
    frame.render_widget(detail_block, chunks[1]);

    let detail_chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Min(1), Constraint::Length(2)])
      .split(detail_inner);

    let detail_body_block = Block::default().padding(Padding::proportional(1));
    let detail_body_inner = detail_body_block.inner(detail_chunks[0]);
    frame.render_widget(detail_body_block, detail_chunks[0]);

    let detail_body = Paragraph::new(detail_text).wrap(Wrap { trim: false });

    let width = detail_chunks[1].width as usize;
    let gap = width.saturating_sub(detail_status_left.len() + detail_status_right.len());

    let detail_bottom_bar = Paragraph::new(Line::from(vec![
      detail_status_left.magenta(),
      " ".repeat(gap).into(),
      detail_status_right.dark_gray(),
    ]))
    .block(
      Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().blue()),
    );

    frame.render_widget(detail_body, detail_body_inner);
    frame.render_widget(detail_bottom_bar, detail_chunks[1]);
  }

  async fn handle_key_press(&mut self) -> AppResult<bool> {
    if crossterm::event::poll(Duration::from_millis(100))?
      && let Some(key) = crossterm::event::read()?.as_key_press_event()
    {
      match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(true),
        KeyCode::Char('j') | KeyCode::Down => self.state.belief_list_ui.select_next(),
        KeyCode::Char('k') | KeyCode::Up => self.state.belief_list_ui.select_previous(),
        _ => {}
      }
    }

    Ok(false)
  }
}

fn truncate(s: &str, max: usize) -> String {
  if s.chars().count() <= max {
    return s.to_string();
  }

  let mut out = s.chars().take(max).collect::<String>();
  out.push('…');
  out
}
