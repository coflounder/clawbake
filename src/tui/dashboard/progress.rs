use crate::tui::dashboard::DashboardState;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render_budget(state: &DashboardState, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Budget ")
        .borders(Borders::ALL);

    let fraction = if state.budget_limit > 0 {
        state.budget_consumed as f64 / state.budget_limit as f64
    } else {
        0.0
    };
    let percent = (fraction * 100.0) as u16;

    let consumed_k = state.budget_consumed / 1000;
    let limit_k = state.budget_limit / 1000;

    let gauge = Gauge::default()
        .block(block)
        .gauge_style(
            Style::default()
                .fg(if fraction > 0.9 {
                    Color::Red
                } else if fraction > 0.7 {
                    Color::Yellow
                } else {
                    Color::Green
                })
                .bg(Color::DarkGray),
        )
        .percent(percent.min(100))
        .label(format!("{}K / {}K tokens ({}%)", consumed_k, limit_k, percent));

    frame.render_widget(gauge, area);
}
