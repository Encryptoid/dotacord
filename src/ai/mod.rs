pub mod chat;
pub mod tools;

use std::sync::OnceLock;

use llm::builder::{LLMBackend, LLMBuilder};
use llm::chat::ToolChoice;
use llm::LLMProvider;
use tracing::info;

use crate::config::AnthropicConfig;
use crate::Error;

struct AiConfig {
    api_key: String,
    model: String,
    max_tokens: u32,
    reasoning_budget_tokens: u32,
    base_system_prompt: String,
    add_players_context: bool,
}

static AI_CONFIG: OnceLock<AiConfig> = OnceLock::new();

pub fn init_client(config: &AnthropicConfig) -> Result<(), Error> {
    let api_key = std::env::var(&config.api_key_var).map_err(|e| {
        format!("Failed to read env var '{}': {}", &config.api_key_var, e)
    })?;

    let base_system_prompt = std::fs::read_to_string(&config.system_prompt_path).map_err(|e| {
        format!("Failed to read system prompt '{}': {}", &config.system_prompt_path, e)
    })?;

    AI_CONFIG
        .set(AiConfig {
            api_key,
            model: config.model.clone(),
            max_tokens: config.max_tokens,
            reasoning_budget_tokens: config.reasoning_budget_tokens,
            base_system_prompt,
            add_players_context: config.add_players_context,
        })
        .map_err(|_already| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "AI config already initialized",
            )) as Error
        })?;

    info!("AI client initialized");
    Ok(())
}

pub fn build_client(extra_context: &str) -> Result<Box<dyn LLMProvider>, Error> {
    let config = AI_CONFIG.get().ok_or_else(|| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "AI config not initialized. Call init_client(...) at startup.",
        )) as Error
    })?;

    let system_prompt = if extra_context.is_empty() {
        config.base_system_prompt.clone()
    } else {
        format!("{}\n\n{}", config.base_system_prompt, extra_context)
    };

    let client = LLMBuilder::new()
        .backend(LLMBackend::Anthropic)
        .api_key(&config.api_key)
        .model(&config.model)
        .max_tokens(config.max_tokens)
        .reasoning_budget_tokens(config.reasoning_budget_tokens)
        .system(system_prompt)
        .function(tools::get_recent_matches_tool())
        .function(tools::get_match_details_tool())
        .function(tools::get_hero_by_nickname_tool())
        .function(tools::top_winrate_heroes_tool())
        .function(tools::get_global_hero_stats_tool())
        .function(tools::get_player_hero_stats_tool())
        .tool_choice(ToolChoice::Auto)
        .build()?;

    Ok(client)
}

pub fn add_players_context() -> bool {
    AI_CONFIG.get().map_or(false, |c| c.add_players_context)
}
