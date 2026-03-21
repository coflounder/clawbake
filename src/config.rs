use crate::error::Result;
use crate::types::{EvalConfig, EvalMode, HeldContext, ModelConfig, OutputConfig, PersonaSpec};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulModeConfig {
    pub session_count: usize,
    pub consistency_threshold: f64,
}

impl Default for SoulModeConfig {
    fn default() -> Self {
        Self {
            session_count: 5,
            consistency_threshold: 0.85,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeModeConfig {
    /// Path to the codebase the project instruction file will be evaluated against.
    /// The sandbox will be seeded from this directory.
    pub project_dir: Option<PathBuf>,
    /// Enable ablation testing (remove one instruction at a time, measure impact).
    #[serde(default = "default_true")]
    pub ablation: bool,
    /// Identify decision points where the instruction file provided no guidance.
    #[serde(default = "default_true")]
    pub coverage_analysis: bool,
    /// Built-in scaffold to use when project_dir is not provided.
    /// Options: "rust-minimal", "typescript-next", "python-fastapi", "monorepo"
    pub scaffold_codebase: Option<String>,
}

fn default_true() -> bool { true }

impl Default for ClaudeModeConfig {
    fn default() -> Self {
        Self {
            project_dir: None,
            ablation: true,
            coverage_analysis: true,
            scaffold_codebase: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConfig {
    pub target: EvalMode,
    #[serde(default)]
    pub hold_constant: HeldContext,
    #[serde(default)]
    pub soul: SoulModeConfig,
    #[serde(default)]
    pub claude: ClaudeModeConfig,
}

impl Default for ModeConfig {
    fn default() -> Self {
        Self {
            target: EvalMode::default(),
            hold_constant: HeldContext::default(),
            soul: SoulModeConfig::default(),
            claude: ClaudeModeConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub persona: PersonaSpec,
    pub eval: EvalConfig,
    pub models: ModelConfig,
    pub output: OutputConfig,
    #[serde(default)]
    pub mode: ModeConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            persona: PersonaSpec::default(),
            eval: EvalConfig::default(),
            models: ModelConfig::default(),
            output: OutputConfig::default(),
            mode: ModeConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EvalMode;

    #[test]
    fn mode_config_defaults() {
        let mc = ModeConfig::default();
        assert!(matches!(mc.target, EvalMode::Soul));
        assert!(mc.hold_constant.claude_md.is_none());
        assert_eq!(mc.soul.session_count, 5);
        assert!((mc.soul.consistency_threshold - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn app_config_roundtrip_with_mode() {
        let mut config = AppConfig::default();
        config.mode.target = EvalMode::Soul;
        config.mode.soul.session_count = 3;

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: AppConfig = toml::from_str(&toml_str).unwrap();

        assert!(matches!(parsed.mode.target, EvalMode::Soul));
        assert_eq!(parsed.mode.soul.session_count, 3);
    }

    #[test]
    fn app_config_backward_compatible_without_mode_section() {
        let toml_str = r#"
[persona]
name = "test"
role = "tester"
responsibility = "testing"
personality_traits = []
tools = []
guardrails = []

[eval]
eval_count = 5
max_iterations = 10
max_budget_tokens = 1000000
max_parallel = 2
max_turns_per_case = 2
interactive = true

[models]
planner = "sonnet"
optimizer = "sonnet"
evaluator = "haiku"
persona = "haiku"
stub = "haiku"

[output]
workspace_files = ["IDENTITY.md"]
format = "single"
"#;
        let parsed: AppConfig = toml::from_str(toml_str).unwrap();
        assert!(matches!(parsed.mode.target, EvalMode::Soul));
    }
}
