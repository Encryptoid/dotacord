use std::collections::HashMap;

use sea_orm::*;

use crate::database::database_access;
use crate::database::entities::{hero, Hero};
use crate::Error;

pub use hero::Model as HeroModel;

#[derive(Debug)]
pub enum Position {
    Carry,
    Mid,
    Offlane,
    Support,
}

pub struct HeroLookup {
    heroes: HashMap<i32, HeroModel>,
}

impl HeroLookup {
    pub async fn load() -> Result<Self, Error> {
        let all = query_all_heroes().await?;
        let heroes = all.into_iter().map(|h| (h.hero_id, h)).collect();
        Ok(Self { heroes })
    }

    pub fn get_name(&self, hero_id: i32) -> Option<&str> {
        self.heroes.get(&hero_id).map(|h| h.name.as_str())
    }

    pub fn contains(&self, hero_id: i32) -> bool {
        self.heroes.contains_key(&hero_id)
    }

    pub fn find_by_name(&self, search: &str) -> Option<&HeroModel> {
        let search_lower = search.to_lowercase();
        let search_nospace = search_lower.replace(' ', "");

        self.heroes.values().find(|h| {
            let name_lower = h.name.to_lowercase();
            name_lower == search_lower || name_lower.replace(' ', "") == search_nospace
        })
    }
}

pub async fn query_all_heroes() -> Result<Vec<hero::Model>, Error> {
    let txn = database_access::get_transaction().await?;
    let rows = Hero::find().all(&txn).await?;
    Ok(rows)
}

pub async fn query_hero_by_id(hero_id: i32) -> Result<Option<hero::Model>, Error> {
    let txn = database_access::get_transaction().await?;
    let row = Hero::find_by_id(hero_id).one(&txn).await?;
    Ok(row)
}

pub async fn query_heroes_by_position(position: &Position) -> Result<Vec<hero::Model>, Error> {
    let txn = database_access::get_transaction().await?;

    let column = match position {
        Position::Carry => hero::Column::IsCarry,
        Position::Mid => hero::Column::IsMid,
        Position::Offlane => hero::Column::IsOfflane,
        Position::Support => hero::Column::IsSupport,
    };

    let rows = Hero::find().filter(column.eq(true)).all(&txn).await?;
    Ok(rows)
}

pub async fn update_hero_position(
    hero_id: i32,
    position: &Position,
    enabled: bool,
) -> Result<(), Error> {
    let txn = database_access::get_transaction().await?;

    let mut active: hero::ActiveModel = hero::ActiveModel {
        hero_id: Set(hero_id),
        ..Default::default()
    };

    match position {
        Position::Carry => active.is_carry = Set(enabled),
        Position::Mid => active.is_mid = Set(enabled),
        Position::Offlane => active.is_offlane = Set(enabled),
        Position::Support => active.is_support = Set(enabled),
    }

    Hero::update(active).exec(&txn).await?;
    txn.commit().await?;
    Ok(())
}
