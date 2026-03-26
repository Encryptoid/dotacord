CREATE TABLE IF NOT EXISTS chat_messages
(
    id                 INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    conversation_id    BIGINT  NOT NULL,
    discord_message_id BIGINT  NOT NULL UNIQUE,
    channel_id         BIGINT  NOT NULL,
    user_id            BIGINT  NOT NULL,
    role               TEXT    NOT NULL,
    content            TEXT    NOT NULL,
    created_at         INTEGER NOT NULL
);
