use crate::claude::client::ClaudeClient;
use crate::claude::models::Tier;
use crate::error::Result;
use crate::types::{AblationAction, AblationResult};

/// Parse a project instruction file into discrete instructions for ablation testing.
/// Uses an LLM call to segment the file into individual directives.
pub async fn parse_instructions(client: &ClaudeClient, content: &str) -> Result<Vec<String>> {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "instructions": {
                "type": "array",
                "items": { "type": "string" }
            }
        },
        "required": ["instructions"]
    });

    let prompt = format!(
        r#"Parse this project instruction file into a list of discrete, individual instructions.

Each item in the list should be a single directive — one rule, one convention, one tool preference, or one workflow step.
If an instruction has multiple sub-points, split them into separate items.
Preserve the original wording of each instruction exactly.
Ignore section headings and comments — only extract actionable directives.

Project instruction file:
{}

Return a JSON object with an "instructions" array."#,
        content
    );

    let response = client
        .build(Tier::Planner, &prompt)
        .with_json_schema(&schema.to_string())
        .execute()
        .await?;

    let parsed = response.parse_json_result("AblationParser")?;
    let instructions: Vec<String> = serde_json::from_value(
        parsed
            .get("instructions")
            .cloned()
            .unwrap_or(serde_json::Value::Array(vec![])),
    )
    .map_err(|e| crate::error::ClawbakeError::Eval(format!("Failed to parse instructions: {}", e)))?;

    Ok(instructions)
}

/// Remove one instruction from a project instruction file content string.
/// Returns the modified content with that instruction removed.
fn remove_instruction(content: &str, instruction: &str) -> String {
    // Try exact line match first
    let lines: Vec<&str> = content.lines().collect();
    let filtered: Vec<&str> = lines
        .iter()
        .filter(|line| {
            let trimmed = line.trim().trim_start_matches('-').trim_start_matches('*').trim();
            !trimmed.eq_ignore_ascii_case(instruction.trim())
        })
        .copied()
        .collect();

    // If nothing was removed, try removing by substring match
    if filtered.len() == lines.len() {
        let filtered2: Vec<&str> = lines
            .iter()
            .filter(|line| !line.contains(instruction.trim()))
            .copied()
            .collect();
        return filtered2.join("\n");
    }

    filtered.join("\n")
}

/// Classify an ablation delta into a recommendation.
fn classify_delta(delta: f64) -> AblationAction {
    if delta < -0.05 {
        // Score dropped significantly after removal — instruction is load-bearing
        AblationAction::Keep
    } else if delta > 0.03 {
        // Score improved after removal — instruction was net-negative
        AblationAction::Rewrite
    } else if delta < -0.01 {
        // Small drop — instruction helps but is weakly expressed
        AblationAction::Strengthen
    } else {
        // No measurable impact — candidate for removal
        AblationAction::Remove
    }
}

/// Run ablation testing: for each discrete instruction in the file, remove it,
/// run a quick eval pass, and compare to the baseline score.
///
/// Returns a list of AblationResults sorted by delta (most impactful first).
///
/// Note: This is intentionally a lightweight scoring function — it uses a fast
/// LLM call to estimate the impact of removing each instruction rather than
/// running the full eval suite. Full eval runs are too expensive for ablation.
pub async fn run_ablation(
    client: &ClaudeClient,
    instruction_file: &str,
    baseline_score: f64,
    sample_prompts: &[String],
) -> Result<Vec<AblationResult>> {
    let instructions = parse_instructions(client, instruction_file).await?;

    if instructions.is_empty() {
        tracing::warn!("Ablation: no discrete instructions found in file");
        return Ok(vec![]);
    }

    tracing::info!(
        "Ablation: testing {} instructions against {} sample prompts",
        instructions.len(),
        sample_prompts.len()
    );

    let mut results = Vec::new();

    for instruction in &instructions {
        let ablated_content = remove_instruction(instruction_file, instruction);

        // Quick score estimate: ask the evaluator LLM how much this removal matters
        let ablated_score = estimate_ablated_score(
            client,
            &ablated_content,
            instruction,
            sample_prompts,
        ).await.unwrap_or(baseline_score);

        let delta = ablated_score - baseline_score;
        let recommendation = classify_delta(delta);

        results.push(AblationResult {
            removed_instruction: instruction.clone(),
            baseline_score,
            ablated_score,
            delta,
            recommendation,
        });
    }

    // Sort by delta ascending (most load-bearing instructions first, i.e. biggest drop)
    results.sort_by(|a, b| a.delta.partial_cmp(&b.delta).unwrap_or(std::cmp::Ordering::Equal));

    Ok(results)
}

/// Estimate the impact of removing an instruction by asking the LLM to predict
/// how well an agent would follow the remaining instructions on a sample of prompts.
async fn estimate_ablated_score(
    client: &ClaudeClient,
    ablated_content: &str,
    removed_instruction: &str,
    sample_prompts: &[String],
) -> Result<f64> {
    if sample_prompts.is_empty() {
        return Ok(0.5);
    }

    let samples = sample_prompts
        .iter()
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .join("\n- ");

    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "predicted_score": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
            "reasoning": { "type": "string" }
        },
        "required": ["predicted_score", "reasoning"]
    });

    let prompt = format!(
        r#"You are evaluating the impact of removing an instruction from a project instruction file.

## Original instruction removed:
{}

## Remaining instruction file (after removal):
{}

## Sample tasks an agent would be asked to perform:
- {}

Predict: if an agent uses the remaining instruction file (without the removed instruction) to handle the sample tasks,
what convention_adherence score would it get? (0.0 = ignores instructions entirely, 1.0 = perfectly follows all instructions)

Consider: does the removed instruction cover any of the sample tasks? If yes, the score will drop. If no, the score stays the same."#,
        removed_instruction, ablated_content, samples
    );

    let response = client
        .build(Tier::Evaluator, &prompt)
        .with_json_schema(&schema.to_string())
        .execute()
        .await?;

    let parsed = response.parse_json_result("AblationEstimator")?;
    Ok(parsed["predicted_score"].as_f64().unwrap_or(0.5))
}

/// Format ablation results into a readable summary for optimizer prompts.
pub fn format_ablation_summary(results: &[AblationResult]) -> String {
    if results.is_empty() {
        return "No ablation results available.".to_string();
    }

    let lines: Vec<String> = results
        .iter()
        .map(|r| {
            format!(
                "- [{}] delta={:+.3} | \"{}\"",
                r.recommendation,
                r.delta,
                if r.removed_instruction.len() > 80 {
                    format!("{}...", &r.removed_instruction[..80])
                } else {
                    r.removed_instruction.clone()
                }
            )
        })
        .collect();

    format!(
        "## Ablation Results ({} instructions tested)\n{}",
        results.len(),
        lines.join("\n")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_delta_keep() {
        assert_eq!(classify_delta(-0.10), AblationAction::Keep);
        assert_eq!(classify_delta(-0.06), AblationAction::Keep);
    }

    #[test]
    fn classify_delta_remove() {
        assert_eq!(classify_delta(0.00), AblationAction::Remove);
        assert_eq!(classify_delta(0.01), AblationAction::Remove);
    }

    #[test]
    fn classify_delta_strengthen() {
        assert_eq!(classify_delta(-0.02), AblationAction::Strengthen);
        assert_eq!(classify_delta(-0.04), AblationAction::Strengthen);
    }

    #[test]
    fn classify_delta_rewrite() {
        assert_eq!(classify_delta(0.05), AblationAction::Rewrite);
        assert_eq!(classify_delta(0.10), AblationAction::Rewrite);
    }

    #[test]
    fn remove_instruction_bullet_match() {
        let content = "# Instructions\n\n- Use ripgrep instead of grep\n- Run tests before committing\n- Never push to main";
        let result = remove_instruction(content, "Use ripgrep instead of grep");
        assert!(!result.contains("ripgrep"));
        assert!(result.contains("Run tests before committing"));
        assert!(result.contains("Never push to main"));
    }

    #[test]
    fn remove_instruction_no_match_unchanged() {
        let content = "- Use ripgrep\n- Run tests";
        let result = remove_instruction(content, "nonexistent instruction");
        assert_eq!(result, content);
    }

    #[test]
    fn format_ablation_summary_empty() {
        let summary = format_ablation_summary(&[]);
        assert!(summary.contains("No ablation"));
    }

    #[test]
    fn format_ablation_summary_with_results() {
        let results = vec![
            AblationResult {
                removed_instruction: "Use ripgrep instead of grep".to_string(),
                baseline_score: 0.80,
                ablated_score: 0.65,
                delta: -0.15,
                recommendation: AblationAction::Keep,
            },
            AblationResult {
                removed_instruction: "Add a newline at the end of files".to_string(),
                baseline_score: 0.80,
                ablated_score: 0.80,
                delta: 0.00,
                recommendation: AblationAction::Remove,
            },
        ];
        let summary = format_ablation_summary(&results);
        assert!(summary.contains("Keep"));
        assert!(summary.contains("Remove"));
        assert!(summary.contains("ripgrep"));
    }
}
