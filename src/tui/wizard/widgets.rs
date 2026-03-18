use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::*;

fn prev_word_boundary(s: &str, pos: usize) -> usize {
    if pos == 0 {
        return 0;
    }
    let bytes = s.as_bytes();
    let mut i = pos;
    // Skip whitespace/punctuation backward
    while i > 0 && !bytes[i - 1].is_ascii_alphanumeric() {
        i -= 1;
    }
    // Skip word characters backward
    while i > 0 && bytes[i - 1].is_ascii_alphanumeric() {
        i -= 1;
    }
    i
}

fn next_word_boundary(s: &str, pos: usize) -> usize {
    let len = s.len();
    if pos >= len {
        return len;
    }
    let bytes = s.as_bytes();
    let mut i = pos;
    // Skip current word characters
    while i < len && bytes[i].is_ascii_alphanumeric() {
        i += 1;
    }
    // Skip whitespace/punctuation
    while i < len && !bytes[i].is_ascii_alphanumeric() {
        i += 1;
    }
    i
}

#[derive(Debug)]
pub struct TextInput {
    pub value: String,
    pub cursor: usize,
    pub label: String,
}

impl TextInput {
    pub fn new(label: &str) -> Self {
        Self {
            value: String::new(),
            cursor: 0,
            label: label.to_string(),
        }
    }

    pub fn set_value(&mut self, val: &str) {
        self.value = val.to_string();
        self.cursor = self.value.len();
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            // Cmd+Left / Home → beginning of line
            KeyCode::Left if key.modifiers.contains(KeyModifiers::SUPER) => {
                self.cursor = 0;
            }
            // Cmd+Right / End → end of line
            KeyCode::Right if key.modifiers.contains(KeyModifiers::SUPER) => {
                self.cursor = self.value.len();
            }
            // Option+Left → previous word
            KeyCode::Left if key.modifiers.contains(KeyModifiers::ALT) => {
                self.cursor = prev_word_boundary(&self.value, self.cursor);
            }
            // Option+Right → next word
            KeyCode::Right if key.modifiers.contains(KeyModifiers::ALT) => {
                self.cursor = next_word_boundary(&self.value, self.cursor);
            }
            // Option+Backspace → delete previous word
            KeyCode::Backspace if key.modifiers.contains(KeyModifiers::ALT) => {
                let target = prev_word_boundary(&self.value, self.cursor);
                self.value.drain(target..self.cursor);
                self.cursor = target;
            }
            KeyCode::Char(c) => {
                self.value.insert(self.cursor, c);
                self.cursor += 1;
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.value.remove(self.cursor);
                }
            }
            KeyCode::Delete => {
                if self.cursor < self.value.len() {
                    self.value.remove(self.cursor);
                }
            }
            KeyCode::Left => {
                self.cursor = self.cursor.saturating_sub(1);
            }
            KeyCode::Right => {
                self.cursor = (self.cursor + 1).min(self.value.len());
            }
            KeyCode::Home => {
                self.cursor = 0;
            }
            KeyCode::End => {
                self.cursor = self.value.len();
            }
            _ => {}
        }
        false
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let block = Block::default()
            .title(format!(" {} ", self.label))
            .borders(Borders::ALL)
            .border_style(style);

        let display_text = if focused {
            let before = &self.value[..self.cursor];
            let cursor_char = if self.cursor < self.value.len() {
                &self.value[self.cursor..self.cursor + 1]
            } else {
                " "
            };
            let after = if self.cursor < self.value.len() {
                &self.value[self.cursor + 1..]
            } else {
                ""
            };
            Line::from(vec![
                Span::raw(before),
                Span::styled(cursor_char, Style::default().add_modifier(Modifier::REVERSED)),
                Span::raw(after),
            ])
        } else {
            Line::from(self.value.as_str())
        };

        let paragraph = Paragraph::new(display_text).block(block);
        frame.render_widget(paragraph, area);
    }
}

#[derive(Debug)]
pub struct TextArea {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub label: String,
}

impl TextArea {
    pub fn new(label: &str) -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            label: label.to_string(),
        }
    }

    pub fn content(&self) -> String {
        self.lines.join("\n")
    }

    pub fn set_content(&mut self, content: &str) {
        self.lines = content.lines().map(|l| l.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        let line = &self.lines[self.cursor_row];
        match key.code {
            // Cmd+Left → beginning of line
            KeyCode::Left if key.modifiers.contains(KeyModifiers::SUPER) => {
                self.cursor_col = 0;
            }
            // Cmd+Right → end of line
            KeyCode::Right if key.modifiers.contains(KeyModifiers::SUPER) => {
                self.cursor_col = line.len();
            }
            // Option+Left → previous word
            KeyCode::Left if key.modifiers.contains(KeyModifiers::ALT) => {
                self.cursor_col = prev_word_boundary(line, self.cursor_col);
            }
            // Option+Right → next word
            KeyCode::Right if key.modifiers.contains(KeyModifiers::ALT) => {
                self.cursor_col = next_word_boundary(line, self.cursor_col);
            }
            // Option+Backspace → delete previous word
            KeyCode::Backspace if key.modifiers.contains(KeyModifiers::ALT) => {
                let target = prev_word_boundary(
                    &self.lines[self.cursor_row],
                    self.cursor_col,
                );
                self.lines[self.cursor_row].drain(target..self.cursor_col);
                self.cursor_col = target;
            }
            KeyCode::Char(c) => {
                self.lines[self.cursor_row].insert(self.cursor_col, c);
                self.cursor_col += 1;
            }
            KeyCode::Enter => {
                let current = self.lines[self.cursor_row].clone();
                let (before, after) = current.split_at(self.cursor_col);
                self.lines[self.cursor_row] = before.to_string();
                self.lines.insert(self.cursor_row + 1, after.to_string());
                self.cursor_row += 1;
                self.cursor_col = 0;
            }
            KeyCode::Backspace => {
                if self.cursor_col > 0 {
                    self.lines[self.cursor_row].remove(self.cursor_col - 1);
                    self.cursor_col -= 1;
                } else if self.cursor_row > 0 {
                    let current = self.lines.remove(self.cursor_row);
                    self.cursor_row -= 1;
                    self.cursor_col = self.lines[self.cursor_row].len();
                    self.lines[self.cursor_row].push_str(&current);
                }
            }
            KeyCode::Up => {
                if self.cursor_row > 0 {
                    self.cursor_row -= 1;
                    self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
                }
            }
            KeyCode::Down => {
                if self.cursor_row + 1 < self.lines.len() {
                    self.cursor_row += 1;
                    self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
                }
            }
            KeyCode::Left => {
                self.cursor_col = self.cursor_col.saturating_sub(1);
            }
            KeyCode::Right => {
                self.cursor_col =
                    (self.cursor_col + 1).min(self.lines[self.cursor_row].len());
            }
            KeyCode::Home => {
                self.cursor_col = 0;
            }
            KeyCode::End => {
                self.cursor_col = self.lines[self.cursor_row].len();
            }
            _ => {}
        }
        false
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let block = Block::default()
            .title(format!(" {} ", self.label))
            .borders(Borders::ALL)
            .border_style(style);

        let text: Vec<Line> = self
            .lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if focused && i == self.cursor_row {
                    let col = self.cursor_col.min(line.len());
                    let before = &line[..col];
                    let cursor_char = if col < line.len() {
                        &line[col..col + 1]
                    } else {
                        " "
                    };
                    let after = if col < line.len() { &line[col + 1..] } else { "" };
                    Line::from(vec![
                        Span::raw(before),
                        Span::styled(
                            cursor_char,
                            Style::default().add_modifier(Modifier::REVERSED),
                        ),
                        Span::raw(after),
                    ])
                } else {
                    Line::from(line.as_str())
                }
            })
            .collect();

        let paragraph = Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    }
}

#[derive(Debug)]
pub struct MultiSelect {
    pub items: Vec<(String, bool)>,
    pub cursor: usize,
    pub label: String,
    pub max_selected: Option<usize>,
}

impl MultiSelect {
    pub fn new(label: &str, items: Vec<String>) -> Self {
        let items = items.into_iter().map(|s| (s, false)).collect();
        Self {
            items,
            cursor: 0,
            label: label.to_string(),
            max_selected: None,
        }
    }

    pub fn with_max(mut self, max: usize) -> Self {
        self.max_selected = Some(max);
        self
    }

    pub fn selected_count(&self) -> usize {
        self.items.iter().filter(|(_, s)| *s).count()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Up => {
                self.cursor = self.cursor.saturating_sub(1);
            }
            KeyCode::Down => {
                if self.cursor + 1 < self.items.len() {
                    self.cursor += 1;
                }
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                if self.cursor < self.items.len() {
                    if self.items[self.cursor].1 {
                        // Always allow deselection
                        self.items[self.cursor].1 = false;
                    } else if let Some(max) = self.max_selected {
                        if self.selected_count() < max {
                            self.items[self.cursor].1 = true;
                        }
                    } else {
                        self.items[self.cursor].1 = true;
                    }
                }
            }
            _ => {}
        }
        false
    }

    pub fn selected(&self) -> Vec<String> {
        self.items
            .iter()
            .filter(|(_, selected)| *selected)
            .map(|(name, _)| name.clone())
            .collect()
    }

    pub fn select_by_names(&mut self, names: &[String]) {
        for item in &mut self.items {
            item.1 = names.contains(&item.0);
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let max_hint = if let Some(max) = self.max_selected {
            format!(" (max {}, {}/{}) ", max, self.selected_count(), max)
        } else {
            String::new()
        };

        let block = Block::default()
            .title(format!(" {} {}", self.label, max_hint))
            .borders(Borders::ALL)
            .border_style(style);

        let inner_height = area.height.saturating_sub(2) as usize;
        let scroll_offset = if self.cursor >= inner_height {
            self.cursor - inner_height + 1
        } else {
            0
        };

        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(inner_height)
            .map(|(i, (name, selected))| {
                let marker = if *selected { "[x]" } else { "[ ]" };
                let at_max = if let Some(max) = self.max_selected {
                    self.selected_count() >= max && !*selected
                } else {
                    false
                };
                let item_style = if i == self.cursor && focused {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if at_max {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };
                ListItem::new(format!("{} {}", marker, name)).style(item_style)
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}
