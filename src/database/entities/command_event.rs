use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "command_events")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub event_id: i64,
    pub server_id: i64,
    pub event_type: String,
    pub event_time: i64,
    pub user_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
