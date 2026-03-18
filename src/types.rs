use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonalityTrait {
    Analytical,
    Creative,
    Empathetic,
    Direct,
    Methodical,
    Collaborative,
    Assertive,
    Patient,
    Curious,
    Concise,
    Thorough,
    Diplomatic,
    Humorous,
    Formal,
    Friendly,
    Skeptical,
    Encouraging,
    Precise,
    Adaptable,
    Proactive,
    Custom(String),
}

impl std::fmt::Display for PersonalityTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom(s) => write!(f, "{}", s),
            other => write!(f, "{:?}", other),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub stub_behavior: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaSpec {
    pub name: String,
    pub role: String,
    pub responsibility: String,
    pub personality_traits: Vec<PersonalityTrait>,
    pub tools: Vec<ToolSpec>,
    pub guardrails: Vec<String>,
}

impl Default for PersonaSpec {
    fn default() -> Self {
        Self {
            name: String::new(),
            role: String::new(),
            responsibility: String::new(),
            personality_traits: Vec::new(),
            tools: Vec::new(),
            guardrails: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalConfig {
    pub eval_count: usize,
    pub max_iterations: usize,
    pub max_budget_tokens: u64,
    pub max_parallel: usize,
    pub max_turns_per_case: u32,
    pub interactive: bool,
}

impl Default for EvalConfig {
    fn default() -> Self {
        Self {
            eval_count: 5,
            max_iterations: 10,
            max_budget_tokens: 1_000_000,
            max_parallel: 2,
            max_turns_per_case: 2,
            interactive: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub planner: String,
    pub optimizer: String,
    pub evaluator: String,
    pub persona: String,
    pub stub: String,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            planner: "sonnet".to_string(),
            optimizer: "sonnet".to_string(),
            evaluator: "haiku".to_string(),
            persona: "haiku".to_string(),
            stub: "haiku".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub workspace_files: Vec<String>,
    pub format: OutputFormat,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            workspace_files: vec!["IDENTITY.md".to_string()],
            format: OutputFormat::Single,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Single,
    Multi,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EvalCategory {
    CoreTask,
    PersonalityProbe,
    EdgeCase,
    GuardrailTest,
    ToolUsage,
}

impl std::fmt::Display for EvalCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CoreTask => write!(f, "Core Task"),
            Self::PersonalityProbe => write!(f, "Personality Probe"),
            Self::EdgeCase => write!(f, "Edge Case"),
            Self::GuardrailTest => write!(f, "Guardrail Test"),
            Self::ToolUsage => write!(f, "Tool Usage"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalCase {
    pub id: String,
    pub title: String,
    pub description: String,
    pub prompt: String,
    pub expected_behaviors: Vec<String>,
    pub category: EvalCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalScore {
    pub case_id: String,
    pub persona_fidelity: f64,
    pub task_quality: f64,
    pub efficiency: f64,
    pub overall: f64,
    pub rationale: String,
}

impl EvalScore {
    pub fn compute_overall(fidelity: f64, quality: f64, efficiency: f64) -> f64 {
        fidelity * 0.4 + quality * 0.4 + efficiency * 0.2
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationResult {
    pub iteration: usize,
    pub scores: Vec<EvalScore>,
    pub average_score: f64,
    pub mutation_summary: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    pub limit: u64,
    pub consumed: u64,
    pub by_tier: HashMap<String, u64>,
}

impl TokenBudget {
    pub fn new(limit: u64) -> Self {
        Self {
            limit,
            consumed: 0,
            by_tier: HashMap::new(),
        }
    }

    pub fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.consumed)
    }

    pub fn exhausted(&self) -> bool {
        self.consumed >= self.limit
    }

    pub fn record(&mut self, tier: &str, tokens: u64) {
        self.consumed += tokens;
        *self.by_tier.entry(tier.to_string()).or_insert(0) += tokens;
    }

    pub fn usage_fraction(&self) -> f64 {
        if self.limit == 0 {
            return 1.0;
        }
        self.consumed as f64 / self.limit as f64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub iteration: usize,
    pub average_score: f64,
    pub best_score: f64,
    pub mutation_summary: String,
    pub tokens_used: u64,
    pub timestamp: DateTime<Utc>,
}
