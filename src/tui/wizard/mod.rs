pub mod autofill;
pub mod config_step;
pub mod name_step;
pub mod personality_step;
pub mod role_step;
pub mod tools_step;
pub mod widgets;

use crate::config::AppConfig;
use crate::types::{EvalConfig, PersonaSpec};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum WizardStep {
    Role,
    Personality,
    Tools,
    Name,
    Config,
    Review,
}

impl WizardStep {
    pub fn index(&self) -> usize {
        match self {
            Self::Role => 0,
            Self::Personality => 1,
            Self::Tools => 2,
            Self::Name => 3,
            Self::Config => 4,
            Self::Review => 5,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Self::Role => "Role & Responsibility",
            Self::Personality => "Personality Traits",
            Self::Tools => "Tools & Skills",
            Self::Name => "Name",
            Self::Config => "Configuration",
            Self::Review => "Review & Start",
        }
    }

    pub fn count() -> usize {
        6
    }

    fn next(&self) -> Option<WizardStep> {
        match self {
            Self::Role => Some(Self::Personality),
            Self::Personality => Some(Self::Tools),
            Self::Tools => Some(Self::Name),
            Self::Name => Some(Self::Config),
            Self::Config => Some(Self::Review),
            Self::Review => None,
        }
    }

    fn previous(&self) -> Option<WizardStep> {
        match self {
            Self::Role => None,
            Self::Personality => Some(Self::Role),
            Self::Tools => Some(Self::Personality),
            Self::Name => Some(Self::Tools),
            Self::Config => Some(Self::Name),
            Self::Review => Some(Self::Config),
        }
    }
}

pub enum WizardAction {
    None,
    Complete(AppConfig),
    Quit,
    RequestAutofill(autofill::AutofillContext),
}

#[derive(Debug)]
pub struct WizardState {
    pub step: WizardStep,
    pub role: role_step::RoleStepState,
    pub personality: personality_step::PersonalityStepState,
    pub tools: tools_step::ToolsStepState,
    pub name: name_step::NameStepState,
    pub config: config_step::ConfigStepState,
    pub reference_content: String,
    pub autofill_pending: bool,
}

// --- Key detection helpers ---

fn is_next_step(key: &KeyEvent) -> bool {
    // CMD+Enter (no Shift, no Alt)
    (key.code == KeyCode::Enter
        && key.modifiers.contains(KeyModifiers::SUPER)
        && !key.modifiers.contains(KeyModifiers::SHIFT)
        && !key.modifiers.contains(KeyModifiers::ALT))
    // F2 fallback
    || key.code == KeyCode::F(2)
    // Ctrl+N fallback
    || (key.code == KeyCode::Char('n') && key.modifiers == KeyModifiers::CONTROL)
}

fn is_prev_step(key: &KeyEvent) -> bool {
    // Shift+CMD+Enter
    (key.code == KeyCode::Enter
        && key.modifiers.contains(KeyModifiers::SUPER)
        && key.modifiers.contains(KeyModifiers::SHIFT))
    // Ctrl+P fallback
    || (key.code == KeyCode::Char('p') && key.modifiers == KeyModifiers::CONTROL)
}

fn is_autofill(key: &KeyEvent) -> bool {
    // Ctrl+G
    key.code == KeyCode::Char('g') && key.modifiers == KeyModifiers::CONTROL
}

fn is_yolo(key: &KeyEvent) -> bool {
    // CMD+Option+Enter
    (key.code == KeyCode::Enter
        && key.modifiers.contains(KeyModifiers::SUPER)
        && key.modifiers.contains(KeyModifiers::ALT))
    // Ctrl+Y fallback
    || (key.code == KeyCode::Char('y') && key.modifiers == KeyModifiers::CONTROL)
}

impl WizardState {
    pub fn new() -> Self {
        Self {
            step: WizardStep::Role,
            role: role_step::RoleStepState::new(),
            personality: personality_step::PersonalityStepState::new(),
            tools: tools_step::ToolsStepState::new(),
            name: name_step::NameStepState::new(),
            config: config_step::ConfigStepState::new(),
            reference_content: String::new(),
            autofill_pending: false,
        }
    }

    /// Save current step's inputs and validate for advancement.
    fn save_and_validate(&mut self) -> bool {
        match self.step {
            WizardStep::Role => {
                self.role.role = self.role.role_input.value.clone();
                self.role.responsibility = self.role.responsibility_input.content();
                !self.role.role.is_empty()
            }
            WizardStep::Personality => true,
            WizardStep::Tools => true,
            WizardStep::Name => {
                self.name.name = self.name.input.value.trim().to_string();
                !self.name.name.is_empty()
            }
            WizardStep::Config => {
                self.config.save_values();
                self.reference_content = self.config.reference.clone();
                true
            }
            WizardStep::Review => true,
        }
    }

    fn build_autofill_context(&self) -> autofill::AutofillContext {
        autofill::AutofillContext {
            step: self.step.clone(),
            role: self.role.role_input.value.clone(),
            responsibility: self.role.responsibility_input.content(),
            selected_traits: self.personality.picker.selected(),
            selected_tools: self.tools.picker.selected(),
            name: self.name.input.value.clone(),
        }
    }

    pub fn apply_autofill(&mut self, result: autofill::AutofillResult) {
        self.autofill_pending = false;
        match result {
            autofill::AutofillResult::Role {
                role,
                responsibility,
            } => {
                if let Some(r) = role {
                    if self.role.role_input.value.is_empty() {
                        self.role.role_input.set_value(&r);
                        self.role.role = r;
                    }
                }
                if let Some(resp) = responsibility {
                    if self.role.responsibility_input.content().trim().is_empty() {
                        self.role.responsibility_input.set_content(&resp);
                        self.role.responsibility = resp;
                    }
                }
            }
            autofill::AutofillResult::Personality { traits } => {
                self.personality.picker.select_by_names(&traits);
            }
            autofill::AutofillResult::Tools { tools } => {
                self.tools.picker.select_by_names(&tools);
            }
            autofill::AutofillResult::Name { name } => {
                self.name.input.set_value(&name);
                self.name.name = name;
            }
            autofill::AutofillResult::Config { guardrails } => {
                let text = guardrails.join("\n");
                self.config.guardrails_input.set_content(&text);
                self.config.guardrails = text;
            }
        }
    }

    fn apply_yolo(&mut self) {
        // Fill empty fields with sensible defaults
        if self.role.role_input.value.is_empty() {
            self.role.role_input.set_value("AI Assistant");
            self.role.role = "AI Assistant".to_string();
        } else {
            self.role.role = self.role.role_input.value.clone();
        }
        if self.role.responsibility_input.content().trim().is_empty() {
            self.role
                .responsibility_input
                .set_content("General-purpose task completion and analysis");
            self.role.responsibility =
                "General-purpose task completion and analysis".to_string();
        } else {
            self.role.responsibility = self.role.responsibility_input.content();
        }

        if self.personality.picker.selected_count() == 0
            && self.personality.custom_traits.is_empty()
        {
            self.personality.picker.select_by_names(&[
                "Analytical".to_string(),
                "Direct".to_string(),
                "Concise".to_string(),
            ]);
        }

        if self.tools.picker.selected_count() == 0 {
            self.tools.picker.select_by_names(&[
                "Bash".to_string(),
                "Read".to_string(),
                "Write".to_string(),
                "Edit".to_string(),
            ]);
        }

        if self.name.input.value.is_empty() {
            self.name.input.set_value("Agent");
            self.name.name = "Agent".to_string();
        } else {
            self.name.name = self.name.input.value.trim().to_string();
        }

        self.config.save_values();
        self.reference_content = self.config.reference.clone();

        self.step = WizardStep::Review;
    }

    fn apply_step_result(&mut self, result: i8) {
        if result == -1 {
            if let Some(prev) = self.step.previous() {
                self.step = prev;
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> WizardAction {
        // Esc → quit
        if key.code == KeyCode::Esc {
            return WizardAction::Quit;
        }

        // YOLO: fill defaults and skip to review
        if is_yolo(&key) {
            self.apply_yolo();
            return WizardAction::None;
        }

        // Autofill: request AI suggestion for current step
        if is_autofill(&key) && !self.autofill_pending && self.step != WizardStep::Review {
            self.autofill_pending = true;
            return WizardAction::RequestAutofill(self.build_autofill_context());
        }

        // Next step (CMD+Enter / F2 / Ctrl+N)
        if is_next_step(&key) {
            if self.step == WizardStep::Review {
                return WizardAction::Complete(self.build_config());
            }
            if self.save_and_validate() {
                if let Some(next) = self.step.next() {
                    self.step = next;
                }
            }
            return WizardAction::None;
        }

        // Prev step (Shift+CMD+Enter / Ctrl+P)
        if is_prev_step(&key) {
            if let Some(prev) = self.step.previous() {
                self.step = prev;
            }
            return WizardAction::None;
        }

        // Dispatch to step handler for field-level input
        match self.step {
            WizardStep::Role => {
                let r = role_step::handle_key(&mut self.role, key);
                self.apply_step_result(r);
            }
            WizardStep::Personality => {
                let r = personality_step::handle_key(&mut self.personality, key);
                self.apply_step_result(r);
            }
            WizardStep::Tools => {
                let r = tools_step::handle_key(&mut self.tools, key);
                self.apply_step_result(r);
            }
            WizardStep::Name => {
                let r = name_step::handle_key(&mut self.name, key);
                self.apply_step_result(r);
            }
            WizardStep::Config => {
                let r = config_step::handle_key(&mut self.config, key);
                self.apply_step_result(r);
            }
            WizardStep::Review => match key.code {
                KeyCode::Enter => {
                    return WizardAction::Complete(self.build_config());
                }
                KeyCode::Backspace | KeyCode::BackTab => {
                    self.step = WizardStep::Config;
                }
                _ => {}
            },
        }

        WizardAction::None
    }

    fn build_config(&self) -> AppConfig {
        let persona = PersonaSpec {
            name: self.name.name.clone(),
            role: self.role.role.clone(),
            responsibility: self.role.responsibility.clone(),
            personality_traits: self.personality.selected_traits(),
            tools: self.tools.tool_specs(),
            guardrails: self.config.guardrails_list(),
        };

        let eval = EvalConfig {
            eval_count: self.config.eval_count,
            max_iterations: self.config.max_iterations,
            max_budget_tokens: self.config.max_budget_tokens,
            max_parallel: self.config.max_parallel,
            max_turns_per_case: 2,
            interactive: true,
            regen_interval: 2,
        };

        AppConfig {
            persona,
            eval,
            ..AppConfig::default()
        }
    }
}

pub fn render_wizard(state: &WizardState, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Title header (large)
            Constraint::Min(1),   // Content
            Constraint::Length(3), // Footer
        ])
        .split(area);

    render_title_bar(state, frame, chunks[0]);

    match state.step {
        WizardStep::Role => role_step::render(&state.role, frame, chunks[1]),
        WizardStep::Personality => personality_step::render(&state.personality, frame, chunks[1]),
        WizardStep::Tools => tools_step::render(&state.tools, frame, chunks[1]),
        WizardStep::Name => name_step::render(&state.name, frame, chunks[1]),
        WizardStep::Config => config_step::render(&state.config, frame, chunks[1]),
        WizardStep::Review => render_review(state, frame, chunks[1]),
    }

    render_footer(state, frame, chunks[2]);
}

fn render_title_bar(state: &WizardState, frame: &mut Frame, area: Rect) {
    let version = env!("CARGO_PKG_VERSION");
    let step_num = state.step.index() + 1;
    let total = WizardStep::count();

    let block = ratatui::widgets::Block::default()
        .title(format!(" CLAWBAKE v{} ", version))
        .borders(ratatui::widgets::Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let step_line = Line::from(vec![
        Span::styled(
            format!("  Step {} of {}", step_num, total),
            Style::default().fg(Color::White).bold(),
        ),
        Span::styled(
            format!("  {}", state.step.title()),
            Style::default().fg(Color::Cyan),
        ),
    ]);

    let status_line = if state.autofill_pending {
        Line::from(Span::styled(
            "  AI suggesting...",
            Style::default().fg(Color::Yellow).italic(),
        ))
    } else {
        Line::from(Span::styled(
            "  Slow-roasting system prompts to perfection.",
            Style::default().fg(Color::DarkGray).italic(),
        ))
    };

    let paragraph =
        ratatui::widgets::Paragraph::new(vec![step_line, status_line]).block(block);
    frame.render_widget(paragraph, area);
}

fn render_footer(state: &WizardState, frame: &mut Frame, area: Rect) {
    let hints = match state.step {
        WizardStep::Review => {
            "[Enter] Start  [Ctrl+P] Back  [Ctrl+Y] YOLO  [Esc] Quit"
        }
        _ => {
            "[Ctrl+N] Next  [Ctrl+P] Back  [Ctrl+G] AI Suggest  [Ctrl+Y] YOLO  [Esc] Quit"
        }
    };
    let paragraph = ratatui::widgets::Paragraph::new(hints)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    frame.render_widget(paragraph, area);
}

fn render_review(state: &WizardState, frame: &mut Frame, area: Rect) {
    use ratatui::widgets::{Block, Borders, Paragraph};

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Name: ", Style::default().bold()),
            Span::raw(&state.name.name),
        ]),
        Line::from(vec![
            Span::styled("Role: ", Style::default().bold()),
            Span::raw(&state.role.role),
        ]),
        Line::from(vec![
            Span::styled("Responsibility: ", Style::default().bold()),
            Span::raw(&state.role.responsibility),
        ]),
        Line::from(vec![
            Span::styled("Traits: ", Style::default().bold()),
            Span::raw(
                state
                    .personality
                    .selected_traits()
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        ]),
        Line::from(vec![
            Span::styled("Tools: ", Style::default().bold()),
            Span::raw(
                state
                    .tools
                    .tool_specs()
                    .iter()
                    .map(|t| t.name.clone())
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        ]),
        Line::from(vec![
            Span::styled("Guardrails: ", Style::default().bold()),
            Span::raw(state.config.guardrails.clone()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Eval cases: ", Style::default().bold()),
            Span::raw(state.config.eval_count.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Max iterations: ", Style::default().bold()),
            Span::raw(state.config.max_iterations.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Token budget: ", Style::default().bold()),
            Span::raw(format!("{}K", state.config.max_budget_tokens / 1000)),
        ]),
    ];

    if !state.reference_content.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Reference material attached",
            Style::default().fg(Color::Green),
        )));
    }

    let block = Block::default()
        .title(" Review ")
        .borders(Borders::ALL);
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
