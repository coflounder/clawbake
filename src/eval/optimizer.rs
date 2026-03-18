use crate::claude::client::ClaudeClient;
use crate::claude::models::Tier;
use crate::error::Result;
use crate::types::{EvalScore, HistoryEntry};

pub struct OptimizationResult {
    pub new_identity: String,
    pub mutation_summary: String,
}

pub async fn optimize_identity(
    client: &ClaudeClient,
    current_identity: &str,
    scores: &[EvalScore],
    history: &[HistoryEntry],
    reference: &str,
) -> Result<OptimizationResult> {
    let scores_summary = scores
        .iter()
        .map(|s| {
            format!(
                "- {}: fidelity={:.2} quality={:.2} efficiency={:.2} overall={:.2} | {}",
                s.case_id, s.persona_fidelity, s.task_quality, s.efficiency, s.overall, s.rationale
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let history_summary = if history.is_empty() {
        "No prior iterations.".to_string()
    } else {
        history
            .iter()
            .map(|h| {
                format!(
                    "- Iter {}: avg={:.2} best={:.2} | {}",
                    h.iteration, h.average_score, h.best_score, h.mutation_summary
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "identity": { "type": "string" },
            "mutation_summary": { "type": "string" }
        },
        "required": ["identity", "mutation_summary"]
    });

    let prompt = format!(
        r#"You are an identity prompt optimizer. Your job is to improve an AI agent's system prompt based on evaluation results.

## Current Identity
{}

## Evaluation Scores
{}

## History of Previous Iterations
{}

## Reference Material
{}

Analyze the scores and rationales. Identify weaknesses and improve the identity document.
Rules:
1. Preserve the markdown structure (headings, lists)
2. Make targeted, specific changes — don't rewrite everything
3. Focus on the lowest-scoring dimensions
4. Keep the core role and personality intact
5. The mutation_summary should be a one-line description of what you changed

Return the complete updated identity document and a summary of mutations."#,
        current_identity, scores_summary, history_summary, reference
    );

    let response = client
        .build(Tier::Optimizer, &prompt)
        .with_json_schema(&schema.to_string())
        .execute()
        .await?;

    let parsed = response.parse_json_result("Optimizer")?;

    Ok(OptimizationResult {
        new_identity: parsed["identity"].as_str().unwrap_or(current_identity).to_string(),
        mutation_summary: parsed["mutation_summary"]
            .as_str()
            .unwrap_or("No changes")
            .to_string(),
    })
}
