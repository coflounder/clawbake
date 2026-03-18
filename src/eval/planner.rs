use crate::claude::client::ClaudeClient;
use crate::claude::models::Tier;
use crate::error::Result;
use crate::types::{EvalCase, PersonaSpec};
use serde_json;

pub async fn generate_eval_cases(
    client: &ClaudeClient,
    spec: &PersonaSpec,
    reference: &str,
    eval_count: usize,
) -> Result<Vec<EvalCase>> {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "cases": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string" },
                        "title": { "type": "string" },
                        "description": { "type": "string" },
                        "prompt": { "type": "string" },
                        "expected_behaviors": {
                            "type": "array",
                            "items": { "type": "string" }
                        },
                        "category": {
                            "type": "string",
                            "enum": ["core_task", "personality_probe", "edge_case", "guardrail_test", "tool_usage"]
                        }
                    },
                    "required": ["id", "title", "description", "prompt", "expected_behaviors", "category"]
                }
            }
        },
        "required": ["cases"]
    });

    let persona_summary = format!(
        "Name: {}\nRole: {}\nResponsibility: {}\nTraits: {}\nTools: {}\nGuardrails: {}",
        spec.name,
        spec.role,
        spec.responsibility,
        spec.personality_traits.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", "),
        spec.tools.iter().map(|t| t.name.clone()).collect::<Vec<_>>().join(", "),
        spec.guardrails.join("; "),
    );

    let prompt = format!(
        r#"Generate exactly {} evaluation test cases for an AI agent with the following persona:

{}

Reference material:
{}

Generate a diverse mix of categories: core_task, personality_probe, edge_case, guardrail_test, and tool_usage.
Each case should test a different aspect of the persona. The prompt field should be a realistic user message that would be sent to this agent.
Expected behaviors should be specific, observable things the agent should do or say."#,
        eval_count, persona_summary, reference
    );

    let response = client
        .build(Tier::Planner, &prompt)
        .with_json_schema(&schema.to_string())
        .execute()
        .await?;

    let wrapper = response.parse_json_result("Planner")?;

    let cases: Vec<EvalCase> = serde_json::from_value(
        wrapper
            .get("cases")
            .cloned()
            .unwrap_or(serde_json::Value::Array(vec![])),
    )
    .map_err(|e| {
        crate::error::ClawbakeError::Eval(format!("Failed to parse eval cases: {}", e))
    })?;

    Ok(cases)
}
