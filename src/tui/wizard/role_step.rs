use crate::tui::wizard::widgets::{TextArea, TextInput};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;

#[derive(Debug)]
pub struct RoleStepState {
    pub role_input: TextInput,
    pub responsibility_input: TextArea,
    pub focus: RoleFocus,
    pub role: String,
    pub responsibility: String,
}

#[derive(Debug, PartialEq)]
pub enum RoleFocus {
    Role,
    Responsibility,
}

impl RoleStepState {
    pub fn new() -> Self {
        Self {
            role_input: TextInput::new("Role"),
            responsibility_input: TextArea::new("Responsibility"),
            focus: RoleFocus::Role,
            role: String::new(),
            responsibility: String::new(),
        }
    }
}

/// Returns: 1 = next step, -1 = previous step, 0 = stay
pub fn handle_key(state: &mut RoleStepState, key: KeyEvent) -> i8 {
    match key.code {
        KeyCode::BackTab => {
            match state.focus {
                RoleFocus::Responsibility => {
                    state.responsibility = state.responsibility_input.content();
                    state.focus = RoleFocus::Role;
                    0
                }
                RoleFocus::Role => -1, // First field, first step — no previous
            }
        }
        KeyCode::Tab => {
            state.role = state.role_input.value.clone();
            state.responsibility = state.responsibility_input.content();
            state.focus = match state.focus {
                RoleFocus::Role => RoleFocus::Responsibility,
                RoleFocus::Responsibility => RoleFocus::Role,
            };
            0
        }
        KeyCode::Enter if state.focus == RoleFocus::Role => {
            state.role = state.role_input.value.clone();
            state.focus = RoleFocus::Responsibility;
            0
        }
        _ => {
            match state.focus {
                RoleFocus::Role => {
                    state.role_input.handle_key(key);
                }
                RoleFocus::Responsibility => {
                    state.responsibility_input.handle_key(key);
                }
            }
            0
        }
    }
}

pub fn render(state: &RoleStepState, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(area);

    state
        .role_input
        .render(frame, chunks[0], state.focus == RoleFocus::Role);
    state
        .responsibility_input
        .render(frame, chunks[1], state.focus == RoleFocus::Responsibility);
}
