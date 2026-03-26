use llm::chat::ChatMessage;

use crate::Error;

#[tracing::instrument(level = "trace", skip(user_text))]
pub async fn send_message(user_text: &str) -> Result<String, Error> {
    let client = super::get_client()?;
    let messages = vec![
        ChatMessage::user().content(user_text).build(),
    ];
    let response = client.chat(&messages).await?;
    response.text()
        .map(|t| t.to_string())
        .ok_or_else(|| "No text in AI response".into())
}
