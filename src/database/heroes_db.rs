use std::collections::HashMap;

use sea_orm::*;

use crate::database::database_access;
use crate::database::entities::{hero, hero_nickname, Hero, HeroNickname};
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
    nicknames: HashMap<i32, Vec<String>>,
}

impl HeroLookup {
    pub async fn load() -> Result<Self, Error> {
        let txn = database_access::get_transaction().await?;
        let all_heroes = Hero::find().all(&txn).await?;
        let all_nicknames = HeroNickname::find().all(&txn).await?;

        let heroes = all_heroes.into_iter().map(|h| (h.hero_id, h)).collect();

        let mut nicknames: HashMap<i32, Vec<String>> = HashMap::new();
        for n in all_nicknames {
            nicknames.entry(n.hero_id).or_default().push(n.nickname);
        }

        Ok(Self { heroes, nicknames })
    }

    pub fn get_name(&self, hero_id: i32) -> Option<&str> {
        self.heroes.get(&hero_id).map(|h| h.name.as_str())
    }

    pub fn contains(&self, hero_id: i32) -> bool {
        self.heroes.contains_key(&hero_id)
    }

    pub fn get_nicknames(&self, hero_id: i32) -> &[String] {
        self.nicknames.get(&hero_id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn find_by_name(&self, search: &str) -> Option<&HeroModel> {
        let search_lower = search.to_lowercase();
        let search_nospace = search_lower.replace(' ', "");

        // Match on hero name
        if let Some(hero) = self.heroes.values().find(|h| {
            let name_lower = h.name.to_lowercase();
            name_lower == search_lower || name_lower.replace(' ', "") == search_nospace
        }) {
            return Some(hero);
        }

        // Match on nicknames
        for (hero_id, nicks) in &self.nicknames {
            for nick in nicks {
                let nick_lower = nick.to_lowercase();
                if nick_lower == search_lower || nick_lower.replace(' ', "") == search_nospace {
                    return self.heroes.get(hero_id);
                }
            }
        }

        None
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

pub async fn query_nicknames(hero_id: i32) -> Result<Vec<String>, Error> {
    let txn = database_access::get_transaction().await?;
    let rows = HeroNickname::find()
        .filter(hero_nickname::Column::HeroId.eq(hero_id))
        .all(&txn)
        .await?;
    Ok(rows.into_iter().map(|r| r.nickname).collect())
}

pub async fn insert_nickname(hero_id: i32, nickname: &str) -> Result<(), Error> {
    let txn = database_access::get_transaction().await?;
    let active = hero_nickname::ActiveModel {
        id: NotSet,
        hero_id: Set(hero_id),
        nickname: Set(nickname.to_string()),
    };
    HeroNickname::insert(active).exec(&txn).await?;
    txn.commit().await?;
    Ok(())
}

pub async fn delete_nickname(hero_id: i32, nickname: &str) -> Result<(), Error> {
    let txn = database_access::get_transaction().await?;
    HeroNickname::delete_many()
        .filter(hero_nickname::Column::HeroId.eq(hero_id))
        .filter(hero_nickname::Column::Nickname.eq(nickname))
        .exec(&txn)
        .await?;
    txn.commit().await?;
    Ok(())
}
