use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const CACHE_FILENAME: &str = "models-dev-cache.json";
const CACHE_MAX_AGE_HOURS: i64 = 24;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsCache {
    pub fetched_at: DateTime<Utc>,
    pub providers: Vec<ProviderInfo>,
    pub api_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub models: Vec<ModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub context_window: Option<u32>,
    pub supports_thinking: Option<bool>,
    pub max_thinking_tokens: Option<u32>,
    pub supports_reasoning_effort: Option<bool>,
    pub max_output_tokens: Option<u32>,
}

pub struct CacheManager {
    cache_dir: PathBuf,
}

impl CacheManager {
    pub fn new(cache_dir: &Path) -> Self {
        Self {
            cache_dir: cache_dir.to_path_buf(),
        }
    }

    pub fn cache_path(&self) -> PathBuf {
        self.cache_dir.join(CACHE_FILENAME)
    }

    /// Get cached data if it exists and is fresh, otherwise fetch fresh data
    pub fn get_or_fetch(&self) -> Result<ModelsCache> {
        if let Some(cached) = self.load_cached()? {
            if self.is_cache_fresh(&cached) {
                return Ok(cached);
            }
        }

        self.fetch_and_cache()
    }

    /// Fetch fresh data from models.dev API
    pub fn fetch_and_cache(&self) -> Result<ModelsCache> {
        println!("Fetching available models from models.dev...");

        let api_url = "https://models.dev/api.json";
        let response = ureq::get(api_url)
            .call()
            .context("Failed to fetch models from models.dev API")?;

        let text = response
            .into_string()
            .context("Failed to read response body")?;

        // Parse the response
        let providers = self.parse_models_response(&text)?;

        let cache = ModelsCache {
            fetched_at: Utc::now(),
            providers,
            api_url: api_url.to_string(),
        };

        // Save to cache
        self.save_cache(&cache)?;

        println!("Fetched {} providers", cache.providers.len());
        Ok(cache)
    }

    pub fn load_cached(&self) -> Result<Option<ModelsCache>> {
        let path = self.cache_path();

        if !path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read cache file: {}", path.display()))?;

        let cache: ModelsCache = serde_json::from_str(&contents)
            .context("Failed to parse cache file")?;

        Ok(Some(cache))
    }

    fn save_cache(&self, cache: &ModelsCache) -> Result<()> {
        let path = self.cache_path();
        let contents = serde_json::to_string_pretty(cache)
            .context("Failed to serialize cache")?;

        fs::write(&path, contents)
            .with_context(|| format!("Failed to write cache file: {}", path.display()))?;

        Ok(())
    }

    fn is_cache_fresh(&self, cache: &ModelsCache) -> bool {
        let age = Utc::now() - cache.fetched_at;
        age < Duration::hours(CACHE_MAX_AGE_HOURS)
    }

    /// Parse models.dev API response into our internal structure
    fn parse_models_response(&self, text: &str) -> Result<Vec<ProviderInfo>> {
        // The exact structure of models.dev API response needs investigation
        // This is a placeholder implementation
        let api_data: serde_json::Value = serde_json::from_str(text)
            .context("Failed to parse models.dev API response")?;

        let mut providers = Vec::new();

        // TODO: Parse actual API structure
        // This will need to be updated once we investigate models.dev API format
        if let Some(obj) = api_data.as_object() {
            for (provider_id, provider_data) in obj {
                let provider_name = provider_data["name"]
                    .as_str()
                    .unwrap_or(provider_id);

                let models = if let Some(models_array) = provider_data["models"].as_array() {
                    models_array
                        .iter()
                        .filter_map(|m| self.parse_model_info(m))
                        .collect()
                } else {
                    Vec::new()
                };

                providers.push(ProviderInfo {
                    id: provider_id.clone(),
                    name: provider_name.to_string(),
                    models,
                });
            }
        }

        Ok(providers)
    }

    fn parse_model_info(&self, value: &serde_json::Value) -> Option<ModelInfo> {
        Some(ModelInfo {
            id: value["id"].as_str()?.to_string(),
            name: value["name"].as_str()?.to_string(),
            context_window: value["context_window"].as_u64().map(|v| v as u32),
            supports_thinking: value["supports_thinking"].as_bool(),
            max_thinking_tokens: value["max_thinking_tokens"].as_u64().map(|v| v as u32),
            supports_reasoning_effort: value["supports_reasoning_effort"].as_bool(),
            max_output_tokens: value["max_output_tokens"].as_u64().map(|v| v as u32),
        })
    }

    /// Force refresh the cache
    pub fn refresh(&self) -> Result<ModelsCache> {
        self.fetch_and_cache()
    }
}
