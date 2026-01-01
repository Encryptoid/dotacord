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
use clap::Parser;
use poise::serenity_prelude::{self as serenity};
use tracing::info;

#[derive(Parser)]
#[command(name = "dotacord")]
struct Args {
    #[arg(short = 'c', long)]
    clear_commands: bool,
    #[arg(short, long)]
    register: bool,
}

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

    let args = Args::parse();
    let cfg = config::load_config().expect("Could not load config");

    logging::init(&cfg)?;
    info!("Logging Initialised. Initialising Dotacord application");

    hero_cache::init_cache(&cfg.heroes_path).expect("Could not init hero cache");
    database_access::init_database(&cfg.database_path).await?;

    let cfg_for_scheduler = cfg.clone();
    let commands = discord::commands().await;

    let token = Token::from_env(&cfg.discord_api_key)?;
    let http = serenity::http::Http::new(token.clone());
    http.set_application_id(http.get_current_application_info().await?.id);

    if args.clear_commands {
        clear_commands(&cfg, &http).await?;
    }

    if args.register {
        info!("Registering application commands");
        if let Some(guild_id) = cfg.test_guild {
            let guild = serenity::GuildId::new(guild_id);
            poise::builtins::register_in_guild(&http, &commands, guild).await?;
        } else {
            poise::builtins::register_globally(&http, &commands).await?;
        }
    }

    let framework = poise::Framework::new(poise::FrameworkOptions {
        commands,
        on_error: |error| {
            Box::pin(async move {
                tracing::error!("Poise error: {:?}", error);
                if let poise::FrameworkError::Command { ctx, error, .. } = &error {
                    let msg = format!("{}", error);
                    if let Err(e) = ctx
                        .send(poise::CreateReply::new().content(msg).ephemeral(true))
                        .await
                    {
                        tracing::error!("Error sending ephemeral error: {:?}", e);
                    }
                } else if let Err(e) = poise::builtins::on_error(error).await {
                    tracing::error!("Error while handling error: {:?}", e);
                }
            })
        },
        ..Default::default()
    });

    let online_status = cfg.online_status;
    let cfg_arc = std::sync::Arc::new(Data { config: cfg });
    let mut client =
        serenity::ClientBuilder::new(token, serenity::GatewayIntents::non_privileged())
            .status(online_status)
            .data(cfg_arc)
            .framework(Box::new(framework))
            .await?;

    let http_for_scheduler = client.http.clone();
    scheduler::spawn_scheduler(cfg_for_scheduler, http_for_scheduler);

    info!("Setup complete. Starting client listener");

    client.start().await?;
    Ok(())
}

async fn clear_commands(cfg: &config::AppConfig, http: &serenity::http::Http) -> Result<(), Error> {
    info!("Clearing commands...");
    let empty_body: Vec<serde_json::Value> = Vec::new();

    info!("Clearing global commands...");
    http.create_global_commands(&empty_body).await?;
    info!("Cleared global application commands");

    if let Some(guild_id) = cfg.test_guild {
        info!("Clearing guild commands for {}...", guild_id);
        let guild = serenity::GuildId::new(guild_id);
        http.create_guild_commands(guild, &empty_body).await?;
        info!("Cleared guild application commands for guild {}", guild_id);
    }

    Ok(())
}
