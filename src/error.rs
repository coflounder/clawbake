use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClawbakeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Claude CLI error: {0}")]
    Claude(String),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Eval error: {0}")]
    Eval(String),

    #[error("Budget exhausted: consumed {consumed} of {limit} tokens")]
    BudgetExhausted { consumed: u64, limit: u64 },

    #[error("Sandbox error: {0}")]
    Sandbox(String),

    #[error("TUI error: {0}")]
    Tui(String),

    #[error("Export error: {0}")]
    Export(String),
}

pub type Result<T> = std::result::Result<T, ClawbakeError>;
