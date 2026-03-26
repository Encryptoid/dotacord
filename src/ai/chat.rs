use llm::chat::ChatMessage;

use crate::database::chat_messages_db::ChatMessageModel;
use crate::Error;

#[tracing::instrument(level = "trace", skip(history, new_user_message))]
pub async fn send_message(history: &[ChatMessageModel], new_user_message: &str) -> Result<String, Error> {
    let client = super::get_client()?;

    let mut messages: Vec<_> = history.iter().map(|m| {
        match m.role.as_str() {
            "assistant" => ChatMessage::assistant().content(&m.content).build(),
            _ => ChatMessage::user().content(&m.content).build(),
        }
    }).collect();

    messages.push(ChatMessage::user().content(new_user_message).build());

    let response = client.chat(&messages).await?;
    response.text()
        .map(|t| t.to_string())
        .ok_or_else(|| "No text in AI response".into())
}
