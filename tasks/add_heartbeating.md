# Implementation Plan: Scheduled Tasks Component

## Overview
Add a new background scheduler component that runs alongside the Discord bot listener to handle periodic automated tasks:
- **Heartbeat**: Log every 10 minutes (health check)
- **Auto-reload**: Reload player matches every hour
- **Auto-leaderboards**: Publish leaderboards on Weekly/Monthly/Year boundaries

## Architecture Design

### Component Structure
Create `src/scheduler/` module with:
- `mod.rs` - Main scheduler orchestration
- `tasks.rs` - Individual task implementations
- `config.rs` - Scheduler-specific configuration

### Execution Model
- Run scheduler as a separate Tokio task spawned in `main.rs` before `client.start()`
- Use `tokio::time::interval()` for periodic tasks
- Share `Data` struct and database access with Discord commands
- Tasks run independently; failures are logged but don't crash the scheduler

## Detailed Implementation Steps

### 1. Add Dependencies to `Cargo.toml`
```toml
[dependencies]
tokio = { version = "1", features = ["time", "sync"] }
chrono = "0.4" # Already present, ensure it's available
```

### 2. Create Scheduler Configuration

**File**: `src/config.rs` (extend existing `FileConfig` and `AppConfig`)

Add field to existing structs:
```rust
struct FileConfig {
    // ... existing fields ...
    pub scheduler: SchedulerConfig,
}

pub struct AppConfig {
    // ... existing fields ...
    pub scheduler: SchedulerConfig,
}
```

In `load_config()`:
```rust
scheduler: cfg.scheduler,
```

**Update**: `dotacord.toml`
```toml
[scheduler]
enabled = true
heartbeat_interval_minutes = 10
auto_reload_start_hour = 16 # Reloads every hour from 16:00 to 03:00 Local time
auto_reload_end_hour = 3 
auto_reload_interval_minutes = 60

# Weekly leaderboard: Publishes every Sunday at 21:00 Local time
weekly_leaderboard_day = 7    # Sunday
weekly_leaderboard_hour = 21   

# Monthly leaderboard: Publishes last month's stats on the 1st day of the month at 18:00 Local time
monthly_leaderboard_day = 1   # First of month
monthly_leaderboard_hour = 18  
```

When a weekly/monthly leaderboard is triggered, a reload is first called, then the leaderboard is published. Yearly and AllTime leaderboards will also be calculated. 

If there has been a new game of the Yearly/AllTime leaderboard, that is within the weekly/monthly period, that leaderboard will also be sent, alerting users to a new all-time high.


### 3. Create Scheduler Module

**File**: `src/scheduler/mod.rs`

Key points:
- Check `config.scheduler.enabled` before spawning tasks
- Use `config.scheduler.heartbeat_interval_minutes` and `config.scheduler.auto_reload_interval_minutes` directly (no defaults)
- Spawn 3 independent Tokio tasks (heartbeat, reload, leaderboard checker)
- Each task loops with `tokio::time::interval()`, catching errors but continuing
- Leaderboard checker runs hourly, checks all three leaderboard types each iteration
- Store `AppConfig` and `SerenityContext` in `Arc<SchedulerContext>` for shared access

### 4. Create Task Implementations

**File**: `src/scheduler/tasks.rs`

Implement three main tasks:

**`heartbeat()`**
- Simple `info!()` log message
- Returns `Ok(())`

**`auto_reload(ctx)`**
- Query ALL players across ALL servers using `player_servers_db::query_server_players(&mut conn, None)`
- For each player, extract reload logic from `reload_command.rs`:
  - Fetch DB matches and API matches
  - Dedupe and insert new matches in transaction
  - Remove player if no API matches found
- Log success/failure counts

**`check_and_publish_leaderboards(ctx)`**
- Check current time (UTC) against configured day/hour pairs
- Get current weekday (1-7), day of month (1-31), day of year (1-366), and hour (0-23)
- If weekly configured and current weekday/hour matches: publish weekly
- If monthly configured and current day/hour matches: publish monthly
- If yearly configured and current day of year/hour matches: publish yearly
- Use `chrono::Datelike::weekday()`, `day()`, `ordinal()` for date components

**`publish_leaderboard(ctx, duration)`**
- Query all servers using `servers_db::query_all_servers()`
- For each server:
  - Get target channel from server's `channel_id` field (from `DiscordServer`)
  - Skip server if `channel_id` is `None`
  - Generate stats using logic from `leaderboard_command.rs`
  - Build sections and batch content
  - Send via `channel_id.say(&ctx.serenity_ctx.http, msg)`

Helper functions:
- Extract `section_to_content()` and `batch_contents()` from `leaderboard_command.rs`

### 5. Update Main Application

**File**: `src/main.rs`

Add module declaration:
```rust
mod scheduler;
```

Before `client.start().await?`, spawn scheduler:
```rust
// Spawn scheduler (needs SerenityContext - may require adjustment for your Poise version)
scheduler::spawn_scheduler(cfg.clone(), serenity_ctx);
```

**NOTE**: Getting `SerenityContext` may require constructing from `client.cache_and_http` or alternative approaches (see Notes section).

### 6. Add Database Helper

**File**: `src/data/servers_db.rs`

Add function to query all servers:
```rust
pub async fn query_all_servers(
    conn: &mut SqliteConnection,
) -> Result<Vec<DiscordServer>, Error> {
    let servers = sqlx::query_as::<_, DiscordServer>(
        r#"
            SELECT server_id, server_name, channel_id
            FROM servers
            ORDER BY server_name
        "#,
    )
    .fetch_all(conn)
    .await?;

    Ok(servers)
}
```

## Configuration Example

```toml
# dotacord.toml

[scheduler]
enabled = true
heartbeat_interval_minutes = 10
auto_reload_interval_minutes = 60

# Weekly: Monday at midnight UTC
weekly_leaderboard_day = 1
weekly_leaderboard_hour = 0

# Monthly: 1st of month at midnight UTC  
monthly_leaderboard_day = 1
monthly_leaderboard_hour = 0

# Yearly: Disabled (comment out to disable)
# yearly_leaderboard_day = 1
# yearly_leaderboard_hour = 0
```

**Note**: Leaderboards are posted to each server's configured `channel_id` from the `servers` table. If a server has no `channel_id`, it will be skipped. To disable a leaderboard type, omit or comment out both its day and hour fields.

## Observability & Monitoring

### Logging
- All tasks use `#[tracing::instrument]` for structured logging
- Heartbeat explicitly logs at INFO level
- Failures log at ERROR/WARN level but don't crash scheduler
- Track success/failure counts for auto-reload

### Metrics to Monitor
- Heartbeat timestamp (last seen)
- Auto-reload: players processed, successes, failures
- Leaderboard publication: servers processed, messages sent
- Task execution duration

## Error Handling Strategy

### Task Isolation
- Each spawned task runs in independent Tokio task
- Task panics won't crash other tasks or main bot
- Errors are logged but execution continues

### Retry Logic
- Network failures on OpenDota API: Already handled in `open_dota_api.rs`
- Database connection issues: Each task gets fresh connection when it needs one
- Discord API failures: Log and skip, retry next interval

### Graceful Degradation
- If server has no `channel_id`: Skip server with warning
- If no players registered: Skip silently
- If OpenDota API down: Log warning, continue on next interval

## Future Enhancements

### Phase 2 (Future)
1. **Persistent state**: Track last execution times in DB to avoid duplicate publications after restarts
2. **Per-server channel override**: Allow servers to specify different channels for auto vs manual leaderboards
3. **Custom schedules**: Cron-like expressions for flexible scheduling
4. **Health endpoint**: HTTP endpoint for external monitoring
5. **Backpressure handling**: Rate limiting for API calls during reload

### Phase 3 (Advanced)
1. **Distributed scheduling**: Support multiple bot instances with leader election
2. **Task queue**: Priority queue for scheduled tasks
3. **User notifications**: DM users about their stats milestones

## Implementation Checklist

- [ ] Update `Cargo.toml` with required dependencies
- [ ] Extend `config.rs` with scheduler config fields
- [ ] Update `dotacord.toml` with scheduler configuration
- [ ] Create `src/scheduler/mod.rs`
- [ ] Create `src/scheduler/tasks.rs`
- [ ] Add `query_all_servers()` to `servers_db.rs` if missing
- [ ] Integrate scheduler spawn in `main.rs`
- [ ] Validate error handling and logging
- [ ] Document configuration in README

## Notes & Considerations

### Serenity Context Issue
The main challenge is getting a valid `SerenityContext` to pass to the scheduler. The approach shown creates a context from the client's internals. If this doesn't work with your Poise/Serenity version:

**Alternative 1**: Pass only `Arc<Http>` to scheduler, use `http.get_channel()` and `channel.say()` directly
**Alternative 2**: Store channel message logic in `discord_helper`, use HTTP client directly
**Alternative 3**: Use event handler to capture context during bot `ready` event

### Time Zone Handling
- All boundary detection uses UTC
- Leaderboard timestamps formatted via existing `dates::format_short()`
- Week starts on Monday (ISO 8601)

### Resource Usage
- Heartbeat: Negligible (<1KB log entry every 10 min)
- Auto-reload: Network-bound, ~1-5 seconds per player depending on API
- Leaderboards: CPU-bound for stats calculation, could take 10-30 seconds for large servers

### Concurrent Safety
- Each task gets its own DB connection (via `get_new_connection()`)
- No shared mutable state between tasks
- Discord API rate limits handled by Serenity's internal rate limiter
