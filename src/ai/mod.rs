pub mod chat;
pub mod tools;

use std::sync::OnceLock;

use llm::builder::{LLMBackend, LLMBuilder};
use llm::chat::ToolChoice;
use llm::LLMProvider;
use tracing::info;

use crate::config::AnthropicConfig;
use crate::Error;

static LLM_CLIENT: OnceLock<Box<dyn LLMProvider>> = OnceLock::new();

pub fn init_client(config: &AnthropicConfig) -> Result<(), Error> {
    let api_key = std::env::var(&config.api_key_var).map_err(|e| {
        format!("Failed to read env var '{}': {}", &config.api_key_var, e)
    })?;

    let system_prompt = std::fs::read_to_string(&config.system_prompt_path).map_err(|e| {
        format!("Failed to read system prompt '{}': {}", &config.system_prompt_path, e)
    })?;

    let client = LLMBuilder::new()
        .backend(LLMBackend::Anthropic)
        .api_key(&api_key)
        .model(&config.model)
        .max_tokens(config.max_tokens)
        .reasoning_budget_tokens(config.reasoning_budget_tokens)
        .system(system_prompt)
        .function(tools::get_recent_matches_tool())
        .function(tools::get_match_details_tool())
        .function(tools::get_hero_by_nickname_tool())
        .tool_choice(ToolChoice::Auto)
        .build()?;

    LLM_CLIENT.set(client).map_err(|_already| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "AI client already initialized",
        )) as Error
    })?;

    info!("AI client initialized");
    Ok(())
}

pub fn get_client() -> Result<&'static Box<dyn LLMProvider>, Error> {
    LLM_CLIENT.get().ok_or_else(|| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "AI client not initialized. Call init_client(...) at startup.",
        )) as Error
    })
}
