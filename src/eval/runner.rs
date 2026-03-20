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

pub struct MultiSessionResult {
    pub case_id: String,
    pub session_transcripts: Vec<String>,
    pub session_count: usize,
    pub combined_transcript: String,
}

impl MultiSessionResult {
    pub fn new(case_id: String, session_transcripts: Vec<String>) -> Self {
        let session_count = session_transcripts.len();
        let combined = session_transcripts
            .iter()
            .enumerate()
            .map(|(i, t)| format!("--- Session {}/{} ---\n{}", i + 1, session_count, t))
            .collect::<Vec<_>>()
            .join("\n\n");
        Self {
            case_id,
            session_transcripts,
            session_count,
            combined_transcript: combined,
        }
    }
}

impl From<MultiSessionResult> for CaseResult {
    fn from(multi: MultiSessionResult) -> Self {
        CaseResult {
            case_id: multi.case_id,
            transcript: multi.combined_transcript,
            success: true,
        }
    }
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

/// Run each case N times with fresh context (no conversation carryover).
/// Used by soul mode to test cross-session identity consistency.
pub async fn run_cases_multi_session(
    client: &ClaudeClient,
    cases: &[EvalCase],
    identity: &str,
    sandbox: &SandboxEnvironment,
    allowed_tools: &[String],
    max_parallel: usize,
    max_turns: u32,
    session_count: usize,
    event_tx: &mpsc::UnboundedSender<EvalEvent>,
) -> Result<Vec<CaseResult>> {
    let semaphore = Arc::new(Semaphore::new(max_parallel));
    let mut tasks = Vec::new();

    for case in cases {
        let sem = semaphore.clone();
        let case_id = case.id.clone();
        let case_title = case.title.clone();
        let case_prompt = case.prompt.clone();
        let tx = event_tx.clone();
        let identity_owned = identity.to_string();
        let tools = allowed_tools.to_vec();
        let env_path = sandbox.env_path.clone();
        let work_dir = sandbox.work_dir.clone();

        // Build N invocations for this case, one per session
        let mut session_invocations = Vec::new();
        for _ in 0..session_count {
            session_invocations.push(
                client
                    .build(Tier::Persona, &case_prompt)
                    .with_system_prompt(&identity_owned)
                    .with_allowed_tools(tools.clone())
                    .with_max_turns(max_turns)
                    .with_env("PATH", &env_path)
                    .with_working_dir(work_dir.clone()),
            );
        }

        tasks.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();

            let _ = tx.send(EvalEvent::CaseStarted {
                case_id: case_id.clone(),
                title: format!("{} ({}x sessions)", case_title, session_count),
            });

            let mut session_transcripts = Vec::new();
            for (i, invocation) in session_invocations.into_iter().enumerate() {
                let _ = tx.send(EvalEvent::Log {
                    message: format!("Case {} session {}/{}", case_id, i + 1, session_count),
                });

                match invocation.execute().await {
                    Ok(response) => {
                        session_transcripts.push(response.result_text().to_string());
                    }
                    Err(e) => {
                        session_transcripts.push(format!("Error in session {}: {}", i + 1, e));
                    }
                }
            }

            let multi = MultiSessionResult::new(case_id.clone(), session_transcripts);

            let _ = tx.send(EvalEvent::Log {
                message: format!("Case {} all {} sessions complete", case_id, session_count),
            });

            CaseResult::from(multi)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multi_session_result_aggregates_transcripts() {
        let sessions = vec![
            "Session 1: I am helpful and friendly.".to_string(),
            "Session 2: I am helpful and kind.".to_string(),
            "Session 3: I am helpful and warm.".to_string(),
        ];
        let result = MultiSessionResult::new("case-1".to_string(), sessions);
        assert_eq!(result.session_count, 3);
        assert!(result.combined_transcript.contains("--- Session 1/3 ---"));
        assert!(result.combined_transcript.contains("--- Session 2/3 ---"));
        assert!(result.combined_transcript.contains("--- Session 3/3 ---"));
        assert!(result.combined_transcript.contains("I am helpful and friendly."));
    }

    #[test]
    fn case_result_from_multi_session() {
        let sessions = vec!["response 1".to_string(), "response 2".to_string()];
        let multi = MultiSessionResult::new("case-1".to_string(), sessions);
        let case_result: CaseResult = multi.into();
        assert_eq!(case_result.case_id, "case-1");
        assert!(case_result.success);
        assert!(case_result.transcript.contains("Session 1/2"));
    }
}
