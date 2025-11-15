# Dotacord – Agent Playbook

## Lifecycle & architecture
- Discord bot built on Poise 0.6 + Serenity git rev 3009d220; entrypoint is `src/main.rs` -> config -> logging -> hero_cache -> database -> Poise builder -> client start.
- Data lives in SQLite (`sql/create_tables.sql`); no migrations, so edit SQL then manually reapply to `data/dotacord.db`.
- Type aliases: `Context<'_>` = `poise::Context<'_, Data, Error>`; `Error` = `Box<dyn std::error::Error + Send + Sync>`.
- Local dependencies: `../ext/poise` fork, sqlx 0.7 (sqlite runtime), hero metadata loaded from `data/heroes.json` via `hero_cache::init_cache()`.

## Discord command patterns
- Commands stay in `src/discord/**`; register through `discord::commands()` where admin commands automatically get `ADMINISTRATOR` permissions.
- Every command must start with `discord_helper::validate_command(ctx).await` to enforce guild registration + logging.
- Respond with `ctx.public_reply()`/`private_reply()` using `MessageFlags::SUPPRESS_EMBEDS`; `send_message()` is used for channel broadcasts.
- Use `#[poise::command(slash_command)]` on `async fn command(ctx: Context<'_>, …) -> crate::Error` and add `#[description]`/`#[name]` metadata for docs.

## Database + OpenDota ingestion
- `database_access::init_database(path)` stores `SqliteConnectOptions` in `OnceLock`; `get_new_connection()` clones per query—never pool.
- `player_matches_db::map_to_player_match()` enforces game modes (Ranked/AllPick) and lobby whitelist plus `hero_cache::get_hero_by_id()` validation; returns `Ok(None)` for filtered matches, `Err(MapperError)` for invalid payloads.
- `reload_command.rs` fetches OpenDota data, dedupes against `player_matches`, and inserts inside a transaction (`conn.begin()`, `tx.commit()`).
- Hard exclusions: `match_id 1439386853` is skipped, `leaver_status` 1/2 rejected, durations must be positive.

## Leaderboard pipeline
- `leaderboard::stats_calculator` turns `PlayerMatch` rows into `PlayerStats` + `SingleMatchStat` using `StatTracker` to keep max kills/deaths/assists/duration history.
- `leaderboard::sections` builds Markdown sections via `section_formatter::build_*` helpers with duration filters and emoji lookups from `leaderboard/emoji.rs`.
- `discord/leaderboard_command.rs` batches `Section` strings with `batch_contents()` to honor `config.max_message_length` (default 1900) before posting.

## Config, logging, workflows
- `dotacord.toml` travels with the exe (copied by `build.rs`); `config::load_config()` verifies every path at startup.
- Required secret: set `KEY_DOTACORD` env var or provide `api_key_var` in config before running `cargo run`.
- Logging layers configured in `logging.rs`: pretty stdout + `log_path` + `log_json_path` + optional `seq_endpoint`; filters set serenity/tokio_tungstenite/h2 to warn, `dotacord=trace`.
- Time formatting uses `util::dates::local_date_yyyy_mm_dd()` + `OffsetTime::local_rfc_3339()`; keep these helpers when adding timestamps.
- Typical dev loop: `cargo check`, `cargo clippy --all-targets --all-features` (unused = deny), `cargo run` (build.rs copies config automatically).

## Error handling + style guardrails
- User-facing issues must reply via `discord_helper` then return `Ok(())`; never bubble to Poise error handler.
- Propagate internal failures with `?` using `crate::Error`; database helpers prefer `query!`/`query_as!` macros with `#[derive(FromRow)]` structs.
- Keep `hero_cache` immutable at runtime; access via `hero_cache::get_hero_by_id()`/`get_random_hero()`.
- Markdown formatting for sections lives under `src/markdown/**`; use `TableBuilder` + `Column` enums rather than ad-hoc strings.
- Command/routine instrumentation should use `#[tracing::instrument(level = "trace", skip(big_args))]` when adding telemetry to async fns.

## File quick hits
- `src/discord/mod.rs` – registers all slash commands + permission tweaks.
- `src/database/*_db.rs` – query modules; follow insert/query function naming convention.
- `leaderboard/section_formatter.rs` – central place for Markdown layout rules.
- `scheduler/*_task.rs` – background heartbeats + leaderboard/reload scheduling if you add automation.
- `util/markdown.rs` + `markdown/*` – reusable formatter helpers shared by Discord replies.
