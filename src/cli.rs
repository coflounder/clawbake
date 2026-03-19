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

        /// Run without the TUI dashboard (log to stdout)
        #[arg(long)]
        headless: bool,
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
