use crate::tui::wizard::widgets::{MultiSelect, TextInput};
use crate::types::PersonalityTrait;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;

const FIXED_TRAITS: [&str; 20] = [
    "Analytical",
    "Creative",
    "Empathetic",
    "Direct",
    "Methodical",
    "Collaborative",
    "Assertive",
    "Patient",
    "Curious",
    "Concise",
    "Thorough",
    "Diplomatic",
    "Humorous",
    "Formal",
    "Friendly",
    "Skeptical",
    "Encouraging",
    "Precise",
    "Adaptable",
    "Proactive",
];

const MAX_TRAITS: usize = 6;

#[derive(Debug, PartialEq)]
pub enum PersonalityFocus {
    Picker,
    Custom,
}

#[derive(Debug)]
pub struct PersonalityStepState {
    pub picker: MultiSelect,
    pub custom_input: TextInput,
    pub custom_traits: Vec<String>,
    pub focus: PersonalityFocus,
}

impl PersonalityStepState {
    pub fn new() -> Self {
        let picker = MultiSelect::new(
            "Personality Traits",
            FIXED_TRAITS.iter().map(|s| s.to_string()).collect(),
        )
        .with_max(MAX_TRAITS);

        Self {
            picker,
            custom_input: TextInput::new("Add custom trait (Enter to add)"),
            custom_traits: Vec::new(),
            focus: PersonalityFocus::Picker,
        }
    }

    fn total_selected(&self) -> usize {
        self.picker.selected_count() + self.custom_traits.len()
    }

    pub fn selected_traits(&self) -> Vec<PersonalityTrait> {
        let mut traits: Vec<PersonalityTrait> = self
            .picker
            .selected()
            .iter()
            .filter_map(|name| match name.as_str() {
                "Analytical" => Some(PersonalityTrait::Analytical),
                "Creative" => Some(PersonalityTrait::Creative),
                "Empathetic" => Some(PersonalityTrait::Empathetic),
                "Direct" => Some(PersonalityTrait::Direct),
                "Methodical" => Some(PersonalityTrait::Methodical),
                "Collaborative" => Some(PersonalityTrait::Collaborative),
                "Assertive" => Some(PersonalityTrait::Assertive),
                "Patient" => Some(PersonalityTrait::Patient),
                "Curious" => Some(PersonalityTrait::Curious),
                "Concise" => Some(PersonalityTrait::Concise),
                "Thorough" => Some(PersonalityTrait::Thorough),
                "Diplomatic" => Some(PersonalityTrait::Diplomatic),
                "Humorous" => Some(PersonalityTrait::Humorous),
                "Formal" => Some(PersonalityTrait::Formal),
                "Friendly" => Some(PersonalityTrait::Friendly),
                "Skeptical" => Some(PersonalityTrait::Skeptical),
                "Encouraging" => Some(PersonalityTrait::Encouraging),
                "Precise" => Some(PersonalityTrait::Precise),
                "Adaptable" => Some(PersonalityTrait::Adaptable),
                "Proactive" => Some(PersonalityTrait::Proactive),
                _ => None,
            })
            .collect();

        for custom in &self.custom_traits {
            traits.push(PersonalityTrait::Custom(custom.clone()));
        }

        traits
    }
}

/// Returns: 1 = next step, -1 = previous step, 0 = stay
pub fn handle_key(state: &mut PersonalityStepState, key: KeyEvent) -> i8 {
    match key.code {
        KeyCode::BackTab => {
            match state.focus {
                PersonalityFocus::Custom => {
                    state.focus = PersonalityFocus::Picker;
                    0
                }
                PersonalityFocus::Picker => -1, // Go to previous wizard step
            }
        }
        KeyCode::Tab => {
            state.focus = match state.focus {
                PersonalityFocus::Picker => PersonalityFocus::Custom,
                PersonalityFocus::Custom => PersonalityFocus::Picker,
            };
            0
        }
        KeyCode::Enter if state.focus == PersonalityFocus::Custom => {
            let val = state.custom_input.value.trim().to_string();
            if !val.is_empty() && state.total_selected() < MAX_TRAITS {
                state.custom_traits.push(val);
                state.custom_input.value.clear();
                state.custom_input.cursor = 0;
            }
            0
        }
        _ => {
            match state.focus {
                PersonalityFocus::Picker => {
                    // Sync max_selected to account for custom traits
                    state.picker.max_selected =
                        Some(MAX_TRAITS.saturating_sub(state.custom_traits.len()));
                    state.picker.handle_key(key);
                }
                PersonalityFocus::Custom => {
                    state.custom_input.handle_key(key);
                }
            }
            0
        }
    }
}

pub fn render(state: &PersonalityStepState, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(area);

    state
        .picker
        .render(frame, chunks[0], state.focus == PersonalityFocus::Picker);
    state
        .custom_input
        .render(frame, chunks[1], state.focus == PersonalityFocus::Custom);

    // Show custom traits
    if !state.custom_traits.is_empty() {
        let custom_text = format!("Custom: {}", state.custom_traits.join(", "));
        let p = ratatui::widgets::Paragraph::new(custom_text)
            .style(Style::default().fg(Color::Green))
            .block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title(" Added "),
            );
        frame.render_widget(p, chunks[2]);
    }
}
