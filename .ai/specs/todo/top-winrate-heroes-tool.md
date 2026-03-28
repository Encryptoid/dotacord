# `top_winrate_heroes` AI Agent Tool

## Context

Add a tool to the AI agent that returns the top heroes by win rate for a given position.
Uses the `heroes` database table (from the heroes-database spec) for position classification
and the OpenDota `/heroStats` API endpoint for current win rate data.

**Depends on**: `heroes-database.md` (heroes table with position flags must exist)

## Example Usage

User in Discord: "@Dotacord what are the best support heroes right now?"

Agent calls: `top_winrate_heroes(position: "Support", count: 5)`

Agent receives:
```json
{
  "position": "Support",
  "heroes": [
    { "hero": "Omniknight", "win_rate_pct": 55.2, "total_picks": 142000, "total_wins": 78384 },
    { "hero": "Abaddon", "win_rate_pct": 54.8, "total_picks": 98000, "total_wins": 53704 },
    ...
  ]
}
```

## Flow

Tool call received in `execute_tool()`
  -> Parse position ("Carry", "Mid", "Offlane", or "Support") + optional count (default 5)
  -> Query heroes DB for hero IDs matching position flag
  -> Get hero stats from cache (or fetch from OpenDota /heroStats if stale/missing)
  -> Filter stats to matching hero IDs
  -> Calculate win rate per hero (sum wins/picks across rank brackets 1-7)
  -> Sort by win rate descending, take top N
  -> Return JSON response

Components touched:
- `src/api/open_dota_api.rs` (add ApiHeroStat struct + get_hero_stats())
- `src/api/hero_stats_cache.rs` (new: time-based cache)
- `src/api/mod.rs` (export)
- `src/ai/tools.rs` (add tool definition + execution)
- `src/ai/mod.rs` (register tool)
- `context/dotacord.md` (document tool in system prompt)

## Implementation

### 1. OpenDota `/heroStats` API integration

**File**: `src/api/open_dota_api.rs`

Follow existing `get_player_matches()` pattern exactly.

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct ApiHeroStat {
    pub id: i32,
    pub localized_name: String,
    #[serde(rename = "1_pick")] pub pick_1: i64,
    #[serde(rename = "1_win")]  pub win_1: i64,
    #[serde(rename = "2_pick")] pub pick_2: i64,
    #[serde(rename = "2_win")]  pub win_2: i64,
    #[serde(rename = "3_pick")] pub pick_3: i64,
    #[serde(rename = "3_win")]  pub win_3: i64,
    #[serde(rename = "4_pick")] pub pick_4: i64,
    #[serde(rename = "4_win")]  pub win_4: i64,
    #[serde(rename = "5_pick")] pub pick_5: i64,
    #[serde(rename = "5_win")]  pub win_5: i64,
    #[serde(rename = "6_pick")] pub pick_6: i64,
    #[serde(rename = "6_win")]  pub win_6: i64,
    #[serde(rename = "7_pick")] pub pick_7: i64,
    #[serde(rename = "7_win")]  pub win_7: i64,
    #[serde(rename = "8_pick")] pub pick_8: i64,
    #[serde(rename = "8_win")]  pub win_8: i64,
}

pub(crate) async fn get_hero_stats() -> Result<Vec<ApiHeroStat>, reqwest::Error> {
    let url = format!("{BASE_URL}/heroStats");
    // same reqwest pattern as get_player_matches
}
```

### 2. Hero stats cache

**File**: `src/api/hero_stats_cache.rs`

Time-based cache using `OnceLock<RwLock<Option<CachedData>>>`:

```rust
use std::sync::OnceLock;
use std::time::Instant;
use tokio::sync::RwLock;

const CACHE_TTL_SECS: u64 = 3600; // 1 hour

struct CachedHeroStats {
    data: Vec<ApiHeroStat>,
    fetched_at: Instant,
}

static CACHE: OnceLock<RwLock<Option<CachedHeroStats>>> = OnceLock::new();

pub async fn get_hero_stats() -> Result<Vec<ApiHeroStat>, Error> {
    // Check cache, fetch if stale/missing, store and return
}
```

- TTL: 1 hour (hero stats don't change frequently)
- Lazily initialized on first call (no startup init needed)
- Single shared cache across all requests

### 3. Tool definition

**File**: `src/ai/tools.rs`

Follow existing `get_recent_matches_tool()` pattern:

```rust
pub fn top_winrate_heroes_tool() -> FunctionBuilder {
    FunctionBuilder::new("top_winrate_heroes")
        .description(
            "Get the top Dota 2 heroes by overall win rate for a position. \
             Returns heroes sorted by win rate with pick and win counts.",
        )
        .param(
            ParamBuilder::new("position")
                .type_of("string")
                .description("The position to filter by")
                .enum_values(vec![
                    "Carry".to_string(),
                    "Mid".to_string(),
                    "Offlane".to_string(),
                    "Support".to_string(),
                ]),
        )
        .param(
            ParamBuilder::new("count")
                .type_of("integer")
                .description("Number of top heroes to return (default: 5)"),
        )
        .required(vec!["position".to_string()])
}
```

### 4. Tool execution

**File**: `src/ai/tools.rs`

```rust
async fn execute_top_winrate_heroes(arguments: &str, _ctx: &ToolContext) -> Result<String, Error> {
    // 1. Parse position + count from JSON arguments
    // 2. Parse position string into heroes_db::Position enum
    //    Query heroes DB: query_heroes_by_position(position)
    // 3. Collect matching hero IDs into a HashSet
    // 4. Get hero stats from cache
    // 5. Filter to matching heroes, calculate total picks/wins across brackets 1-7
    //    (skip bracket 8/Immortal which often returns 0)
    // 6. Filter out heroes with very low pick counts (< 100) to avoid statistical noise
    // 7. Sort by win rate descending, take top N
    // 8. Return JSON
}
```

Response structs:
```rust
#[derive(Serialize)]
struct HeroWinRate {
    hero: String,
    win_rate_pct: f64,
    total_picks: i64,
    total_wins: i64,
}

#[derive(Serialize)]
struct TopWinrateResponse {
    position: String,
    heroes: Vec<HeroWinRate>,
}
```

Add match arm in `execute_tool()`:
```rust
"top_winrate_heroes" => execute_top_winrate_heroes(&tool_call.function.arguments, ctx).await,
```

### 5. Register tool

**File**: `src/ai/mod.rs`

Add `.function(tools::top_winrate_heroes_tool())` alongside existing tools.

### 6. System prompt

**File**: `context/dotacord.md`

Add to Tools section:
```
- `top_winrate_heroes` - Get top heroes by win rate for a position (Carry, Mid, Offlane, or Support)
```

## Notes

- Win rate calculation sums across brackets 1-7 (Herald through Divine). Bracket 8 (Immortal)
  often returns 0 from the API and should be skipped.
- Heroes with <100 total picks should be filtered out to avoid statistical outliers.
- The `_ctx: &ToolContext` parameter is unused (no server-specific data needed) but kept for
  consistency with other tool signatures.
- The `count` parameter defaults to 5 if not provided by the agent.
