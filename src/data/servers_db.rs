use sqlx::{Error, FromRow, SqliteConnection};

#[derive(Debug, Clone, FromRow)]
pub(crate) struct DiscordServer {
    pub server_id: i64,
    pub server_name: Option<String>,
    pub channel_id: Option<i64>,
}

pub async fn query_server_by_id(
    conn: &mut SqliteConnection,
    server_id: i64,
) -> Result<Option<DiscordServer>, Error> {
    let server = sqlx::query_as::<_, DiscordServer>(
        r#"
            SELECT server_id, server_name, channel_id
            FROM servers
            WHERE server_id = ?
        "#,
    )
    .bind(server_id)
    .fetch_optional(conn)
    .await?;

    Ok(server)
}

pub async fn insert_server(
    conn: &mut SqliteConnection,
    server_id: i64,
    server_name: String,
    channel_id: Option<i64>,
) -> Result<(), Error> {
    sqlx::query(
        r#"
            INSERT INTO servers (server_id, server_name, channel_id)
            VALUES (?, ?, ?)
        "#,
    )
    .bind(server_id)
    .bind(server_name)
    .bind(channel_id)
    .execute(conn)
    .await?;
    Ok(())
}
