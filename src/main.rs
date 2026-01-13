use anyhow::Result;
use clap::Parser;
use clap::CommandFactory;
use clap_complete::generate;
use directories::ProjectDirs;
use std::path::{Path, PathBuf};

mod cli;
mod config;
mod models;
mod ui;
mod cache;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    // Determine cache directory
    let cache_dir = if let Some(override_dir) = cli.cache_dir {
        override_dir
    } else {
        let proj_dirs = ProjectDirs::from("org", "rekon", "oc-variance")
            .expect("Failed to determine project directories");
        proj_dirs.cache_dir().to_path_buf()
    };

    // Determine config directory (where opencode.json is loaded from)
    let config_dir = if let Some(opencode_path) = cli.config.as_deref() {
        // Use parent directory of specified config file as config_dir
        opencode_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        // Fallback to XDG config directory
        let proj_dirs = ProjectDirs::from("org", "rekon", "oc-variance")
            .expect("Failed to determine project directories");
        proj_dirs.config_dir().to_path_buf()
    };

    // Ensure directories exist
    std::fs::create_dir_all(&cache_dir)?;
    std::fs::create_dir_all(&config_dir)?;

    // Run appropriate command
    match cli.command {
        cli::Commands::Interactive => ui::run_interactive(&config_dir, &cache_dir),
        cli::Commands::FetchModels => models::fetch_models(&config_dir, &cache_dir),
        cli::Commands::Validate => {
            let opencode_path = cli.config.as_ref().map(|p| p.as_path());
            config::validate(&config_dir, opencode_path)
        }
        cli::Commands::Completions { shell } => {
            generate(shell, &mut cli::Cli::command(), "oc-variance", &mut std::io::stdout());
            Ok(())
        }
    }
}
