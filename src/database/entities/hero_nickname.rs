use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "hero_nicknames")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub hero_id: i32,
    pub nickname: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::hero::Entity",
        from = "Column::HeroId",
        to = "super::hero::Column::HeroId"
    )]
    Hero,
}

impl Related<super::hero::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Hero.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
