mod api;
mod config;
mod database;
mod discord;
mod leaderboard;
mod logging;
mod markdown;
mod scheduler;
mod util;
use ::serenity::all::Token;
use poise::serenity_prelude::{self as serenity};
use tracing::info;

use crate::database::{database_access, hero_cache};

#[derive(Debug)]
struct Data {
    config: config::AppConfig,
}
pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    let cfg = config::load_config().expect("Could not load config");

    logging::init(&cfg)?;
    info!("Logging Initialised. Initialising Dotacord application");

    hero_cache::init_cache(&cfg.heroes_path).expect("Could not init hero cache");
    database_access::init_database(&cfg.database_path).await?;

    // if cfg.clear_commands_on_startup {
        // clear_commands_from_server(&cfg).await?;
    // }

    let cfg_for_scheduler = cfg.clone();
    let commands = discord::commands().await;

    let token = Token::from_env(&cfg.discord_api_key)?;
    let http = serenity::http::Http::new(token.clone());
    http.set_application_id(http.get_current_application_info().await?.id);

    info!("Registering application commands");
    if let Some(guild_id) = cfg.test_guild {
        let guild = serenity::GuildId::new(guild_id);
        poise::builtins::register_in_guild(&http, &commands, guild).await?;
    } else {
        poise::builtins::register_globally(&http, &commands).await?;
    }

    let framework = poise::Framework::new(poise::FrameworkOptions {
        commands,
        on_error: |error| {
            Box::pin(async move {
                tracing::error!("Poise error: {:?}", error);
                if let Err(e) = poise::builtins::on_error(error).await {
                    tracing::error!("Error while handling error: {:?}", e);
                }
            })
        },
        ..Default::default()
    });

    let cfg_arc = std::sync::Arc::new(Data { config: cfg });
    let mut client =
        serenity::ClientBuilder::new(token, serenity::GatewayIntents::non_privileged())
            .data(cfg_arc)
            .framework(framework)
            .await?;

    let http_for_scheduler = client.http.clone();
    scheduler::spawn_scheduler(cfg_for_scheduler, http_for_scheduler);

    info!("Setup complete. Starting client listener");

    client.start().await?;
    Ok(())
}

async fn clear_commands_from_server(cfg: &config::AppConfig) -> Result<(), Error> {
    // Create a simple HTTP client using serenity to call the Discord API
    let token = Token::from_env(&cfg.discord_api_key)?;
    let http = serenity::http::Http::new(token);
    // Ensure the Http knows the application id; some serenity methods require it and will
    // return Http(ApplicationIdMissing) if not set. Fetch current application info and set it.
    let app_info = http.get_current_application_info().await?;
    http.set_application_id(app_info.id);

    if let Some(guild_id) = cfg.test_guild {
        let guild = serenity::GuildId::new(guild_id as u64);
        // Overwrite guild commands with empty list by calling create_guild_commands with an empty Vec
        // The serenity HTTP method expects a serializable body (e.g. a Vec of command definitions),
        // so passing an empty Vec will clear/replace existing guild commands.
        let empty_body: Vec<serde_json::Value> = Vec::new();
        http.create_guild_commands(guild, &empty_body).await?;
        info!("Cleared guild application commands for guild {}", guild_id);
    } else {
        // Overwrite global (application) commands with empty list by calling create_global_commands with an empty Vec
        let empty_body: Vec<serde_json::Value> = Vec::new();
        http.create_global_commands(&empty_body).await?;
        info!("Cleared global application commands");
    }
    info!("Cleared commands as requested; exiting.");
    return Ok(());
}
