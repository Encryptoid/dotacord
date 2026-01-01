use sea_orm::*;

use crate::database::database_access;
use crate::database::entities::{command_event, CommandEvent};
use crate::Error;

pub use command_event::Model as CommandEventModel;

#[derive(Debug, Clone)]
pub enum EventType {
    UserRefresh,
    AdminRefresh,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::UserRefresh => "UserRefresh",
            EventType::AdminRefresh => "AdminRefresh",
        }
    }
}

pub async fn query_last_event(
    server_id: i64,
    event_type: EventType,
    user_id: Option<i64>,
) -> Result<Option<CommandEventModel>, Error> {
    let txn = database_access::get_transaction().await?;

    let mut query = CommandEvent::find()
        .filter(command_event::Column::ServerId.eq(server_id))
        .filter(command_event::Column::EventType.eq(event_type.as_str()));

    if let Some(uid) = user_id {
        query = query.filter(command_event::Column::UserId.eq(uid));
    }

    let event = query
        .order_by_desc(command_event::Column::EventTime)
        .one(&txn)
        .await?;

    Ok(event)
}

pub async fn insert_event(
    server_id: i64,
    event_type: EventType,
    user_id: i64,
    event_time: i64,
) -> Result<(), Error> {
    let txn = database_access::get_transaction().await?;

    let new_event = command_event::ActiveModel {
        event_id: NotSet,
        server_id: Set(server_id),
        event_type: Set(event_type.as_str().to_string()),
        event_time: Set(event_time),
        user_id: Set(user_id),
    };

    CommandEvent::insert(new_event).exec(&txn).await?;
    txn.commit().await?;
    Ok(())
}
