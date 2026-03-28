use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "heroes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub hero_id: i32,
    pub name: String,
    pub is_carry: bool,
    pub is_mid: bool,
    pub is_offlane: bool,
    pub is_support: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
