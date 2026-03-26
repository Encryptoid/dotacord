use poise::serenity_prelude::{self as serenity, async_trait, Context, FullEvent};

use crate::ai::tools::ToolContext;
use crate::database::{chat_messages_db, database_access, player_servers_db};

pub struct MentionHandler {
    pub max_conversation_messages: u32,
    pub max_recent_match_days: u64,
    pub max_recent_matches: usize,
}

#[async_trait]
impl serenity::EventHandler for MentionHandler {
    async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
        if let FullEvent::Message { new_message, .. } = event {
            if new_message.author.bot() {
                return;
            }

            let replied_to = check_reply_to_tracked_message(new_message).await;
            let mentioned = new_message.mentions_me(&ctx).await.unwrap_or(false);

            if replied_to.is_none() && !mentioned {
                return;
            }

            let display_name = resolve_display_name(new_message).await;
            let text = strip_mentions(&new_message.content);
            if text.is_empty() {
                return;
            }

            let user_text = format!("@{display_name}: {text}");
            tracing::info!(user = %new_message.author.name, display_name = %display_name, content = %new_message.content, "Received AI chat message");

            match replied_to {
                Some(replied_msg) => {
                    self.handle_continuation(ctx, new_message, &replied_msg, &user_text).await;
                }
                None => {
                    self.handle_new_conversation(ctx, new_message, &user_text).await;
                }
            }
        }
    }
}

impl MentionHandler {
    async fn handle_new_conversation(
        &self,
        ctx: &Context,
        new_message: &serenity::Message,
        user_text: &str,
    ) {
        let conversation_id = new_message.id.get() as i64;
        tracing::info!(message_text = %user_text, "Sending new conversation to AI");

        let tool_ctx = self.build_tool_context(new_message);
        match crate::ai::chat::send_message(&[], user_text, &tool_ctx).await {
            Ok(response) => {
                tracing::info!(response = %response, "AI response");
                match new_message.reply(&ctx.http, &response).await {
                    Ok(bot_reply) => {
                        if let Err(e) = self.persist_messages(
                            conversation_id,
                            new_message,
                            user_text,
                            &bot_reply,
                            &response,
                        ).await {
                            tracing::error!("Failed to persist chat messages: {:?}", e);
                        }
                    }
                    Err(e) => tracing::error!("Failed to reply with AI response: {:?}", e),
                }
            }
            Err(e) => tracing::error!("AI chat error: {:?}", e),
        }
    }

    async fn handle_continuation(
        &self,
        ctx: &Context,
        new_message: &serenity::Message,
        replied_msg: &chat_messages_db::ChatMessageModel,
        user_text: &str,
    ) {
        let conversation_id = replied_msg.conversation_id;

        let latest = match chat_messages_db::query_latest_in_conversation(conversation_id).await {
            Ok(Some(latest)) => latest,
            Ok(None) => return,
            Err(e) => {
                tracing::error!("Failed to query latest message: {:?}", e);
                return;
            }
        };

        if latest.discord_message_id != replied_msg.discord_message_id {
            if let Err(e) = new_message.reply(&ctx.http, "Please reply to the latest message in the conversation to continue.").await {
                tracing::error!("Failed to send error reply: {:?}", e);
            }
            return;
        }

        let history = match chat_messages_db::query_last_n_messages(
            conversation_id,
            self.max_conversation_messages as u64,
        ).await {
            Ok(h) => h,
            Err(e) => {
                tracing::error!("Failed to query conversation history: {:?}", e);
                return;
            }
        };

        tracing::info!(message_text = %user_text, history_len = history.len(), "Sending continuation to AI");

        let tool_ctx = self.build_tool_context(new_message);
        match crate::ai::chat::send_message(&history, user_text, &tool_ctx).await {
            Ok(response) => {
                tracing::info!(response = %response, "AI response");
                match new_message.reply(&ctx.http, &response).await {
                    Ok(bot_reply) => {
                        if let Err(e) = self.persist_messages(
                            conversation_id,
                            new_message,
                            user_text,
                            &bot_reply,
                            &response,
                        ).await {
                            tracing::error!("Failed to persist chat messages: {:?}", e);
                        }
                    }
                    Err(e) => tracing::error!("Failed to reply with AI response: {:?}", e),
                }
            }
            Err(e) => tracing::error!("AI chat error: {:?}", e),
        }
    }

    fn build_tool_context(&self, message: &serenity::Message) -> ToolContext {
        ToolContext {
            server_id: message.guild_id.map(|g| g.get() as i64).unwrap_or(0),
            max_recent_match_days: self.max_recent_match_days,
            max_recent_matches: self.max_recent_matches,
        }
    }

    async fn persist_messages(
        &self,
        conversation_id: i64,
        user_message: &serenity::Message,
        user_text: &str,
        bot_reply: &serenity::Message,
        bot_text: &str,
    ) -> Result<(), crate::Error> {
        let txn = database_access::get_transaction().await?;
        let now = chrono::Utc::now().timestamp();

        chat_messages_db::insert_message(
            &txn,
            conversation_id,
            user_message.id.get() as i64,
            user_message.channel_id.get() as i64,
            user_message.author.id.get() as i64,
            "user",
            user_text,
            now,
        ).await?;

        chat_messages_db::insert_message(
            &txn,
            conversation_id,
            bot_reply.id.get() as i64,
            bot_reply.channel_id.get() as i64,
            bot_reply.author.id.get() as i64,
            "assistant",
            bot_text,
            now,
        ).await?;

        txn.commit().await?;
        Ok(())
    }
}

async fn check_reply_to_tracked_message(
    message: &serenity::Message,
) -> Option<chat_messages_db::ChatMessageModel> {
    let msg_ref = message.message_reference.as_ref()?;
    let replied_to_id = msg_ref.message_id?.get() as i64;
    chat_messages_db::query_message_by_discord_id(replied_to_id).await.ok()?
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
