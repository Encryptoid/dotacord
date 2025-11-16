use sea_orm::*;

use crate::database::entities::{server, Server};
use crate::Error;

pub use server::Model as DiscordServer;

pub async fn query_server_by_id(
    db: &DatabaseTransaction,
    server_id: i64,
) -> Result<Option<DiscordServer>, Error> {
    let server = Server::find_by_id(server_id).one(db).await?;
    Ok(server)
}

pub async fn insert_server(
    db: &DatabaseTransaction,
    server_id: i64,
    server_name: String,
    channel_id: Option<i64>,
) -> Result<(), Error> {
    let new_server = server::ActiveModel {
        server_id: Set(server_id),
        server_name: Set(server_name),
        channel_id: Set(channel_id),
        is_sub_week: Set(0),
        is_sub_month: Set(0),
    };

    Server::insert(new_server).exec(db).await?;
    Ok(())
}

pub async fn query_all_servers(db: &DatabaseTransaction) -> Result<Vec<DiscordServer>, Error> {
    let servers = Server::find()
        .order_by_asc(server::Column::ServerName)
        .all(db)
        .await?;

    Ok(servers)
}

pub async fn update_server_channel(
    db: &DatabaseTransaction,
    server_id: i64,
    channel_id: i64,
) -> Result<(), Error> {
    let server = Server::find_by_id(server_id).one(db).await?;
    
    if let Some(s) = server {
        let mut s_active: server::ActiveModel = s.into();
        s_active.channel_id = Set(Some(channel_id));
        s_active.update(db).await?;
    }

    Ok(())
}

pub async fn update_server_sub_week(
    db: &DatabaseTransaction,
    server_id: i64,
    is_sub_week: bool,
) -> Result<(), Error> {
    let server = Server::find_by_id(server_id).one(db).await?;
    
    if let Some(s) = server {
        let mut s_active: server::ActiveModel = s.into();
        s_active.is_sub_week = Set(if is_sub_week { 1 } else { 0 });
        s_active.update(db).await?;
    }

    Ok(())
}

pub async fn update_server_sub_month(
    db: &DatabaseTransaction,
    server_id: i64,
    is_sub_month: bool,
) -> Result<(), Error> {
    let server = Server::find_by_id(server_id).one(db).await?;
    
    if let Some(s) = server {
        let mut s_active: server::ActiveModel = s.into();
        s_active.is_sub_month = Set(if is_sub_month { 1 } else { 0 });
        s_active.update(db).await?;
    }

    Ok(())
}
