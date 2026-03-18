use crate::claude::client::ClaudeClient;
use crate::claude::models::Tier;
use crate::error::Result;
use crate::eval::loop_runner::EvalEvent;
use crate::sandbox::environment::SandboxEnvironment;
use crate::types::EvalCase;
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};

pub struct CaseResult {
    pub case_id: String,
    pub transcript: String,
    pub success: bool,
}

pub async fn run_cases(
    client: &ClaudeClient,
    cases: &[EvalCase],
    identity: &str,
    sandbox: &SandboxEnvironment,
    allowed_tools: &[String],
    max_parallel: usize,
    max_turns: u32,
    event_tx: &mpsc::UnboundedSender<EvalEvent>,
) -> Result<Vec<CaseResult>> {
    let semaphore = Arc::new(Semaphore::new(max_parallel));

    // Build all invocations upfront (borrows client), then execute in parallel
    let mut tasks = Vec::new();

    for case in cases {
        let invocation = client
            .build(Tier::Persona, &case.prompt)
            .with_system_prompt(identity)
            .with_allowed_tools(allowed_tools.to_vec())
            .with_max_turns(max_turns)
            .with_env("PATH", &sandbox.env_path)
            .with_working_dir(sandbox.work_dir.clone());

        let sem = semaphore.clone();
        let case_id = case.id.clone();
        let case_title = case.title.clone();
        let tx = event_tx.clone();

        tasks.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();

            let _ = tx.send(EvalEvent::CaseStarted {
                case_id: case_id.clone(),
                title: case_title.clone(),
            });
            let _ = tx.send(EvalEvent::Log {
                message: format!("Running case: {}", case_title),
            });

            let result = match invocation.execute().await {
                Ok(response) => CaseResult {
                    case_id: case_id.clone(),
                    transcript: response.result_text().to_string(),
                    success: !response.is_error,
                },
                Err(e) => {
                    let _ = tx.send(EvalEvent::Log {
                        message: format!("Case {} failed: {}", case_id, e),
                    });
                    CaseResult {
                        case_id: case_id.clone(),
                        transcript: format!("Error: {}", e),
                        success: false,
                    }
                }
            };

            let _ = tx.send(EvalEvent::Log {
                message: format!(
                    "Case {} complete ({})",
                    case_id,
                    if result.success { "ok" } else { "failed" }
                ),
            });

            result
        }));
    }

    let mut results = Vec::new();
    for handle in tasks {
        match handle.await {
            Ok(result) => results.push(result),
            Err(e) => {
                results.push(CaseResult {
                    case_id: "unknown".to_string(),
                    transcript: format!("Task join error: {}", e),
                    success: false,
                });
            }
        }
    }

    Ok(results)
}
