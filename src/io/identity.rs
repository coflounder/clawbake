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

/// Generate a SOUL.md document — identity and values only, no tools or task instructions.
pub fn generate_soul(spec: &PersonaSpec) -> String {
    let mut doc = String::new();

    doc.push_str(&format!("# {}\n\n", spec.name));

    doc.push_str("## Identity\n\n");
    doc.push_str(&format!("{}\n\n", spec.role));

    doc.push_str("## Purpose\n\n");
    doc.push_str(&format!("{}\n\n", spec.responsibility));

    if !spec.personality_traits.is_empty() {
        doc.push_str("## Voice & Personality\n\n");
        for trait_ in &spec.personality_traits {
            doc.push_str(&format!("- {}\n", trait_));
        }
        doc.push('\n');
    }

    if !spec.guardrails.is_empty() {
        doc.push_str("## Principles\n\n");
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

/// Detect the project instruction file inside a project directory.
/// Checks in order: CLAUDE.md, AGENTS.md, .claude/CLAUDE.md, .claude/AGENTS.md
/// Returns the content and the canonical filename detected.
pub fn detect_project_instruction_file(project_dir: &Path) -> Option<(String, String)> {
    let candidates = [
        "CLAUDE.md",
        "AGENTS.md",
        ".claude/CLAUDE.md",
        ".claude/AGENTS.md",
    ];
    for candidate in &candidates {
        let path = project_dir.join(candidate);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("CLAUDE.md")
                    .to_string();
                return Some((content, filename));
            }
        }
    }
    None
}

/// Generate a minimal CLAUDE.md scaffold when no project instruction file exists.
pub fn scaffold_claude_md(spec: &crate::types::PersonaSpec) -> String {
    let mut doc = String::new();
    doc.push_str("# Project Instructions\n\n");
    doc.push_str(&format!("This project is worked on by {}.\n\n", spec.name));
    doc.push_str("## Conventions\n\n");
    doc.push_str("- Follow existing code style and patterns\n");
    doc.push_str("- Write clear, descriptive commit messages\n");
    doc.push_str("- Run tests before committing\n\n");
    if !spec.tools.is_empty() {
        doc.push_str("## Preferred Tools\n\n");
        for tool in &spec.tools {
            doc.push_str(&format!("- Use `{}` for {}\n", tool.name, tool.description));
        }
        doc.push('\n');
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn read_soul_md() {
        let dir = TempDir::new().unwrap();
        let soul_path = dir.path().join("SOUL.md");
        std::fs::write(&soul_path, "# My Agent\n\nI am helpful.").unwrap();
        let content = read_identity(&soul_path).unwrap();
        assert!(content.contains("I am helpful"));
    }

    #[test]
    fn generate_soul_document_from_spec() {
        let spec = PersonaSpec {
            name: "TestBot".to_string(),
            role: "assistant".to_string(),
            responsibility: "helping users".to_string(),
            personality_traits: vec![
                crate::types::PersonalityTrait::Friendly,
                crate::types::PersonalityTrait::Thorough,
            ],
            tools: vec![],
            guardrails: vec!["Never lie".to_string()],
        };
        let soul = generate_soul(&spec);
        assert!(soul.contains("# TestBot"));
        assert!(soul.contains("Friendly"));
        assert!(soul.contains("Thorough"));
        assert!(!soul.contains("## Tools"));
        assert!(soul.contains("Never lie"));
    }

    #[test]
    fn generate_identity_unchanged() {
        let spec = PersonaSpec {
            name: "Old".to_string(),
            role: "r".to_string(),
            responsibility: "resp".to_string(),
            personality_traits: vec![],
            tools: vec![crate::types::ToolSpec {
                name: "Bash".to_string(),
                description: "run commands".to_string(),
                stub_behavior: None,
            }],
            guardrails: vec![],
        };
        let doc = generate_identity(&spec);
        assert!(doc.contains("## Tools"));
        assert!(doc.contains("### Bash"));
    }
}
