use rand::Rng;
use tracing::info;

use crate::discord::discord_helper::{self, CmdCtx, Ephemeral};
use crate::leaderboard::emoji::Emoji;
use crate::{Context, Error};

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

    let base_content = format!("### Rolling: `1` -> `{max}` {}\n", Emoji::WIZ_GLHF);

    let result = rand::rng().random_range(1..=max);
    info!(max = max, result = result, "Roll command executed");

    let member = ctx.author_member().await;
    let user = match &member {
        Some(m) => m.display_name().to_string(),
        None => ctx.author().name.to_string(),
    };
    let emoji = Emoji::BOUNTYRUNE;
    let final_content = format!("## `{user}` rolled: {emoji} `{result}` {emoji}");
    discord_helper::reply_countdown(
        &cmd_ctx,
        &base_content,
        "",
        final_content,
        cmd_ctx.app_cfg.roll_countdown_duration_sec,
    )
    .await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn flip(
    ctx: Context<'_>,
    choice1: Option<String>,
    choice2: Option<String>,
) -> Result<(), Error> {
    let cmd_ctx = discord_helper::get_command_ctx(ctx).await?;
    flip_inner(&cmd_ctx, choice1, choice2).await?;
    Ok(())
}

async fn flip_inner(
    ctx: &CmdCtx<'_>,
    choice1: Option<String>,
    choice2: Option<String>,
) -> Result<(), Error> {
    info!("Starting coin flip");
    let (heads_choice, tails_choice) = assign_coin_sides(choice1, choice2);

    let initial_content = flip_initial_content(&heads_choice, &tails_choice);

    let heads_wins = rand::rng().random_bool(0.5);
    let winner = if heads_wins {
        &heads_choice
    } else {
        &tails_choice
    };
    let coin_side = if heads_wins { "Heads" } else { "Tails" };

    info!(
        winner = winner.as_str(),
        coin_side = coin_side,
        "Coin flip result"
    );

    let final_content = format!(
        "{} **{}** has been chosen! {}",
        Emoji::AEGIS2015,
        winner,
        Emoji::AGHS_SCEPTER
    );
    discord_helper::reply_countdown(
        &ctx,
        &initial_content,
        "Flipping... ",
        final_content,
        ctx.app_cfg.flip_countdown_duration_sec,
    )
    .await?;

    Ok(())
}

fn flip_initial_content(heads_choice: &str, tails_choice: &str) -> String {
    let mut content = format!(
        "# {} Aghanim's Amazing Ambuguity Arbiter {}\n\n",
        Emoji::AWOOGA,
        Emoji::APEXMAGE
    );
    content.push_str(&format!("## {} vs {}\n\n", heads_choice, tails_choice));
    content
}

fn assign_coin_sides(choice1: Option<String>, choice2: Option<String>) -> (String, String) {
    let c1 = format!("`{}`", choice1.unwrap_or_else(|| "Heads".to_string()));
    let c2 = format!("`{}`", choice2.unwrap_or_else(|| "Tails".to_string()));
    if rand::rng().random_bool(0.5) {
        (c1, c2)
    } else {
        (c2, c1)
    }
}

