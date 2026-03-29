use llm::chat::ChatMessage;
use llm::{FunctionCall, ToolCall};
use tracing::info;

use super::tools::{self, ToolContext};
use crate::database::chat_messages_db::ChatMessageModel;
use crate::database::{player_rules_db, player_servers_db};
use crate::Error;

#[tracing::instrument(level = "trace", skip(history, new_user_message, tool_ctx))]
pub async fn send_message(
    history: &[ChatMessageModel],
    new_user_message: &str,
    tool_ctx: &ToolContext,
) -> Result<String, Error> {
    let rules_context = if super::add_players_context() {
        build_rules_context(tool_ctx.server_id).await?
    } else {
        String::new()
    };
    let client = super::build_client(&rules_context)?;

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

async fn build_rules_context(server_id: i64) -> Result<String, Error> {
    let players = player_servers_db::query_server_players(server_id).await?;
    if players.is_empty() {
        return Ok(String::new());
    }

    let rules = player_rules_db::query_rules_by_server(server_id).await?;

    let mut rules_by_user: std::collections::HashMap<i64, Vec<&str>> =
        std::collections::HashMap::new();
    for rule in &rules {
        rules_by_user
            .entry(rule.discord_user_id)
            .or_default()
            .push(&rule.rule_text);
    }

    let mut output = String::from(
        "## Registered Players\n\nThese are the players registered on this Discord server. \
         Messages from them will appear as `@DisplayName: message`. \
         When a user asks about themselves (e.g. \"my stats\", \"how am I doing\"), use their @DisplayName for tool calls. \
         When they mention another player, it will appear as @OtherName in the message. \
         Follow any rules listed under each player.\n",
    );
    for player in &players {
        let display_name = player
            .player_name
            .as_deref()
            .unwrap_or(&player.discord_name);
        output.push_str(&format!("\n### {}\n", display_name));

        if let Some(discord_user_id) = player.discord_user_id {
            if let Some(rule_texts) = rules_by_user.get(&discord_user_id) {
                for text in rule_texts {
                    output.push_str(&format!("- {}\n", text));
                }
            }
        }
    }

    Ok(output)
}
