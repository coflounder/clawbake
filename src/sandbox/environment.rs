use crate::error::Result;
use crate::sandbox::stubs::{is_builtin_tool, write_stub, StubSpec};
use crate::types::ToolSpec;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub struct SandboxEnvironment {
    _temp_dir: TempDir,
    pub work_dir: PathBuf,
    pub env_path: String,
}

impl SandboxEnvironment {
    pub fn new(tools: &[ToolSpec]) -> Result<Self> {
        Self::new_with_project(tools, None)
    }

    /// Create a sandbox, optionally seeding the workspace from a project directory.
    /// When `project_dir` is provided, its contents are copied into the sandbox workspace
    /// so the agent can work against a realistic codebase.
    pub fn new_with_project(tools: &[ToolSpec], project_dir: Option<&Path>) -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let work_dir = temp_dir.path().join("workspace");
        std::fs::create_dir_all(&work_dir)?;

        // Seed workspace from project_dir if provided
        if let Some(src) = project_dir {
            if src.exists() {
                copy_dir_recursive(src, &work_dir)?;
            }
        }

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

/// Recursively copy directory contents from `src` to `dst`.
/// Skips .git directories to avoid unnecessary sandbox pollution.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Skip .git — would pollute sandbox with full repo history
        if name_str == ".git" {
            continue;
        }

        let src_path = entry.path();
        let dst_path = dst.join(&name);

        if src_path.is_dir() {
            std::fs::create_dir_all(&dst_path)?;
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
