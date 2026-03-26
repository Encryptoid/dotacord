# Ready Check Command

## Context

Recreate Dota 2's party ready check feature in Discord. When an admin
triggers the command, all players in their voice channel see a public
message with Ready/Not Ready buttons. The message updates in real-time
as players respond, showing greyed-out names for those who haven't
readied. After 30 seconds, the final status is displayed.

## Example Usage

```
/ready
```

Admin must be in a voice channel. The command:
1. Posts a public message listing all voice channel members
2. Each player clicks Ready or Not Ready on the public message
3. Message updates live as responses come in
4. After 30 seconds, shows final status and disables buttons

## Flow

Entry: `/ready` slash command (admin only)
  -> Validate admin is in a voice channel
  -> Fetch all members in that voice channel
  -> Post public message with player list + buttons
  -> Start ComponentInteractionCollector (30s timeout)
  -> On button click: update player status, refresh message
  -> On timeout: show final status, disable buttons

Components touched:
- `src/discord/ready_command.rs` (new)
- `src/discord/mod.rs` (register command)
- `src/discord/commands.rs` (add to command list with admin perms)

## Implementation

1. Create `src/discord/ready_command.rs`
   - Follow pattern from `server_settings_command.rs:1-50` for structure
   - Admin permission via `discord::commands()` like other admin commands

2. Get admin's voice channel
   - Use `ctx.guild_id()` to get guild
   - Use `guild.voice_states` to find admin's voice state
   - Extract `channel_id` from voice state

3. Fetch voice channel members
   - Iterate `guild.voice_states` filtering by `channel_id`
   - Build list of `(UserId, display_name)` tuples
   - Error if channel is empty or admin is alone

4. Build initial message
   - List each player with greyed-out indicator (e.g., dimmed emoji)
   - Two buttons: Ready (green) + Not Ready (red)
   - Use emojis from `src/leaderboard/emoji.rs` if suitable

5. Interaction loop
   - `ComponentInteractionCollector` with 30s timeout
   - Filter by `message_id` (anyone in channel can respond)
   - Track responses in `HashMap<UserId, bool>` (true=ready, false=not)
   - On each click: ephemeral ack, update HashMap, rebuild message
   - Prevent duplicate responses (player can only respond once)

6. Timeout handling
   - Disable buttons using `.disabled(true)` on rebuild
   - Show final tally: X ready, Y not ready, Z no response
   - Edit message one last time with final state

## Notes

- No database storage needed (30s ephemeral session)
- Voice channel member fetching is new to codebase - may need
  `GuildId::to_guild_cached()` or fetch via HTTP
- Consider showing countdown in message footer (optional)
