# Dotacord

A Discord bot for calculating and displaying a leaderboard of Dota players registered to the current server.

Uses the [OpenDota API](https://docs.opendota.com/) to fetch player statistics and match history and formats into multiple leaderboard sections for a specified time period.

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

## Commands

### Public Commands

#### `/register_to_leaderboard <player_id>`

Allows a player to register themselves to the leaderboard by providing their Dota Player ID.
This command should be configured to be usable/restricted by anyone.

#### `/reload_matches`

Allows a user to to reload their own matches from the API. This should be on a cooldown timer to prevent API spam.

#### `/leaderboard [duration]`

Responds with an ephemeral leaderboard for the specified duration.

### Admin Commands

#### `/reload_server_matches`

Forces a reload of API matches for all players on the current server.

#### `/list_players`

Lists all players on the current server.

#### `/register_player <discord_name> <player_id>`

Adds a player to be tracked by the bot. Attempts to load player data from the API and store in the database.
If no dota matches are found for the player id, the player is not added.
Similar to register_to_leaderboard, but allows an admin to add other players.

#### `/remove_player <discord_name>`

Removes a player from being tracked by the bot.

#### `/nickname <discord_name> <nickname>`

Renames a tracked player to a new name.

#### Subscription Commmands

##### `/subscribe_channel <channel_id>`

Sets the current server's subscription channel to the specified channel ID.

##### `/subscribe_week`

Toggles weekly leaderboard subscription for the current server.

##### `/subscribe_month`

Toggles monthly leaderboard subscription for the current server.

### Owner Commands

#### `/register_server <server_id>`

![Month Demo](./resources/demo_month.png)
