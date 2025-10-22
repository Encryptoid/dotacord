use tracing::info;

use crate::database::{database_access, hero_cache};
use crate::discord::discord_helper;
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
