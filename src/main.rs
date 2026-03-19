mod cli;
mod claude;
mod config;
mod error;
mod eval;
mod export;
mod io;
mod sandbox;
mod tui;
mod types;

use crate::cli::{Cli, Commands};
use crate::config::AppConfig;
use crate::eval::loop_runner::{EvalEvent, LoopRunner};
use crate::io::state::StateDir;
use crate::types::TokenBudget;
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let base_dir = cli.dir.unwrap_or_else(|| PathBuf::from("."));
    let state_dir = StateDir::new(&base_dir);

    match cli.command {
        Commands::Init => cmd_init(&state_dir).await?,
        Commands::Run { no_wizard, headless } => cmd_run(&state_dir, no_wizard, headless).await?,
        Commands::Status => cmd_status(&state_dir)?,
        Commands::Export { output } => cmd_export(&state_dir, output)?,
    }

    Ok(())
}

async fn cmd_init(state_dir: &StateDir) -> anyhow::Result<()> {
    state_dir.init()?;

    // Run wizard TUI to collect persona spec
    let config = tui::run_tui(None, None).await?;

    if let Some(config) = config {
        config.save(&state_dir.config_path())?;
        println!("Project initialized at {}", state_dir.root().display());
        println!("Config saved to {}", state_dir.config_path().display());
    } else {
        println!("Setup cancelled.");
    }

    Ok(())
}

async fn cmd_run(state_dir: &StateDir, no_wizard: bool, headless: bool) -> anyhow::Result<()> {
    if no_wizard {
        // No wizard — load config from file and run eval directly
        if !state_dir.config_path().exists() {
            anyhow::bail!(
                "No config found at {}. Run 'clawbake init' first.",
                state_dir.config_path().display()
            );
        }
        let config = AppConfig::load(&state_dir.config_path())?;
        state_dir.init()?;
        state_dir.clean_run_data()?;
        if headless {
            return run_eval_headless(state_dir, config).await;
        }
        return run_eval_with_dashboard(state_dir, config).await;
    }

    // With wizard: single TUI session that transitions wizard → dashboard.
    // The eval loop spawns when the wizard completes, via a config channel.
    state_dir.init()?;

    let (event_tx, event_rx) = mpsc::unbounded_channel::<EvalEvent>();
    let (config_tx, mut config_rx) = mpsc::unbounded_channel::<AppConfig>();

    // Spawn a listener that starts the eval loop once the wizard produces a config
    let state_root = state_dir.root().to_path_buf();
    let eval_handle = tokio::spawn(async move {
        let Some(config) = config_rx.recv().await else {
            return Ok(());
        };

        let sd = StateDir::new(
            state_root
                .parent()
                .unwrap_or(&state_root),
        );

        config.save(&sd.config_path())?;

        // Write reference material if provided
        let reference = config
            .persona
            .guardrails
            .iter()
            .map(|g| g.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        if !reference.is_empty() {
            // Reference is handled separately via state_dir
        }

        let budget = Arc::new(Mutex::new(TokenBudget::new(config.eval.max_budget_tokens)));
        let client = claude::client::ClaudeClient::new(config.clone(), budget);
        let event_tx_err = event_tx.clone();
        let runner = LoopRunner::new(client, config, sd, event_tx);
        match runner.run().await {
            Ok(_) => {}
            Err(e) => {
                let _ = event_tx_err.send(EvalEvent::LoopComplete {
                    reason: format!("Error: {}", e),
                    best_score: 0.0,
                });
            }
        }
        Ok::<(), crate::error::ClawbakeError>(())
    });

    // Run the TUI — wizard first, then dashboard in the same session
    let _tui_result = tui::run_tui(Some(event_rx), Some(config_tx)).await?;

    // Wait for eval loop to finish (or it may have already stopped)
    match eval_handle.await? {
        Ok(()) => println!("Eval loop completed."),
        Err(e) => println!("Eval loop error: {}", e),
    }

    Ok(())
}

/// Run eval loop with dashboard TUI (no wizard, config already known).
async fn run_eval_with_dashboard(
    state_dir: &StateDir,
    config: AppConfig,
) -> anyhow::Result<()> {
    let (event_tx, event_rx) = mpsc::unbounded_channel::<EvalEvent>();

    let max_iterations = config.eval.max_iterations;
    let max_budget = config.eval.max_budget_tokens;

    let budget = Arc::new(Mutex::new(TokenBudget::new(max_budget)));
    let client = claude::client::ClaudeClient::new(config.clone(), budget);
    let event_tx_err = event_tx.clone();
    let runner = LoopRunner::new(
        client,
        config,
        StateDir::new(state_dir.root().parent().unwrap_or(state_dir.root())),
        event_tx,
    );

    let stop_handle = runner.stop_handle();

    let eval_handle = tokio::spawn(async move {
        let result = runner.run().await;
        if let Err(ref e) = result {
            let _ = event_tx_err.send(EvalEvent::LoopComplete {
                reason: format!("Error: {}", e),
                best_score: 0.0,
            });
        }
        result
    });

    // Run dashboard TUI directly (skip wizard)
    let _tui_result = tui::run_dashboard(event_rx, max_iterations, max_budget).await?;

    // Signal stop if TUI exits early
    *stop_handle.lock().await = true;

    match eval_handle.await? {
        Ok(reason) => println!("Eval loop completed: {}", reason),
        Err(e) => println!("Eval loop error: {}", e),
    }

    Ok(())
}

/// Run eval loop without TUI — log events to stdout.
async fn run_eval_headless(
    state_dir: &StateDir,
    config: AppConfig,
) -> anyhow::Result<()> {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<EvalEvent>();

    let budget = Arc::new(Mutex::new(TokenBudget::new(config.eval.max_budget_tokens)));
    let client = claude::client::ClaudeClient::new(config.clone(), budget);
    let runner = LoopRunner::new(
        client,
        config,
        StateDir::new(state_dir.root().parent().unwrap_or(state_dir.root())),
        event_tx,
    );

    let eval_handle = tokio::spawn(async move { runner.run().await });

    // Drain events and print them
    let printer = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                EvalEvent::Log { message } => println!("  {}", message),
                EvalEvent::IterationStarted { iteration } => {
                    if iteration > 0 {
                        println!("\n=== Iteration {} ===", iteration);
                    }
                }
                EvalEvent::PlanningComplete { case_count } => {
                    println!("  Planned {} eval cases", case_count);
                }
                EvalEvent::CaseComplete { case_id, score } => {
                    println!("  {} => {:.3}", case_id, score);
                }
                EvalEvent::EvaluationComplete {
                    iteration,
                    average_score,
                } => {
                    println!("  Iteration {} avg: {:.3}", iteration, average_score);
                }
                EvalEvent::OptimizationComplete { mutation, .. } => {
                    println!("  Mutation: {}", mutation);
                }
                EvalEvent::BudgetUpdate { consumed, limit } => {
                    println!(
                        "  Budget: {}K / {}K ({:.1}%)",
                        consumed / 1000,
                        limit / 1000,
                        consumed as f64 / limit as f64 * 100.0
                    );
                }
                EvalEvent::LoopComplete { reason, best_score } => {
                    println!("\n=== Complete ===");
                    println!("  Reason: {}", reason);
                    println!("  Best score: {:.3}", best_score);
                }
                EvalEvent::Error { message } => {
                    eprintln!("  ERROR: {}", message);
                }
                _ => {}
            }
        }
    });

    match eval_handle.await? {
        Ok(reason) => println!("Eval loop completed: {}", reason),
        Err(e) => println!("Eval loop error: {}", e),
    }

    drop(printer);
    Ok(())
}

fn cmd_status(state_dir: &StateDir) -> anyhow::Result<()> {
    if !state_dir.exists() {
        println!("No clawbake project found. Run 'clawbake init' first.");
        return Ok(());
    }

    let config = AppConfig::load(&state_dir.config_path())?;
    println!("Project: {}", config.persona.name);
    println!("Role: {}", config.persona.role);

    // Load history
    let history = io::history::load_history(&state_dir.history_path())?;

    if history.is_empty() {
        println!("No eval runs yet. Run 'clawbake run' to start.");
    } else {
        let last = history.last().unwrap();
        println!("Iterations completed: {}", last.iteration);
        println!("Best score: {:.2}", last.best_score);
        println!("Average score: {:.2}", last.average_score);
        println!("Tokens used: {}K", last.tokens_used / 1000);
        println!("Last mutation: {}", last.mutation_summary);
    }

    if state_dir.best_identity_path().exists() {
        println!(
            "\nBest identity: {}",
            state_dir.best_identity_path().display()
        );
    }

    Ok(())
}

fn cmd_export(state_dir: &StateDir, output: Option<PathBuf>) -> anyhow::Result<()> {
    if !state_dir.exists() {
        anyhow::bail!("No clawbake project found. Run 'clawbake init' first.");
    }

    let config = AppConfig::load(&state_dir.config_path())?;
    let output_dir = output.unwrap_or_else(|| PathBuf::from("."));

    let written = export::export_identity(state_dir, &config, &output_dir)?;

    println!("Exported identity files:");
    for path in &written {
        println!("  {}", path);
    }

    Ok(())
}
