use llm::chat::ChatMessage;
use llm::{FunctionCall, ToolCall};
use tracing::info;

use super::tools::{self, ToolContext};
use crate::database::chat_messages_db::ChatMessageModel;
use crate::Error;

#[tracing::instrument(level = "trace", skip(history, new_user_message, tool_ctx))]
pub async fn send_message(
    history: &[ChatMessageModel],
    new_user_message: &str,
    tool_ctx: &ToolContext,
) -> Result<String, Error> {
    let client = super::get_client()?;

    let mut messages: Vec<_> = history
        .iter()
        .map(|m| match m.role.as_str() {
            "assistant" => ChatMessage::assistant().content(&m.content).build(),
            _ => ChatMessage::user().content(&m.content).build(),
        })
        .collect();

    messages.push(ChatMessage::user().content(new_user_message).build());

    for round in 0..tools::max_tool_rounds() {
        let response = client.chat(&messages).await?;

        match response.tool_calls() {
            Some(tool_calls) if !tool_calls.is_empty() => {
                info!(round, tool_count = tool_calls.len(), "LLM requested tool calls");

                messages.push(
                    ChatMessage::assistant()
                        .tool_use(tool_calls.clone())
                        .build(),
                );

                let mut results = Vec::new();
                for tc in &tool_calls {
                    let result_json = match tools::execute_tool(tc, tool_ctx).await {
                        Ok(json) => json,
                        Err(e) => format!("{{\"error\": \"{}\"}}", e),
                    };

                    info!(tool = %tc.function.name, result = %result_json, "Tool executed");

                    results.push(ToolCall {
                        id: tc.id.clone(),
                        call_type: tc.call_type.clone(),
                        function: FunctionCall {
                            name: tc.function.name.clone(),
                            arguments: result_json,
                        },
                    });
                }

                messages.push(ChatMessage::user().tool_result(results).build());
            }
            _ => {
                return response
                    .text()
                    .map(|t| t.to_string())
                    .ok_or_else(|| "No text in AI response".into());
            }
        }
    }

    Err("Too many tool call rounds".into())
}
