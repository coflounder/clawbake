use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "clawbake", version, about = "Identity-based system prompt generator and evaluator")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Working directory (defaults to current)
    #[arg(short, long, global = true)]
    pub dir: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new clawbake project with the setup wizard
    Init,

    /// Run the eval loop to generate and optimize the identity
    Run {
        /// Skip the wizard and use existing config
        #[arg(long)]
        no_wizard: bool,

        /// Evaluation mode: soul, claude, agents, memory, skills
        #[arg(long)]
        mode: Option<String>,

        /// Context files to hold constant during evaluation (can be repeated)
        #[arg(long)]
        hold: Vec<PathBuf>,
    },

    /// Show current status: best score, iteration count, budget
    Status,

    /// Export the best identity to the output directory
    Export {
        /// Output directory (defaults to current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_run_with_mode_soul() {
        let cli = Cli::parse_from(["clawbake", "run", "--mode", "soul"]);
        match cli.command {
            Commands::Run { mode, .. } => assert_eq!(mode.unwrap(), "soul"),
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn parse_run_with_hold_flags() {
        let cli = Cli::parse_from([
            "clawbake", "run",
            "--mode", "soul",
            "--hold", "path/to/CLAUDE.md",
            "--hold", "path/to/AGENTS.md",
        ]);
        match cli.command {
            Commands::Run { hold, .. } => {
                assert_eq!(hold.len(), 2);
                assert_eq!(hold[0], PathBuf::from("path/to/CLAUDE.md"));
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn parse_run_defaults_no_mode() {
        let cli = Cli::parse_from(["clawbake", "run"]);
        match cli.command {
            Commands::Run { mode, hold, .. } => {
                assert!(mode.is_none());
                assert!(hold.is_empty());
            }
            _ => panic!("Expected Run command"),
        }
    }
}
