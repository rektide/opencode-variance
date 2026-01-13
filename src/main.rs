use anyhow::Result;
use clap::Parser;
use clap::CommandFactory;
use clap_complete::generate;
use directories::ProjectDirs;

mod cli;
mod config;
mod models;
mod ui;
mod cache;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    // Initialize project directories
    let proj_dirs = ProjectDirs::from("org", "rekon", "oc-variance")
        .expect("Failed to determine project directories");

    let config_dir = proj_dirs.config_dir();
    let cache_dir = proj_dirs.cache_dir();

    // Ensure directories exist
    std::fs::create_dir_all(config_dir)?;
    std::fs::create_dir_all(cache_dir)?;

    // Run the appropriate command
    match cli.command {
        cli::Commands::Interactive => ui::run_interactive(config_dir, cache_dir),
        cli::Commands::FetchModels => models::fetch_models(config_dir, cache_dir),
        cli::Commands::Validate => config::validate(config_dir),
        cli::Commands::Completions { shell } => {
            generate(shell, &mut cli::Cli::command(), "oc-variance", &mut std::io::stdout());
            Ok(())
        }
    }
}
