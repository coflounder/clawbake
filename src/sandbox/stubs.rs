use crate::error::Result;
use std::path::Path;

pub struct StubSpec {
    pub name: String,
    pub description: String,
    pub behavior_hint: Option<String>,
}

/// Claude Code built-in tools that should NOT be stubbed.
const BUILTIN_TOOLS: &[&str] = &[
    "Bash", "Read", "Write", "Edit", "Glob", "Grep", "WebFetch", "WebSearch", "NotebookEdit",
    "Agent",
];

pub fn is_builtin_tool(name: &str) -> bool {
    BUILTIN_TOOLS.contains(&name)
}

/// Generate a simple echo stub (no claude calls) for non-builtin tools.
pub fn generate_stub_script(spec: &StubSpec) -> String {
    let hint = spec
        .behavior_hint
        .as_deref()
        .unwrap_or(&spec.description);

    format!(
        r#"#!/usr/bin/env bash
# Stub for: {} - {}
echo "[stub] {} $*"
echo "{}"
"#,
        spec.name,
        hint,
        spec.name,
        hint.replace('"', r#"\""#),
    )
}

pub fn write_stub(dir: &Path, spec: &StubSpec) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let script = generate_stub_script(spec);
    let path = dir.join(&spec.name);
    std::fs::write(&path, script)?;

    let mut perms = std::fs::metadata(&path)?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&path, perms)?;

    Ok(())
}
