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
