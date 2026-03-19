use crate::claude::client::ClaudeClient;
use crate::claude::models::Tier;
use crate::error::Result;
use crate::types::PersonaSpec;
use std::path::Path;

pub fn generate_identity(spec: &PersonaSpec) -> String {
    let mut doc = String::new();

    doc.push_str(&format!("# {}\n\n", spec.name));

    doc.push_str("## Role\n\n");
    doc.push_str(&format!("{}\n\n", spec.role));

    doc.push_str("## Responsibility\n\n");
    doc.push_str(&format!("{}\n\n", spec.responsibility));

    if !spec.personality_traits.is_empty() {
        doc.push_str("## Personality\n\n");
        for trait_ in &spec.personality_traits {
            doc.push_str(&format!("- {}\n", trait_));
        }
        doc.push('\n');
    }

    if !spec.tools.is_empty() {
        doc.push_str("## Tools\n\n");
        for tool in &spec.tools {
            doc.push_str(&format!("### {}\n\n", tool.name));
            doc.push_str(&format!("{}\n\n", tool.description));
        }
    }

    if !spec.guardrails.is_empty() {
        doc.push_str("## Guardrails\n\n");
        for rule in &spec.guardrails {
            doc.push_str(&format!("- {}\n", rule));
        }
        doc.push('\n');
    }

    doc
}

pub async fn bootstrap_identity(
    client: &ClaudeClient,
    spec: &PersonaSpec,
    reference: &str,
) -> Result<String> {
    let bare_identity = generate_identity(spec);

    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "identity": { "type": "string" }
        },
        "required": ["identity"]
    });

    let prompt = format!(
        r#"Expand this AI agent identity document. For each personality trait, add a brief behavioral definition (one line) explaining how this trait manifests in the agent's responses. Add a Communication Style section describing the agent's tone and formatting preferences. Keep the same markdown structure. Do not change the role, responsibility, or tools sections.

## Identity Document
{}

## Reference Material
{}"#,
        bare_identity, reference
    );

    let response = client
        .build(Tier::Planner, &prompt)
        .with_json_schema(&schema.to_string())
        .execute()
        .await?;

    match response.parse_json_result("Bootstrap") {
        Ok(parsed) => Ok(parsed["identity"]
            .as_str()
            .unwrap_or(&bare_identity)
            .to_string()),
        Err(_) => {
            // Model may have returned raw markdown instead of JSON
            let text = response.result_text();
            if text.len() > 50 && text.contains("##") {
                tracing::warn!("Bootstrap returned raw markdown instead of JSON; using as identity");
                Ok(text.to_string())
            } else {
                // Fall back to bare identity rather than failing the whole run
                tracing::warn!("Bootstrap failed to parse; falling back to bare identity");
                Ok(bare_identity)
            }
        }
    }
}

pub fn write_identity(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)?;
    Ok(())
}

pub fn read_identity(path: &Path) -> Result<String> {
    Ok(std::fs::read_to_string(path)?)
}
