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
    transcripts_summary: &str,
    history_summary: &str,
    reference: &str,
    is_regression: bool,
    persona_model: &str,
) -> String {
    let regression_note = if is_regression {
        "\n\nIMPORTANT: The current iteration scored below the best. You are working from the best-scoring identity. Focus on targeted improvements that avoid the regression seen in recent iterations. The previous optimization added too many rules — consider simplifying instead.\n"
    } else {
        ""
    };

    match mode {
        EvalMode::Claude | EvalMode::Agents => format!(
            r#"You are a project instruction optimizer. Your job is to improve a CLAUDE.md or AGENTS.md file based on how well an agent followed its instructions during evaluation.

## Current Project Instruction File
{identity}

## Evaluation Scores
{scores}

## Transcript Summaries
{transcripts}

## History of Previous Iterations
{history}

## Project Reference
{reference}
{regression}
Analyze the scores, rationales, and summaries. The key metric is convention_adherence — did the agent follow the stated instructions?

Rules:
1. **Mutations are structural**: Add instructions, remove instructions, reorder them, clarify ambiguous ones. This is NOT a prose document — it's a specification. Use clear directives.
2. **Ablation principle**: If a rule exists but convention_adherence scores don't improve when it's tested, the rule is weakly worded. Rewrite it to be unambiguous.
3. **Coverage gaps**: If instruction_coverage cases show the agent guessing, add explicit instructions for those situations.
4. **Conflict resolution**: If instruction_conflict scores are low, reorder rules so higher-priority rules appear first, and add explicit "when X conflicts with Y, prefer X" statements.
5. **Remove ineffective rules**: If a rule was added in a previous iteration and scores didn't improve, REMOVE it. Context window is precious.
6. **Length limit**: Keep the instruction file under 600 words. Every instruction must earn its place.
7. **Clarity over completeness**: One clear directive beats three hedged ones. The persona agent runs on "{model}" — prefer simple, unambiguous instructions.
8. The mutation_summary should be a one-line description of what changed and why.

Return the complete updated instruction file and a summary of mutations."#,
            identity = current_identity,
            scores = scores_summary,
            transcripts = transcripts_summary,
            history = history_summary,
            reference = reference,
            regression = regression_note,
            model = persona_model,
        ),
        EvalMode::Soul => format!(
            r#"You are a SOUL.md optimizer. Your job is to refine an AI agent's deep identity document — its values, voice, worldview, and behavioral principles.

## Current SOUL Document
{identity}

## Evaluation Scores
{scores}

## Transcript Summaries
{transcripts}

## History of Previous Iterations
{history}

## Reference Material
{reference}
{regression}
Analyze the scores, rationales, and transcript summaries. Improve the SOUL document following these rules:
1. Mutations target prose, not structure. SOUL.md is a narrative document. Rewrite sections for clarity and specificity — do NOT add bullet points or restructure into lists.
2. Strengthen voice: If voice_preservation scores are low, make the personality description more vivid and distinctive.
3. Sharpen values: If value_conflict scores are low, make value hierarchies explicit ("thoroughness over speed").
4. Add boundary language: If boundary_holding scores are low, add clear statements about what this agent is NOT.
5. Avoid anti-patterns: Don't make the soul too vague ("be helpful") or too rigid ("always respond in exactly 3 paragraphs").
6. Keep the core identity intact — refine, don't reinvent.
7. The mutation_summary should be a one-line description of what you changed and what you removed
8. If a rule was added in a previous iteration and scores did not improve or worsened, REMOVE it. Pruning ineffective rules is as important as adding new ones
9. Keep the total SOUL document under 800 words. If adding a section would exceed this, consolidate or remove less effective sections first
10. The persona agent runs on model tier "{model}". Smaller models need simpler, clearer instructions — prefer vivid, concrete descriptions over complex conditional rules

Return the complete updated SOUL document and a summary of mutations."#,
            identity = current_identity,
            scores = scores_summary,
            transcripts = transcripts_summary,
            history = history_summary,
            reference = reference,
            regression = regression_note,
            model = persona_model,
        ),
        _ => format!(
            r#"You are an identity prompt optimizer. Your job is to improve an AI agent's system prompt based on evaluation results.

## Current Identity
{identity}

## Evaluation Scores
{scores}

## Transcript Summaries
{transcripts}

## History of Previous Iterations
{history}

## Reference Material
{reference}
{regression}
Analyze the scores, rationales, and transcript summaries. Identify weaknesses and improve the identity document.

Rules:
1. Preserve the markdown structure (headings, lists)
2. Make targeted, specific changes — don't rewrite everything
3. Focus on the lowest-scoring dimensions
4. Keep the core role and personality intact
5. The mutation_summary should be a one-line description of what you changed and what you removed
6. If a rule was added in a previous iteration and scores did not improve or worsened, REMOVE it. Pruning ineffective rules is as important as adding new ones
7. Keep the total identity document under 800 words. If adding a rule would exceed this, remove or consolidate a less effective rule first
8. The persona agent runs on model tier "{model}". Smaller models need simpler, clearer instructions — avoid complex conditional rules, multi-step procedural instructions, or rules with many exceptions. Prefer one clear directive over three nuanced ones
9. If a case consistently scores below 0.5 across multiple iterations despite identity changes, the failure is likely a model capability limit, not an identity problem. Do not add more rules targeting that case — focus optimization budget on cases where identity changes can actually help

Return the complete updated identity document and a summary of mutations."#,
            identity = current_identity,
            scores = scores_summary,
            transcripts = transcripts_summary,
            history = history_summary,
            reference = reference,
            regression = regression_note,
            model = persona_model,
        ),
    }
}

pub async fn optimize_identity(
    client: &ClaudeClient,
    optimization_base: &str,
    scores: &[EvalScore],
    transcripts: &[(String, String)],
    history: &[HistoryEntry],
    reference: &str,
    mode: &EvalMode,
    is_regression: bool,
    persona_model: &str,
) -> Result<OptimizationResult> {
    let scores_summary = scores
        .iter()
        .map(|s| {
            if s.convention_adherence > 0.0 {
                format!(
                    "- {}: fidelity={:.2} quality={:.2} efficiency={:.2} convention={:.2} overall={:.2} | {}",
                    s.case_id, s.persona_fidelity, s.task_quality, s.efficiency, s.convention_adherence, s.overall, s.rationale
                )
            } else {
                format!(
                    "- {}: fidelity={:.2} quality={:.2} efficiency={:.2} overall={:.2} | {}",
                    s.case_id, s.persona_fidelity, s.task_quality, s.efficiency, s.overall, s.rationale
                )
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let transcripts_summary = if transcripts.is_empty() {
        "No transcript summaries available.".to_string()
    } else {
        transcripts
            .iter()
            .map(|(case_id, summary)| format!("### {}\n{}", case_id, summary))
            .collect::<Vec<_>>()
            .join("\n\n")
    };

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

    let prompt = build_optimizer_prompt(mode, optimization_base, &scores_summary, &transcripts_summary, &history_summary, reference, is_regression, persona_model);

    let response = client
        .build(Tier::Optimizer, &prompt)
        .with_json_schema(&schema.to_string())
        .execute()
        .await?;

    // The optimizer returns a JSON object with "identity" (multiline markdown) and
    // "mutation_summary". When structured_output works, this parses cleanly. When
    // it falls back to result text, the markdown inside the JSON string can cause
    // parse failures. Handle both cases.
    match response.parse_json_result("Optimizer") {
        Ok(parsed) => Ok(OptimizationResult {
            new_identity: parsed["identity"]
                .as_str()
                .unwrap_or(optimization_base)
                .to_string(),
            mutation_summary: parsed["mutation_summary"]
                .as_str()
                .unwrap_or("No changes")
                .to_string(),
        }),
        Err(_) => {
            // Fallback: the result text itself may be the identity markdown
            // (model ignored JSON schema and wrote markdown directly)
            let text = response.result_text();
            if text.len() > 50 && text.contains("##") {
                tracing::warn!("Optimizer returned raw markdown instead of JSON; using as identity");
                Ok(OptimizationResult {
                    new_identity: text.to_string(),
                    mutation_summary: "Raw markdown response — structured parse failed".to_string(),
                })
            } else {
                Err(crate::error::ClawbakeError::Eval(format!(
                    "Optimizer: failed to parse response and result text is not a valid identity. First 500 chars: {}",
                    &text[..text.len().min(500)]
                )))
            }
        }
    }
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
            "transcripts summary",
            "history summary",
            "reference",
            false,
            "haiku",
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
            "transcripts",
            "history",
            "reference",
            false,
            "sonnet",
        );
        assert!(prompt.contains("identity prompt optimizer"));
        assert!(prompt.contains("Preserve the markdown structure"));
    }
}
