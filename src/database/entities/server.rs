use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "servers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub server_id: i64,
    pub server_name: String,
    pub channel_id: Option<i64>,
    pub is_sub_week: i32,
    pub is_sub_month: i32,
    pub is_sub_reload: i32,
    pub weekly_day: Option<i32>,
    pub weekly_hour: Option<i32>,
    pub monthly_week: Option<i32>,
    pub monthly_weekday: Option<i32>,
    pub monthly_hour: Option<i32>,
    pub reload_start: Option<i32>,
    pub reload_end: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
