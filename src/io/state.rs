use crate::error::Result;
use std::path::{Path, PathBuf};

pub struct StateDir {
    root: PathBuf,
}

impl StateDir {
    pub fn new(base: &Path) -> Self {
        Self {
            root: base.join(".clawbake"),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn config_path(&self) -> PathBuf {
        self.root.join("config.toml")
    }

    pub fn reference_path(&self) -> PathBuf {
        self.root.join("reference.md")
    }

    pub fn evals_dir(&self) -> PathBuf {
        self.root.join("evals")
    }

    pub fn cases_path(&self) -> PathBuf {
        self.evals_dir().join("cases.json")
    }

    pub fn runs_dir(&self) -> PathBuf {
        self.root.join("runs")
    }

    pub fn iteration_dir(&self, iteration: usize) -> PathBuf {
        self.runs_dir().join(format!("{:03}", iteration))
    }

    pub fn iteration_scores_path(&self, iteration: usize) -> PathBuf {
        self.iteration_dir(iteration).join("scores.json")
    }

    pub fn iteration_transcripts_dir(&self, iteration: usize) -> PathBuf {
        self.iteration_dir(iteration).join("transcripts")
    }

    pub fn iteration_identity_path(&self, iteration: usize) -> PathBuf {
        self.iteration_dir(iteration).join("identity.md")
    }

    pub fn best_dir(&self) -> PathBuf {
        self.root.join("best")
    }

    pub fn best_identity_path(&self) -> PathBuf {
        self.best_dir().join("identity.md")
    }

    pub fn best_soul_path(&self) -> PathBuf {
        self.best_dir().join("SOUL.md")
    }

    pub fn iteration_soul_path(&self, iteration: usize) -> PathBuf {
        self.iteration_dir(iteration).join("SOUL.md")
    }

    pub fn history_path(&self) -> PathBuf {
        self.root.join("history.json")
    }

    pub fn exists(&self) -> bool {
        self.root.exists()
    }

    pub fn init(&self) -> Result<()> {
        std::fs::create_dir_all(self.root())?;
        std::fs::create_dir_all(self.evals_dir())?;
        std::fs::create_dir_all(self.runs_dir())?;
        std::fs::create_dir_all(self.best_dir())?;
        Ok(())
    }

    pub fn ensure_iteration_dir(&self, iteration: usize) -> Result<()> {
        std::fs::create_dir_all(self.iteration_dir(iteration))?;
        std::fs::create_dir_all(self.iteration_transcripts_dir(iteration))?;
        Ok(())
    }
}
