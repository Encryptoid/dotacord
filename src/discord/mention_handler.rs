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
                if let Err(e) = new_message.reply(&ctx.http, "Hello World!").await {
                    tracing::error!("Failed to reply to mention: {:?}", e);
                }
            }
        }
    }
}
