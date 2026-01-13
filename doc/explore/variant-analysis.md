# OpenCode Model Variants Analysis

## Summary

**Critical Discovery**: Model variants are **NOT** stored in models.dev API. They are **programmatically generated** by OpenCode based on:
1. Model capabilities (specifically `reasoning` capability)
2. Provider npm package type
3. Model ID and release date

## What OpenCode Actually Supports

### Model Schema (from `/opt/opencode-git/packages/opencode/src/provider/provider.ts`)

```typescript
{
  id: string
  providerID: string
  name: string
  family?: string
  
  api: {
    id: string
    url: string
    npm: string  // Provider's npm package
  }
  
  capabilities: {
    temperature: boolean
    reasoning: boolean
    attachment: boolean
    toolcall: boolean
    
    input: {
      text: boolean
      audio: boolean
      image: boolean
      video: boolean
      pdf: boolean
    }
    
    output: {
      text: boolean
      audio: boolean
      image: boolean
      video: boolean
      pdf: boolean
    }
    
    interleaved: boolean | {
      field: "reasoning_content" | "reasoning_details"
    }
  }
  
  cost: {
    input: number
    output: number
    cache: {
      read: number
      write: number
    }
    experimentalOver200K?: {
      input: number
      output: number
      cache: { read: number; write: number }
    }
  }
  
  limit: {
    context: number
    output: number
  }
  
  status: "alpha" | "beta" | "deprecated" | "active"
  
  options: Record<string, any>
  headers: Record<string, string>
  release_date: string
  
  variants: Record<string, Record<string, any>>  // KEY FIELD
}
```

### How Variants Are Generated

The `ProviderTransform.variants(model)` function (lines 274-454 in transform.ts) generates variants based on:

```typescript
// Only models with reasoning capability get variants
if (!model.capabilities.reasoning) return {}

// Some providers/models explicitly return {}
if (id.includes("deepseek") || id.includes("minimax") || 
    id.includes("glm") || id.includes("mistral")) return {}
```

### Provider-Specific Variant Structures

#### 1. @openrouter/ai-sdk-provider
Only for GPT, Gemini-3, Grok-4:
```typescript
{
  "none": { reasoning: { effort: "none" } },
  "minimal": { reasoning: { effort: "minimal" } },
  "low": { reasoning: { effort: "low" } },
  "medium": { reasoning: { effort: "medium" } },
  "high": { reasoning: { effort: "high" } },
  "xhigh": { reasoning: { effort: "xhigh" } }
}
```

#### 2. @ai-sdk/gateway
```typescript
{
  "none": { reasoningEffort: "none" },
  "minimal": { reasoningEffort: "minimal" },
  "low": { reasoningEffort: "low" },
  "medium": { reasoningEffort: "medium" },
  "high": { reasoningEffort: "high" },
  "xhigh": { reasoningEffort: "xhigh" }
}
```

#### 3. @ai-sdk/cerebras, @ai-sdk/togetherai, @ai-sdk/xai, @ai-sdk/deepinfra, @ai-sdk/openai-compatible
```typescript
{
  "low": { reasoningEffort: "low" },
  "medium": { reasoningEffort: "medium" },
  "high": { reasoningEffort: "high" }
}
```

#### 4. @ai-sdk/azure
```typescript
// For o1-mini: {}
// For gpt-5-* or gpt-5:
{
  "minimal": { 
    reasoningEffort: "minimal",
    reasoningSummary: "auto",
    include: ["reasoning.encrypted_content"]
  },
  "low": { ... },
  "medium": { ... },
  "high": { ... }
}

// For Codex 5.2:
{
  "low": { ... },
  "medium": { ... },
  "high": { ... },
  "xhigh": { ... }
}

// For GPT-5 with release_date >= 2025-11-13:
{
  "none": { ... },
  "minimal": { ... },
  "low": { ... },
  "medium": { ... },
  "high": { ... }
}

// For GPT-5 with release_date >= 2025-12-04:
{
  "none": { ... },
  "minimal": { ... },
  "low": { ... },
  "medium": { ... },
  "high": { ... },
  "xhigh": { ... }
}
```

#### 5. @ai-sdk/openai
```typescript
// For gpt-5-pro: {}
// For Codex variants (depends on version)
// Similar to Azure but with different logic
```

#### 6. @ai-sdk/anthropic
```typescript
{
  "high": {
    thinking: {
      type: "enabled",
      budgetTokens: 16000
    }
  },
  "max": {
    thinking: {
      type: "enabled",
      budgetTokens: 31999
    }
  }
}
```

#### 7. @ai-sdk/amazon-bedrock
For Anthropic models:
```typescript
{
  "high": {
    reasoningConfig: {
      type: "enabled",
      budgetTokens: 16000
    }
  },
  "max": {
    reasoningConfig: {
      type: "enabled",
      budgetTokens: 31999
    }
  }
}
```

For Amazon Nova models:
```typescript
{
  "low": {
    reasoningConfig: {
      type: "enabled",
      maxReasoningEffort: "low"
    }
  },
  "medium": { ... },
  "high": { ... }
}
```

#### 8. @ai-sdk/google, @ai-sdk/google-vertex
For Google 2.5:
```typescript
{
  "high": {
    thinkingConfig: {
      includeThoughts: true,
      thinkingBudget: 16000
    }
  },
  "max": {
    thinkingConfig: {
      includeThoughts: true,
      thinkingBudget: 24576
    }
  }
}
```

For other Google models:
```typescript
{
  "low": {
    includeThoughts: true,
    thinkingLevel: "low"
  },
  "high": {
    includeThoughts: true,
    thinkingLevel: "high"
  }
}
```

#### 9. @ai-sdk/groq
```typescript
{
  "none": {
    includeThoughts: true,
    thinkingLevel: "none"
  },
  "low": { ... },
  "medium": { ... },
  "high": { ... }
}
```

#### 10. @ai-sdk/mistral, @ai-sdk/cohere, @ai-sdk/perplexity
```typescript
// No variants: {}
```

### How Variants Are Used

From `llm.ts` (lines 91-100):

```typescript
const variant = !input.small && 
                 input.model.variants && 
                 input.user.variant 
  ? input.model.variants[input.user.variant] 
  : {}

const options: Record<string, any> = pipe(
  base,              // Provider options
  mergeDeep(input.model.options),  // Model options from config
  mergeDeep(input.agent.options),   // Agent options
  mergeDeep(variant)               // Selected variant (lowest precedence)
)
```

**Precedence (highest to lowest)**:
1. Base provider options
2. Model options from config
3. Agent options
4. **Variant options (applied last)**

### Config File Structure (variants)

From `config.ts` (lines 791-807):

```json
{
  "provider": {
    "anthropic": {
      "models": {
        "claude-sonnet-4-5-20250929": {
          "variants": {
            "high": {
              "disabled": false  // Disable this variant
            },
            "custom-high": {
              "thinking": {
                "type": "enabled",
                "budgetTokens": 20000  // Override budget
              }
            }
          }
        }
      }
    }
  }
}
```

**Key points**:
- Config can disable variants with `{ "disabled": true }`
- Config can define custom variants
- Config variants are merged with auto-generated variants (lines 878-886 in provider.ts)

### User-Selectable Variants

From `message-v2.ts` (line 317):

```typescript
export const User = Base.extend({
  role: z.literal("user"),
  time: z.object({ created: z.number() }),
  model: z.object({
    providerID: z.string(),
    modelID: z.string(),
  }),
  system: z.string().optional(),
  tools: z.record(z.string(), z.boolean()).optional(),
  variant: z.string().optional()  // User can select variant
})
```

From CLI `run.ts` (lines 90-92):

```typescript
.option("variant", {
  describe: "model variant (provider-specific reasoning effort, e.g., high, max, minimal)",
  default: undefined
})
```

## Critique of Current Approach

### What You're Doing Correctly ✅

1. **Fetching from models.dev API**: This is the right source for discovering available models and their basic capabilities
2. **Parsing capabilities fields**: You're correctly identifying which models support reasoning, tool calls, etc.
3. **Caching approach**: 24-hour cache is reasonable for models.dev data

### What's Wrong ❌

1. **You're NOT capturing variants**: Variants are not in models.dev - they're generated programmatically
2. **Field naming mismatch**: 
   - You use: `supports_thinking`, `supports_reasoning`, `supports_tool_call`
   - OpenCode uses: `capabilities.reasoning`, `capabilities.toolcall`
3. **Missing critical fields**:
   - `provider.npm` - **ESSENTIAL** for determining variant generation
   - `release_date` - **CRITICAL** for OpenAI variant logic
   - `api.url`, `api.npm`
   - `capabilities.input`/`output` arrays vs your separate arrays
   - `options` - Model-specific options
   - `headers` - Provider headers
   - `status` - alpha/beta/deprecated/active
   - `family` - Model family/grouping

4. **Modality structure**: You have separate `modalities_input` and `modalities_output`, but OpenCode uses nested objects:
   ```typescript
   // Your approach:
   modalities_input: ["text", "image"]
   modalities_output: ["text"]
   
   // OpenCode's structure:
   capabilities: {
     input: { text: true, image: true },
     output: { text: true }
   }
   ```

5. **Cost structure**: You're missing cache costs and 200K+ context costs

## What You Need to Change

### 1. Update ModelInfo struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub family: Option<String>,
    
    // Provider info - CRITICAL for variant generation
    pub provider_npm: String,  // e.g., "@ai-sdk/openai"
    pub provider_url: String,
    
    // Capabilities
    pub capabilities: ModelCapabilities,
    
    // Limits
    pub context_limit: u32,
    pub output_limit: u32,
    
    // Costs
    pub cost_input: f64,
    pub cost_output: f64,
    pub cost_cache_read: Option<f64>,
    pub cost_cache_write: Option<f64>,
    pub cost_over_200k_input: Option<f64>,
    pub cost_over_200k_output: Option<f64>,
    
    // Other
    pub release_date: String,  // CRITICAL for OpenAI variant logic
    pub status: ModelStatus,
    pub options: Option<serde_json::Value>,  // Model-specific options
    pub headers: Option<HashMap<String, String>>,
    
    // Note: variants is NOT parsed from models.dev
    // It will be computed separately
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub temperature: bool,
    pub reasoning: bool,  // Key for variant generation
    pub attachment: bool,
    pub tool_call: bool,
    pub input: ModalityCapabilities,
    pub output: ModalityCapabilities,
    pub interleaved: InterleavedConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalityCapabilities {
    pub text: bool,
    pub audio: bool,
    pub image: bool,
    pub video: bool,
    pub pdf: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InterleavedConfig {
    Enabled(bool),
    Field { field: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelStatus {
    #[serde(rename = "alpha")]
    Alpha,
    #[serde(rename = "beta")]
    Beta,
    #[serde(rename = "deprecated")]
    Deprecated,
    #[serde(rename = "active")]
    Active,
}
```

### 2. Implement Variant Generation Logic

You need to port the TypeScript `ProviderTransform.variants()` function to Rust. This will be **complex** but essential:

```rust
pub fn generate_variants(model: &ModelInfo) -> HashMap<String, serde_json::Value> {
    // Only models with reasoning capability get variants
    if !model.capabilities.reasoning {
        return HashMap::new();
    }
    
    // Some models explicitly return no variants
    let id = model.id.to_lowercase();
    if id.contains("deepseek") || id.contains("minimax") || 
       id.contains("glm") || id.contains("mistral") {
        return HashMap::new();
    }
    
    match model.provider_npm.as_str() {
        "@ai-sdk/anthropic" => generate_anthropic_variants(),
        "@ai-sdk/openai" => generate_openai_variants(&model.id, &model.release_date),
        "@ai-sdk/azure" => generate_azure_variants(&model.id, &model.release_date),
        "@openrouter/ai-sdk-provider" => generate_openrouter_variants(&model.id),
        "@ai-sdk/google" | "@ai-sdk/google-vertex" => generate_google_variants(&model.id),
        "@ai-sdk/groq" => generate_groq_variants(),
        "@ai-sdk/cerebras" | "@ai-sdk/togetherai" | 
        "@ai-sdk/xai" | "@ai-sdk/deepinfra" | 
        "@ai-sdk/openai-compatible" => generate_widely_supported_variants(),
        "@ai-sdk/gateway" => generate_gateway_variants(),
        "@ai-sdk/mistral" | "@ai-sdk/cohere" | "@ai-sdk/perplexity" => HashMap::new(),
        _ => HashMap::new()
    }
}
```

### 3. Update parse_model_info()

Parse all the fields correctly from models.dev:

```rust
fn parse_model_info(&self, provider_id: &str, provider_npm: &str, 
                     provider_api: &str, value: &serde_json::Value) -> Option<ModelInfo> {
    let limit = value.get("limit").and_then(|v| v.as_object());
    let cost = value.get("cost").and_then(|v| v.as_object());
    let modalities = value.get("modalities").and_then(|v| v.as_object());
    
    Some(ModelInfo {
        id: value["id"].as_str()?.to_string(),
        name: value["name"].as_str()?.to_string(),
        family: value.get("family").and_then(|v| v.as_str()).map(|s| s.to_string()),
        
        provider_npm: value.get("provider")
            .and_then(|p| p.get("npm"))
            .and_then(|npm| npm.as_str())
            .unwrap_or(provider_npm)
            .to_string(),
        provider_url: provider_api.to_string(),
        
        capabilities: ModelCapabilities {
            temperature: value.get("temperature").and_then(|v| v.as_bool()).unwrap_or(false),
            reasoning: value.get("reasoning").and_then(|v| v.as_bool()).unwrap_or(false),
            attachment: value.get("attachment").and_then(|v| v.as_bool()).unwrap_or(false),
            tool_call: value.get("tool_call").and_then(|v| v.as_bool()).unwrap_or(true),
            input: self.parse_modalities(modalities, "input"),
            output: self.parse_modalities(modalities, "output"),
            interleaved: self.parse_interleaved(value),
        },
        
        context_limit: limit
            .and_then(|l| l.get("context").and_then(|v| v.as_u64()).map(|v| v as u32))
            .unwrap_or(0),
        output_limit: limit
            .and_then(|l| l.get("output").and_then(|v| v.as_u64()).map(|v| v as u32))
            .unwrap_or(0),
        
        cost_input: cost.and_then(|c| c.get("input").and_then(|v| v.as_f64())).unwrap_or(0.0),
        cost_output: cost.and_then(|c| c.get("output").and_then(|v| v.as_f64())).unwrap_or(0.0),
        cost_cache_read: cost.and_then(|c| c.get("cache_read").and_then(|v| v.as_f64())),
        cost_cache_write: cost.and_then(|c| c.get("cache_write").and_then(|v| v.as_f64())),
        
        cost_over_200k_input: cost
            .and_then(|c| c.get("context_over_200k"))
            .and_then(|o| o.get("input").and_then(|v| v.as_f64())),
        cost_over_200k_output: cost
            .and_then(|c| c.get("context_over_200k"))
            .and_then(|o| o.get("output").and_then(|v| v.as_f64())),
        
        release_date: value.get("release_date")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        
        status: value.get("status")
            .and_then(|s| s.as_str())
            .and_then(|s| match s {
                "alpha" => Some(ModelStatus::Alpha),
                "beta" => Some(ModelStatus::Beta),
                "deprecated" => Some(ModelStatus::Deprecated),
                "active" => Some(ModelStatus::Active),
                _ => None,
            })
            .unwrap_or(ModelStatus::Active),
        
        options: value.get("options").cloned(),
        headers: value.get("headers").cloned().and_then(|h| {
            serde_json::from_value(h).ok()
        }),
    })
}
```

### 4. Consider Adding a Variant Computation Module

Since variants are computed, not stored, consider:

```rust
pub mod variants {
    use super::*;
    
    pub fn compute_all_variants(providers: &[ProviderInfo]) {
        for provider in providers {
            for model in &provider.models {
                let variants = generate_variants(model);
                // Store or display variants
            }
        }
    }
    
    pub fn get_variant_for_model(model_id: &str, variant_name: &str) 
        -> Option<serde_json::Value> {
        // Look up model and return the specified variant
    }
    
    pub fn list_available_variants(model: &ModelInfo) -> Vec<String> {
        generate_variants(model).keys().cloned().collect()
    }
}
```

## Recommendations

1. **Keep using models.dev**: It's the correct source for model discovery
2. **Don't parse variants from models.dev**: Implement the variant generation logic in Rust
3. **Capture all fields from models.dev**: Especially `provider.npm`, `release_date`, and full capabilities
4. **Match OpenCode's structure exactly**: Use the same field names and nesting
5. **Implement the variant generation**: Port the TypeScript logic from `transform.ts`
6. **Support config file variants**: Allow users to add custom variants via opencode.json
7. **Display generated variants**: Show users what variants are available for each model
8. **Validate variant selection**: Warn if user selects a non-existent variant

## Next Steps

1. Update `cache.rs` with the corrected ModelInfo structure
2. Implement the variant generation module
3. Update parsing logic to capture all required fields
4. Add variant display to your TUI
5. Support editing variants in config files
6. Add validation for variant selection

