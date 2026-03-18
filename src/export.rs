use crate::config::AppConfig;
use crate::error::Result;
use crate::io::state::StateDir;
use std::path::Path;

pub fn export_identity(
    state_dir: &StateDir,
    config: &AppConfig,
    output_dir: &Path,
) -> Result<Vec<String>> {
    let identity = std::fs::read_to_string(state_dir.best_identity_path()).map_err(|_| {
        crate::error::ClawbakeError::Export(
            "No best identity found. Run the eval loop first.".to_string(),
        )
    })?;

    std::fs::create_dir_all(output_dir)?;

    let mut written = Vec::new();

    match config.output.format {
        crate::types::OutputFormat::Single => {
            let filename = config
                .output
                .workspace_files
                .first()
                .map(|s| s.as_str())
                .unwrap_or("IDENTITY.md");
            let path = output_dir.join(filename);
            std::fs::write(&path, &identity)?;
            written.push(path.display().to_string());
        }
        crate::types::OutputFormat::Multi => {
            // Split identity by ## sections into separate files
            let sections = split_identity_sections(&identity);
            for (name, content) in sections {
                let filename = format!("{}.md", name.to_lowercase().replace(' ', "-"));
                let path = output_dir.join(&filename);
                std::fs::write(&path, content)?;
                written.push(path.display().to_string());
            }
        }
    }

    Ok(written)
}

fn split_identity_sections(identity: &str) -> Vec<(String, String)> {
    let mut sections = Vec::new();
    let mut current_name = String::from("header");
    let mut current_content = String::new();

    for line in identity.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            if !current_content.trim().is_empty() {
                sections.push((current_name, current_content));
            }
            current_name = heading.trim().to_string();
            current_content = String::new();
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    if !current_content.trim().is_empty() {
        sections.push((current_name, current_content));
    }

    sections
}
