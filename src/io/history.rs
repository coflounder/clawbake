use crate::error::Result;
use crate::types::HistoryEntry;
use std::path::Path;

pub fn load_history(path: &Path) -> Result<Vec<HistoryEntry>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(path)?;
    let entries: Vec<HistoryEntry> = serde_json::from_str(&content)?;
    Ok(entries)
}

pub fn save_history(path: &Path, entries: &[HistoryEntry]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(entries)?;
    std::fs::write(path, content)?;
    Ok(())
}

pub fn append_history(path: &Path, entry: HistoryEntry) -> Result<()> {
    let mut entries = load_history(path)?;
    entries.push(entry);
    save_history(path, &entries)
}
