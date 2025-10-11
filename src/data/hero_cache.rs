use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::OnceLock;

use serde_json;

static HEROES_BY_ID: OnceLock<HashMap<i32, String>> = OnceLock::new();

pub fn init_cache(hero_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(hero_path)?;
    let reader = std::io::BufReader::new(file);
    let json: serde_json::Value = serde_json::from_reader(reader)?;

    let heroes_map: HashMap<i32, String> = json
        .as_array()
        .ok_or("Expected JSON array")?
        .iter()
        .filter_map(|hero| {
            let id = hero["id"].as_i64()? as i32;
            let name = hero["localized_name"].as_str()?.to_string();
            Some((id, name))
        })
        .collect();

    HEROES_BY_ID
        .set(heroes_map)
        .map_err(|_| "Heroes already initialized")?;

    Ok(())
}

pub fn get_hero_by_id(id: i32) -> Option<&'static str> {
    HEROES_BY_ID
        .get()
        .expect("Heroes not initialized. Call init() first.")
        .get(&id)
        .map(|s| s.as_str())
}

pub fn get_random_hero() -> &'static String {
    use rand::seq::IteratorRandom;
    let heroes = HEROES_BY_ID.get().expect("Heroes not initialized");
    let mut rng = rand::rng();
    heroes
        .iter()
        .map(|h| h.1)
        .choose(&mut rng)
        .expect("Could not get random hero")
}
