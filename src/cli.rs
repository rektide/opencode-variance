use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

#[derive(Parser)]
#[command(
    name = "oc-variance",
    about = "Interactive TUI for OpenCode model configuration and variance",
    long_about = "Manage OpenCode model providers, models, and variants through an intuitive terminal UI. \
                 Fetch available models from models.dev and configure custom parameter combinations.",
    version,
    author,
)]
pub struct Cli {
    /// Path to opencode.json configuration file
    #[arg(short = 'C', long, global = true, env = "OPENCODE_CONFIG")]
    pub config: Option<std::path::PathBuf>,

    /// Path to cache directory (default: XDG cache directory)
    #[arg(short, long, global = true, env = "OC_VARIANCE_CACHE")]
    pub cache_dir: Option<std::path::PathBuf>,

    /// Increase logging verbosity
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Launch interactive TUI (default)
    Interactive,

    /// Fetch and cache available models from models.dev
    FetchModels,

    /// Validate current opencode.json configuration
    Validate,

    /// Generate shell completions
    #[command(arg_required_else_help = true)]
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}
