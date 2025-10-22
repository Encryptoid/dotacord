use poise::serenity_prelude::Permissions;

use crate::{Data, Error};

mod discord_helper;
pub(crate) mod leaderboard_command;
mod misc_commands;
mod player_commands;
mod reload_command;
mod subscription_commands;

pub(crate) async fn commands() -> Vec<poise::Command<crate::Data, crate::Error>> {
    let mut cmds: Vec<poise::Command<crate::Data, crate::Error>> = vec![
        reload_command::reload_matches(),
        player_commands::list_players(),
        misc_commands::random_hero(),
        leaderboard_command::leaderboard(),
    ];

    let admin_cmds: Vec<poise::Command<Data, Error>> = vec![
        player_commands::add_player(),
        player_commands::remove_player(),
        player_commands::rename_player(),
        player_commands::register_server(),
        subscription_commands::subscribe_channel(),
        subscription_commands::subscribe_week(),
        subscription_commands::subscribe_month(),
    ];

    for mut admin_cmd in admin_cmds.into_iter() {
        admin_cmd.required_permissions = Permissions::ADMINISTRATOR;
        admin_cmd.default_member_permissions = Permissions::ADMINISTRATOR;
        cmds.push(admin_cmd);
    }

    cmds
}
