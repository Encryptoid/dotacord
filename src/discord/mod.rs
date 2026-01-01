use poise::serenity_prelude::Permissions;

use crate::{Data, Error};

mod discord_helper;
pub(crate) mod leaderboard_command;
mod manage_players_command;
mod misc_commands;
mod register_command;
mod register_server;
mod reload_command;
mod server_settings_command;

pub(crate) async fn commands() -> Vec<poise::Command<crate::Data, crate::Error>> {
    let mut cmds: Vec<poise::Command<crate::Data, crate::Error>> = vec![
        reload_command::reload_matches(),
        misc_commands::roll(),
        misc_commands::flip(),
        leaderboard_command::leaderboard(),
        register_command::register(),
    ];

    let admin_cmds: Vec<poise::Command<Data, Error>> = vec![
        register_server::register_server(),
        server_settings_command::server_settings(),
        manage_players_command::players(),
    ];

    for mut admin_cmd in admin_cmds.into_iter() {
        admin_cmd.required_permissions = Permissions::ADMINISTRATOR;
        admin_cmd.default_member_permissions = Permissions::ADMINISTRATOR;
        cmds.push(admin_cmd);
    }

    cmds
}
