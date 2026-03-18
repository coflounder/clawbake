use crate::tui::wizard::widgets::TextInput;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::*;

#[derive(Debug)]
pub struct NameStepState {
    pub input: TextInput,
    pub name: String,
}

impl NameStepState {
    pub fn new() -> Self {
        Self {
            input: TextInput::new("Agent Name"),
            name: String::new(),
        }
    }
}

/// Returns: -1 = previous step, 0 = stay
pub fn handle_key(state: &mut NameStepState, key: KeyEvent) -> i8 {
    match key.code {
        KeyCode::BackTab => -1,
        _ => {
            state.input.handle_key(key);
            0
        }
    }
}

pub fn render(state: &NameStepState, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3)])
        .split(area);

    state.input.render(frame, chunks[0], true);

    let hint = Paragraph::new("Enter a name for your AI agent persona")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(hint, chunks[1]);
}
