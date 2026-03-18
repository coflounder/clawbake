use crate::types::ToolSpec;
use crate::tui::wizard::widgets::MultiSelect;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::*;

const PREDEFINED_TOOLS: [(&str, &str); 10] = [
    ("Bash", "Execute shell commands"),
    ("Read", "Read file contents"),
    ("Write", "Create or overwrite files"),
    ("Edit", "Make targeted edits to files"),
    ("Glob", "Find files by pattern"),
    ("Grep", "Search file contents"),
    ("WebFetch", "Fetch web page contents"),
    ("WebSearch", "Search the web"),
    ("NotebookEdit", "Edit Jupyter notebooks"),
    ("Agent", "Launch sub-agents for complex tasks"),
];

#[derive(Debug)]
pub struct ToolsStepState {
    pub picker: MultiSelect,
}

impl ToolsStepState {
    pub fn new() -> Self {
        Self {
            picker: MultiSelect::new(
                "Allowed Tools",
                PREDEFINED_TOOLS.iter().map(|(n, _)| n.to_string()).collect(),
            ),
        }
    }

    pub fn tool_specs(&self) -> Vec<ToolSpec> {
        self.picker
            .selected()
            .iter()
            .map(|name| {
                let desc = PREDEFINED_TOOLS
                    .iter()
                    .find(|(n, _)| *n == name.as_str())
                    .map(|(_, d)| d.to_string())
                    .unwrap_or_default();
                ToolSpec {
                    name: name.clone(),
                    description: desc,
                    stub_behavior: None,
                }
            })
            .collect()
    }
}

/// Returns: 1 = next step, -1 = previous step, 0 = stay
pub fn handle_key(state: &mut ToolsStepState, key: KeyEvent) -> i8 {
    match key.code {
        KeyCode::BackTab => -1, // Single widget, go to previous step
        _ => {
            state.picker.handle_key(key);
            0
        }
    }
}

pub fn render(state: &ToolsStepState, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(2)])
        .split(area);

    state.picker.render(frame, chunks[0], true);

    // Show description of highlighted tool
    if state.picker.cursor < PREDEFINED_TOOLS.len() {
        let (name, desc) = PREDEFINED_TOOLS[state.picker.cursor];
        let p = Paragraph::new(format!("  {}: {}", name, desc))
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(p, chunks[1]);
    }
}
