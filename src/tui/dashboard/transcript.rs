use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct TranscriptState {
    pub content: String,
    pub scroll: u16,
}

impl TranscriptState {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            scroll: 0,
        }
    }
}

pub fn render_transcript(state: &TranscriptState, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Transcript Preview ")
        .borders(Borders::ALL);

    let paragraph = Paragraph::new(state.content.as_str())
        .block(block)
        .scroll((state.scroll, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
