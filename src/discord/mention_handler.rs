use poise::serenity_prelude::{self as serenity, async_trait, Context, FullEvent};

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
                tracing::info!(user = %new_message.author.name, content = %new_message.content, "Received mention");
                let text = strip_mentions(&new_message.content);
                if text.is_empty() {
                    return;
                }
                match crate::ai::chat::send_message(&text).await {
                    Ok(response) => {
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
