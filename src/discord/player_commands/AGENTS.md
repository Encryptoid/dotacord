# Breakdown of Files

## add_player.rs

### Commands

`add_player(DiscordUser, PlayerId, Option<Nickname>)`

This command will addd a player to the database.

#### Process

- Checks for max number of players allowed on server
- Checks if the player is already on ther server
- Inserts into database
- Privately replies to the user

## list_players.rs

### Commands

`list_players()`

Lists all registered players for the current server.

#### Process

- Fetches all player mappings for the server from the database
- Formats the players into a markdown table
- Replies privately with the formatted list

## remove_player.rs

### Commands

`remove_player(DiscordUser)`

Removes a player's registration from the current server.

#### Process

- Begins a database transaction and attempts to delete the player mapping
- Commits the transaction after the delete attempt
- Replies privately indicating whether the player existed

## rename_player.rs

### Commands

`rename_player(DiscordUser, NewName)`

Updates the custom name for a registered player on this server.

#### Process

- Runs the rename inside a database transaction via `player_servers_db`
- Commits the transaction and replies privately with success/failure status