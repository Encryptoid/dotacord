
use crate::discord::discord_helper::{self, CmdCtx, Ephemeral};
use crate::{Context, Error};

mod add_player;
mod list_players;
mod remove_player;
mod rename_player;

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    subcommands(
        "list_players::list_players_command",
        "add_player::add_player_command",
        "remove_player::remove_player_command",
        "rename_player::rename_player_command"
    )
)]
pub async fn players(_: Context<'_>) -> Result<(), Error> {
    unreachable!()
}
