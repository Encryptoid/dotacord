pub mod chat;

use std::sync::OnceLock;

use llm::builder::{LLMBackend, LLMBuilder};
use llm::LLMProvider;
use tracing::info;

use crate::config::AnthropicConfig;
use crate::Error;

static LLM_CLIENT: OnceLock<Box<dyn LLMProvider>> = OnceLock::new();

pub fn init_client(config: &AnthropicConfig) -> Result<(), Error> {
    let api_key = std::env::var(&config.api_key_var).map_err(|e| {
        format!("Failed to read env var '{}': {}", &config.api_key_var, e)
    })?;

    let client = LLMBuilder::new()
        .backend(LLMBackend::Anthropic)
        .api_key(&api_key)
        .model(&config.model)
        .max_tokens(config.max_tokens)
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
