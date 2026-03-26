use poise::serenity_prelude::{self as serenity, async_trait, Context, FullEvent};

use crate::database::player_servers_db;

pub struct MentionHandler;

#[async_trait]
impl serenity::EventHandler for MentionHandler {
    async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
        if let FullEvent::Message { new_message, .. } = event {
            if new_message.author.bot() {
                return;
            }

            let mentioned = new_message.mentions_me(&ctx).await.unwrap_or(false);
            if mentioned {
                let display_name = resolve_display_name(new_message).await;
                tracing::info!(user = %new_message.author.name, display_name = %display_name, content = %new_message.content, "Received mention");
                let text = strip_mentions(&new_message.content);
                if text.is_empty() {
                    return;
                }
                let message_text = format!("@{display_name}: {text}");
                tracing::info!(message_text = %message_text, "Sending to AI");
                match crate::ai::chat::send_message(&message_text).await {
                    Ok(response) => {
                        tracing::info!(response = %response, "AI response");
                        if let Err(e) = new_message.reply(&ctx.http, &response).await {
                            tracing::error!("Failed to reply with AI response: {:?}", e);
                        }
                    }
                    Err(e) => {
                        tracing::error!("AI chat error: {:?}", e);
                    }
                }
            }
        }
    }
}

async fn resolve_display_name(message: &serenity::Message) -> String {
    if let Some(guild_id) = message.guild_id {
        if let Ok(Some(player)) = player_servers_db::query_player_by_discord_user(
            guild_id.get() as i64,
            message.author.id.get() as i64,
        ).await {
            return player.discord_name;
        }
    }
    message.author.display_name().to_string()
}

fn strip_mentions(content: &str) -> String {
    let mut result = content.to_string();
    while let Some(start) = result.find("<@") {
        if let Some(end) = result[start..].find('>') {
            result = format!("{}{}", &result[..start], &result[start + end + 1..]);
        } else {
            break;
        }
    }
    result.trim().to_string()
}
