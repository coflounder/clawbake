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
#[serde(default)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub stub_behavior: Option<String>,
}

impl Default for ToolSpec {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            stub_behavior: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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
#[serde(default)]
pub struct EvalConfig {
    pub eval_count: usize,
    pub max_iterations: usize,
    pub max_budget_tokens: u64,
    pub max_parallel: usize,
    pub max_turns_per_case: u32,
    pub interactive: bool,
    pub regen_interval: usize,
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
            regen_interval: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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
            evaluator: "sonnet".to_string(),
            persona: "haiku".to_string(),
            stub: "haiku".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Single,
    Multi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EvalMode {
    Soul,
    Claude,
    Agents,
    Memory,
    Skills,
}

impl Default for EvalMode {
    fn default() -> Self {
        Self::Soul
    }
}

impl std::fmt::Display for EvalMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Soul => write!(f, "soul"),
            Self::Claude => write!(f, "claude"),
            Self::Agents => write!(f, "agents"),
            Self::Memory => write!(f, "memory"),
            Self::Skills => write!(f, "skills"),
        }
    }
}

impl std::str::FromStr for EvalMode {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "soul" => Ok(Self::Soul),
            "claude" => Ok(Self::Claude),
            "agents" => Ok(Self::Agents),
            "memory" => Ok(Self::Memory),
            "skills" => Ok(Self::Skills),
            other => Err(format!("Unknown eval mode: '{}'", other)),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeldContext {
    pub claude_md: Option<std::path::PathBuf>,
    pub agents_md: Option<std::path::PathBuf>,
    pub memory_md: Option<std::path::PathBuf>,
    pub skills: Vec<std::path::PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringWeights {
    pub persona_fidelity: f64,
    pub task_quality: f64,
    pub efficiency: f64,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            persona_fidelity: 0.4,
            task_quality: 0.4,
            efficiency: 0.2,
        }
    }
}

impl ScoringWeights {
    pub fn for_mode(mode: &EvalMode) -> Self {
        match mode {
            EvalMode::Soul => Self {
                persona_fidelity: 0.60,
                task_quality: 0.25,
                efficiency: 0.15,
            },
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EvalCategory {
    CoreTask,
    PersonalityProbe,
    EdgeCase,
    GuardrailTest,
    ToolUsage,
    IdentityConsistency,
    ValueConflict,
    VoicePreservation,
    BoundaryHolding,
    NovelSituation,
}

impl std::fmt::Display for EvalCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CoreTask => write!(f, "Core Task"),
            Self::PersonalityProbe => write!(f, "Personality Probe"),
            Self::EdgeCase => write!(f, "Edge Case"),
            Self::GuardrailTest => write!(f, "Guardrail Test"),
            Self::ToolUsage => write!(f, "Tool Usage"),
            Self::IdentityConsistency => write!(f, "Identity Consistency"),
            Self::ValueConflict => write!(f, "Value Conflict"),
            Self::VoicePreservation => write!(f, "Voice Preservation"),
            Self::BoundaryHolding => write!(f, "Boundary Holding"),
            Self::NovelSituation => write!(f, "Novel Situation"),
        }
    }
}

impl EvalCategory {
    pub fn for_soul_mode() -> Vec<Self> {
        vec![
            Self::IdentityConsistency,
            Self::ValueConflict,
            Self::VoicePreservation,
            Self::BoundaryHolding,
            Self::NovelSituation,
        ]
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
    pub fn compute_overall_weighted(fidelity: f64, quality: f64, efficiency: f64, weights: &ScoringWeights) -> f64 {
        fidelity * weights.persona_fidelity + quality * weights.task_quality + efficiency * weights.efficiency
    }

    pub fn compute_overall(fidelity: f64, quality: f64, efficiency: f64) -> f64 {
        let result = fidelity * 0.4 + quality * 0.4 + efficiency * 0.2;
        (result * 1000.0).round() / 1000.0
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
    #[serde(default)]
    pub tokens_delta: u64,
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_mode_default_is_soul() {
        assert!(matches!(EvalMode::default(), EvalMode::Soul));
    }

    #[test]
    fn eval_mode_display() {
        assert_eq!(EvalMode::Soul.to_string(), "soul");
        assert_eq!(EvalMode::Claude.to_string(), "claude");
        assert_eq!(EvalMode::Agents.to_string(), "agents");
        assert_eq!(EvalMode::Memory.to_string(), "memory");
        assert_eq!(EvalMode::Skills.to_string(), "skills");
    }

    #[test]
    fn eval_mode_from_str() {
        assert_eq!("soul".parse::<EvalMode>().unwrap(), EvalMode::Soul);
        assert_eq!("claude".parse::<EvalMode>().unwrap(), EvalMode::Claude);
        assert!("invalid".parse::<EvalMode>().is_err());
    }

    #[test]
    fn held_context_default_is_empty() {
        let held = HeldContext::default();
        assert!(held.claude_md.is_none());
        assert!(held.agents_md.is_none());
        assert!(held.memory_md.is_none());
        assert!(held.skills.is_empty());
    }

    #[test]
    fn soul_scoring_weights() {
        let weights = ScoringWeights::for_mode(&EvalMode::Soul);
        assert!((weights.persona_fidelity - 0.60).abs() < f64::EPSILON);
        assert!((weights.task_quality - 0.25).abs() < f64::EPSILON);
        assert!((weights.efficiency - 0.15).abs() < f64::EPSILON);
    }

    #[test]
    fn default_scoring_weights_unchanged() {
        let weights = ScoringWeights::default();
        assert!((weights.persona_fidelity - 0.4).abs() < f64::EPSILON);
        assert!((weights.task_quality - 0.4).abs() < f64::EPSILON);
        assert!((weights.efficiency - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn compute_overall_uses_weights() {
        let weights = ScoringWeights::for_mode(&EvalMode::Soul);
        let overall = EvalScore::compute_overall_weighted(0.9, 0.8, 0.7, &weights);
        let expected = 0.9 * 0.60 + 0.8 * 0.25 + 0.7 * 0.15;
        assert!((overall - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn soul_eval_categories_exist() {
        let cats = EvalCategory::for_soul_mode();
        assert_eq!(cats.len(), 5);
        assert!(cats.contains(&EvalCategory::IdentityConsistency));
    }

    #[test]
    fn existing_categories_still_work() {
        assert_eq!(EvalCategory::CoreTask.to_string(), "Core Task");
    }
}
