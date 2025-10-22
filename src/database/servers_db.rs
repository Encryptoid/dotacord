use sqlx::{Error, FromRow, SqliteConnection};

#[derive(Debug, Clone, FromRow)]
pub(crate) struct DiscordServer {
    pub server_id: i64,
    pub server_name: Option<String>,
    pub channel_id: Option<i64>,
    pub is_sub_week: i64,
    pub is_sub_month: i64,
}

pub async fn query_server_by_id(
    conn: &mut SqliteConnection,
    server_id: i64,
) -> Result<Option<DiscordServer>, Error> {
    let server = sqlx::query_as::<_, DiscordServer>(
        r#"
            SELECT server_id, server_name, channel_id, is_sub_week, is_sub_month
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
            INSERT INTO servers (server_id, server_name, channel_id, is_sub_week, is_sub_month)
            VALUES (?, ?, ?, 0, 0)
        "#,
    )
    .bind(server_id)
    .bind(server_name)
    .bind(channel_id)
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn query_all_servers(
    conn: &mut SqliteConnection,
) -> Result<Vec<DiscordServer>, Error> {
    let servers = sqlx::query_as::<_, DiscordServer>(
        r#"
            SELECT server_id, server_name, channel_id, is_sub_week, is_sub_month
            FROM servers
            ORDER BY server_name
        "#,
    )
    .fetch_all(conn)
    .await?;

    Ok(servers)
}

pub async fn update_server_channel(
    conn: &mut SqliteConnection,
    server_id: i64,
    channel_id: i64,
) -> Result<(), Error> {
    sqlx::query(
        r#"
            UPDATE servers
            SET channel_id = ?
            WHERE server_id = ?
        "#,
    )
    .bind(channel_id)
    .bind(server_id)
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn update_server_sub_week(
    conn: &mut SqliteConnection,
    server_id: i64,
    is_sub_week: bool,
) -> Result<(), Error> {
    sqlx::query(
        r#"
            UPDATE servers
            SET is_sub_week = ?
            WHERE server_id = ?
        "#,
    )
    .bind(is_sub_week as i64)
    .bind(server_id)
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn update_server_sub_month(
    conn: &mut SqliteConnection,
    server_id: i64,
    is_sub_month: bool,
) -> Result<(), Error> {
    sqlx::query(
        r#"
            UPDATE servers
            SET is_sub_month = ?
            WHERE server_id = ?
        "#,
    )
    .bind(is_sub_month as i64)
    .bind(server_id)
    .execute(conn)
    .await?;
    Ok(())
}
