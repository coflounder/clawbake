use crate::tui::dashboard::DashboardState;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render_mutations(state: &DashboardState, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Mutation Log ")
        .borders(Borders::ALL);

    let items: Vec<ListItem> = state
        .mutations
        .iter()
        .rev()
        .enumerate()
        .map(|(i, mutation)| {
            let iter_num = state.mutations.len() - i;
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("iter {}: ", iter_num),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(mutation),
            ]))
        })
        .collect();

    if items.is_empty() {
        let empty = Paragraph::new(Span::styled(
            "No mutations yet...",
            Style::default().fg(Color::DarkGray),
        ))
        .block(block);
        frame.render_widget(empty, area);
    } else {
        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}
