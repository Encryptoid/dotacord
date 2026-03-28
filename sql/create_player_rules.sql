CREATE TABLE IF NOT EXISTS player_rules
(
    id              INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    server_id       BIGINT  NOT NULL,
    discord_user_id BIGINT  NOT NULL,
    rule_text       TEXT    NOT NULL,

    FOREIGN KEY (server_id) REFERENCES servers (server_id)
);
