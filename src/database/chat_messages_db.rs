use sea_orm::*;

use crate::database::database_access;
use crate::database::entities::{chat_message, ChatMessage};
use crate::Error;

pub use chat_message::Model as ChatMessageModel;

pub async fn query_message_by_discord_id(
    discord_message_id: i64,
) -> Result<Option<ChatMessageModel>, Error> {
    let txn = database_access::get_transaction().await?;
    let row = ChatMessage::find()
        .filter(chat_message::Column::DiscordMessageId.eq(discord_message_id))
        .one(&txn)
        .await?;
    Ok(row)
}

pub async fn query_latest_in_conversation(
    conversation_id: i64,
) -> Result<Option<ChatMessageModel>, Error> {
    let txn = database_access::get_transaction().await?;
    let row = ChatMessage::find()
        .filter(chat_message::Column::ConversationId.eq(conversation_id))
        .order_by_desc(chat_message::Column::Id)
        .one(&txn)
        .await?;
    Ok(row)
}

pub async fn query_last_n_messages(
    conversation_id: i64,
    limit: u64,
) -> Result<Vec<ChatMessageModel>, Error> {
    let txn = database_access::get_transaction().await?;
    let mut rows = ChatMessage::find()
        .filter(chat_message::Column::ConversationId.eq(conversation_id))
        .order_by_desc(chat_message::Column::Id)
        .limit(limit)
        .all(&txn)
        .await?;
    rows.reverse();
    Ok(rows)
}

pub async fn insert_message(
    txn: &DatabaseTransaction,
    conversation_id: i64,
    discord_message_id: i64,
    channel_id: i64,
    user_id: i64,
    role: &str,
    content: &str,
    created_at: i64,
) -> Result<(), Error> {
    let new_msg = chat_message::ActiveModel {
        id: NotSet,
        conversation_id: Set(conversation_id),
        discord_message_id: Set(discord_message_id),
        channel_id: Set(channel_id),
        user_id: Set(user_id),
        role: Set(role.to_string()),
        content: Set(content.to_string()),
        created_at: Set(created_at),
    };
    ChatMessage::insert(new_msg).exec(txn).await?;
    Ok(())
}
