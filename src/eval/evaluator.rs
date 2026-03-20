use crate::claude::client::ClaudeClient;
use crate::claude::models::Tier;
use crate::error::Result;
use crate::types::{EvalCase, EvalMode, EvalScore, ScoringWeights};
use crate::eval::runner::CaseResult;

pub fn build_evaluator_prompt(
    mode: &EvalMode,
    identity: &str,
    title: &str,
    description: &str,
    category: &str,
    expected_behaviors: &str,
    transcript: &str,
) -> String {
    match mode {
        EvalMode::Soul => format!(
            r#"Evaluate this AI agent's response against its SOUL document. Focus on identity fidelity, not task completion.

## SOUL Document
{}

## Test Case
Title: {}
Description: {}
Category: {}
Expected behaviors: {}

## Agent's Response
{}

Score each dimension from 0.0 to 1.0:
- persona_fidelity: Does the response embody the agent's identity, voice, and values as defined in the SOUL? Is the tone consistent? Would you recognize this agent across sessions?
- task_quality: Despite focusing on identity, does the response still accomplish something useful?
- efficiency: Is the response appropriately sized? Soul-driven agents may be more verbose to express personality — that's fine if it serves identity.

Weight persona_fidelity most heavily. This is a soul evaluation — identity coherence is the primary signal.

Provide a brief rationale explaining your scores."#,
            identity, title, description, category, expected_behaviors, transcript
        ),
        _ => format!(
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

Provide a brief rationale explaining your scores."#,
            identity, title, description, category, expected_behaviors, transcript
        ),
    }
}

pub async fn evaluate_case(
    client: &ClaudeClient,
    case: &EvalCase,
    result: &CaseResult,
    identity: &str,
    mode: &EvalMode,
) -> Result<EvalScore> {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "persona_fidelity": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
            "task_quality": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
            "efficiency": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
            "rationale": { "type": "string" }
        },
        "required": ["persona_fidelity", "task_quality", "efficiency", "rationale"]
    });

    let prompt = build_evaluator_prompt(
        mode,
        identity,
        &case.title,
        &case.description,
        &case.category.to_string(),
        &case.expected_behaviors.join(", "),
        &result.transcript,
    );

    let response = client
        .build(Tier::Evaluator, &prompt)
        .with_json_schema(&schema.to_string())
        .execute()
        .await?;

    let parsed = response.parse_json_result("Evaluator")?;

    let fidelity = parsed["persona_fidelity"].as_f64().unwrap_or(0.0);
    let quality = parsed["task_quality"].as_f64().unwrap_or(0.0);
    let efficiency = parsed["efficiency"].as_f64().unwrap_or(0.0);
    let weights = ScoringWeights::for_mode(mode);

    Ok(EvalScore {
        case_id: case.id.clone(),
        persona_fidelity: fidelity,
        task_quality: quality,
        efficiency,
        overall: EvalScore::compute_overall_weighted(fidelity, quality, efficiency, &weights),
        rationale: parsed["rationale"].as_str().unwrap_or("").to_string(),
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EvalMode;

    #[test]
    fn soul_mode_evaluator_prompt_emphasizes_identity() {
        let prompt = build_evaluator_prompt(
            &EvalMode::Soul,
            "identity doc",
            "test title",
            "test desc",
            "Identity Consistency",
            "should be consistent",
            "agent response here",
        );
        assert!(prompt.contains("SOUL"));
        assert!(prompt.contains("identity"));
        assert!(prompt.contains("voice"));
    }

    #[test]
    fn default_mode_evaluator_prompt_unchanged() {
        let prompt = build_evaluator_prompt(
            &EvalMode::Claude,
            "identity doc",
            "test title",
            "test desc",
            "Core Task",
            "expected",
            "response",
        );
        assert!(prompt.contains("persona_fidelity"));
        assert!(prompt.contains("task_quality"));
    }
}
