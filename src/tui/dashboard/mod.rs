pub mod mutations;
pub mod progress;
pub mod scores;
pub mod transcript;

use ratatui::prelude::*;
use ratatui::widgets::*;
use std::time::Instant;

#[derive(Debug)]
pub struct DashboardState {
    pub started_at: Instant,
    pub current_iteration: usize,
    pub max_iterations: usize,
    pub phase: String,
    pub total_cases: usize,
    pub completed_cases_in_iter: usize,
    pub active_case: Option<String>,
    pub score_history: Vec<f64>,
    pub last_scores: Vec<f64>,
    pub mutations: Vec<String>,
    pub budget_consumed: u64,
    pub budget_limit: u64,
    pub errors: Vec<String>,
    pub logs: Vec<String>,
}

impl DashboardState {
    pub fn new(max_iterations: usize, budget_limit: u64) -> Self {
        Self {
            started_at: Instant::now(),
            current_iteration: 0,
            max_iterations,
            phase: "Initializing".to_string(),
            total_cases: 0,
            completed_cases_in_iter: 0,
            active_case: None,
            score_history: Vec::new(),
            last_scores: Vec::new(),
            mutations: Vec::new(),
            budget_consumed: 0,
            budget_limit,
            errors: Vec::new(),
            logs: Vec::new(),
        }
    }
}

pub fn render_dashboard(state: &DashboardState, frame: &mut Frame, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(6),   // Top: scores + budget + mutations
            Constraint::Min(5),   // Bottom: live logs
            Constraint::Length(3), // Footer
        ])
        .split(area);

    // Title bar with timer
    let elapsed = state.started_at.elapsed();
    let mins = elapsed.as_secs() / 60;
    let secs = elapsed.as_secs() % 60;
    let timer = format!("{}:{:02}", mins, secs);

    let title = format!(
        " Iteration {}/{} - {} ",
        state.current_iteration, state.max_iterations, state.phase
    );
    let title_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = title_block.inner(main_chunks[0]);
    frame.render_widget(title_block, main_chunks[0]);
    // Render timer right-aligned inside the title block
    let timer_span = Span::styled(
        format!("{} ", timer),
        Style::default().fg(Color::DarkGray),
    );
    let timer_para = Paragraph::new(timer_span).alignment(Alignment::Right);
    frame.render_widget(timer_para, inner);

    // Top row: three columns — scores | budget+cases | mutations
    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Percentage(30),
            Constraint::Percentage(35),
        ])
        .split(main_chunks[1]);

    scores::render_scores(state, frame, top_cols[0]);

    // Middle column: budget gauge + active cases stacked
    let mid_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(1)])
        .split(top_cols[1]);
    progress::render_budget(state, frame, mid_chunks[0]);
    render_active_cases(state, frame, mid_chunks[1]);

    mutations::render_mutations(state, frame, top_cols[2]);

    // Live logs
    render_logs(state, frame, main_chunks[2]);

    // Footer
    let footer = Paragraph::new("[q] Quit  [s] Stop eval loop")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    frame.render_widget(footer, main_chunks[3]);
}

fn render_active_cases(state: &DashboardState, frame: &mut Frame, area: Rect) {
    let mut lines = vec![];

    if let Some(ref active) = state.active_case {
        lines.push(Line::from(vec![
            Span::styled("[running] ", Style::default().fg(Color::Yellow)),
            Span::raw(active),
        ]));
    }

    lines.push(Line::from(format!(
        "Completed: {}/{}",
        state.completed_cases_in_iter, state.total_cases
    )));

    if !state.errors.is_empty() {
        lines.push(Line::from(""));
        for err in state.errors.iter().rev().take(3) {
            lines.push(Line::from(Span::styled(
                err.as_str(),
                Style::default().fg(Color::Red),
            )));
        }
    }

    let block = Block::default()
        .title(" Active Cases ")
        .borders(Borders::ALL);
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

fn render_logs(state: &DashboardState, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Live Log ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner_height = area.height.saturating_sub(2) as usize;

    let lines: Vec<Line> = state
        .logs
        .iter()
        .rev()
        .take(inner_height)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|msg| {
            let style = if msg.starts_with("[ERR]") {
                Style::default().fg(Color::Red)
            } else if msg.starts_with("---") {
                Style::default().fg(Color::Cyan).bold()
            } else {
                Style::default().fg(Color::DarkGray)
            };
            Line::from(Span::styled(msg.as_str(), style))
        })
        .collect();

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
