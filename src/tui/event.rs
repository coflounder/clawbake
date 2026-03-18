use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use std::time::{Duration, Instant};
use crate::eval::loop_runner::EvalEvent;

#[derive(Debug)]
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    Eval(EvalEvent),
    Resize(u16, u16),
}

pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate_ms: u64) -> Self {
        Self {
            tick_rate: Duration::from_millis(tick_rate_ms),
        }
    }

    pub fn poll(&self, deadline: Instant) -> Option<AppEvent> {
        let timeout = deadline.saturating_duration_since(Instant::now());
        if timeout.is_zero() {
            return Some(AppEvent::Tick);
        }

        if event::poll(timeout).ok()? {
            match event::read().ok()? {
                CrosstermEvent::Key(key) => Some(AppEvent::Key(key)),
                CrosstermEvent::Resize(w, h) => Some(AppEvent::Resize(w, h)),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn tick_rate(&self) -> Duration {
        self.tick_rate
    }
}
