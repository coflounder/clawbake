use crate::tui::wizard::widgets::{TextArea, TextInput};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;

#[derive(Debug, PartialEq)]
pub enum ConfigFocus {
    EvalCount,
    MaxIterations,
    BudgetTokens,
    MaxParallel,
    Guardrails,
    Reference,
}

#[derive(Debug)]
pub struct ConfigStepState {
    pub eval_count_input: TextInput,
    pub iterations_input: TextInput,
    pub budget_input: TextInput,
    pub parallel_input: TextInput,
    pub guardrails_input: TextArea,
    pub reference_input: TextArea,
    pub focus: ConfigFocus,
    pub eval_count: usize,
    pub max_iterations: usize,
    pub max_budget_tokens: u64,
    pub max_parallel: usize,
    pub guardrails: String,
    pub reference: String,
}

impl ConfigStepState {
    pub fn new() -> Self {
        let mut eval_count_input = TextInput::new("Eval cases per iteration");
        eval_count_input.value = "5".to_string();
        eval_count_input.cursor = 1;

        let mut iterations_input = TextInput::new("Max iterations");
        iterations_input.value = "10".to_string();
        iterations_input.cursor = 2;

        let mut budget_input = TextInput::new("Token budget (K)");
        budget_input.value = "1000".to_string();
        budget_input.cursor = 4;

        let mut parallel_input = TextInput::new("Max parallel");
        parallel_input.value = "2".to_string();
        parallel_input.cursor = 1;

        Self {
            eval_count_input,
            iterations_input,
            budget_input,
            parallel_input,
            guardrails_input: TextArea::new("Guardrails (one per line)"),
            reference_input: TextArea::new("Reference material"),
            focus: ConfigFocus::EvalCount,
            eval_count: 5,
            max_iterations: 10,
            max_budget_tokens: 1_000_000,
            max_parallel: 2,
            guardrails: String::new(),
            reference: String::new(),
        }
    }

    pub fn guardrails_list(&self) -> Vec<String> {
        self.guardrails
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn save_values(&mut self) {
        self.eval_count = self.eval_count_input.value.parse().unwrap_or(5);
        self.max_iterations = self.iterations_input.value.parse().unwrap_or(10);
        self.max_budget_tokens = self.budget_input.value.parse::<u64>().unwrap_or(1000) * 1000;
        self.max_parallel = self.parallel_input.value.parse().unwrap_or(2);
        self.guardrails = self.guardrails_input.content();
        self.reference = self.reference_input.content();
    }
}

/// Returns: 1 = next step, -1 = previous step, 0 = stay
pub fn handle_key(state: &mut ConfigStepState, key: KeyEvent) -> i8 {
    match key.code {
        KeyCode::BackTab => {
            state.save_values();
            match state.focus {
                ConfigFocus::EvalCount => -1, // First field, go to previous step
                ConfigFocus::MaxIterations => {
                    state.focus = ConfigFocus::EvalCount;
                    0
                }
                ConfigFocus::BudgetTokens => {
                    state.focus = ConfigFocus::MaxIterations;
                    0
                }
                ConfigFocus::MaxParallel => {
                    state.focus = ConfigFocus::BudgetTokens;
                    0
                }
                ConfigFocus::Guardrails => {
                    state.focus = ConfigFocus::MaxParallel;
                    0
                }
                ConfigFocus::Reference => {
                    state.focus = ConfigFocus::Guardrails;
                    0
                }
            }
        }
        KeyCode::Tab => {
            state.save_values();
            state.focus = match state.focus {
                ConfigFocus::EvalCount => ConfigFocus::MaxIterations,
                ConfigFocus::MaxIterations => ConfigFocus::BudgetTokens,
                ConfigFocus::BudgetTokens => ConfigFocus::MaxParallel,
                ConfigFocus::MaxParallel => ConfigFocus::Guardrails,
                ConfigFocus::Guardrails => ConfigFocus::Reference,
                ConfigFocus::Reference => ConfigFocus::EvalCount,
            };
            0
        }
        _ => {
            match state.focus {
                ConfigFocus::EvalCount => {
                    state.eval_count_input.handle_key(key);
                }
                ConfigFocus::MaxIterations => {
                    state.iterations_input.handle_key(key);
                }
                ConfigFocus::BudgetTokens => {
                    state.budget_input.handle_key(key);
                }
                ConfigFocus::MaxParallel => {
                    state.parallel_input.handle_key(key);
                }
                ConfigFocus::Guardrails => {
                    state.guardrails_input.handle_key(key);
                }
                ConfigFocus::Reference => {
                    state.reference_input.handle_key(key);
                }
            }
            0
        }
    }
}

pub fn render(state: &ConfigStepState, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Min(4),
        ])
        .split(area);

    state
        .eval_count_input
        .render(frame, chunks[0], state.focus == ConfigFocus::EvalCount);
    state
        .iterations_input
        .render(frame, chunks[1], state.focus == ConfigFocus::MaxIterations);
    state
        .budget_input
        .render(frame, chunks[2], state.focus == ConfigFocus::BudgetTokens);
    state
        .parallel_input
        .render(frame, chunks[3], state.focus == ConfigFocus::MaxParallel);
    state
        .guardrails_input
        .render(frame, chunks[4], state.focus == ConfigFocus::Guardrails);
    state
        .reference_input
        .render(frame, chunks[5], state.focus == ConfigFocus::Reference);
}
