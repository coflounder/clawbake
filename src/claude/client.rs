use crate::claude::models::{ClaudeResponse, Tier};
use crate::config::AppConfig;
use crate::error::{ClawbakeError, Result};
use crate::types::TokenBudget;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;

pub struct ClaudeClient {
    config: AppConfig,
    budget: Arc<Mutex<TokenBudget>>,
}

pub struct ClaudeInvocation {
    model: String,
    tier: Tier,
    prompt: String,
    system_prompt: Option<String>,
    json_schema: Option<String>,
    allowed_tools: Option<Vec<String>>,
    max_turns: Option<u32>,
    env_vars: HashMap<String, String>,
    working_dir: Option<PathBuf>,
    budget: Arc<Mutex<TokenBudget>>,
}

impl ClaudeClient {
    pub fn new(config: AppConfig, budget: Arc<Mutex<TokenBudget>>) -> Self {
        Self { config, budget }
    }

    pub fn build(&self, tier: Tier, prompt: &str) -> ClaudeInvocation {
        let model = match tier {
            Tier::Planner => self.config.models.planner.clone(),
            Tier::Optimizer => self.config.models.optimizer.clone(),
            Tier::Evaluator => self.config.models.evaluator.clone(),
            Tier::Persona => self.config.models.persona.clone(),
            Tier::Stub => self.config.models.stub.clone(),
        };

        ClaudeInvocation {
            model,
            tier,
            prompt: prompt.to_string(),
            system_prompt: None,
            json_schema: None,
            allowed_tools: None,
            max_turns: None,
            env_vars: HashMap::new(),
            working_dir: None,
            budget: Arc::clone(&self.budget),
        }
    }

    pub fn budget(&self) -> Arc<Mutex<TokenBudget>> {
        Arc::clone(&self.budget)
    }
}

impl ClaudeInvocation {
    pub fn with_system_prompt(mut self, prompt: &str) -> Self {
        self.system_prompt = Some(prompt.to_string());
        self
    }

    pub fn with_json_schema(mut self, schema: &str) -> Self {
        self.json_schema = Some(schema.to_string());
        self
    }

    pub fn with_allowed_tools(mut self, tools: Vec<String>) -> Self {
        self.allowed_tools = Some(tools);
        self
    }

    pub fn with_max_turns(mut self, turns: u32) -> Self {
        self.max_turns = Some(turns);
        self
    }

    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.env_vars.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }

    pub async fn execute(self) -> Result<ClaudeResponse> {
        // Check budget before executing
        {
            let budget = self.budget.lock().await;
            if budget.exhausted() {
                return Err(ClawbakeError::BudgetExhausted {
                    consumed: budget.consumed,
                    limit: budget.limit,
                });
            }
        }

        let mut cmd = Command::new("claude");
        cmd.arg("-p")
            .arg("--output-format").arg("json")
            .arg("--model").arg(&self.model);

        if let Some(ref sp) = self.system_prompt {
            cmd.arg("--system-prompt").arg(sp);
        }

        if let Some(ref schema) = self.json_schema {
            cmd.arg("--json-schema").arg(schema);
        }

        if let Some(ref tools) = self.allowed_tools {
            cmd.arg("--allowedTools").arg(tools.join(","));
        }

        if let Some(turns) = self.max_turns {
            cmd.arg("--max-turns").arg(turns.to_string());
        }

        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }

        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        // Pipe prompt via stdin
        cmd.stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            ClawbakeError::Claude(format!("Failed to spawn claude CLI: {}", e))
        })?;

        // Write prompt to stdin
        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            stdin.write_all(self.prompt.as_bytes()).await.map_err(|e| {
                ClawbakeError::Claude(format!("Failed to write to claude stdin: {}", e))
            })?;
            drop(stdin);
        }

        let output = child.wait_with_output().await.map_err(|e| {
            ClawbakeError::Claude(format!("Failed to wait for claude: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ClawbakeError::Claude(format!(
                "claude exited with status {}: {}",
                output.status, stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let response: ClaudeResponse = serde_json::from_str(&stdout).map_err(|e| {
            ClawbakeError::Claude(format!(
                "Failed to parse claude response: {}. Output: {}",
                e,
                &stdout[..stdout.len().min(500)]
            ))
        })?;

        // Record token usage
        let tokens = response.total_tokens();
        if tokens > 0 {
            let mut budget = self.budget.lock().await;
            budget.record(self.tier.as_str(), tokens);
        }

        Ok(response)
    }

    pub async fn execute_structured<T: DeserializeOwned>(self) -> Result<(T, ClaudeResponse)> {
        let response = self.execute().await?;
        let text = response.result_text();
        let parsed: T = serde_json::from_str(text).map_err(|e| {
            ClawbakeError::Claude(format!(
                "Failed to parse structured output: {}. Text: {}",
                e,
                &text[..text.len().min(500)]
            ))
        })?;
        Ok((parsed, response))
    }
}
