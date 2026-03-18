use crate::error::Result;
use crate::sandbox::stubs::{is_builtin_tool, write_stub, StubSpec};
use crate::types::ToolSpec;
use std::path::PathBuf;
use tempfile::TempDir;

pub struct SandboxEnvironment {
    _temp_dir: TempDir,
    pub work_dir: PathBuf,
    pub env_path: String,
}

impl SandboxEnvironment {
    pub fn new(tools: &[ToolSpec]) -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let work_dir = temp_dir.path().join("workspace");
        std::fs::create_dir_all(&work_dir)?;

        let stub_dir = temp_dir.path().join("stubs");
        std::fs::create_dir_all(&stub_dir)?;

        // Only generate stubs for non-builtin tools
        let mut stub_count = 0;
        for tool in tools {
            if !is_builtin_tool(&tool.name) {
                let spec = StubSpec {
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                    behavior_hint: tool.stub_behavior.clone(),
                };
                write_stub(&stub_dir, &spec)?;
                stub_count += 1;
            }
        }

        // Build PATH with stub dir prepended (only matters for non-builtin tools)
        let original_path = std::env::var("PATH").unwrap_or_default();
        let env_path = if stub_count > 0 {
            format!("{}:{}", stub_dir.display(), original_path)
        } else {
            original_path
        };

        Ok(Self {
            _temp_dir: temp_dir,
            work_dir,
            env_path,
        })
    }

    /// List of built-in Claude tools from the tool specs.
    pub fn builtin_tools(tools: &[ToolSpec]) -> Vec<String> {
        tools
            .iter()
            .filter(|t| is_builtin_tool(&t.name))
            .map(|t| t.name.clone())
            .collect()
    }
}
