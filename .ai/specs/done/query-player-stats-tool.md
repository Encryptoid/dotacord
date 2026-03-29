# Query Player Stats AI Tool

## Context

When users ask the bot questions like "what is the average time of the games I've played?" or "how am I doing this week?", the AI has no tool to answer with aggregated player statistics. The leaderboard system already computes these stats (winrates, hero spam, kills/assists/deaths/duration) but they're only exposed via the `/leaderboard` slash command which renders all players as markdown tables.

This spec adds a `query_player_stats(username, duration)` tool that returns a single player's aggregated stats as structured JSON, reusing the existing `PlayerStats` calculation pipeline.

## Diagram

```
User: "what's my avg game time this month?"
        │
        ▼
  AI receives message
        │
        ▼
  query_player_stats("username", "Month")
        │
        ├─ find_player_by_name() ─► player_servers_db
        ├─ query_matches_by_duration() ─► player_matches_db
        ├─ player_matches_to_stats() ─► PlayerStats
        ├─ HeroLookup::load() ─► hero names
        │
        ▼
  JSON response (two sections)
  ┌─────────────────────────────┐
  │ overall:                    │
  │   winrate, ranked_winrate,  │
  │   most_played_hero          │
  │ single_match:               │
  │   kills, assists, deaths,   │
  │   longest_match             │
  └─────────────────────────────┘
        │
        ▼
  AI composes natural language answer
```

## Example Usage

Discord: `@Dotacord what is the average time of the games ive played`

Tool call: `query_player_stats(username: "PlayerName", duration: "Month")`

Response: JSON with `overall` and `single_match` sections. The `longest_match` field within `single_match` contains `average_duration_minutes` which directly answers the question.

Discord: `@Dotacord how am i doing this year?`

Tool call: `query_player_stats(username: "PlayerName", duration: "Year")`

Response: Same shape. AI uses overall winrate, ranked winrate, most played hero, and peak stats to compose a summary.

## Flow

Entry: `tools::query_player_stats_tool()` (tool definition) + `tools::execute_query_player_stats()` (execution handler)

1. Tool definition registered in `ai/mod.rs::build_client()` alongside existing tools
2. Dispatch added to `tools::execute_tool()` match arm
3. Execution handler:
   - Parse `username` and `duration` from arguments
   - Resolve player via existing `find_player_by_name()`
   - Calculate date range from duration (reuse `leaderboard/duration.rs` logic)
   - Fetch matches via `player_matches_db::query_matches_by_duration()`
   - Compute stats via `stats_calculator::player_matches_to_stats()`
   - Load `HeroLookup` for hero name resolution
   - Serialize into two-section JSON response
4. Error cases: player not found (list available), no matches in period (clear message)

## Implementation

### 1. Tool definition and response structs in `src/ai/tools.rs`

Add `query_player_stats_tool()` following existing tool definition pattern. Parameters:
- `username` (string, required) — Discord display name or Dota username
- `duration` (string, required, enum) — Day, Week, Month, Year, AllTime

Add response structs for the two sections:

**Overall section fields:** total_matches, wins, winrate_pct, ranked_matches, ranked_wins, ranked_winrate_pct, most_played_hero (name, matches, pick_pct, winrate_pct)

**SingleMatch section fields:** For each of kills, assists, deaths — peak value, average, total, hero name, match_id, won. For longest_match — same shape but with duration formatted as minutes.

### 2. Execution handler in `src/ai/tools.rs`

Add `execute_query_player_stats()` following existing execute function patterns (argument parsing, player resolution, error responses).

Duration-to-date-range conversion: reuse the approach from `leaderboard_task.rs` / `duration.rs` — map the string to start/end timestamps. Reference `src/leaderboard/duration.rs` for the `start_date()` logic.

Call `player_matches_to_stats()` from `stats_calculator` to compute `PlayerStats`, then map fields into the response structs. Use `HeroLookup` for hero ID to name resolution.

Handle gracefully: zero matches returns a clear "no matches found for player in this period" error response (not a Rust error).

### 3. Registration in `src/ai/mod.rs`

Add `.function(tools::query_player_stats_tool())` to the builder chain in `build_client()`.

### 4. Dispatch in `src/ai/tools.rs`

Add `"query_player_stats"` match arm in `execute_tool()` pointing to `execute_query_player_stats()`.
