use crate::tui::wizard::WizardStep;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct AutofillContext {
    pub step: WizardStep,
    pub role: String,
    pub responsibility: String,
    pub selected_traits: Vec<String>,
    pub selected_tools: Vec<String>,
    pub name: String,
}

#[derive(Debug)]
pub enum AutofillResult {
    Role {
        role: Option<String>,
        responsibility: Option<String>,
    },
    Personality {
        traits: Vec<String>,
    },
    Tools {
        tools: Vec<String>,
    },
    Name {
        name: String,
    },
    Config {
        guardrails: Vec<String>,
    },
}

const AVAILABLE_TRAITS: &[&str] = &[
    "Analytical",
    "Creative",
    "Empathetic",
    "Direct",
    "Methodical",
    "Collaborative",
    "Assertive",
    "Patient",
    "Curious",
    "Concise",
    "Thorough",
    "Diplomatic",
    "Humorous",
    "Formal",
    "Friendly",
    "Skeptical",
    "Encouraging",
    "Precise",
    "Adaptable",
    "Proactive",
];

const AVAILABLE_TOOLS: &[&str] = &[
    "Bash", "Read", "Write", "Edit", "Glob", "Grep", "WebFetch", "WebSearch", "NotebookEdit",
    "Agent",
];

fn build_prompt(ctx: &AutofillContext) -> String {
    match ctx.step {
        WizardStep::Role => {
            if !ctx.role.is_empty() && ctx.responsibility.is_empty() {
                // Role already provided — only suggest responsibility
                format!(
                    "Given an AI agent with the role '{}', suggest a fitting core responsibility \
                     (1-2 sentences describing what the agent does). \
                     Respond with ONLY a JSON object, no markdown fences: \
                     {{\"responsibility\": \"<1-2 sentence responsibility>\"}}",
                    ctx.role
                )
            } else if !ctx.role.is_empty() && !ctx.responsibility.is_empty() {
                // Both filled — nothing to suggest
                return String::new();
            } else {
                "Suggest a creative and specific AI agent role and its core responsibility. \
                 Be imaginative but practical. \
                 Respond with ONLY a JSON object, no markdown fences: \
                 {\"role\": \"<specific role>\", \"responsibility\": \"<1-2 sentence responsibility>\"}"
                    .to_string()
            }
        }
        WizardStep::Personality => {
            let traits_list = AVAILABLE_TRAITS.join(", ");
            format!(
                "Given an AI agent with role '{}' and responsibility '{}', \
                 suggest 3-5 personality traits from this list that best fit: [{}]. \
                 Respond with ONLY a JSON object, no markdown fences: \
                 {{\"traits\": [\"Trait1\", \"Trait2\", ...]}}",
                ctx.role, ctx.responsibility, traits_list
            )
        }
        WizardStep::Tools => {
            let tools_list = AVAILABLE_TOOLS.join(", ");
            format!(
                "Given an AI agent with role '{}' and responsibility '{}', \
                 which tools from this list would be most useful: [{}]. \
                 Respond with ONLY a JSON object, no markdown fences: \
                 {{\"tools\": [\"Tool1\", \"Tool2\", ...]}}",
                ctx.role, ctx.responsibility, tools_list
            )
        }
        WizardStep::Name => {
            format!(
                "Suggest a creative, memorable single-word or short name for an AI agent \
                 with role '{}', responsibility '{}', traits [{}], tools [{}]. \
                 Respond with ONLY a JSON object, no markdown fences: \
                 {{\"name\": \"<name>\"}}",
                ctx.role,
                ctx.responsibility,
                ctx.selected_traits.join(", "),
                ctx.selected_tools.join(", ")
            )
        }
        WizardStep::Config => {
            format!(
                "Suggest 2-3 important behavioral guardrails for an AI agent named '{}' \
                 with role '{}' and responsibility '{}'. \
                 Respond with ONLY a JSON object, no markdown fences: \
                 {{\"guardrails\": [\"Rule 1\", \"Rule 2\"]}}",
                ctx.name, ctx.role, ctx.responsibility
            )
        }
        WizardStep::Review => String::new(),
    }
}

fn extract_json(text: &str) -> Option<&str> {
    let text = text.trim();
    if text.starts_with('{') {
        if let Some(end) = text.rfind('}') {
            return Some(&text[..=end]);
        }
    }
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            return Some(&text[start..=end]);
        }
    }
    None
}

pub async fn run_autofill(ctx: AutofillContext) -> Option<AutofillResult> {
    let prompt = build_prompt(&ctx);
    if prompt.is_empty() {
        return None;
    }

    let mut child = Command::new("claude")
        .arg("-p")
        .arg("--model")
        .arg("haiku")
        .arg("--max-turns")
        .arg("1")
        .arg("--output-format")
        .arg("text")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(prompt.as_bytes()).await.ok()?;
        drop(stdin);
    }

    let output = child.wait_with_output().await.ok()?;
    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let json_str = extract_json(&text)?;
    let json: serde_json::Value = serde_json::from_str(json_str).ok()?;

    match ctx.step {
        WizardStep::Role => {
            let role = json
                .get("role")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let responsibility = json
                .get("responsibility")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            Some(AutofillResult::Role {
                role,
                responsibility,
            })
        }
        WizardStep::Personality => {
            let arr = json.get("traits")?.as_array()?;
            let traits: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .filter(|t| AVAILABLE_TRAITS.contains(&t.as_str()))
                .collect();
            Some(AutofillResult::Personality { traits })
        }
        WizardStep::Tools => {
            let arr = json.get("tools")?.as_array()?;
            let tools: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .filter(|t| AVAILABLE_TOOLS.contains(&t.as_str()))
                .collect();
            Some(AutofillResult::Tools { tools })
        }
        WizardStep::Name => {
            let name = json.get("name")?.as_str()?.to_string();
            Some(AutofillResult::Name { name })
        }
        WizardStep::Config => {
            let arr = json.get("guardrails")?.as_array()?;
            let guardrails: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            Some(AutofillResult::Config { guardrails })
        }
        WizardStep::Review => None,
    }
}
