# ADR-001: OpenCode Model Configuration CLI Tool

## Status

In Progress

## Program: o-variance

**Purpose:** Interactive TUI for managing OpenCode model providers, models, and variants with caching from models.dev API.

## Context

OpenCode uses a JSON-based configuration system that allows users to configure multiple providers, models, and model variants with different thinking/reasoning levels. Currently, users must manually edit `opencode.json` to:

- Add or modify providers
- Configure models within providers
- Create and customize model variants
- Set model-specific options (thinking budget, reasoning effort, verbosity)

This manual editing process is error-prone and lacks discoverability. Users need a better way to:

1. See all configured providers and models
2. Browse existing variants and their parameters
3. Understand what parameters are available for each model
4. Add new models from providers
5. Create custom variants with specific configurations

## Decision Drivers

- **User Experience**: Need a friendly, interactive CLI interface for configuration
- **Discoverability**: Users should easily see what models and parameters are available
- **Validation**: Prevent invalid configurations before writing to files
- **Completeness**: Support all opencode model configuration features
- **Programmatic Discovery**: Leverage external APIs (models.dev) for model information
- **Performance**: No high-performance requirements - simple blocking I/O is sufficient
- **Caching**: Avoid repeated API calls with local cache and periodic refresh

## Considered Options

### Option 1: Simple TUI with dialoguer/demand/inquire

Use a question-based CLI library that presents sequential menus for navigating the provider/model/variant hierarchy.

**Pros:**

- Simple to implement
- Familiar pattern for CLI users
- Easy to validate input

**Cons:**

- Lost context between menu levels
- Cannot see the full tree at once
- Tedious to navigate deep hierarchies

### Option 2: Tree-based TUI with ratatui

Use a terminal UI framework that displays the full provider/model/variant tree with persistent state and split-pane editing.

**Pros:**

- Full tree visibility at all times
- Navigate up/down the hierarchy freely
- Split-pane editing preserves context
- More professional appearance

**Cons:**

- More complex implementation
- Higher learning curve for users
- Requires more terminal features

### Option 3: Inline TUI within existing CLI

Build a lightweight tree UI that appears inline with the normal CLI flow, using a simplified rendering approach.

**Pros:**

- Less disruptive than full-screen TUI
- Can be integrated with existing opencode commands
- Simpler than full ratatui app

**Cons:**

- Limited screen real estate
- May be difficult to maintain tree state
- Potential compatibility issues

## Decision

**Choose Option 2: Tree-based TUI with ratatui**

The tree-based approach provides the best user experience for exploring and editing complex hierarchical configurations. The ability to see the entire tree and navigate freely between levels is critical for understanding how providers, models, and variants relate to each other.

## Proposed Architecture

### Data Model

```rust
pub struct ModelConfig {
    pub providers: HashMap<String, ProviderConfig>,
}

pub struct ProviderConfig {
    pub npm: Option<String>,
    pub name: Option<String>,
    pub options: ProviderOptions,
    pub models: HashMap<String, ModelEntry>,
}

pub struct ModelEntry {
    pub name: Option<String>,
    pub limit: Option<ContextLimits>,
    pub variants: HashMap<String, VariantConfig>,
}

pub struct VariantConfig {
    pub options: ModelOptions,
}

pub struct ModelOptions {
    // Anthropic-specific
    pub thinking: Option<ThinkingConfig>,

    // OpenAI-specific
    pub reasoning_effort: Option<String>, // "low", "medium", "high"
    pub text_verbosity: Option<String>,   // "low", "high"
    pub reasoning_summary: Option<String>, // "auto", "none"

    // Generic
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
}

pub struct ThinkingConfig {
    pub type_: String, // "enabled", "disabled"
    pub budget_tokens: u32,
}

pub struct ContextLimits {
    pub context: u32,
    pub output: u32,
}
```

### Tree Structure

The UI will display a hierarchical tree that is **fully expanded by default**. The tree has **no borders** to maximize screen space and reduce visual clutter.

```
OpenCode Model Configuration

  anthropic
    claude-sonnet-4-5-20250929
      thinking-high (variant)
      thinking-low (variant)
    claude-opus-4-20250514
      [new variant]
    [new model]
  openai
    gpt-5
      high-reasoning (variant)
    [new model]
  [new provider]

────────────────────────────────────────────────────────────
Configuration Panel

Selected: anthropic/claude-sonnet-4-5-20250929/thinking-high

Thinking Config:
  [X] Enabled
  Budget Tokens: [16000      ]

[Save] [Cancel]
```

### User Interaction Flow

1. **Navigation**: Use arrow keys or vim-style navigation to move through the tree
2. **Expansion/Collapse**:
   - Tree starts fully expanded
   - Press `[` to expand the selected subtree
   - Press `]` to collapse the selected subtree
   - Press Enter or → to toggle expansion state
3. **Selection**: Press Space to select a node and show in configuration panel
4. **New Items**: Select "[new ...]" entries to create new providers/models/variants
5. **Editing**: Modify parameters in the configuration panel
6. **Persistence**: Press "Save" to write changes to `opencode.json`

### Keyboard Shortcuts

| Key       | Action                                      |
| --------- | ------------------------------------------- |
| ↑ / ↓     | Navigate up/down in tree                    |
| → / Enter | Toggle expand/collapse on selected node     |
| `[`       | Expand the selected subtree                 |
| `]`       | Collapse the selected subtree               |
| Space     | Select node and show in configuration panel |
| q         | Quit application                            |
| s         | Save configuration (when valid)             |
| ?         | Show help                                   |

### Integration with models.dev with Caching

The tool will use a caching layer to avoid repeated API calls:

```rust
// Cache structure
pub struct ModelsCache {
    pub fetched_at: DateTime<Utc>,
    pub providers: Vec<ProviderInfo>,
    pub api_url: String,
}

// Cache manager implementation
impl CacheManager {
    /// Get cached data if fresh, otherwise fetch fresh data
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
        let response = ureq::get("https://models.dev/api.json").call()?;
        let providers = self.parse_models_response(&response.into_string()?)?;

        let cache = ModelsCache {
            fetched_at: Utc::now(),
            providers,
            api_url: "https://models.dev/api.json".to_string(),
        };

        self.save_cache(&cache)
    }

    /// Cache is fresh if less than 24 hours old
    fn is_cache_fresh(&self, cache: &ModelsCache) -> bool {
        let age = Utc::now() - cache.fetched_at;
        age < Duration::hours(24)
    }
}
```

**Cache Strategy:**

- Cache file stored in XDG cache directory (e.g., `~/.cache/oc-variance/models-dev-cache.json`)
- Maximum cache age: 24 hours
- Users can force refresh with `fetch-models` command
- Cache includes timestamp for freshness checking

**Model Metadata Structure:**

```rust
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub models: Vec<ModelInfo>,
}

pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub context_window: Option<u32>,
    pub supports_thinking: Option<bool>,
    pub max_thinking_tokens: Option<u32>,
    pub supports_reasoning_effort: Option<bool>,
    pub max_output_tokens: Option<u32>,
}
```

### Known Model Parameters

Based on opencode documentation, we understand these parameters:

#### Anthropic Models

- `thinking.type`: "enabled" or "disabled"
- `thinking.budget_tokens`: integer (typically up to 200000)
- `temperature`: float (0.0 - 1.0)
- `max_tokens`: integer

#### OpenAI Models

- `reasoning_effort`: "low", "medium", or "high"
- `text_verbosity`: "low" or "high"
- `reasoning_summary`: "auto" or "none"
- `temperature`: float (0.0 - 2.0)
- `max_tokens`: integer

#### Generic Parameters

- `temperature`: Controls randomness (0.0 - 2.0 depending on model)
- `max_tokens`: Maximum output tokens

#### Provider Options

- `baseURL`: Custom API endpoint
- `apiKey`: Authentication key
- `headers`: Custom HTTP headers

### Error Handling Strategy

Following modern CLI best practices with anyhow for user-friendly error messages:

```rust
use anyhow::{anyhow, bail, Context, Result};
use thiserror::Error;

// Custom error types for specific domains
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found at {path}")]
    NotFound { path: PathBuf },

    #[error("Invalid configuration: {message}")]
    Invalid { message: String },

    #[error("Provider '{provider}' not found")]
    ProviderNotFound { provider: String },

    #[error("Model '{model}' not available for provider '{provider}'")]
    ModelNotFound { provider: String, model: String },
}

// Example: Load configuration with helpful context
pub async fn load_config(path: &Path) -> Result<ModelConfig> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file at {}", path.display()))?;

    let config: ModelConfig = serde_json::from_str(&contents)
        .with_context(|| format!("Invalid JSON in config file {}", path.display()))?;

    validate_config(&config)
        .with_context(|| "Configuration validation failed")?;

    Ok(config)
}

// Display errors with suggestions
pub fn display_error(err: &anyhow::Error) -> String {
    let mut output = String::new();

    writeln!(
        &mut output,
        "{} {}",
        style("Error:").red().bold(),
        err
    ).unwrap();

    // Chain of causes
    let mut source = err.source();
    while let Some(cause) = source {
        writeln!(
            &mut output,
            "  {} {}",
            style("Caused by:").yellow(),
            cause
        ).unwrap();
        source = cause.source();
    }

    // Add helpful suggestions
    if let Some(suggestion) = suggest_fix(err) {
        writeln!(
            &mut output,
            "\n{} {}",
            style("Suggestion:").green(),
            suggestion
        ).unwrap();
    }

    output
}

// Provide context-aware suggestions
fn suggest_fix(err: &anyhow::Error) -> Option<&'static str> {
    let msg = err.to_string();

    if msg.contains("not found") && msg.contains("config") {
        Some("Run 'opencode config init' to create a default configuration")
    } else if msg.contains("Invalid JSON") {
        Some("Check your opencode.json file for syntax errors. Consider using a JSON validator.")
    } else if msg.contains("Provider") && msg.contains("not found") {
        Some("Verify the provider name is correct. Use 'opencode models' to list available providers.")
    } else {
        None
    }
}
```

### Configuration Management

The tool should support layered configuration from multiple sources:

```rust
use directories::ProjectDirs;

impl ConfigLoader {
    pub async fn load(cli_path: Option<&Path>) -> Result<ModelConfig> {
        let mut config = ModelConfig::default();

        // 1. Load from default locations with proper precedence
        for path in Self::default_paths() {
            if path.exists() {
                config.merge_file(&path)
                    .with_context(|| format!("Failed to load config from {}", path.display()))?;
            }
        }

        // 2. Load from CLI-specified path (highest precedence)
        if let Some(path) = cli_path {
            if !path.exists() {
                bail!("Config file not found: {}", path.display());
            }
            config.merge_file(path)?;
        }

        // 3. Apply environment variable overrides
        config.merge_env()?;

        // 4. Validate final configuration
        config.validate()
            .context("Configuration validation failed. See errors above.")?;

        Ok(config)
    }

    fn default_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // System-wide config (lowest precedence)
        paths.push(PathBuf::from("/etc/opencode/config.json"));

        // User config from XDG directories
        if let Some(proj_dirs) = ProjectDirs::from("org", "rekon", "rekon-config") {
            paths.push(proj_dirs.config_dir().join("opencode.json"));
        }

        // Project-local config (medium precedence)
        if let Ok(cwd) = std::env::current_dir() {
            paths.push(cwd.join("opencode.json"));
        }

        paths
    }

    fn merge_env(&mut self) -> Result<()> {
        // Override with environment variables
        if let Ok(endpoint) = std::env::var("OPENCODE_API_ENDPOINT") {
            if let Some(ref mut api) = self.provider.get_mut("anthropic") {
                if let Some(ref mut opts) = api.options.as_mut() {
                    opts.base_url = Some(endpoint);
                }
            }
        }

        Ok(())
    }
}

impl VariantConfig {
    fn validate_with_context(
        &self,
        provider: &str,
        model: &str,
        variant: &str,
    ) -> Result<()> {
        if let Some(ref thinking) = self.options.thinking {
            if thinking.type_ == "enabled" && thinking.budget_tokens == 0 {
                bail!(
                    "Variant '{}/{}:{}' has enabled thinking with 0 budget tokens. \
                     Set a positive value for thinking.budget_tokens.",
                    provider, model, variant
                );
            }
        }

        // Validate thinking budget doesn't exceed model limits
        if let Some(ref thinking) = self.options.thinking {
            if thinking.budget_tokens > 200000 {
                bail!(
                    "Variant '{}/{}:{}' has thinking budget of {} tokens, which exceeds maximum of 200000. \
                     Use a smaller value or remove this variant.",
                    provider, model, variant, thinking.budget_tokens
                );
            }
        }

        Ok(())
    }
}
```

### Testing Strategy

Following CLI testing best practices with assert_cmd and insta:

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_load_valid_config() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("opencode.json");
    std::fs::write(
        &config_path,
        r#"{
            "provider": {
                "anthropic": {
                    "models": {
                        "claude-sonnet-4-5": {}
                    }
                }
            }
        }"#
    ).unwrap();

    Command::cargo_bin("rekon-config")
        .unwrap()
        .arg("--config")
        .arg(&config_path)
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration is valid"));
}

#[test]
fn test_invalid_thinking_budget() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("opencode.json");
    std::fs::write(
        &config_path,
        r#"{
            "provider": {
                "anthropic": {
                    "models": {
                        "claude-sonnet-4-5": {
                            "variants": {
                                "overflow": {
                                    "options": {
                                        "thinking": {
                                            "type": "enabled",
                                            "budget_tokens": 999999999
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }"#
    ).unwrap();

    Command::cargo_bin("rekon-config")
        .unwrap()
        .arg("--config")
        .arg(&config_path)
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("thinking budget"))
        .stderr(predicate::str::contains("exceeds maximum"));
}

// Snapshot testing for UI output
#[test]
fn test_help_output() {
    let output = Command::cargo_bin("rekon-config")
        .unwrap()
        .arg("--help")
        .output()
        .unwrap();

    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}
```

### Missing Information

We need to research the following to build a complete solution:

1. **Parameter Discovery**: Does `models.dev` API provide schema information for model parameters?
   - Available parameter names
   - Valid value ranges
   - Default values
   - Required vs optional parameters

2. **Model Capabilities**: Can we determine from `models.dev`:
   - Which models support extended thinking?
   - Maximum thinking budget per model?
   - Which models support reasoning effort?
   - Context window limits?

3. **Provider-Specific Validation**: Need to understand:
   - What parameters are valid for each provider type?
   - Are there mutually exclusive parameters?
   - How do parameters interact?

4. **Opencode Schema**: Need to verify:
   - Full list of supported parameters
   - Schema validation rules
   - Version-specific differences

### Implementation Plan

#### Phase 1: Project Setup & Structure

1. Initialize Rust project in `variance/` subdirectory
2. Set up modular project structure following CLI best practices
3. Configure dependencies in `Cargo.toml`

**Project Structure:**

```
variance/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point
│   ├── cli.rs              # CLI argument parsing with clap
│   ├── config.rs            # OpenCode config loading and validation
│   ├── models.rs            # Model discovery entry point
│   ├── cache.rs            # models.dev API caching
│   └── ui.rs              # TUI framework (placeholder)
└── (future) tests/
```

#### Phase 2: models.dev Integration with Caching

1. Implement blocking HTTP client using ureq
2. Add caching layer with freshness checking (24h max age)
3. Store cache in XDG cache directory
4. Parse models.dev API response (placeholder structure)

**Caching Features:**

- Cache stored in `~/.cache/oc-variance/models-dev-cache.json`
- Automatic cache refresh when >24 hours old
- `fetch-models` command to force refresh
- Timestamp tracking for freshness validation

#### Phase 3: Configuration Management

1. Load and parse `opencode.json` with serde
2. Implement multi-source config discovery (CLI arg, config dir, current dir)
3. Add validation for OpenCode schema and model parameters
4. Support `validate` command for standalone validation

**Config Discovery Order:**

1. `--config` CLI argument (highest priority)
2. XDG config directory (`~/.config/oc-variance/opencode.json`)
3. Current directory (`./opencode.json`)

#### Phase 4: Core Data Structures ✅ PARTIAL

1. ✅ Define Rust types for opencode configuration with serde
2. ✅ Implement parsing and serialization of `opencode.json`
3. ⏳ Create in-memory tree representation (TUI phase)
4. ✅ Add configuration validation with context

#### Phase 5: Basic UI Framework ⏳ TODO

1. Set up ratatui application with crossterm terminal handling
2. Implement tree view rendering (fully expanded by default, no borders)
3. Add navigation (vim-style, `[`/`]` for expand/collapse)
4. Handle keyboard input for selection and editing

#### Phase 6: Configuration Panel ⏳ TODO

1. Create split-pane layout (tree + config panel)
2. Implement form fields for parameter editing
3. Add real-time validation with helpful error messages
4. Display model info from cached models.dev data

#### Phase 7: CRUD Operations ⏳ TODO

1. Implement create (new provider/model/variant)
2. Implement read (parse and display)
3. Implement update (edit and save)
4. Implement delete (remove items with confirmation)
5. Add user prompts using dialoguer for destructive operations

#### Phase 8: Testing & Quality ⏳ TODO

1. Integration tests with assert_cmd
2. Snapshot testing with insta for UI output
3. Mock external dependencies (models.dev API)
4. Configuration validation tests

#### Phase 9: Advanced Features ⏳ TODO

1. Search/filter functionality in tree view
2. Import/export configurations
3. Configuration templates
4. Shell integration and completions

### Technology Stack

- **Language**: Rust (2021 edition, no nightly needed)
- **TUI Framework**: ratatui with crossterm
- **CLI Parsing**: clap (derive API) for command-line arguments
- **Error Handling**: anyhow with context, thiserror for custom types
- **Configuration**: serde with JSON support
- **Model Discovery**: ureq (blocking HTTP) to fetch models.dev API with caching
- **User Interaction**: dialoguer for prompts, indicatif for progress
- **Terminal Handling**: console for styled output, colored for colors
- **Directory Management**: directories crate for XDG-compliant config and cache paths

### Project Location

The `oc-variance` tool lives in the `variance/` subdirectory of the rekon project.

### Dependencies

See `variance/Cargo.toml` for the complete dependency list. Key dependencies include:

- **clap** - CLI argument parsing with derive macros
- **ratatui** - Terminal UI framework for tree-based interface
- **crossterm** - Terminal handling and input
- **anyhow** - Ergonomic error handling with context
- **thiserror** - Custom error types
- **serde/serde_json** - Configuration serialization
- **ureq** - Simple blocking HTTP client for API requests
- **directories** - XDG-compliant config and cache directories
- **dialoguer** - Interactive prompts and confirmations
- **indicatif** - Progress bars and spinners
- **chrono** - Time handling for cache freshness checking

### Alternatives Considered

**dialoguer** - Good for simple prompts, but lacks persistent tree views  
**demand** - Focused on input validation, not suitable for complex UIs  
**inquire** - Modern and ergonomic, but designed for sequential workflows

### Success Criteria

1. Users can view all configured providers, models, and variants
2. Users can navigate the tree without losing context
3. Users can edit existing configurations with real-time validation
4. Users can add new models using `models.dev` discovery
5. Users can create custom variants with specific parameter combinations
6. Changes are validated before writing to `opencode.json`
7. Tool handles edge cases (invalid configs, missing files, etc.)
8. Error messages are helpful and provide actionable suggestions
9. All functionality is covered by integration tests
10. CLI supports shell completions for bash, zsh, fish, and PowerShell
11. Configuration follows XDG Base Directory specification
12. Async operations handle failures gracefully with retry logic

### Open Questions

1. What is the exact API for the `models.dev` crate?
2. Does `models.dev` provide parameter schemas?
3. How should we handle providers not supported by `models.dev`?
4. Should we support local model providers (Ollama, LM Studio)?
5. How do we handle configuration migrations when opencode schema changes?

### References

- [OpenCode Configuration Documentation](https://opencode.ai/docs/config)
- [ratatui Documentation](https://ratatui.rs/)
- [models.dev API](https://models.dev/api) (investigate actual URL)
- OpenCode configuration examples in `provider/opencode-model-config.json`
- [The Definitive Guide to High-Performance CLI and Automation Tools with Rust](../../prompt/rust-cli.md) - Best practices for CLI development
- [Clap Documentation](https://docs.rs/clap/latest/clap/)
- [Anyhow Documentation](https://docs.rs/anyhow/latest/anyhow/)
- [Tokio Documentation](https://docs.rs/tokio/latest/tokio/)

---

**Document Version**: 1.1
**Created**: January 13, 2026
**Last Updated**: January 13, 2026
