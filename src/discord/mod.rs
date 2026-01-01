use poise::serenity_prelude::Permissions;

use crate::{Data, Error};

mod discord_helper;
pub(crate) mod leaderboard_command;
mod misc_commands;
mod register_command;
mod reload_command;
mod server_settings_command;

pub(crate) async fn commands() -> Vec<poise::Command<crate::Data, crate::Error>> {
    let mut cmds: Vec<poise::Command<crate::Data, crate::Error>> = vec![
        reload_command::refresh_matches(),
        misc_commands::roll(),
        misc_commands::flip(),
        leaderboard_command::leaderboard(),
        register_command::register(),
    ];

    let admin_cmds: Vec<poise::Command<Data, Error>> = vec![
        server_settings_command::server_settings(),
        reload_command::refresh_server_matches(),
    ];

    for mut admin_cmd in admin_cmds.into_iter() {
        admin_cmd.required_permissions = Permissions::ADMINISTRATOR;
        admin_cmd.default_member_permissions = Permissions::ADMINISTRATOR;
        cmds.push(admin_cmd);
    }

    cmds
}

