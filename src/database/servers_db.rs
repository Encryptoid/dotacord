use sea_orm::*;

use crate::database::database_access;
use crate::database::entities::{server, Server};
use crate::Error;

pub use server::Model as DiscordServer;

pub async fn query_server_by_id(server_id: i64) -> Result<Option<DiscordServer>, Error> {
    let txn = database_access::get_transaction().await?;
    let server = Server::find_by_id(server_id).one(&txn).await?;
    Ok(server)
}

pub async fn query_all_servers() -> Result<Vec<DiscordServer>, Error> {
    let txn = database_access::get_transaction().await?;
    let servers = Server::find()
        .order_by_asc(server::Column::ServerName)
        .all(&txn)
        .await?;

    Ok(servers)
}

pub async fn update_server_channel(server_id: i64, channel_id: i64) -> Result<(), Error> {
    let txn = database_access::get_transaction().await?;
    let server = Server::find_by_id(server_id).one(&txn).await?;

    if let Some(s) = server {
        let mut s_active: server::ActiveModel = s.into();
        s_active.channel_id = Set(Some(channel_id));
        s_active.update(&txn).await?;
    }

    txn.commit().await?;
    Ok(())
}

pub async fn update_server_sub_week(server_id: i64, is_sub_week: i32) -> Result<(), Error> {
    let txn = database_access::get_transaction().await?;
    let server = Server::find_by_id(server_id).one(&txn).await?;

    if let Some(s) = server {
        let mut s_active: server::ActiveModel = s.into();
        s_active.is_sub_week = Set(is_sub_week);
        s_active.update(&txn).await?;
    }

    txn.commit().await?;
    Ok(())
}

pub async fn update_server_sub_month(server_id: i64, is_sub_month: i32) -> Result<(), Error> {
    let txn = database_access::get_transaction().await?;
    let server = Server::find_by_id(server_id).one(&txn).await?;

    if let Some(s) = server {
        let mut s_active: server::ActiveModel = s.into();
        s_active.is_sub_month = Set(is_sub_month);
        s_active.update(&txn).await?;
    }

    txn.commit().await?;
    Ok(())
}

pub async fn update_server_sub_reload(server_id: i64, is_sub_reload: i32) -> Result<(), Error> {
    let txn = database_access::get_transaction().await?;
    let server = Server::find_by_id(server_id).one(&txn).await?;

    if let Some(s) = server {
        let mut s_active: server::ActiveModel = s.into();
        s_active.is_sub_reload = Set(is_sub_reload);
        s_active.update(&txn).await?;
    }

    txn.commit().await?;
    Ok(())
}

pub async fn update_server_weekly_day(server_id: i64, weekly_day: i32) -> Result<(), Error> {
    let txn = database_access::get_transaction().await?;
    let server = Server::find_by_id(server_id).one(&txn).await?;

    if let Some(s) = server {
        let mut s_active: server::ActiveModel = s.into();
        s_active.weekly_day = Set(Some(weekly_day));
        s_active.update(&txn).await?;
    }

    txn.commit().await?;
    Ok(())
}

pub async fn update_server_weekly_hour(server_id: i64, weekly_hour: i32) -> Result<(), Error> {
    let txn = database_access::get_transaction().await?;
    let server = Server::find_by_id(server_id).one(&txn).await?;

    if let Some(s) = server {
        let mut s_active: server::ActiveModel = s.into();
        s_active.weekly_hour = Set(Some(weekly_hour));
        s_active.update(&txn).await?;
    }

    txn.commit().await?;
    Ok(())
}

pub async fn update_server_monthly_week(server_id: i64, monthly_week: i32) -> Result<(), Error> {
    let txn = database_access::get_transaction().await?;
    let server = Server::find_by_id(server_id).one(&txn).await?;

    if let Some(s) = server {
        let mut s_active: server::ActiveModel = s.into();
        s_active.monthly_week = Set(Some(monthly_week));
        s_active.update(&txn).await?;
    }

    txn.commit().await?;
    Ok(())
}

pub async fn update_server_monthly_weekday(server_id: i64, monthly_weekday: i32) -> Result<(), Error> {
    let txn = database_access::get_transaction().await?;
    let server = Server::find_by_id(server_id).one(&txn).await?;

    if let Some(s) = server {
        let mut s_active: server::ActiveModel = s.into();
        s_active.monthly_weekday = Set(Some(monthly_weekday));
        s_active.update(&txn).await?;
    }

    txn.commit().await?;
    Ok(())
}

pub async fn update_server_monthly_hour(server_id: i64, monthly_hour: i32) -> Result<(), Error> {
    let txn = database_access::get_transaction().await?;
    let server = Server::find_by_id(server_id).one(&txn).await?;

    if let Some(s) = server {
        let mut s_active: server::ActiveModel = s.into();
        s_active.monthly_hour = Set(Some(monthly_hour));
        s_active.update(&txn).await?;
    }

    txn.commit().await?;
    Ok(())
}

