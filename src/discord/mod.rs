use poise::serenity_prelude::Permissions;

use crate::{Data, Error};

mod discord_helper;
mod flip_command;
pub(crate) mod leaderboard_command;
mod manage_players_command;
mod misc_commands;
mod player_commands;
mod register_command;
mod register_server;
mod reload_command;
mod server_settings_command;
mod subscription_commands;

pub(crate) async fn commands() -> Vec<poise::Command<crate::Data, crate::Error>> {
    let mut cmds: Vec<poise::Command<crate::Data, crate::Error>> = vec![
        reload_command::reload_matches(),
        player_commands::players(),
        misc_commands::random_hero(),
        misc_commands::roll(),
        leaderboard_command::leaderboard(),
        flip_command::flip(),
        register_command::register(),
    ];

    let test = poise::Command {
        name: std::borrow::Cow::Borrowed("test"),
        description: Some(std::borrow::Cow::Borrowed("Test command for development")),
        slash_action: Some(|ctx| {
            Box::pin(async move {
                ctx.say("Test command executed").await.unwrap();
                Ok(())
            })
        }),
        ..Default::default()
    };

    cmds.push(test);

    let admin_cmds: Vec<poise::Command<Data, Error>> = vec![
        register_server::register_server(),
        subscription_commands::subscribe_channel(),
        subscription_commands::subscribe(),
        server_settings_command::server_settings(),
        manage_players_command::manage_players(),
    ];

    for mut admin_cmd in admin_cmds.into_iter() {
        admin_cmd.required_permissions = Permissions::ADMINISTRATOR;
        admin_cmd.default_member_permissions = Permissions::ADMINISTRATOR;
        cmds.push(admin_cmd);
    }

    cmds
}
