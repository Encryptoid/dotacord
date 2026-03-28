use std::sync::OnceLock;
use std::time::Instant;

use tokio::sync::RwLock;
use tracing::info;

use super::open_dota_api::{self, ApiHeroStat};
use crate::Error;

const CACHE_TTL_SECS: u64 = 3600;

struct CachedHeroStats {
    data: Vec<ApiHeroStat>,
    fetched_at: Instant,
}

static CACHE: OnceLock<RwLock<Option<CachedHeroStats>>> = OnceLock::new();

fn get_cache() -> &'static RwLock<Option<CachedHeroStats>> {
    CACHE.get_or_init(|| RwLock::new(None))
}

pub async fn get_hero_stats() -> Result<Vec<ApiHeroStat>, Error> {
    let lock = get_cache();

    {
        let reader = lock.read().await;
        if let Some(cached) = reader.as_ref() {
            if cached.fetched_at.elapsed().as_secs() < CACHE_TTL_SECS {
                return Ok(cached.data.clone());
            }
        }
    }

    info!("Hero stats cache miss or stale, fetching from OpenDota");
    let stats = open_dota_api::get_hero_stats().await?;

    let mut writer = lock.write().await;
    *writer = Some(CachedHeroStats {
        data: stats.clone(),
        fetched_at: Instant::now(),
    });

    Ok(stats)
}
