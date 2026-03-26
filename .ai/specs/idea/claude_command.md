# Claude Command

I want to introduce a new command `/claude` that will allow users to ask questions to a Claude AI agent.
I want to expose MCP tools to the agent, so it will be able to get data about the user's Dota games/account.

## Tools

- `get_recent_matches(username: str) -> RecentMatch` - Get's the last 7 days of matches and returns a summary of each match.

```rust
struct MatchSummary {
    match_id: u64,
    hero_played: String,
    won: Bool, // "win" or "loss"
    kills: u32,
    deaths: u32,
    assists: u32,
    date: String,
    friends: Vec<String>, // List of friends played with in the match(@discord username)
}
```

