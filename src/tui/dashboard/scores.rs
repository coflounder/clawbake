use crate::tui::dashboard::DashboardState;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render_scores(state: &DashboardState, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Scores ")
        .borders(Borders::ALL);

    let mut lines = vec![];

    if state.score_history.is_empty() {
        lines.push(Line::from(Span::styled(
            "No scores yet...",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        // Show sparkline-like representation
        let sparkline_chars = ['_', '.', '-', '~', '+', '*', '#', '@'];

        lines.push(Line::from(vec![
            Span::styled("Overall: ", Style::default().bold()),
            Span::raw(render_sparkline(&state.score_history, &sparkline_chars)),
            Span::raw(format!(
                " {:.2}",
                state.score_history.last().unwrap_or(&0.0)
            )),
        ]));

        // Show current iteration scores if available
        if !state.last_scores.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Current iteration scores:",
                Style::default().fg(Color::DarkGray),
            )));
            for (i, score) in state.last_scores.iter().enumerate() {
                let color = if *score >= 0.8 {
                    Color::Green
                } else if *score >= 0.5 {
                    Color::Yellow
                } else {
                    Color::Red
                };
                lines.push(Line::from(vec![
                    Span::raw(format!("  Case {}: ", i + 1)),
                    Span::styled(format!("{:.2}", score), Style::default().fg(color)),
                ]));
            }
        }
    }

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

fn render_sparkline(values: &[f64], chars: &[char; 8]) -> String {
    values
        .iter()
        .map(|&v| {
            let idx = ((v * 7.0).round() as usize).min(7);
            chars[idx]
        })
        .collect()
}
