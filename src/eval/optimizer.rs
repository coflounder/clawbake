use crate::claude::client::ClaudeClient;
use crate::claude::models::Tier;
use crate::error::Result;
use crate::types::{EvalMode, EvalScore, HistoryEntry};

pub struct OptimizationResult {
    pub new_identity: String,
    pub mutation_summary: String,
}

pub fn build_optimizer_prompt(
    mode: &EvalMode,
    current_identity: &str,
    scores_summary: &str,
    history_summary: &str,
    reference: &str,
) -> String {
    match mode {
        EvalMode::Soul => format!(
            r#"You are a SOUL.md optimizer. Your job is to refine an AI agent's deep identity document — its values, voice, worldview, and behavioral principles.

## Current SOUL Document
{}

## Evaluation Scores
{}

## History of Previous Iterations
{}

## Reference Material
{}

Analyze the scores and rationales. Improve the SOUL document following these rules:
1. Mutations target prose, not structure. SOUL.md is a narrative document. Rewrite sections for clarity and specificity — do NOT add bullet points or restructure into lists.
2. Strengthen voice: If voice_preservation scores are low, make the personality description more vivid and distinctive.
3. Sharpen values: If value_conflict scores are low, make value hierarchies explicit ("thoroughness over speed").
4. Add boundary language: If boundary_holding scores are low, add clear statements about what this agent is NOT.
5. Avoid anti-patterns: Don't make the soul too vague ("be helpful") or too rigid ("always respond in exactly 3 paragraphs").
6. Keep the core identity intact — refine, don't reinvent.
7. The mutation_summary should be a one-line description of what you changed.

Return the complete updated SOUL document and a summary of mutations."#,
            current_identity, scores_summary, history_summary, reference
        ),
        _ => format!(
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
        ),
    }
}

pub async fn optimize_identity(
    client: &ClaudeClient,
    current_identity: &str,
    scores: &[EvalScore],
    history: &[HistoryEntry],
    reference: &str,
    mode: &EvalMode,
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

    let prompt = build_optimizer_prompt(mode, current_identity, &scores_summary, &history_summary, reference);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EvalMode;

    #[test]
    fn soul_mode_optimizer_prompt_targets_prose() {
        let prompt = build_optimizer_prompt(
            &EvalMode::Soul,
            "# Bot\n## Identity\nI am helpful",
            "scores summary",
            "history summary",
            "reference",
        );
        assert!(prompt.contains("prose"));
        assert!(prompt.contains("SOUL"));
        assert!(prompt.contains("voice"));
        assert!(prompt.contains("narrative"));
    }

    #[test]
    fn default_mode_optimizer_prompt_unchanged() {
        let prompt = build_optimizer_prompt(
            &EvalMode::Claude,
            "identity",
            "scores",
            "history",
            "reference",
        );
        assert!(prompt.contains("identity prompt optimizer"));
        assert!(prompt.contains("Preserve the markdown structure"));
    }
}
