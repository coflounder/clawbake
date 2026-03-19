use crate::claude::client::{ClaudeClient, ClaudeInvocation};
use crate::claude::models::{ClaudeResponse, Tier};
use crate::error::Result;
use crate::types::{EvalCase, EvalScore};
use crate::eval::runner::CaseResult;

fn eval_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "persona_fidelity": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
            "task_quality": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
            "efficiency": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
            "rationale": { "type": "string" },
            "summary": { "type": "string" }
        },
        "required": ["persona_fidelity", "task_quality", "efficiency", "rationale", "summary"]
    })
}

fn eval_prompt(
    identity: &str,
    case: &EvalCase,
    transcript: &str,
) -> String {
    format!(
        r#"Evaluate this AI agent's response against its identity and the test case.

## Identity (System Prompt)
{}

## Test Case
Title: {}
Description: {}
Category: {}
Expected behaviors: {}

## Agent's Response
{}

Score each dimension from 0.0 to 1.0:
- persona_fidelity: How well does the response match the defined personality, role, and guardrails?
- task_quality: How well does the response accomplish the task described in the test case?
- efficiency: How concise and focused is the response? Does it avoid unnecessary verbosity?

Provide a brief rationale explaining your scores.

Also provide a concise summary (under 200 words) of the transcript focusing on: decisions made, tools used, personality expression, any issues."#,
        identity,
        case.title,
        case.description,
        case.category,
        case.expected_behaviors.join(", "),
        transcript,
    )
}

/// Build a ClaudeInvocation for evaluation. This borrows &ClaudeClient but returns
/// an owned invocation that can be moved into a spawned task.
pub fn build_eval_invocation(
    client: &ClaudeClient,
    case: &EvalCase,
    result: &CaseResult,
    identity: &str,
) -> ClaudeInvocation {
    let schema = eval_schema();
    let prompt = eval_prompt(identity, case, &result.transcript);

    client
        .build(Tier::Evaluator, &prompt)
        .with_json_schema(&schema.to_string())
}

/// Parse a ClaudeResponse from an evaluation invocation into an EvalScore and summary.
pub fn parse_eval_response(
    response: &ClaudeResponse,
    case_id: &str,
) -> Result<(EvalScore, String)> {
    let parsed = response.parse_json_result("Evaluator")?;

    let fidelity = parsed["persona_fidelity"].as_f64().unwrap_or(0.0);
    let quality = parsed["task_quality"].as_f64().unwrap_or(0.0);
    let efficiency = parsed["efficiency"].as_f64().unwrap_or(0.0);
    let summary = parsed["summary"].as_str().unwrap_or("").to_string();

    Ok((
        EvalScore {
            case_id: case_id.to_string(),
            persona_fidelity: fidelity,
            task_quality: quality,
            efficiency,
            overall: EvalScore::compute_overall(fidelity, quality, efficiency),
            rationale: parsed["rationale"].as_str().unwrap_or("").to_string(),
        },
        summary,
    ))
}

/// Evaluate a case end-to-end (builds invocation, executes, parses).
/// Returns (EvalScore, summary).
pub async fn evaluate_case(
    client: &ClaudeClient,
    case: &EvalCase,
    result: &CaseResult,
    identity: &str,
) -> Result<(EvalScore, String)> {
    let invocation = build_eval_invocation(client, case, result, identity);
    let response = invocation.execute().await?;
    parse_eval_response(&response, &case.id)
}

pub async fn summarize_transcript(
    client: &ClaudeClient,
    transcript: &str,
    case_title: &str,
) -> Result<String> {
    let prompt = format!(
        r#"Condense this AI agent transcript for the test case "{}" into key moments only.
Keep it under 200 words. Focus on: decisions made, tools used, personality expression, any issues.

Transcript:
{}"#,
        case_title, transcript
    );

    let response = client
        .build(Tier::Evaluator, &prompt)
        .execute()
        .await?;

    Ok(response.result_text().to_string())
}
