# Dotacord – Agent Playbook

## Architecture & startup flow
Discord bot tracking Dota 2 player stats via OpenDota API. Built on Poise 0.6 + Serenity (git rev 3009d220).

**Startup sequence** (`src/main.rs`):
1. `config::load_config()` – loads `dotacord.toml` from exe directory, validates all paths
2. `logging::init()` – configures stdout + file + JSON + optional Seq endpoint
3. `hero_cache::init_cache()` – loads `data/heroes.json` into immutable `OnceLock<HashMap<i32, String>>`
4. `database_access::init_database()` – stores SQLite options in `OnceLock`, no connection pooling
5. `database_access::init_sea_orm_database()` – initializes SeaORM connection for entities
6. Command registration – `discord::commands()` builds slash commands; admin commands get `ADMINISTRATOR` perms
7. Framework + client start – Poise framework wraps Serenity client
8. `scheduler::spawn_scheduler()` – launches heartbeat/reload/leaderboard background tasks

**Type system conventions**:
- `Context<'_>` = `poise::Context<'_, Data, Error>`
- `Error` = `Box<dyn std::error::Error + Send + Sync>`
- SQLite via sqlx 0.7 + SeaORM 1.x for entities

## Data layer & dual ORM usage
**SQLite schema**: `sql/create_tables.sql` defines `players`, `servers`, `player_servers`, `player_matches`. No migrations – manually reapply schema changes to `data/dotacord.db`.

**Database access patterns**:
- **sqlx raw queries**: `database_access::get_new_connection()` clones `SqliteConnectOptions` per query (no pooling). Use `query!`/`query_as!` macros with `#[derive(FromRow)]` structs in `src/database/*_db.rs` modules.
- **SeaORM entities**: `database_access::get_sea_orm_connection()` returns `&'static DatabaseConnection` from `OnceLock`. Entity models in `src/database/entities/` generated from schema.

**OpenDota ingestion** (`player_matches_db::map_to_player_match`):
- Filters by game mode (Ranked/AllPick) and whitelisted lobby types
- Validates `hero_id` via `hero_cache::get_hero_by_id()` – returns `Ok(None)` for filtered, `Err(MapperError)` for invalid
- Hard exclusions: `match_id 1439386853`, `leaver_status` 1/2, negative durations
- `reload_command.rs` fetches API data, dedupes against existing matches, inserts in transaction

## Discord command lifecycle
**Command structure** (`src/discord/**`):
1. Register via `discord::commands()` in `mod.rs`
2. Every command starts with `discord_helper::validate_command(ctx, conn, guild_id).await` – checks server registration + logs invocation
3. Reply methods: `discord_helper::public_reply()`/`private_reply()` always use `MessageFlags::SUPPRESS_EMBEDS`; `send_message()` for channel posts
4. Attribute format: `#[poise::command(slash_command)]` + `#[description]`/`#[name]` for metadata

**Error handling contract**:
- User-facing errors: reply via `discord_helper` then `return Ok(())` – never bubble to Poise handler
- Internal failures: propagate with `?` using `crate::Error`

**Example pattern** (see `reload_command.rs`):
```rust
#[poise::command(slash_command)]
#[tracing::instrument(name = "COMMAND_NAME", level = "trace", skip(ctx))]
pub async fn my_command(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = discord_helper::guild_id(&ctx)?;
    let mut conn = database_access::get_new_connection().await?;
    if !discord_helper::validate_command(&ctx, &mut conn, guild_id).await? {
        return Ok(());
    }
    // ... command logic
    discord_helper::public_reply(&ctx, "Done".to_string()).await?;
    Ok(())
}
```

## Leaderboard rendering pipeline
**Data flow**: `PlayerMatch` rows → `stats_calculator` → `PlayerStats` → `section_formatter` → `LeaderboardSection` → batched Discord messages

**Stats aggregation** (`leaderboard/stats_calculator.rs`):
- `StatTracker` accumulates per-player: overall/ranked win rates, hero spam, max kills/deaths/assists/duration
- Returns `Vec<PlayerStats>` with embedded `SingleMatchStat` for extremes

**Markdown rendering** (`src/markdown/**`):
- `TableBuilder` constructs tables from `Column` enum (Text/Link variants)
- Link columns use `LINK_SYMBOL` + `mask_link()` to keep URLs compact
- Builds `LeaderboardSection` (title + lines) consumed by Discord commands
- See `src/markdown/README.md` for design rationale

**Message batching** (`discord/leaderboard_command.rs`):
- `batch_contents()` splits sections to honor `config.max_message_length` (default 1900 chars)
- Posts sequentially to avoid rate limits

## Scheduler & background tasks
**Task spawning** (`scheduler/mod.rs`):
- Enabled via `config.scheduler.enabled`
- Three tokio tasks with `time::interval`:
  1. **Heartbeat** (`heartbeat_task.rs`) – logs keepalive every N minutes
  2. **Auto-reload** (`reload_task.rs`) – fetches player matches during configured hours
  3. **Leaderboard publisher** (`leaderboard_task.rs`) – checks weekly/monthly schedules, posts to subscribed channels

**Scheduler context**:
```rust
struct SchedulerContext {
    config: AppConfig,
    http: Arc<serenity::Http>,
}
```

## Config & secrets management
**Config file**: `dotacord.toml` copied by `build.rs` to target directory alongside exe. `config::load_config()` validates all paths at startup.

**Log configuration**:
- `logging.rs` sets up tracing layers: pretty stdout + `log_path` + `log_json_path` + optional `seq_endpoint`
- Filters: `serenity`, `tokio_tungstenite`, `h2` → warn; `dotacord` → trace
- Date formatting: `util::dates::local_date_yyyy_mm_dd()` for `{DATE}` replacement in log paths

**Database changes**:
1. Edit `sql/create_tables.sql`
2. Manually apply to `data/dotacord.db` (no migration system)
3. Regenerate SeaORM entities if schema changed

**Instrumentation**: Add `#[tracing::instrument(level = "trace", skip(large_args))]` to async fns for telemetry.

## Immutable runtime caches
- **Hero cache** (`hero_cache.rs`): `OnceLock<HashMap<i32, String>>` populated at startup. Access via `get_hero_by_id()`/`get_random_hero()`. Never mutate at runtime.

## Key file locations
- `src/discord/mod.rs` – command registration + admin permission assignment
- `src/database/*_db.rs` – query modules (naming: `query_*`, `insert_*`, `update_*`)
- `src/database/entities/` – SeaORM generated models
- `leaderboard/section_formatter.rs` – Markdown layout rules
- `scheduler/*_task.rs` – background task implementations
- `util/dates.rs` – time formatting helpers used by logging + stats
