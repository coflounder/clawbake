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
    optimization_base: &str,
    scores: &[EvalScore],
    transcripts: &[(String, String)],
    history: &[HistoryEntry],
    reference: &str,
    is_regression: bool,
    persona_model: &str,
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

    let regression_note = if is_regression {
        "\n\nIMPORTANT: The current iteration scored below the best. You are working from the best-scoring identity. Focus on targeted improvements that avoid the regression seen in recent iterations. The previous optimization added too many rules — consider simplifying instead.\n"
    } else {
        ""
    };

    let prompt = format!(
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
        identity = optimization_base,
        scores = scores_summary,
        transcripts = transcripts_summary,
        history = history_summary,
        reference = reference,
        regression = regression_note,
        model = persona_model,
    );

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
