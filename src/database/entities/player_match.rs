use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "player_matches")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub match_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub player_id: i64,
    pub hero_id: i32,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub rank: i32,
    pub party_size: i32,
    pub faction: i32,
    pub is_victory: bool,
    pub start_time: i64,
    pub duration: i32,
    pub game_mode: i32,
    pub lobby_type: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::player::Entity",
        from = "Column::PlayerId",
        to = "super::player::Column::PlayerId"
    )]
    Player,
}

impl Related<super::player::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Player.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
