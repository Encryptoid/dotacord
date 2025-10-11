# Dotacord

A Discord bot for calculating and displaying a leaderboard of Dota players registered to the current server.

Uses the [OpenDota API](https://docs.opendota.com/) to fetch player statistics and match history and formats into multiple leaderboard sections for a specified time period.

## Duration

* Day
* Week
* Month
* Year
* All Time

## Sections

These sections will be displayed in a formated grid, either published to the registered channel, or as a private(ephemeral) response.

### Overall Stats

* Overall Win Rate
* Ranked Win Rate
* Hero Spam Rate

### Single Match Stats

* Most Kills
* Most Deaths
* Most Assists
* Longest Match

# Commands

## Public Commands

### `/register_to_leaderboard <player_id>`

Allows a player to register themselves to the leaderboard by providing their Dota Player ID.
This command should be configured to be usable/restricted by anyone.

### `/leaderboard [duration]`

Responds with an ephemeral leaderboard for the specified duration.

## Admin Commands

### `/reload`

Forces a reload of API matches for all players on the current server.

### `/list_players`

Lists all players on the current server.

### `/add_player <discord_name> <player_id> [nickname]`

Adds a player to be tracked by the bot. Attempts to load player data from the API and store in the database.
If no dota matches are found for the player id, the player is not added. 

### `/remove_player <discord_name>`

Removes a player from being tracked by the bot.

### `/nickname <discord_name> <nickname>`

Renames a tracked player to a new name.

## Owner Commands

# TODO

## Dingus

- [x] Do not allow adding more than one player
- [-] default /leaderboard to ephemeral, with optional parameter to make it public.
- [ ] Ordering before title

## Coy

- [ ] Check for sending too long a message
- [x] Admin level commands
- [ ] Reorganise player commands into a subcommand group
- [ ] Way to clear commands from all servers, to update
- [ ] Owner commands
- [ ] Heartbeating
- [ ] Reload

## Ireland

- [ ] Streaming all messages: list players, etc.
- [ ] More customisation on sections
- [ ] Timed reloading
- [ ] Timed publishing, config for ephemeral, etc.
- [ ] SQL Injection protection
- [ ] Status
- [ ] Database backup

![Month Demo](./resources/demo_month.png)
