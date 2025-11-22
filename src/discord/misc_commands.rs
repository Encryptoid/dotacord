use rand::Rng;
use tracing::info;

use crate::database::hero_cache;
use crate::{
    discord::discord_helper::{self, Ephemeral},
    leaderboard::emoji::Emoji,
};
use crate::{Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn random_hero(ctx: Context<'_>) -> Result<(), Error> {
    let cmd_ctx = discord_helper::get_command_ctx(ctx).await?;
    info!("Selecting random hero from cache");

    let hero = hero_cache::get_random_hero();
    info!(hero = hero.as_str(), "Random hero selected");

    cmd_ctx
        .reply(Ephemeral::Public, format!("Random Hero: {hero}"))
        .await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn roll(
    ctx: Context<'_>,
    #[description = "Maximum Number (default: 100)"] max: Option<i32>,
) -> Result<(), Error> {
    let cmd_ctx = discord_helper::get_command_ctx(ctx).await?;
    let max = max.unwrap_or(100);
    if max < 1 {
        cmd_ctx
            .reply(Ephemeral::Private, "Maximum must be at least 1".to_string())
            .await?;
        return Ok(());
    }

    let base_content = format!("Rolling: `1` -> `{max}`\n\n");

    let result = rand::rng().random_range(1..=max);
    info!(max = max, result = result, "Roll command executed");

    let final_content = format!("Rolled: `{result}` {}", Emoji::BOUNTYRUNE);
    discord_helper::reply_countdown(&cmd_ctx, &base_content, "", final_content).await?;

    Ok(())
}
