use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// OpenCode configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct OpenCodeConfig {
    /// JSON Schema URL
    #[serde(default, rename = "$schema")]
    pub schema: Option<String>,

    #[serde(default)]
    pub model: Option<String>,

    #[serde(default)]
    pub small_model: Option<String>,

    #[serde(default)]
    pub provider: ProviderConfigMap,

    #[serde(default)]
    pub agent: serde_json::Map<String, serde_json::Value>,

    #[serde(default)]
    pub mode: serde_json::Map<String, serde_json::Value>,

    #[serde(default)]
    pub command: serde_json::Map<String, serde_json::Value>,
}

pub type ProviderConfigMap = serde_json::Map<String, serde_json::Value>;

/// Load and validate OpenCode configuration
pub fn load_config(config_dir: &Path, cli_path: Option<&Path>) -> Result<OpenCodeConfig> {
    let config_path = find_config_path(config_dir, cli_path)?;

    let contents = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config: {}", config_path.display()))?;

    let config: OpenCodeConfig = serde_json::from_str(&contents)
        .with_context(|| format!("Invalid JSON in config: {}", config_path.display()))?;

    validate_config(&config)?;

    Ok(config)
}

/// Save OpenCode configuration
pub fn save_config(config_dir: &Path, config: &OpenCodeConfig) -> Result<()> {
    let config_path = config_dir.join("opencode.json");

    let contents = serde_json::to_string_pretty(config)
        .context("Failed to serialize configuration")?;

    fs::write(&config_path, contents)
        .with_context(|| format!("Failed to write config: {}", config_path.display()))?;

    Ok(())
}

/// Find the appropriate config file path
fn find_config_path(config_dir: &Path, cli_path: Option<&Path>) -> Result<PathBuf> {
    // 1. CLI-specified path has highest priority
    if let Some(path) = cli_path {
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "Config file not found: {}",
                path.display()
            ));
        }
        return Ok(path.to_path_buf());
    }

    // 2. Config directory
    let config_path = config_dir.join("opencode.json");
    if config_path.exists() {
        return Ok(config_path);
    }

    // 3. Current directory
    let cwd_config = PathBuf::from("opencode.json");
    if cwd_config.exists() {
        return Ok(cwd_config);
    }

    Err(anyhow::anyhow!(
        "No opencode.json found. \
         Expected location: {} or ./opencode.json. \
         Use --config to specify a path.",
        config_path.display()
    ))
}

/// Validate configuration
fn validate_config(config: &OpenCodeConfig) -> Result<()> {
    if let Some(ref schema) = config.schema {
        if !schema.starts_with("https://opencode.ai/") && !schema.starts_with("http://opencode.ai/") {
            return Err(anyhow::anyhow!(
                "Invalid schema URL: {}. Expected opencode.ai schema.",
                schema
            ));
        }
    }

    // Validate provider configurations
    if let Some(providers) = config.provider.get("anthropic") {
        if let Some(models) = providers.get("models") {
            if let Some(models_obj) = models.as_object() {
                for (model_id, model_config) in models_obj {
                    validate_model_config("anthropic", model_id, model_config)?;
                }
            }
        }
    }

    Ok(())
}

/// Validate individual model configuration
fn validate_model_config(
    provider: &str,
    model: &str,
    config: &serde_json::Value,
) -> Result<()> {
    // Check variants if present
    if let Some(variants) = config.get("variants") {
        if let Some(variants_obj) = variants.as_object() {
            for (variant_name, variant_config) in variants_obj {
                // Validate variant's options
                if let Some(options) = variant_config.get("options") {
                    if let Some(thinking) = options.get("thinking") {
                        validate_thinking_config(provider, model, variant_name, thinking)?;
                    }
                }
            }
        }
    }

    // Check options if present (default variant)
    if let Some(options) = config.get("options") {
        if let Some(thinking) = options.get("thinking") {
            validate_thinking_config(provider, model, "default", thinking)?;
        }
    }

    Ok(())
}

/// Validate thinking configuration
fn validate_thinking_config(
    provider: &str,
    model: &str,
    variant: &str,
    thinking: &serde_json::Value,
) -> Result<()> {
    if let Some(type_) = thinking.get("type") {
        let _type_str = type_.as_str().unwrap_or("disabled");

        if type_.as_str().unwrap_or("disabled") == "enabled" {
            if let Some(budget) = thinking.get("budget_tokens") {
                let budget_value = budget.as_u64().unwrap_or(0);

                if budget_value == 0 {
                    return Err(anyhow::anyhow!(
                        "Variant '{}/{}:{}' has enabled thinking but budget_tokens is 0. \
                         Set a positive value.",
                        provider, model, variant
                    ));
                }

                if budget_value > 200000 {
                    return Err(anyhow::anyhow!(
                        "Variant '{}/{}:{}' has thinking budget of {} tokens, \
                         which exceeds maximum of 200000.",
                        provider, model, variant, budget_value
                    ));
                }
            }
        }
    }

    Ok(())
}

/// Validate current configuration without modifying it
pub fn validate(config_dir: &Path) -> Result<()> {
    let _config = load_config(config_dir, None)?;
    println!("Configuration is valid");
    Ok(())
}
