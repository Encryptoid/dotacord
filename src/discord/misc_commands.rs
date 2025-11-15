use rand::Rng;
use tracing::info;

use crate::database::{database_access, hero_cache};
use crate::discord::discord_helper;
use crate::leaderboard::emoji::Emoji;
use crate::{Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn random_hero(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = discord_helper::guild_id(&ctx)?;
    let mut conn = database_access::get_new_connection().await?;
    if !discord_helper::validate_command(&ctx, &mut conn, guild_id).await? {
        return Ok(());
    }
    info!("Selecting random hero from cache");
    let hero = hero_cache::get_random_hero();
    info!(hero = hero.as_str(), "Random hero selected");
    discord_helper::public_reply(&ctx, format!("Random Hero: {hero}")).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn roll(
    ctx: Context<'_>,
    #[description = "Maximum Number (default: 100)"] max: Option<i32>,
) -> Result<(), Error> {
    let guild_id = discord_helper::guild_id(&ctx)?;
    let mut conn = database_access::get_new_connection().await?;
    if !discord_helper::validate_command(&ctx, &mut conn, guild_id).await? {
        return Ok(());
    }
    let max = max.unwrap_or(100);
    if max < 1 {
        discord_helper::private_reply(&ctx, "Maximum must be at least 1".to_string()).await?;
        return Ok(());
    }

    let base_content = format!("Rolling: `1` -> `{max}`\n\n");

    let result = rand::rng().random_range(1..=max);
    info!(max = max, result = result, "Roll command executed");

    let final_content = format!("Rolled: `{result}` {}", Emoji::BOUNTYRUNE);
    discord_helper::reply_countdown(&ctx, &base_content, "", final_content).await?;

    Ok(())
}
