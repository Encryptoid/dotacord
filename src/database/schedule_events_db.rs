use sea_orm::*;

use crate::database::database_access;
use crate::database::entities::{schedule_event, ScheduleEvent};
use crate::Error;

pub use schedule_event::Model as ScheduleEventModel;

#[derive(Debug, Clone)]
pub enum EventType {
    LeaderboardWeek,
    LeaderboardMonth,
    Reload,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::LeaderboardWeek => "LeaderboardWeek",
            EventType::LeaderboardMonth => "LeaderboardMonth",
            EventType::Reload => "Reload",
        }
    }
}

#[derive(Debug, Clone)]
pub enum EventSource {
    Manual,
    Schedule,
}

impl EventSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventSource::Manual => "Manual",
            EventSource::Schedule => "Schedule",
        }
    }
}

pub async fn query_last_event(
    server_id: i64,
    event_type: EventType,
) -> Result<Option<ScheduleEventModel>, Error> {
    let txn = database_access::get_transaction().await?;
    let event = ScheduleEvent::find()
        .filter(schedule_event::Column::ServerId.eq(server_id))
        .filter(schedule_event::Column::EventType.eq(event_type.as_str()))
        .order_by_desc(schedule_event::Column::EventTime)
        .one(&txn)
        .await?;

    Ok(event)
}

pub async fn insert_event(
    server_id: i64,
    event_type: EventType,
    event_source: EventSource,
    event_time: i64,
) -> Result<(), Error> {
    let txn = database_access::get_transaction().await?;

    let new_event = schedule_event::ActiveModel {
        event_id: NotSet,
        server_id: Set(server_id),
        event_type: Set(event_type.as_str().to_string()),
        event_source: Set(event_source.as_str().to_string()),
        event_time: Set(event_time),
    };

    ScheduleEvent::insert(new_event).exec(&txn).await?;
    txn.commit().await?;
    Ok(())
}
