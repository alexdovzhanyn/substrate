use crate::error::AppResult;
use ratatui::{DefaultTerminal, Frame};

pub fn app(terminal: &mut DefaultTerminal) -> AppResult<()> {
  loop {
    terminal.draw(render)?;

    if crossterm::event::read()?.is_key_press() {
      break Ok(());
    }
  }
}

fn render(frame: &mut Frame) {
  frame.render_widget("Hello world", frame.area());
}
