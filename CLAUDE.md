# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
cargo build                    # Compiles; build.rs copies dotacord.toml to target/debug/
cargo run                      # Runs from target/debug/
```

**Required files at runtime:**
- `dotacord.toml` (copied to target dir by build.rs)
- `data/dotacord.db` (SQLite database)
- `data/heroes.json` (hero ID→name mapping)

**No test suite exists** - there are no `#[test]` annotations in the codebase.

## Database Schema

SQLite with SeaORM 1.x. Tables: `players`, `servers`, `player_servers`, `player_matches`, `schedule_events`.

**To create/recreate database:**
```bash
sqlite3 data/dotacord.db ".read sql/create_tables.sql"
```

**To regenerate SeaORM entities after schema changes:**
```bash
sea-orm-cli generate entity -o src/database/entities
```

No migration system exists - manually apply schema changes and regenerate entities.

## Architecture

Discord bot tracking Dota 2 stats via OpenDota API. Built on Poise + Serenity (serenity-next branch).

**Startup sequence** (`src/main.rs`):
1. `config::load_config()` - loads `dotacord.toml` from exe directory
2. `logging::init()` - configures tracing (stdout + file + JSON + optional Seq)
3. `hero_cache::init_cache()` - loads heroes.json into `OnceLock<HashMap<i32, String>>`
4. `database_access::init_database()` - initializes SeaORM connection in `OnceLock`
5. `discord::commands()` - builds slash commands; admin commands get `ADMINISTRATOR` perms
6. Framework + client start
7. `scheduler::spawn_scheduler()` - launches background tasks

**Key types:**
- `Context<'_>` = `poise::Context<'_, Data, Error>`
- `Error` = `Box<dyn std::error::Error + Send + Sync>`

## Code Organization

| Directory | Purpose |
|-----------|---------|
| `src/api/` | OpenDota API client and link generation |
| `src/database/` | SeaORM entities and query modules (`*_db.rs`) |
| `src/discord/` | Slash commands and Discord interaction logic |
| `src/leaderboard/` | Stats calculation and section formatting |
| `src/scheduler/` | Background tasks (reload, leaderboard publishing) |
| `src/markdown/` | Table/link rendering for Discord messages |

**Database query naming:** `query_*`, `insert_*`, `update_*`, `delete_*`

## Discord Command Pattern

Every command follows this structure:

```rust
#[poise::command(slash_command)]
#[tracing::instrument(name = "COMMAND_NAME", level = "trace", skip(ctx))]
pub async fn my_cmd(ctx: Context<'_>) -> Result<(), Error> {
    let cmd_ctx = discord_helper::get_command_ctx(ctx).await?;
    // command logic
    cmd_ctx.private_reply(response).await?;
    Ok(())
}
```

- All commands use `discord_helper::get_command_ctx(ctx).await?` which validates server registration
- User-facing errors: reply via `discord_helper` then `return Ok(())` - never bubble to Poise
- Admin commands defined in `discord::commands()` get `Permissions::ADMINISTRATOR`

## Utility Macros

- `fmt!()` - alias for `format!()`
- `str!()` - alias for `.to_string()`

## Configuration

`dotacord.toml` fields:
- `api_key_var` - environment variable name containing Discord token
- `test_guild` - optional guild ID for faster command registration during dev
- `scheduler.enabled` - controls background task spawning
- `max_message_length` - Discord message batching threshold (default 1900)
