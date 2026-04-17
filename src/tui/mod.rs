use crossterm::event::KeyCode;
use std::time::Duration;
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
  style::{Style, Stylize},
  symbols::border,
  text::Line,
  widgets::{Block, Borders, List, ListItem, Paragraph},
};

struct ConsoleState {
  beliefs: Option<Vec<Belief>>,
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
        .draw(|frame| Self::render(frame, &self.state))?;

      if self.handle_key_press().await? {
        break Ok(());
      }
    }
  }

  fn render(frame: &mut Frame, state: &ConsoleState) {
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

    let beliefs_list = List::new(belief_blurbs).block(
      Block::bordered()
        .title(Line::from(" Beliefs ".blue().bold()).centered())
        .border_set(border::THICK)
        .border_style(Style::new().blue()),
    );

    frame.render_widget(beliefs_list, chunks[0]);

    let right_text = if state.error.is_empty() {
      "Select a belief to view details"
    } else {
      &state.error
    };

    let detail = Paragraph::new(right_text).block(
      Block::default()
        .title("Belief Details")
        .borders(Borders::ALL),
    );

    frame.render_widget(detail, chunks[1]);
  }

  async fn handle_key_press(&self) -> AppResult<bool> {
    if crossterm::event::poll(Duration::from_millis(100))?
      && let Some(key) = crossterm::event::read()?.as_key_press_event()
    {
      match key.code {
        KeyCode::Char('q') => return Ok(true),
        _ => {
          self
            .cmd_tx
            .send(IPCRequest::ListBeliefs {
              search: None,
              limit: None,
              after: None,
            })
            .await?;
        }
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
