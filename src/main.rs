mod config;
mod data;
mod discord;
mod leaderboard;
mod logging;
mod markdown;
mod util;
use ::serenity::all::Token;
use poise::serenity_prelude::{self as serenity};
use tracing::info;

use crate::data::{database_access, hero_cache};

struct Data {
    config: config::AppConfig,
}
pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cfg = config::load_config().expect("Could not load config");

    logging::init(&cfg)?;
    info!("Logging Initialised. Initialising Dotacord application");

    hero_cache::init_cache(&cfg.heroes_path).expect("Could not init hero cache");
    database_access::init_database(&cfg.database_path)?;

    // if cfg.clear_commands_on_startup {
    //     clear_commands_from_server(&cfg).await?;
    // }

    // let options = poise::FrameworkOptions {
    //     commands: discord::commands().await,
    //     ..Default::default()
    // };

    let cfg_for_setup = cfg.clone();
    let framework = poise::Framework::new(poise::FrameworkOptions {
        commands: discord::commands().await,
        ..Default::default()
    });
    // .options(poise::FrameworkOptions {
    //     commands: discord::commands().await,
    //     // reply_callback: Some(|_ctx, mut builder| {
    //     //     // builder.embeds.clear();
    //     //     // builder.attachments.clear();
    //     //     // builder.components = None;
    //     //     // builder.allowed_mentions = None;
    //     //     builder
    //     // }),
    //     ..Default::default()
    // })
    // (move |ctx, _ready, framework| {
    //     let config_clone = cfg_for_setup.clone();
    //     Box::pin(async move {
    //         info!(
    //             "Setting up Discord client with online status: {:?}",
    //             cfg_for_setup.online_status
    //         );
    //         ctx.set_presence(None, cfg_for_setup.online_status);
    //         if let Some(guild_id) = cfg_for_setup.test_guild {
    //             let guild = serenity::GuildId::new(guild_id as u64);
    //             poise::builtins::register_in_guild(ctx, &framework.options().commands, guild)
    //                 .await?;
    //         } else {
    //             poise::builtins::register_globally(ctx, &framework.options().commands).await?;
    //         }
    //         Ok(Data {
    //             config: config_clone,
    //         })
    //     })
    // })
    // .build();

    let mut client = serenity::ClientBuilder::new(
        Token::from_env(cfg.discord_api_key)?,
        serenity::GatewayIntents::non_privileged(),
    ).data(cfg_for_setup)
    .framework(framework)
    .await?;

    info!("Setup complete. Starting client listener");

    client.start().await?;
    Ok(())
}

// async fn clear_commands_from_server(cfg: &config::AppConfig) -> Result<(), Error> {
//     // Create a simple HTTP client using serenity to call the Discord API
//     let http = serenity::http::Http::new(&cfg.discord_api_key);
//     // Ensure the Http knows the application id; some serenity methods require it and will
//     // return Http(ApplicationIdMissing) if not set. Fetch current application info and set it.
//     let app_info = http.get_current_application_info().await?;
//     http.set_application_id(app_info.id);

//     if let Some(guild_id) = cfg.test_guild {
//         let guild = serenity::GuildId::new(guild_id as u64);
//         // Overwrite guild commands with empty list by calling create_guild_commands with an empty Vec
//         // The serenity HTTP method expects a serializable body (e.g. a Vec of command definitions),
//         // so passing an empty Vec will clear/replace existing guild commands.
//         let empty_body: Vec<serde_json::Value> = Vec::new();
//         http.create_guild_commands(guild, &empty_body).await?;
//         info!("Cleared guild application commands for guild {}", guild_id);
//     } else {
//         // Overwrite global (application) commands with empty list by calling create_global_commands with an empty Vec
//         let empty_body: Vec<serde_json::Value> = Vec::new();
//         http.create_global_commands(&empty_body).await?;
//         info!("Cleared global application commands");
//     }
//     info!("Cleared commands as requested; exiting.");
//     return Ok(());
// }
