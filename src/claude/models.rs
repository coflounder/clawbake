use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tier {
    Planner,
    Optimizer,
    Evaluator,
    Persona,
    Stub,
}

impl Tier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planner => "planner",
            Self::Optimizer => "optimizer",
            Self::Evaluator => "evaluator",
            Self::Persona => "persona",
            Self::Stub => "stub",
        }
    }
}

impl fmt::Display for Tier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeUsage {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
}

impl ClaudeUsage {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeResponse {
    pub result: Option<String>,
    #[serde(default)]
    pub structured_output: Option<serde_json::Value>,
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub usage: Option<ClaudeUsage>,
    #[serde(default)]
    pub num_turns: u32,
    #[serde(default)]
    pub duration_ms: u64,
    #[serde(default)]
    pub total_cost_usd: f64,
    #[serde(default)]
    pub is_error: bool,
}

impl ClaudeResponse {
    pub fn total_tokens(&self) -> u64 {
        self.usage.as_ref().map_or(0, |u| u.total_tokens())
    }

    pub fn result_text(&self) -> &str {
        self.result.as_deref().unwrap_or("")
    }

    /// Extract JSON from the response. Prefers `structured_output` (set by --json-schema),
    /// falls back to parsing `result` text (stripping markdown fences if present).
    pub fn parse_json_result(&self, context: &str) -> crate::error::Result<serde_json::Value> {
        if self.is_error {
            return Err(crate::error::ClawbakeError::Claude(format!(
                "{}: claude returned an error: {}",
                context,
                self.result_text()
            )));
        }

        // Prefer structured_output (populated by --json-schema)
        if let Some(ref structured) = self.structured_output {
            return Ok(structured.clone());
        }

        // Fall back to parsing result text
        let text = self.result_text();
        if text.is_empty() {
            return Err(crate::error::ClawbakeError::Claude(format!(
                "{}: empty result and no structured_output. Check that claude CLI is authenticated and the model is available.",
                context,
            )));
        }

        let json_text = text.trim();
        let json_text = json_text
            .strip_prefix("```json")
            .or_else(|| json_text.strip_prefix("```"))
            .unwrap_or(json_text);
        let json_text = json_text.strip_suffix("```").unwrap_or(json_text).trim();

        serde_json::from_str(json_text).map_err(|e| {
            crate::error::ClawbakeError::Eval(format!(
                "{}: failed to parse JSON: {}. Raw (first 500 chars): {}",
                context,
                e,
                &text[..text.len().min(500)]
            ))
        })
    }
}
