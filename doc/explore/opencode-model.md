# OpenCode Model State

## Overview

OpenCode stores user model preferences and history in a single JSON file. This file tracks:
- Recently used models
- User-pinned favorite models
- Variant preferences for specific models

## File Location

The model state file location varies by platform following XDG Base Directory specification:

| Platform | Path |
|-----------|-------|
| Linux | `~/.local/state/opencode/model.json` |
| macOS | `~/Library/Application Support/opencode/model.json` |
| Windows | `%LOCALAPPDATA%\opencode\model.json` |

## Schema

```json
{
  "recent": [
    {
      "providerID": "provider-name",
      "modelID": "model-id"
    }
  ],
  "favorite": [
    {
      "providerID": "provider-name",
      "modelID": "model-id"
    }
  ],
  "variant": {
    "provider/model-id": "variant-name"
  }
}
```

## Fields

### `recent` (Array)
- **Purpose**: Tracks up to 10 most recently used models
- **Order**: Most recent first
- **Example**:
  ```json
  [
    {
      "providerID": "opencode",
      "modelID": "kimi-k2-thinking"
    },
    {
      "providerID": "openrouter",
      "modelID": "google/gemini-3-flash-preview"
    }
  ]
  ```

### `favorite` (Array)
- **Purpose**: User-pinned favorite models for quick access
- **Empty by default**: `[]` if no favorites set
- **Example**:
  ```json
  [
    {
      "providerID": "anthropic",
      "modelID": "claude-sonnet-4-5-20250929"
    }
  ]
  ```

### `variant` (Object)
- **Purpose**: Maps specific models to preferred variant configurations
- **Keys**: Model identifier in format `"providerID/modelID"`
- **Values**: Variant name (e.g., "high", "max", "medium")
- **Optional**: May be omitted or empty if no variant preferences set
- **Example**:
  ```json
  {
    "opencode/kimi-k2-thinking": "high",
    "anthropic/claude-sonnet-4-5-20250929": "max"
  }
  ```

## Example Complete File

```json
{
  "recent": [
    {
      "providerID": "zai-coding-plan",
      "modelID": "glm-4.7"
    },
    {
      "providerID": "cerebras",
      "modelID": "zai-glm-4.7"
    },
    {
      "providerID": "opencode",
      "modelID": "kimi-k2-thinking"
    },
    {
      "providerID": "openrouter",
      "modelID": "moonshotai/kimi-k2-thinking"
    },
    {
      "providerID": "openrouter",
      "modelID": "google/gemini-3-flash-preview"
    },
    {
      "providerID": "opencode",
      "modelID": "gpt-5.2"
    },
    {
      "providerID": "opencode",
      "modelID": "glm-4.7-free"
    },
    {
      "providerID": "deepseek",
      "modelID": "deepseek-chat"
    },
    {
      "providerID": "opencode",
      "modelID": "glm-4.6"
    },
    {
      "providerID": "opencode",
      "modelID": "kimi-k2"
    }
  ],
  "favorite": [],
  "variant": {}
}
```

## Reading the File (Rust)

```rust
use serde::{Deserialize, Serialize};
use directories::ProjectDirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRef {
    #[serde(rename = "providerID")]
    pub provider_id: String,
    #[serde(rename = "modelID")]
    pub model_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeModelState {
    pub recent: Vec<ModelRef>,
    pub favorite: Vec<ModelRef>,
    #[serde(default)]
    pub variant: HashMap<String, String>,
}

fn load_model_state() -> Result<OpenCodeModelState> {
    let proj_dirs = ProjectDirs::from("org", "opencode", "opencode")
        .expect("Failed to determine project directories");

    let state_dir = proj_dirs
        .state_dir()
        .expect("Failed to determine state directory");

    let model_state_path = state_dir.join("model.json");

    let contents = std::fs::read_to_string(&model_state_path)
        .with_context(|| format!("Failed to read model state from {}", model_state_path.display()))?;

    let state: OpenCodeModelState = serde_json::from_str(&contents)
        .with_context(|| "Failed to parse OpenCode model state JSON")?;

    Ok(state)
}
```

## Reading the File (TypeScript/Node.js)

```typescript
import { readFile } from 'node:fs/promises'
import { xdgState } from 'xdg-basedir'
import path from 'node:path'

const modelFile = path.join(xdgState!, 'opencode', 'model.json')
const data = await readFile(modelFile, 'utf-8')
const modelState = JSON.parse(data)

// Recent models
modelState.recent.forEach((m: {providerID: string, modelID: string}) => {
  console.log(`${m.providerID}/${m.modelID}`)
})

// Favorite models
modelState.favorite.forEach((m: {providerID: string, modelID: string}) => {
  console.log(`${m.providerID}/${m.modelID}`)
})

// Variant preferences
Object.entries(modelState.variant).forEach(([modelKey, variant]) => {
  console.log(`${modelKey} → ${variant}`)
})
```

## Usage in oc-variance

The `show-current` command reads this file to display:

1. **📋 Recently Used Models** - Shows last 10 models used
2. **⭐ Favorite Models** - Shows user-pinned favorites
3. **⚙️ Model Variants** - Shows variant preferences for specific models

```bash
cd ~/src/variance
cargo run -- show-current
```

Example output:
```
OpenCode Current Models

📋 Recently Used Models:
  1. zai-coding-plan/glm-4.7
  2. cerebras/zai-glm-4.7
  3. opencode/kimi-k2-thinking
  4. openrouter/moonshotai/kimi-k2-thinking
  5. openrouter/google/gemini-3-flash-preview
  6. opencode/gpt-5.2
  7. opencode/glm-4.7-free
  8. deepseek/deepseek-chat
  9. opencode/glm-4.6
  10. opencode/kimi-k2

⭐ No favorite models set

⚙️  Model Variants (0 configured):

Model state file: /home/rektide/.local/state/opencode/model.json
```

## Key Points

- **File is user-specific**: Each user has their own model state file
- **File is dynamic**: Updated whenever user switches models
- **recent array is FIFO**: Oldest entries removed when limit (10) exceeded
- **variant keys use full path**: Format is `"providerID/modelID"` to avoid conflicts
- **Field names use camelCase**: `providerID` and `modelID` (not snake_case)
- **Cross-platform**: Uses XDG directories for consistent behavior across OS
- **May be missing fields**: favorite and variant are optional, use `#[serde(default)]` in Rust

## Related Files

- **TypeScript source**: `/opt/opencode-git/packages/opencode/src/provider/model.ts` (likely location)
- **Rust implementation**: `~/src/variance/src/models.rs` (show_current_models function)
- **OC Variance CLI**: `~/src/variance/src/cli.rs` (ShowCurrent command)
