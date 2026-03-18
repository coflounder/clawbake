pub mod app;
pub mod dashboard;
pub mod event;
pub mod wizard;

use crate::config::AppConfig;
use crate::error::Result;
use crate::eval::loop_runner::EvalEvent;
use crate::tui::app::{App, AppMode};
use crate::tui::event::{AppEvent, EventHandler};
use crate::tui::wizard::autofill;
use crossterm::{
    event::{KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::io;
use std::time::Instant;
use tokio::sync::mpsc;

/// Constrain area to half terminal height, centered vertically.
fn constrained_area(full: Rect) -> Rect {
    let max_h = (full.height / 2).max(12);
    let y = (full.height.saturating_sub(max_h)) / 2;
    Rect::new(full.x, y, full.width, max_h)
}

/// Run TUI starting in dashboard mode (no wizard).
pub async fn run_dashboard(
    eval_rx: mpsc::UnboundedReceiver<EvalEvent>,
    max_iterations: usize,
    budget_limit: u64,
) -> Result<Option<AppConfig>> {
    run_tui_inner(
        Some(eval_rx),
        None,
        App::new_dashboard(max_iterations, budget_limit),
    )
    .await
}

pub async fn run_tui(
    eval_rx: Option<mpsc::UnboundedReceiver<EvalEvent>>,
    config_tx: Option<mpsc::UnboundedSender<AppConfig>>,
) -> Result<Option<AppConfig>> {
    run_tui_inner(eval_rx, config_tx, App::new()).await
}

async fn run_tui_inner(
    eval_rx: Option<mpsc::UnboundedReceiver<EvalEvent>>,
    config_tx: Option<mpsc::UnboundedSender<AppConfig>>,
    app: App,
) -> Result<Option<AppConfig>> {
    enable_raw_mode().map_err(|e| crate::error::ClawbakeError::Tui(e.to_string()))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)
        .map_err(|e| crate::error::ClawbakeError::Tui(e.to_string()))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| crate::error::ClawbakeError::Tui(e.to_string()))?;

    let mut app = app;
    let event_handler = EventHandler::new(250);
    let mut eval_rx = eval_rx;
    let mut result_config: Option<AppConfig> = None;

    // Autofill channel
    let (autofill_tx, mut autofill_rx) =
        mpsc::unbounded_channel::<Option<autofill::AutofillResult>>();

    loop {
        // Render
        terminal
            .draw(|frame| render(&app, frame))
            .map_err(|e| crate::error::ClawbakeError::Tui(e.to_string()))?;

        if app.should_quit {
            break;
        }

        // Drain autofill results
        while let Ok(result) = autofill_rx.try_recv() {
            if let AppMode::Wizard(wizard) = &mut app.mode {
                if let Some(r) = result {
                    wizard.apply_autofill(r);
                } else {
                    wizard.autofill_pending = false;
                }
            }
        }

        // Drain eval events
        if let Some(ref mut rx) = eval_rx {
            while let Ok(event) = rx.try_recv() {
                app.handle_eval_event(event);
            }
        }

        // Poll for input events
        let deadline = Instant::now() + event_handler.tick_rate();
        if let Some(event) = event_handler.poll(deadline) {
            match event {
                AppEvent::Key(key) => {
                    // Global quit
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        app.should_quit = true;
                        continue;
                    }

                    match &mut app.mode {
                        AppMode::Wizard(wizard) => {
                            let action = wizard.handle_key(key);
                            match action {
                                wizard::WizardAction::None => {}
                                wizard::WizardAction::Complete(config) => {
                                    result_config = Some(config.clone());
                                    // Notify caller so it can spawn the eval loop
                                    if let Some(ref tx) = config_tx {
                                        let _ = tx.send(config.clone());
                                    }
                                    app.mode = AppMode::Dashboard(
                                        dashboard::DashboardState::new(
                                            config.eval.max_iterations,
                                            config.eval.max_budget_tokens,
                                        ),
                                    );
                                }
                                wizard::WizardAction::Quit => {
                                    app.should_quit = true;
                                }
                                wizard::WizardAction::RequestAutofill(ctx) => {
                                    let tx = autofill_tx.clone();
                                    tokio::spawn(async move {
                                        let result = autofill::run_autofill(ctx).await;
                                        let _ = tx.send(result);
                                    });
                                }
                            }
                        }
                        AppMode::Dashboard(_dashboard) => match key.code {
                            KeyCode::Char('q') => {
                                app.should_quit = true;
                            }
                            KeyCode::Char('s') => {
                                // Signal stop - handled externally
                            }
                            _ => {}
                        },
                        AppMode::Done { .. } => {
                            if key.code == KeyCode::Char('q') || key.code == KeyCode::Enter {
                                app.should_quit = true;
                            }
                        }
                    }
                }
                AppEvent::Tick => {}
                AppEvent::Eval(event) => {
                    app.handle_eval_event(event);
                }
                AppEvent::Resize(_, _) => {}
            }
        }
    }

    // Restore terminal
    disable_raw_mode().map_err(|e| crate::error::ClawbakeError::Tui(e.to_string()))?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .map_err(|e| crate::error::ClawbakeError::Tui(e.to_string()))?;
    terminal
        .show_cursor()
        .map_err(|e| crate::error::ClawbakeError::Tui(e.to_string()))?;

    Ok(result_config)
}

fn render(app: &App, frame: &mut Frame) {
    let area = constrained_area(frame.area());

    match &app.mode {
        AppMode::Wizard(wizard) => wizard::render_wizard(wizard, frame, area),
        AppMode::Dashboard(dashboard) => dashboard::render_dashboard(dashboard, frame, area),
        AppMode::Done { reason, best_score } => {
            let block = Block::default()
                .title(" Clawbake Complete ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green));
            let text = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("Result: ", Style::default().bold()),
                    Span::raw(reason),
                ]),
                Line::from(vec![
                    Span::styled("Best Score: ", Style::default().bold()),
                    Span::styled(
                        format!("{:.2}", best_score),
                        Style::default().fg(Color::Yellow),
                    ),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "Press 'q' or Enter to exit",
                    Style::default().fg(Color::DarkGray),
                )),
            ];
            let paragraph = Paragraph::new(text)
                .block(block)
                .alignment(Alignment::Center);
            frame.render_widget(paragraph, area);
        }
    }
}
