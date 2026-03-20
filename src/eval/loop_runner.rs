use crate::claude::client::ClaudeClient;
use crate::config::AppConfig;
use crate::error::Result;
use crate::eval::convergence::{ConvergenceChecker, StopReason};
use crate::eval::evaluator;
use crate::eval::optimizer;
use crate::eval::planner;
use crate::eval::runner;
use crate::io::{history, identity, state::StateDir};
use crate::sandbox::environment::SandboxEnvironment;
use crate::types::HistoryEntry;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

#[derive(Debug, Clone)]
pub enum EvalEvent {
    IterationStarted { iteration: usize },
    PlanningComplete { case_count: usize },
    CaseStarted { case_id: String, title: String },
    CaseComplete { case_id: String, score: f64 },
    EvaluationComplete { iteration: usize, average_score: f64 },
    OptimizationComplete { iteration: usize, mutation: String },
    BudgetUpdate { consumed: u64, limit: u64 },
    ConvergenceCheck { stop_reason: Option<String> },
    LoopComplete { reason: String, best_score: f64 },
    Error { message: String },
    Log { message: String },
}

pub struct LoopRunner {
    client: ClaudeClient,
    config: AppConfig,
    state_dir: StateDir,
    event_tx: mpsc::UnboundedSender<EvalEvent>,
    user_stopped: Arc<Mutex<bool>>,
}

/// Load held-constant context files and concatenate them into a single string
/// that gets prepended to the system prompt alongside the target document.
fn load_held_context(held: &crate::types::HeldContext) -> String {
    let mut context = String::new();

    if let Some(ref path) = held.claude_md {
        if let Ok(content) = std::fs::read_to_string(path) {
            context.push_str("---\n# Held Constant: CLAUDE.md\n");
            context.push_str(&content);
            context.push_str("\n---\n\n");
        }
    }

    if let Some(ref path) = held.agents_md {
        if let Ok(content) = std::fs::read_to_string(path) {
            context.push_str("---\n# Held Constant: AGENTS.md\n");
            context.push_str(&content);
            context.push_str("\n---\n\n");
        }
    }

    if let Some(ref path) = held.memory_md {
        if let Ok(content) = std::fs::read_to_string(path) {
            context.push_str("---\n# Held Constant: MEMORY.md\n");
            context.push_str(&content);
            context.push_str("\n---\n\n");
        }
    }

    for path in &held.skills {
        if let Ok(content) = std::fs::read_to_string(path) {
            let label = path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("skill");
            context.push_str(&format!("---\n# Held Constant: {}\n", label));
            context.push_str(&content);
            context.push_str("\n---\n\n");
        }
    }

    context
}

impl LoopRunner {
    pub fn new(
        client: ClaudeClient,
        config: AppConfig,
        state_dir: StateDir,
        event_tx: mpsc::UnboundedSender<EvalEvent>,
    ) -> Self {
        Self {
            client,
            config,
            state_dir,
            event_tx,
            user_stopped: Arc::new(Mutex::new(false)),
        }
    }

    pub fn stop_handle(&self) -> Arc<Mutex<bool>> {
        Arc::clone(&self.user_stopped)
    }

    fn log(&self, msg: impl Into<String>) {
        let _ = self.event_tx.send(EvalEvent::Log {
            message: msg.into(),
        });
    }

    pub async fn run(self) -> Result<StopReason> {
        let reference = std::fs::read_to_string(self.state_dir.reference_path())
            .unwrap_or_default();

        // Phase 1: Plan - generate eval cases
        self.log("Planning eval cases via claude -p --model sonnet...");
        let _ = self.event_tx.send(EvalEvent::IterationStarted { iteration: 0 });

        let cases = match planner::generate_eval_cases(
            &self.client,
            &self.config.persona,
            &reference,
            self.config.eval.eval_count,
            &self.config.mode.target,
        )
        .await
        {
            Ok(cases) => cases,
            Err(e) => {
                let _ = self.event_tx.send(EvalEvent::Error {
                    message: format!("Planning failed: {}", e),
                });
                return Err(e);
            }
        };

        // Save cases
        let cases_json = serde_json::to_string_pretty(&cases)?;
        std::fs::write(self.state_dir.cases_path(), &cases_json)?;

        self.log(format!("Planning complete: {} eval cases generated", cases.len()));
        let _ = self.event_tx.send(EvalEvent::PlanningComplete {
            case_count: cases.len(),
        });

        // Generate initial identity (mode-aware)
        let mut current_identity = match self.config.mode.target {
            crate::types::EvalMode::Soul => identity::generate_soul(&self.config.persona),
            _ => identity::generate_identity(&self.config.persona),
        };
        let mut convergence = ConvergenceChecker::new(self.config.eval.max_iterations);
        let mut best_score: f64 = 0.0;
        let mut hist = history::load_history(&self.state_dir.history_path())?;

        // Load held-constant context
        let held_context = load_held_context(&self.config.mode.hold_constant);

        // Set up sandbox
        let sandbox = SandboxEnvironment::new(&self.config.persona.tools)?;
        let allowed_tools = SandboxEnvironment::builtin_tools(&self.config.persona.tools);
        self.log(format!(
            "Sandbox ready (tools: {})",
            if allowed_tools.is_empty() {
                "none".to_string()
            } else {
                allowed_tools.join(", ")
            }
        ));

        for iteration in 1..=self.config.eval.max_iterations {
            // Check user stop
            if *self.user_stopped.lock().await {
                return Ok(StopReason::UserStopped);
            }

            self.log(format!(
                "--- Iteration {}/{} ---",
                iteration, self.config.eval.max_iterations
            ));
            let _ = self.event_tx.send(EvalEvent::IterationStarted { iteration });

            self.state_dir.ensure_iteration_dir(iteration)?;

            // Build system prompt: identity + held context for runners
            let system_prompt = if held_context.is_empty() {
                current_identity.clone()
            } else {
                format!("{}\n\n{}", current_identity, held_context)
            };

            // Phase 2: Run cases (mode-aware)
            let case_results = match self.config.mode.target {
                crate::types::EvalMode::Soul => {
                    let session_count = self.config.mode.soul.session_count;
                    self.log(format!(
                        "Running {} cases x {} sessions (max {} parallel, {} turns)...",
                        cases.len(),
                        session_count,
                        self.config.eval.max_parallel,
                        self.config.eval.max_turns_per_case,
                    ));
                    runner::run_cases_multi_session(
                        &self.client,
                        &cases,
                        &system_prompt,
                        &sandbox,
                        &allowed_tools,
                        self.config.eval.max_parallel,
                        self.config.eval.max_turns_per_case,
                        session_count,
                        &self.event_tx,
                    )
                    .await?
                }
                _ => {
                    self.log(format!(
                        "Running {} cases (max {} parallel, {} turns)...",
                        cases.len(),
                        self.config.eval.max_parallel,
                        self.config.eval.max_turns_per_case,
                    ));
                    runner::run_cases(
                        &self.client,
                        &cases,
                        &system_prompt,
                        &sandbox,
                        &allowed_tools,
                        self.config.eval.max_parallel,
                        self.config.eval.max_turns_per_case,
                        &self.event_tx,
                    )
                    .await?
                }
            };

            // Phase 3: Evaluate (uses current_identity without held context)
            self.log("Evaluating transcripts...");
            let mut scores = Vec::new();
            for (case, result) in cases.iter().zip(case_results.iter()) {
                let score = evaluator::evaluate_case(
                    &self.client,
                    case,
                    result,
                    &current_identity,
                    &self.config.mode.target,
                )
                .await?;

                let _ = self.event_tx.send(EvalEvent::CaseComplete {
                    case_id: case.id.clone(),
                    score: score.overall,
                });

                scores.push(score);
            }

            // Summarize transcripts
            self.log("Summarizing transcripts...");
            for (case, result) in cases.iter().zip(case_results.iter()) {
                let summary = evaluator::summarize_transcript(
                    &self.client,
                    &result.transcript,
                    &case.title,
                )
                .await?;
                let transcript_path = self
                    .state_dir
                    .iteration_transcripts_dir(iteration)
                    .join(format!("{}.md", case.id));
                std::fs::write(transcript_path, &summary)?;
            }

            // Compute average
            let avg_score = if scores.is_empty() {
                0.0
            } else {
                scores.iter().map(|s| s.overall).sum::<f64>() / scores.len() as f64
            };

            // Save scores
            let scores_json = serde_json::to_string_pretty(&scores)?;
            std::fs::write(self.state_dir.iteration_scores_path(iteration), &scores_json)?;

            self.log(format!("Iteration {} avg score: {:.3}", iteration, avg_score));
            let _ = self.event_tx.send(EvalEvent::EvaluationComplete {
                iteration,
                average_score: avg_score,
            });

            convergence.record_score(avg_score);

            // Update best (mode-aware path)
            if avg_score > best_score {
                best_score = avg_score;
                let best_path = match self.config.mode.target {
                    crate::types::EvalMode::Soul => self.state_dir.best_soul_path(),
                    _ => self.state_dir.best_identity_path(),
                };
                identity::write_identity(&best_path, &current_identity)?;
            }

            // Send budget update
            {
                let budget_arc = self.client.budget();
                let budget = budget_arc.lock().await;
                let _ = self.event_tx.send(EvalEvent::BudgetUpdate {
                    consumed: budget.consumed,
                    limit: budget.limit,
                });
            }

            // Phase 4: Check convergence before optimizing
            {
                let budget_arc = self.client.budget();
                let budget = budget_arc.lock().await;
                let user_stopped = *self.user_stopped.lock().await;
                if let Some(reason) = convergence.check(iteration, &budget, user_stopped) {
                    // Save final identity for this iteration (mode-aware path)
                    let iter_path = match self.config.mode.target {
                        crate::types::EvalMode::Soul => self.state_dir.iteration_soul_path(iteration),
                        _ => self.state_dir.iteration_identity_path(iteration),
                    };
                    identity::write_identity(&iter_path, &current_identity)?;

                    let _ = self.event_tx.send(EvalEvent::LoopComplete {
                        reason: reason.to_string(),
                        best_score,
                    });

                    return Ok(reason);
                }
            }

            // Phase 5: Optimize (uses current_identity without held context)
            self.log("Optimizing identity document...");
            let opt_result = optimizer::optimize_identity(
                &self.client,
                &current_identity,
                &scores,
                &hist,
                &reference,
                &self.config.mode.target,
            )
            .await?;

            current_identity = opt_result.new_identity.clone();

            // Save iteration identity (mode-aware path)
            let iter_path = match self.config.mode.target {
                crate::types::EvalMode::Soul => self.state_dir.iteration_soul_path(iteration),
                _ => self.state_dir.iteration_identity_path(iteration),
            };
            identity::write_identity(&iter_path, &current_identity)?;

            self.log(format!("Mutation: {}", opt_result.mutation_summary));
            let _ = self.event_tx.send(EvalEvent::OptimizationComplete {
                iteration,
                mutation: opt_result.mutation_summary.clone(),
            });

            // Append history
            let entry = HistoryEntry {
                iteration,
                average_score: avg_score,
                best_score: scores
                    .iter()
                    .map(|s| s.overall)
                    .fold(0.0_f64, f64::max),
                mutation_summary: opt_result.mutation_summary,
                tokens_used: self.client.budget().lock().await.consumed,
                timestamp: Utc::now(),
            };
            hist.push(entry.clone());
            history::append_history(&self.state_dir.history_path(), entry)?;
        }

        let _ = self.event_tx.send(EvalEvent::LoopComplete {
            reason: StopReason::MaxIterations.to_string(),
            best_score,
        });

        Ok(StopReason::MaxIterations)
    }
}
