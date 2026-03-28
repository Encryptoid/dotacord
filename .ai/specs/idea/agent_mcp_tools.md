# Agent MCP Tools

I want to expose MCP tools to the agent, so it will be able to get data about the user's Dota games/account.

## Tools

- `get_recent_matches(username: str) -> RecentMatch` - Get's the last 7 days(or max 20 matches, both configurable values) of matches and returns a summary of each match.
- `get_match_details(match_id: int) -> MatchDetails` - Get's detailed information about a specific match.
