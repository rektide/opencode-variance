use crate::cache::{CacheManager, ModelsCache};
use anyhow::Result;
use std::path::Path;

/// Fetch models from models.dev API with caching
pub fn fetch_models(_config_dir: &Path, cache_dir: &Path) -> Result<()> {
    let cache_manager = CacheManager::new(cache_dir);

    match cache_manager.get_or_fetch() {
        Ok(cache) => {
            println!("Successfully loaded {} providers:", cache.providers.len());

            for provider in &cache.providers {
                println!("  {}", provider.name);
                for model in &provider.models {
                    println!("    - {}", model.name);
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching models: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Get cached models without forcing refresh
pub fn get_cached_models(cache_dir: &Path) -> Result<Option<ModelsCache>> {
    let cache_manager = CacheManager::new(cache_dir);
    Ok(cache_manager.load_cached()?)
}
