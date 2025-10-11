# Dotacord – AI implementation notes

## Project architecture
- **Framework**: Discord bot built with [Poise](https://github.com/serenity-rs/poise) (0.6) + Serenity, tracking Dota 2 match stats from OpenDota API
- **Dependencies**: Local Poise fork at `../ext/poise`, Serenity from git rev `3009d220`, sqlx 0.7 with SQLite runtime
- **Bootstrap sequence** (`src/main.rs`): config → logging → hero_cache → database → Poise framework → client start
- **Type aliases**: `Context<'_>` = `poise::Context<'a, Data, Error>`, `Error` = `Box<dyn std::error::Error + Send + Sync>`
- **Framework setup**: Commands registered via `discord::commands()` vector; admin commands get `ADMINISTRATOR` permission requirements applied in loop

## Discord commands
- **Location**: Commands live in `src/discord/`; register via `discord::commands()` async function in `mod.rs`
- **Signature**: Async functions taking `Context<'_>`, returning `crate::Error`, decorated with `#[poise::command(slash_command)]`
- **Poise attributes**: Use `#[description]` for command/param docs, `#[name]` to rename params
- **Permission pattern**: Admin commands get `required_permissions` and `default_member_permissions` set to `ADMINISTRATOR` in `mod.rs` loop
- **Helpers**: Use `discord_helper::{guild_id, channel_id, guild_name, validate_command}` for context extraction
- **Reply modes**: `public_reply()` for visible messages, `private_reply()` for ephemeral; `send_message()` directly to channels
- **Command validation**: Always call `discord_helper::validate_command()` FIRST—checks server registration, logs invocation details
- **Reply flags**: All replies use `MessageFlags::SUPPRESS_EMBEDS` to prevent automatic link unfurling

## Database architecture (SQLite + sqlx)
- **Init pattern**: `database_access::init_database(path)` stores `SqliteConnectOptions` in `OnceLock<>` at startup
- **Connection pattern**: Call `get_new_connection()` per operation—NEVER pool connections, always clone options from OnceLock
- **Schema**: `sql/create_tables.sql` defines `players`, `servers`, `player_servers`, `player_matches` tables with foreign keys
- **Query pattern**: Use `sqlx::query!`/`query_as!` macros + `#[derive(FromRow)]` for compile-time verification
- **WAL mode**: SQLite configured with `journal_mode = Wal` and `foreign_keys = true` for concurrency/integrity
- **Transactions**: Wrap batch inserts in `conn.begin().await?` / `tx.commit().await?` pattern
- **No migrations**: Schema changes require manual SQL edits to `create_tables.sql` and reapplying to database file

## OpenDota integration & match filtering
- **Reload flow** (`reload_command.rs`): Fetch API matches → map/filter → dedupe against DB → transactional batch insert
- **Filtering logic** (`player_matches_db::map_to_player_match`):
  - **Whitelist**: Only `GameMode::{Ranked, AllPick}` AND `LobbyType::{Unranked, Ranked, RankedSolo}`
  - **Blacklist**: Exclude `leaver_status` 1 or 2, invalid hero IDs, negative durations, missing required fields
  - Returns `Ok(None)` for filtered matches (soft rejection), `Err(MapperError)` for validation failures
- **Victory calculation**: `Faction::from_player_slot(player_slot)` checks `< 128` for Radiant, compared against `radiant_win` boolean
- **Hardcoded exclusion**: Match ID `1439386853` always filtered at line 30 of `player_matches_db.rs`
- **Hero validation**: Always check `hero_cache::get_hero_by_id(id).is_some()` before persisting to catch API inconsistencies

## Hero cache pattern
- **Storage**: In-memory `OnceLock<HashMap<i32, String>>` loaded from `data/heroes.json` at startup
- **Initialization**: `hero_cache::init_cache(path)` MUST be called before database init (enforced in `main.rs` bootstrap)
- **Validation**: Check `get_hero_by_id(id).is_some()` before accepting matches—returns `MapperError::UnknownHero` if missing
- **Random selection**: `get_random_hero()` uses `rand::seq::IteratorRandom` for uniform sampling
- **Immutable**: Cache is loaded once at startup and never modified—no runtime updates

## Leaderboard system (3-tier pipeline)
1. **Stats calculator** (`leaderboard/stats_calculator.rs`):
   - Aggregates `PlayerMatch[]` → `PlayerStats` with nested `OverallStats`, `RankedStats`, `HeroPickStats`, `SingleMatchStat`
   - `player_matches_to_stats()` uses internal `StatTracker` to track max values (kills/assists/deaths/duration)
   - `SingleMatchStat` stores both the max value AND metadata (match_id, date, hero_id, is_victory)
2. **Sections** (`leaderboard/sections.rs`):
   - Calls `section_formatter::build_*_section()` with duration filter, stats data, sort predicate
   - Example: `build_winrate_section(duration, stats, |s| (s.overall_stats.wins, s.total), emojis, title)`
   - Each section returns formatted markdown string ready for Discord
3. **Command** (`discord/leaderboard_command.rs`):
   - Fetches player stats, builds 7 sections, batches by `max_message_length` (1900 default), posts via `send_message()`
   - **Message batching**: `batch_contents()` accumulates sections and splits when approaching Discord 2000-char limit

## Config & environment
- **File location**: `dotacord.toml` co-located with exe (copied by `build.rs` from workspace root to `target/debug/`)
- **Required env var**: `KEY_DOTACORD` Discord bot token—stored in `api_key_var` config field, NOT read from env by default
- **Path validation**: `config::load_config()` validates ALL paths exist at startup, fails fast if missing
- **Log rotation**: Supports `{DATE}` placeholder in log paths (replaced by `YYYY-MM-DD` via `dates::local_date_yyyy_mm_dd()`)
- **Dev config**: Set `test_guild` for instant guild-level command registration (skips global 1hr delay); `clear_commands_on_startup` to wipe (currently commented out)
- **Config fields**: `max_message_length` (Discord batching), `max_players_per_server` (enforcement TBD), `seq_endpoint` (optional structured logging)

## Logging & observability
- **4-layer output**: stdout (pretty) + `log_path` (pretty file) + `log_json_path` (JSON file) + optional Seq endpoint
- **Custom filters** (`logging.rs`):
  - `serenity=warn`, `tokio_tungstenite=warn`, `h2=warn` to suppress noisy dependencies
  - `dotacord=trace` for full app-level tracing
  - `dotacord::data::player_matches_db=info` to reduce match filtering spam
- **Instrumentation**: Add `#[tracing::instrument(level = "trace", skip(large_args))]` to async functions
- **Structured logs**: Use `info!(field = value, "message")` format for JSON compatibility
- **Time formatting**: Uses `OffsetTime::local_rfc_3339()` for timestamps (requires local time offset)

## Development workflow
```powershell
# Setup
$env:KEY_DOTACORD = "your_discord_bot_token"

# Edit dotacord.toml paths to absolute Windows paths (forward slashes OK)
# Set test_guild = 927307976497315930 for faster command registration during dev

# Dev commands
cargo check              # Fast syntax/type validation
cargo clippy --all-targets --all-features  # Linting (unused = "deny" enforced)
cargo run                # Starts bot (build.rs auto-copies dotacord.toml to target/)

# Database changes
# 1. Edit sql/create_tables.sql manually
# 2. Reapply to data/dotacord.db (no automated migration system)
# 3. Update *_db.rs query files if schema changes affect queries
```

## Error handling conventions
- **User-facing errors**: Send reply via `discord_helper::private_reply()` or `public_reply()`, then return `Ok(())`—NEVER propagate to Discord framework
- **Internal errors**: Bubble with `?` operator using `crate::Error` type alias (boxed trait object)
- **Validation failures**: Custom `MapperError` enum in `data/types.rs` for API data issues (missing fields, invalid enums, unknown heroes)
- **Example pattern** (from `leaderboard_command.rs`):
  ```rust
  if players.is_empty() {
      discord_helper::private_reply(&ctx, "No players registered".to_string()).await?;
      return Ok(()); // Return success after user notification, not Err
  }
  ```
- **Command validation**: Check `validate_command()` result and return `Ok(())` early if server unregistered

## Key files & patterns
- **Command registration**: `src/discord/mod.rs` exports `commands()` async function, applies permission requirements in loop
- **Database queries**: `src/data/*_db.rs` files follow `query_*` / `insert_*` naming convention
- **Type conversions**: `src/data/types.rs` defines `Faction`, `GameMode`, `LobbyType` enums with `TryFrom<i32>` for API deserialization
- **Markdown formatting**: `src/markdown/` (not `discord/markdown/`) handles table generation with emoji support
- **Build system**: `build.rs` copies `dotacord.toml` to target dir using `OUT_DIR` env var + ancestor traversal
- **Workspace lints**: `unused = "deny"` in `Cargo.toml` [workspace.lints.rust] section enforces zero unused code
