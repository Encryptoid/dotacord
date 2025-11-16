use rand::Rng;
use tracing::info;

use crate::discord::discord_helper::{self, CommandCtx};
use crate::leaderboard::emoji::Emoji;
use crate::{fmt, Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn flip(
    ctx: Context<'_>,
    choice1: Option<String>,
    choice2: Option<String>,
) -> Result<(), Error> {
    let cmd_ctx = discord_helper::get_command_ctx(ctx).await?;
    flip_command(&cmd_ctx, choice1, choice2).await?;
    Ok(())
}

async fn flip_command(
    ctx: &CommandCtx<'_>,
    choice1: Option<String>,
    choice2: Option<String>,
) -> Result<(), Error> {
    info!("Starting coin flip");
    let (heads_choice, tails_choice) = assign_coin_sides(choice1, choice2);

    let initial_content = create_initial_content(&heads_choice, &tails_choice);

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

    let final_content = fmt!(
        "{} **{}** has been chosen! {}",
        Emoji::AEGIS2015,
        winner,
        Emoji::AGHS_SCEPTER
    );
    discord_helper::reply_countdown(
        &ctx.discord_ctx,
        &initial_content,
        "Flipping... ",
        final_content,
    )
    .await?;

    Ok(())
}

fn create_initial_content(heads_choice: &str, tails_choice: &str) -> String {
    let mut content = fmt!(
        "# {} Aghanim's Amazing Ambuguity Arbiter {}\n\n",
        Emoji::AWOOGA,
        Emoji::APEXMAGE
    );
    content.push_str(&fmt!("## {} vs {}\n\n", heads_choice, tails_choice));
    content
}

fn assign_coin_sides(choice1: Option<String>, choice2: Option<String>) -> (String, String) {
    let c1 = fmt!("`{}`", choice1.unwrap_or_else(|| "Heads".to_string()));
    let c2 = fmt!("`{}`", choice2.unwrap_or_else(|| "Tails".to_string()));
    if rand::rng().random_bool(0.5) {
        (c1, c2)
    } else {
        (c2, c1)
    }
}
