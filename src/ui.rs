use anyhow::Result;
use std::path::Path;

/// Run the interactive TUI for model configuration
pub fn run_interactive(config_dir: &Path, cache_dir: &Path) -> Result<()> {
    println!("oc-variance interactive mode");
    println!("Config dir: {}", config_dir.display());
    println!("Cache dir: {}", cache_dir.display());

    // TODO: Implement ratatui-based TUI
    // - Parse and load opencode.json
    // - Build tree structure of providers/models/variants
    // - Render fully expanded tree without borders
    // - Implement navigation with vim keys
    // - Support [ and ] for expand/collapse
    // - Show configuration panel below tree on selection
    // - Allow editing parameters
    // - Save changes back to opencode.json

    println!("\nTODO: Interactive TUI not yet implemented");
    println!("Available commands:");
    println!("  fetch-models  - Fetch available models from models.dev");
    println!("  validate      - Validate current configuration");

    Ok(())
}
