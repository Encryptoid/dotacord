# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## [MANDATORY] Convention-First Development

Before writing ANY new code, you MUST:

1. **Find existing examples** - Search for similar code patterns in the codebase (use Grep/Glob)
2. **Read multiple examples** - Read at least 2-3 existing files that do similar things
3. **Match exactly** - Copy the exact style, structure, attributes, imports, and patterns from existing code
4. **Never invent** - Do not add attributes, parameters, or patterns not present in existing code
5. **When in doubt, search** - If unsure about any convention, search the codebase first

This applies to: function signatures, attributes, imports, error handling, naming, file structure, and all other patterns.

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

## Discord Commands

Reference files for command patterns:
- `src/discord/reload_command.rs` - simple public command
- `src/discord/register_server.rs` - command without standard validation
- `src/discord/server_settings_command.rs` - interactive panel with buttons/selects
- `src/discord/manage_players_command.rs` - interactive panel with modals and sub-panels

Key points:
- Commands use `discord_helper::get_command_ctx(ctx).await?` for server validation
- User-facing errors: reply then `return Ok(())` - never bubble to Poise
- Admin commands get `Permissions::ADMINISTRATOR` in `discord::commands()`

**Always read existing command files before writing new ones.**

**Custom emojis:** Use emojis from `src/leaderboard/emoji.rs`. For buttons, use `discord_helper::parse_custom_emoji()` with `.emoji()` method, not in `.label()`.

## Configuration

`dotacord.toml` fields:
- `api_key_var` - environment variable name containing Discord token
- `test_guild` - optional guild ID for faster command registration during dev
- `clear_commands_on_startup` - clears all Discord commands (global + guild) before registering
- `scheduler.enabled` - master toggle for background task spawning
- `scheduler.auto_reload.enabled` - toggle auto-reload task
- `scheduler.weekly_leaderboard.enabled` - toggle weekly leaderboard task
- `scheduler.monthly_leaderboard.enabled` - toggle monthly leaderboard task
- `max_message_length` - Discord message batching threshold (default 1900)

