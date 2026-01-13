use crate::cache::{CacheManager, ModelInfo, ProviderInfo};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TreeNodeKind {
    Provider(String),
    Model(String),
    Variant(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub kind: TreeNodeKind,
    pub name: String,
    pub expanded: bool,
    pub children: Vec<TreeNode>,
    pub data: Option<TreeNodeData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNodeData {
    pub provider_npm: Option<String>,
    pub model_id: Option<String>,
    pub variant_name: Option<String>,
    pub capabilities: Option<ModelCapabilities>,
    pub variant_config: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub reasoning: bool,
    pub temperature: bool,
    pub tool_call: bool,
}

impl TreeNode {
    pub fn new(kind: TreeNodeKind, name: String) -> Self {
        Self {
            kind,
            name,
            expanded: true,
            children: Vec::new(),
            data: None,
        }
    }

    pub fn provider(name: String, npm: String) -> Self {
        let mut node = Self::new(TreeNodeKind::Provider(name.clone()), name);
        node.data = Some(TreeNodeData {
            provider_npm: Some(npm),
            model_id: None,
            variant_name: None,
            capabilities: None,
            variant_config: None,
        });
        node
    }

    pub fn model(name: String, id: String, capabilities: ModelCapabilities) -> Self {
        let mut node = Self::new(TreeNodeKind::Model(name.clone()), name);
        node.data = Some(TreeNodeData {
            provider_npm: None,
            model_id: Some(id),
            variant_name: None,
            capabilities: Some(capabilities),
            variant_config: None,
        });
        node
    }

    pub fn variant(name: String, config: Value) -> Self {
        let mut node = Self::new(TreeNodeKind::Variant(name.clone()), name.clone());
        node.data = Some(TreeNodeData {
            provider_npm: None,
            model_id: None,
            variant_name: Some(name),
            capabilities: None,
            variant_config: Some(config),
        });
        node
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    pub fn expand(&mut self) {
        self.expanded = true;
    }

    pub fn collapse(&mut self) {
        self.expanded = false;
    }

    pub fn toggle(&mut self) {
        self.expanded = !self.expanded;
    }

    pub fn add_child(&mut self, child: TreeNode) {
        self.children.push(child);
    }

    pub fn count_nodes(&self) -> usize {
        1 + self.children.iter().map(|c| c.count_nodes()).sum::<usize>()
    }

    pub fn depth(&self) -> usize {
        match &self.kind {
            TreeNodeKind::Provider(_) => 0,
            TreeNodeKind::Model(_) => 1,
            TreeNodeKind::Variant(_) => 2,
        }
    }

    pub fn find_path(&self, path: &[String]) -> Option<&TreeNode> {
        if path.is_empty() {
            return Some(self);
        }

        match &self.kind {
            TreeNodeKind::Provider(ref name) if name == &path[0] => {
                if path.len() == 1 {
                    Some(self)
                } else {
                    self.children
                        .iter()
                        .find_map(|c| c.find_path(&path[1..]))
                }
            }
            TreeNodeKind::Model(ref name) if name == &path[0] => {
                if path.len() == 1 {
                    Some(self)
                } else {
                    self.children
                        .iter()
                        .find_map(|c| c.find_path(&path[1..]))
                }
            }
            TreeNodeKind::Variant(ref name) if name == &path[0] => Some(self),
            _ => None,
        }
    }

    pub fn find_path_mut(&mut self, path: &[String]) -> Option<&mut TreeNode> {
        if path.is_empty() {
            return Some(self);
        }

        let name = self.name.clone();
        match &self.kind {
            TreeNodeKind::Provider(_) if name == path[0] => {
                if path.len() == 1 {
                    Some(self)
                } else {
                    self.children
                        .iter_mut()
                        .find_map(|c| c.find_path_mut(&path[1..]))
                }
            }
            TreeNodeKind::Model(_) if name == path[0] => {
                if path.len() == 1 {
                    Some(self)
                } else {
                    self.children
                        .iter_mut()
                        .find_map(|c| c.find_path_mut(&path[1..]))
                }
            }
            TreeNodeKind::Variant(_) if name == path[0] => Some(self),
            _ => None,
        }
    }
}

pub fn build_tree_from_providers(providers: &[ProviderInfo]) -> TreeNode {
    let mut root = TreeNode::new(TreeNodeKind::Provider("root".to_string()), "root".to_string());
    root.expanded = true;

    for provider in providers {
        let mut provider_node = TreeNode::provider(provider.name.clone(), provider.npm.clone());
        provider_node.expanded = true;

        for model in &provider.models {
            let capabilities = ModelCapabilities {
                reasoning: model.capabilities.reasoning,
                temperature: model.capabilities.temperature,
                tool_call: model.capabilities.tool_call,
            };

            let mut model_node = TreeNode::model(model.name.clone(), model.id.clone(), capabilities);
            model_node.expanded = true;

            if model.capabilities.reasoning {
                let variants = generate_variants(model, &provider.npm);
                for (variant_name, variant_config) in variants {
                    model_node.add_child(TreeNode::variant(variant_name, variant_config));
                }
            }

            provider_node.add_child(model_node);
        }

        root.add_child(provider_node);
    }

    root
}

pub fn generate_variants(model: &ModelInfo, provider_npm: &str) -> HashMap<String, Value> {
    if !model.capabilities.reasoning {
        return HashMap::new();
    }

    let id = model.id.to_lowercase();
    
    if id.contains("deepseek") || id.contains("minimax") || id.contains("glm") || id.contains("mistral") {
        return HashMap::new();
    }

    match provider_npm {
        "@openrouter/ai-sdk-provider" => {
            if !id.contains("gpt") && !id.contains("gemini-3") && !id.contains("grok-4") {
                return HashMap::new();
            }
            let efforts = ["none", "minimal", "low", "medium", "high", "xhigh"];
            efforts
                .iter()
                .map(|effort| {
                    let config = serde_json::json!({
                        "reasoning": { "effort": effort }
                    });
                    (effort.to_string(), config)
                })
                .collect()
        }

        "@ai-sdk/gateway" => {
            let efforts = ["none", "minimal", "low", "medium", "high", "xhigh"];
            efforts
                .iter()
                .map(|effort| {
                    let config = serde_json::json!({
                        "reasoningEffort": effort
                    });
                    (effort.to_string(), config)
                })
                .collect()
        }

        "@ai-sdk/cerebras"
        | "@ai-sdk/togetherai"
        | "@ai-sdk/xai"
        | "@ai-sdk/deepinfra"
        | "@ai-sdk/openai-compatible" => {
            let efforts = ["low", "medium", "high"];
            efforts
                .iter()
                .map(|effort| {
                    let config = serde_json::json!({
                        "reasoningEffort": effort
                    });
                    (effort.to_string(), config)
                })
                .collect()
        }

        "@ai-sdk/azure" => {
            if id == "o1-mini" {
                return HashMap::new();
            }
            let mut efforts = vec!["low", "medium", "high"];
            if id.contains("gpt-5-") || id == "gpt-5" {
                efforts.insert(0, "minimal");
            }
            efforts
                .iter()
                .map(|effort| {
                    let config = serde_json::json!({
                        "reasoningEffort": effort,
                        "reasoningSummary": "auto",
                        "include": ["reasoning.encrypted_content"]
                    });
                    (effort.to_string(), config)
                })
                .collect()
        }

        "@ai-sdk/openai" => {
            if id == "gpt-5-pro" {
                return HashMap::new();
            }

            let mut efforts = vec!["low", "medium", "high"];
            
            if id.contains("gpt-5-") || id == "gpt-5" {
                efforts.insert(0, "minimal");
            }
            
            if model.release_date >= String::from("2025-11-13") {
                efforts.insert(0, "none");
            }

            if model.release_date >= String::from("2025-12-04") {
                efforts.push("xhigh");
            }

            efforts
                .iter()
                .map(|effort| {
                    let config = serde_json::json!({
                        "reasoningEffort": effort,
                        "reasoningSummary": "auto",
                        "include": ["reasoning.encrypted_content"]
                    });
                    (effort.to_string(), config)
                })
                .collect()
        }

        "@ai-sdk/anthropic" => {
            let mut variants = HashMap::new();
            variants.insert(
                "high".to_string(),
                serde_json::json!({
                    "thinking": {
                        "type": "enabled",
                        "budgetTokens": 16000
                    }
                }),
            );
            variants.insert(
                "max".to_string(),
                serde_json::json!({
                    "thinking": {
                        "type": "enabled",
                        "budgetTokens": 31999
                    }
                }),
            );
            variants
        }

        "@ai-sdk/amazon-bedrock" => {
            if provider_npm.contains("anthropic") {
                let mut variants = HashMap::new();
                variants.insert(
                    "high".to_string(),
                    serde_json::json!({
                        "reasoningConfig": {
                            "type": "enabled",
                            "budgetTokens": 16000
                        }
                    }),
                );
                variants.insert(
                    "max".to_string(),
                    serde_json::json!({
                        "reasoningConfig": {
                            "type": "enabled",
                            "budgetTokens": 31999
                        }
                    }),
                );
                return variants;
            }

            let efforts = ["low", "medium", "high"];
            efforts
                .iter()
                .map(|effort| {
                    let config = serde_json::json!({
                        "reasoningConfig": {
                            "type": "enabled",
                            "maxReasoningEffort": effort
                        }
                    });
                    (effort.to_string(), config)
                })
                .collect()
        }

        "@ai-sdk/google-vertex" | "@ai-sdk/google" => {
            if id.contains("2.5") {
                let mut variants = HashMap::new();
                variants.insert(
                    "high".to_string(),
                    serde_json::json!({
                        "thinkingConfig": {
                            "includeThoughts": true,
                            "thinkingBudget": 16000
                        }
                    }),
                );
                variants.insert(
                    "max".to_string(),
                    serde_json::json!({
                        "thinkingConfig": {
                            "includeThoughts": true,
                            "thinkingBudget": 24576
                        }
                    }),
                );
                return variants;
            }

            let efforts = ["low", "high"];
            efforts
                .iter()
                .map(|effort| {
                    let config = serde_json::json!({
                        "includeThoughts": true,
                        "thinkingLevel": effort
                    });
                    (effort.to_string(), config)
                })
                .collect()
        }

        "@ai-sdk/mistral" | "@ai-sdk/cohere" | "@ai-sdk/perplexity" => HashMap::new(),

        "@ai-sdk/groq" => {
            let efforts = ["none", "low", "medium", "high"];
            efforts
                .iter()
                .map(|effort| {
                    let config = serde_json::json!({
                        "includeThoughts": true,
                        "thinkingLevel": effort
                    });
                    (effort.to_string(), config)
                })
                .collect()
        }

        _ => HashMap::new(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelOptions {
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub reasoning: Option<ReasoningConfig>,
    pub custom: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ReasoningConfig {
    Anthropic {
        #[serde(rename = "type")]
        mode: String,
        budget_tokens: u32,
    },
    OpenAI {
        effort: String,
    },
    Google {
        thinking_level: Option<String>,
        thinking_budget: Option<u32>,
    },
    Generic(Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl ModelOptions {
    pub fn new() -> Self {
        Self {
            temperature: None,
            max_tokens: None,
            reasoning: None,
            custom: HashMap::new(),
        }
    }

    pub fn from_config(config: &Value) -> Result<Self> {
        let mut options = Self::new();

        if let Some(temp) = config.get("temperature").and_then(|v| v.as_f64()) {
            options.temperature = Some(temp);
        }

        if let Some(max_tokens) = config.get("max_tokens").and_then(|v| v.as_u64()) {
            options.max_tokens = Some(max_tokens as u32);
        }

        if let Some(thinking) = config.get("thinking") {
            if let Some(thinking_type) = thinking.get("type").and_then(|v| v.as_str()) {
                if thinking_type == "enabled" {
                    if let Some(budget) = thinking.get("budgetTokens").and_then(|v| v.as_u64()) {
                        options.reasoning = Some(ReasoningConfig::Anthropic {
                            mode: "enabled".to_string(),
                            budget_tokens: budget as u32,
                        });
                    }
                }
            }
        }

        if let Some(effort) = config.get("reasoningEffort").and_then(|v| v.as_str()) {
            options.reasoning = Some(ReasoningConfig::OpenAI {
                effort: effort.to_string(),
            });
        }

        if let Some(thinking_config) = config.get("thinkingConfig") {
            let thinking_level = thinking_config
                .get("thinkingLevel")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let thinking_budget = thinking_config
                .get("thinkingBudget")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32);

            options.reasoning = Some(ReasoningConfig::Google {
                thinking_level,
                thinking_budget,
            });
        }

        Ok(options)
    }

    pub fn validate(&self, capabilities: &ModelCapabilities) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if let Some(temp) = self.temperature {
            if !capabilities.temperature {
                errors.push(ValidationError {
                    field: "temperature".to_string(),
                    message: "This model does not support temperature adjustment".to_string(),
                });
            } else if temp < 0.0 || temp > 2.0 {
                errors.push(ValidationError {
                    field: "temperature".to_string(),
                    message: "Temperature must be between 0.0 and 2.0".to_string(),
                });
            }
        }

        if let Some(ref reasoning) = self.reasoning {
            if !capabilities.reasoning {
                errors.push(ValidationError {
                    field: "reasoning".to_string(),
                    message: "This model does not support reasoning".to_string(),
                });
            }

            if let ReasoningConfig::Anthropic { budget_tokens, .. } = reasoning {
                if *budget_tokens > 200000 {
                    errors.push(ValidationError {
                        field: "reasoning.budgetTokens".to_string(),
                        message: "Budget tokens must not exceed 200000".to_string(),
                    });
                }
                if *budget_tokens == 0 {
                    errors.push(ValidationError {
                        field: "reasoning.budgetTokens".to_string(),
                        message: "Budget tokens must be greater than 0 when reasoning is enabled".to_string(),
                    });
                }
            }

            if let ReasoningConfig::OpenAI { effort } = reasoning {
                let valid_efforts = ["none", "minimal", "low", "medium", "high", "xhigh"];
                if !valid_efforts.contains(&effort.as_str()) {
                    errors.push(ValidationError {
                        field: "reasoning.reasoningEffort".to_string(),
                        message: format!(
                            "Invalid reasoning effort: {}. Valid options: {}",
                            effort,
                            valid_efforts.join(", ")
                        ),
                    });
                }
            }

            if let ReasoningConfig::Google {
                thinking_level,
                thinking_budget,
            } = reasoning
            {
                if let Some(level) = thinking_level {
                    let valid_levels = ["low", "high", "medium"];
                    if !valid_levels.contains(&level.as_str()) {
                        errors.push(ValidationError {
                            field: "reasoning.thinkingLevel".to_string(),
                            message: format!(
                                "Invalid thinking level: {}. Valid options: {}",
                                level,
                                valid_levels.join(", ")
                            ),
                        });
                    }
                }
                if let Some(budget) = thinking_budget {
                    if *budget > 24576 {
                        errors.push(ValidationError {
                            field: "reasoning.thinkingBudget".to_string(),
                            message: "Thinking budget must not exceed 24576 for Google models".to_string(),
                        });
                    }
                }
            }
        }

        errors
    }
}

impl Default for ModelOptions {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TreeTraversal {
    flat_nodes: Vec<(TreeNode, usize)>,
    current_index: usize,
}

impl TreeTraversal {
    pub fn new(root: &TreeNode) -> Self {
        let mut flat_nodes = Vec::new();
        flatten_tree(root, 0, &mut flat_nodes);
        Self {
            flat_nodes,
            current_index: 0,
        }
    }

    pub fn current(&self) -> Option<&TreeNode> {
        self.flat_nodes.get(self.current_index).map(|(node, _)| node)
    }

    pub fn move_down(&mut self) -> bool {
        if self.current_index < self.flat_nodes.len() - 1 {
            self.current_index += 1;
            true
        } else {
            false
        }
    }

    pub fn move_up(&mut self) -> bool {
        if self.current_index > 0 {
            self.current_index -= 1;
            true
        } else {
            false
        }
    }

    pub fn move_to(&mut self, index: usize) -> bool {
        if index < self.flat_nodes.len() {
            self.current_index = index;
            true
        } else {
            false
        }
    }

    pub fn find_path_mut(&mut self, path: &[String]) -> Option<&mut TreeNode> {
        for (i, (node, _)) in self.flat_nodes.iter_mut().enumerate() {
            if check_path(node, path) {
                self.current_index = i;
                return Some(node);
            }
        }
        None
    }
}

fn flatten_tree(node: &TreeNode, depth: usize, result: &mut Vec<(TreeNode, usize)>) {
    result.push((node.clone(), depth));
    if node.is_expanded() {
        for child in &node.children {
            flatten_tree(child, depth + 1, result);
        }
    }
}

fn check_path(node: &TreeNode, path: &[String]) -> bool {
    if path.is_empty() {
        return true;
    }

    match &node.kind {
        TreeNodeKind::Provider(ref name) => {
            name == &path[0] && (path.len() == 1 || check_children(node, &path[1..]))
        }
        TreeNodeKind::Model(ref name) => {
            name == &path[0] && (path.len() == 1 || check_children(node, &path[1..]))
        }
        TreeNodeKind::Variant(ref name) => name == &path[0] && path.len() == 1,
    }
}

fn check_children(node: &TreeNode, path: &[String]) -> bool {
    node.children.iter().any(|child| check_path(child, path))
}

pub fn extract_tree_path(node: &TreeNode) -> Vec<String> {
    let mut path = Vec::new();
    let mut current = Some(node);

    while let Some(n) = current {
        path.push(n.name.clone());
        current = None;
    }

    path.reverse();
    path
}

pub fn build_tree_from_opencode_config(config: &crate::config::OpenCodeConfig) -> TreeNode {
    let mut root = TreeNode::new(TreeNodeKind::Provider("root".to_string()), "root".to_string());
    root.expanded = true;

    for (provider_name, provider_config) in &config.provider {
        let mut provider_node = TreeNode::provider(provider_name.clone(), "".to_string());
        provider_node.expanded = true;

        if let Some(models) = provider_config.get("models").and_then(|v| v.as_object()) {
            for (model_name, model_config) in models {
                let mut model_node = TreeNode::model(model_name.clone(), String::new(), ModelCapabilities {
                    reasoning: false,
                    temperature: false,
                    tool_call: false,
                });
                model_node.expanded = true;

                if let Some(variants) = model_config.get("variants").and_then(|v| v.as_object()) {
                    for (variant_name, variant_config) in variants {
                        model_node.add_child(TreeNode::variant(variant_name.clone(), variant_config.clone()));
                    }
                }

                model_node.add_child(TreeNode::new(TreeNodeKind::Variant("[new variant]".to_string()), "[new variant]".to_string()));
                provider_node.add_child(model_node);
            }
        }

        provider_node.add_child(TreeNode::new(TreeNodeKind::Model("[new model]".to_string()), "[new model]".to_string()));
        root.add_child(provider_node);
    }

    root
}

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

pub fn get_cached_models(cache_dir: &Path) -> Result<Option<crate::cache::ModelsCache>> {
    let cache_manager = CacheManager::new(cache_dir);
    Ok(cache_manager.load_cached()?)
}

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

pub fn show_current_models() -> Result<()> {
    use directories::ProjectDirs;

    let proj_dirs = ProjectDirs::from("org", "opencode", "opencode")
        .expect("Failed to determine project directories");

    let state_dir = proj_dirs
        .state_dir()
        .expect("Failed to determine state directory");
    let model_state_path = state_dir.join("model.json");

    if !model_state_path.exists() {
        println!("No OpenCode model state found at: {}", model_state_path.display());
        println!("Make sure OpenCode has been run at least once.");
        return Ok(());
    }

    let contents = std::fs::read_to_string(&model_state_path)
        .with_context(|| format!("Failed to read model state from {}", model_state_path.display()))?;

    let state: OpenCodeModelState = serde_json::from_str(&contents)
        .with_context(|| "Failed to parse OpenCode model state JSON")?;

    println!("OpenCode Current Models\n");

    if !state.recent.is_empty() {
        println!("\n📋 Recently Used Models:");
        for (i, model_ref) in state.recent.iter().enumerate() {
            println!("  {}. {}/{}", i + 1, model_ref.provider_id, model_ref.model_id);

            let model_key = format!("{}/{}", model_ref.provider_id, model_ref.model_id);
            if let Some(variant) = state.variant.get(&model_key) {
                println!("     Variant: {}", variant);
            }
        }
    } else {
        println!("\n📋 No recent models found");
    }

    if !state.favorite.is_empty() {
        println!("\n⭐ Favorite Models:");
        for (i, model_ref) in state.favorite.iter().enumerate() {
            println!("  {}. {}/{}", i + 1, model_ref.provider_id, model_ref.model_id);

            let model_key = format!("{}/{}", model_ref.provider_id, model_ref.model_id);
            if let Some(variant) = state.variant.get(&model_key) {
                println!("     Variant: {}", variant);
            }
        }
    } else {
        println!("\n⭐ No favorite models set");
    }

    if !state.variant.is_empty() {
        println!("\n⚙️  Model Variants ({} configured):", state.variant.len());
        for (model_key, variant) in state.variant.iter() {
            println!("  {} → {}", model_key, variant);
        }
    }

    println!("\nModel state file: {}", model_state_path.display());

    Ok(())
}
