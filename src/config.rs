use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

    #[serde(default)]
    pub mcp: serde_json::Map<String, serde_json::Value>,
}

pub type ProviderConfigMap = serde_json::Map<String, serde_json::Value>;

/// Load and validate OpenCode configuration with layered merging
pub fn load_config(config_dir: &Path, cli_path: Option<&Path>) -> Result<OpenCodeConfig> {
    let mut merged_config = serde_json::Map::new();

    // Priority 1: System-wide config (optional)
    if let Some(system_config) = find_system_config() {
        if system_config.exists() {
            println!("Loading system config: {}", system_config.display());
            let contents = fs::read_to_string(&system_config)
                .with_context(|| format!("Failed to read system config: {}", system_config.display()))?;
            if let Ok(config) = serde_json::from_str::<Value>(&contents) {
                if let Some(obj) = config.as_object() {
                    for (key, value) in obj {
                        merged_config.insert(key.clone(), value.clone());
                    }
                }
            }
        }
    }

    // Priority 2: User config directory (optional)
    if let Some(user_config) = find_user_config() {
        if user_config.exists() {
            println!("Loading user config: {}", user_config.display());
            let contents = fs::read_to_string(&user_config)
                .with_context(|| format!("Failed to read user config: {}", user_config.display()))?;
            if let Ok(config) = serde_json::from_str::<Value>(&contents) {
                if let Some(obj) = config.as_object() {
                    for (key, value) in obj {
                        merged_config.insert(key.clone(), value.clone());
                    }
                }
            }
        }
    }

    // Priority 3: Current directory config (optional)
    let cwd_config = PathBuf::from("./opencode.json");
    if cwd_config.exists() {
        println!("Loading local config: {}", cwd_config.display());
        let contents = fs::read_to_string(&cwd_config)
            .with_context(|| format!("Failed to read local config: {}", cwd_config.display()))?;
        if let Ok(config) = serde_json::from_str::<Value>(&contents) {
            if let Some(obj) = config.as_object() {
                for (key, value) in obj {
                    merged_config.insert(key.clone(), value.clone());
                }
            }
        }
    }

    // Priority 4: Environment variable for config path (optional)
    if let Ok(env_path) = std::env::var("OPENCODE_CONFIG") {
        let env_path_buf = PathBuf::from(&env_path);
        if env_path_buf.exists() {
            println!("Loading env config: {}", env_path_buf.display());
            let contents = fs::read_to_string(&env_path_buf)
                .with_context(|| format!("Failed to read env config: {}", env_path_buf.display()))?;
            if let Ok(config) = serde_json::from_str::<Value>(&contents) {
                if let Some(obj) = config.as_object() {
                    for (key, value) in obj {
                        merged_config.insert(key.clone(), value.clone());
                    }
                }
            }
        }
    }

    // Priority 5: CLI-specified config path (highest priority)
    if let Some(cli_path) = cli_path {
        if !cli_path.exists() {
            return Err(anyhow::anyhow!(
                "CLI-specified config file not found: {}",
                cli_path.display()
            ));
        }
        println!("Loading CLI config: {}", cli_path.display());
        let contents = fs::read_to_string(cli_path)
            .with_context(|| format!("Failed to read CLI config: {}", cli_path.display()))?;
        if let Ok(config) = serde_json::from_str::<Value>(&contents) {
            if let Some(obj) = config.as_object() {
                for (key, value) in obj {
                    merged_config.insert(key.clone(), value.clone());
                }
            }
        }
    } else {
        // If no CLI path and no configs found, use config_dir default
        let config_dir_config = config_dir.join("opencode.json");
        if config_dir_config.exists() {
            println!("Loading default config: {}", config_dir_config.display());
            let contents = fs::read_to_string(&config_dir_config)
                .with_context(|| format!("Failed to read default config: {}", config_dir_config.display()))?;
            if let Ok(config) = serde_json::from_str::<Value>(&contents) {
                if let Some(obj) = config.as_object() {
                    for (key, value) in obj {
                        merged_config.insert(key.clone(), value.clone());
                    }
                }
            }
        } else if merged_config.is_empty() {
            // Return default config if no files found
            return Ok(OpenCodeConfig::default());
        }
    }

    // Convert merged config to OpenCodeConfig
    let merged_value = Value::Object(merged_config);
    let config: OpenCodeConfig = serde_json::from_value(merged_value)
        .with_context(|| "Merged configuration does not match OpenCode schema")?;

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

/// Find system-wide configuration file
/// Checks /etc/opencode/opencode.json and /etc/opencode.json
fn find_system_config() -> Option<PathBuf> {
    let paths = [
        "/etc/opencode/opencode.json",
        "/etc/opencode.json",
    ];

    for path in paths {
        let path_buf = PathBuf::from(path);
        if path_buf.exists() {
            return Some(path_buf);
        }
    }

    None
}

/// Find user configuration file following XDG spec
/// Primary: ~/.config/opencode/opencode.json
/// Fallback: ~/opencode/opencode.json
fn find_user_config() -> Option<PathBuf> {
    // Primary XDG location: ~/.config/opencode/opencode.json
    if let Some(proj_dirs) = ProjectDirs::from("org", "rekon", "opencode") {
        let opencode_config = proj_dirs.config_dir().join("opencode.json");
        if opencode_config.exists() {
            return Some(opencode_config);
        }
    }

    // Fallback: ~/opencode/opencode.json
    if let Ok(home_str) = std::env::var("HOME") {
        let home = PathBuf::from(&home_str);
        let fallback_config = home.join("opencode/opencode.json");
        if fallback_config.exists() {
            return Some(fallback_config);
        }
    }

    None
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
    config: &Value,
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
    thinking: &Value,
) -> Result<()> {
    if let Some(type_) = thinking.get("type") {
        let _type_str = type_.as_str().unwrap_or("disabled");

        if type_.as_str().unwrap_or("disabled") == "enabled" {
            if let Some(budget) = thinking.get("budgetTokens") {
                let budget_value = budget.as_u64().unwrap_or(0);

                if budget_value == 0 {
                    return Err(anyhow::anyhow!(
                        "Variant '{}/{}:{}' has enabled thinking but budgetTokens is 0. \
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
pub fn validate(config_dir: &Path, cli_path: Option<&Path>) -> Result<()> {
    let _config = load_config(config_dir, cli_path)?;
    println!("Configuration is valid");
    Ok(())
}
