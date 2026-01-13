# oc-variance

Interactive TUI for managing OpenCode model providers, models, and variants.

## Overview

**oc-variance** is a tree-based interactive terminal UI for configuring OpenCode model providers, models, and model variants with custom parameters. It uses [`ratatui`](https://ratatui.rs/) to provide an intuitive, borderless tree view that is fully expanded by default, with `[` and `]` keys for expand/collapse navigation.

The tool integrates with [models.dev](https://models.dev/) API to discover available providers and their supported models, with intelligent 24-hour caching to minimize network calls while keeping data fresh.

### Key Features

- 🌳 **Interactive Tree UI** - Fully expanded tree by default, no borders, vim-style navigation
- ⚙️ **Model Configuration** - Edit thinking budgets, reasoning effort, and provider-specific parameters
- 📦 **Model Discovery** - Fetch available models from models.dev API
- 💾 **Smart Caching** - 24-hour cache in XDG-compliant directory
- ✅ **Configuration Validation** - Real-time validation with helpful error messages
- 🔧 **CRUD Operations** - Create, read, update, and delete providers, models, and variants
- 🎨 **Split-Pane Layout** - Tree view + configuration panel for seamless editing
- ⌨️ **Keyboard-Driven** - Efficient navigation without mouse required

## Installation

### From Source

```bash
cd variance
cargo build --release
```

The binary will be available at `target/release/oc-variance`.

### Install System-Wide

```bash
cd variance
cargo install --path .
```

This installs `oc-variance` to `~/.cargo/bin/` (ensure this is in your PATH).

## Usage

### Commands

```bash
# Launch interactive TUI (default)
oc-variance

# Fetch and cache available models from models.dev
oc-variance fetch-models

# Validate current opencode.json configuration
oc-variance validate

# Use a specific configuration file
oc-variance --config /path/to/opencode.json interactive

# Use a specific cache directory
oc-variance --cache-dir /custom/cache/path
```

### Interactive Mode

When running `oc-variance` without arguments, you enter the interactive TUI:

```
oc-variance
```

**Keyboard Shortcuts:**

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate up/down in tree |
| `→` / `Enter` | Toggle expand/collapse on selected node |
| `[` | Expand selected subtree |
| `]` | Collapse selected subtree |
| `Space` | Select node and show in configuration panel |
| `q` / `ESC` | Quit application |
| `s` | Save changes (when in editing mode) |

### Configuration Discovery

`oc-variance` searches for `opencode.json` in the following order:

1. `--config` CLI argument (highest priority)
2. XDG config directory (`~/.config/oc-variance/opencode.json`)
3. Current directory (`./opencode.json`)

Cache files are stored in XDG cache directory (`~/.cache/oc-variance/models-dev-cache.json`).

## Configuration

### opencode.json Structure

OpenCode uses a JSON configuration file to define providers, models, and variants:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "model": "anthropic/claude-sonnet-4-5-20250929",
  "provider": {
    "anthropic": {
      "models": {
        "claude-sonnet-4-5-20250929": {
          "variants": {
            "high-thinking": {
              "options": {
                "thinking": {
                  "type": "enabled",
                  "budgetTokens": 16000
                }
              }
            }
          }
        }
      }
    }
  }
}
```

### Model Parameters

#### Anthropic Models
- `thinking.type`: `"enabled"` or `"disabled"`
- `thinking.budgetTokens`: Integer (1-200000)
- `temperature`: Float (0.0-1.0)
- `max_tokens`: Integer

#### OpenAI Models
- `reasoningEffort`: `"low"`, `"medium"`, or `"high"`
- `textVerbosity`: `"low"` or `"high"`
- `reasoningSummary`: `"auto"` or `"none"`
- `temperature`: Float (0.0-2.0)
- `max_tokens`: Integer

#### Generic Parameters
- `temperature`: Controls randomness (model-dependent range)
- `max_tokens`: Maximum output tokens

## Development

### Prerequisites

- Rust 2021 edition or later
- `cargo` package manager

### Setup

```bash
cd variance
cargo install
cargo build
```

### Project Structure

```
variance/
├── Cargo.toml              # Dependencies and metadata
├── src/
│   ├── main.rs             # Entry point
│   ├── cli.rs              # CLI argument parsing with clap
│   ├── config.rs            # Configuration loading and validation
│   ├── models.rs            # Model discovery entry point
│   ├── cache.rs            # models.dev API caching
│   └── ui.rs              # TUI framework
└── (future) tests/
    └── integration/         # Integration tests
```

### Testing

```bash
cd variance

# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration
```

### Build

```bash
cd variance

# Development build
cargo build

# Release build (optimized)
cargo build --release

# Check compilation
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Environment Variables

- `OPENCODE_CONFIG` - Path to custom opencode.json file
- `OC_VARIANCE_CACHE` - Path to custom cache directory
- `NO_COLOR` - Disable colored output

## Roadmap

See [issue tracker](https://github.com/rektide/rekon/issues) for detailed progress.

- [x] Phase 1: Project Setup & Structure
- [x] Phase 2: models.dev Integration with Caching
- [x] Phase 3: Configuration Management
- [ ] Phase 4: Core Data Structures
- [ ] Phase 5: Basic UI Framework
- [ ] Phase 6: Configuration Panel
- [ ] Phase 7: CRUD Operations
- [ ] Phase 8: Testing & Quality
- [ ] Phase 9: Advanced Features

## Contributing

Contributions are welcome! Please read our contributing guidelines and submit pull requests to [rektide/rekon](https://github.com/rektide/rekon/pulls).

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

Distributed under the terms of both the MIT license and the Apache License, Version 2.0.

See [LICENSE-MIT](../../LICENSE-MIT) and [LICENSE-APACHE](../../LICENSE-APACHE) files for details.

## Acknowledgments

- [OpenCode](https://opencode.ai/) - Open-source AI coding agent
- [ratatui](https://ratatui.rs/) - TUI framework
- [clap](https://github.com/clap-rs/clap) - Command-line argument parser
- [models.dev](https://models.dev/) - Model information API
- [Rust CLI Guide](../prompt/rust-cli.md) - Best practices for CLI development

## Related Projects

- [rekon](../) - Main rekon project
- [provider/opencode-model-config.json](../provider/opencode-model-config.json) - Example OpenCode configuration
