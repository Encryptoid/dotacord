use std::time::Duration;

use serenity::all::{
    ButtonStyle, ComponentInteractionCollector, CreateActionRow, CreateButton, CreateComponent,
    CreateInteractionResponse, CreateInteractionResponseMessage,
};
use tracing::info;

use crate::database::heroes_db::{self, HeroLookup, HeroModel, Position};
use crate::discord::discord_helper::{self, CmdCtx};
use crate::leaderboard::emoji::Emoji;
use crate::{Context, Error};

const BUTTON_ID_CARRY: &str = "dotacord_hero_carry";
const BUTTON_ID_MID: &str = "dotacord_hero_mid";
const BUTTON_ID_OFFLANE: &str = "dotacord_hero_offlane";
const BUTTON_ID_SUPPORT: &str = "dotacord_hero_support";

/// View and edit hero position flags
#[poise::command(slash_command, guild_only)]
pub async fn heroes(
    ctx: Context<'_>,
    #[description = "Hero name (e.g. Storm Spirit or stormspirit)"] hero_name: String,
) -> Result<(), Error> {
    let cmd_ctx = discord_helper::get_command_ctx(ctx).await?;

    let hero_lookup = HeroLookup::load().await?;
    let hero = match hero_lookup.find_by_name(&hero_name) {
        Some(h) => h.clone(),
        None => {
            cmd_ctx
                .reply(
                    discord_helper::Ephemeral::Private,
                    format!("{} Hero **{}** not found.", Emoji::SILENCE, hero_name),
                )
                .await?;
            return Ok(());
        }
    };

    generate_hero_panel(&cmd_ctx, hero).await?;
    Ok(())
}

struct HeroState {
    hero_id: i32,
    name: String,
    is_carry: bool,
    is_mid: bool,
    is_offlane: bool,
    is_support: bool,
}

impl HeroState {
    fn from_model(model: &HeroModel) -> Self {
        Self {
            hero_id: model.hero_id,
            name: model.name.clone(),
            is_carry: model.is_carry,
            is_mid: model.is_mid,
            is_offlane: model.is_offlane,
            is_support: model.is_support,
        }
    }

    fn active_count(&self) -> u8 {
        self.is_carry as u8 + self.is_mid as u8 + self.is_offlane as u8 + self.is_support as u8
    }
}

async fn generate_hero_panel(ctx: &CmdCtx<'_>, hero: HeroModel) -> Result<(), Error> {
    let mut state = HeroState::from_model(&hero);

    let (content, components) = build_hero_panel(&state);

    let reply = ctx
        .discord_ctx
        .send(
            poise::CreateReply::default()
                .content(content)
                .components(components)
                .ephemeral(true),
        )
        .await?;

    let message = reply.message().await?;
    let message_id = message.id;

    while let Some(interaction) =
        ComponentInteractionCollector::new(ctx.discord_ctx.serenity_context())
            .author_id(ctx.discord_ctx.author().id)
            .message_id(message_id)
            .timeout(Duration::from_secs(60))
            .await
    {
        let custom_id = interaction.data.custom_id.as_str();

        let (position, current_value) = match custom_id {
            BUTTON_ID_CARRY => (Position::Carry, state.is_carry),
            BUTTON_ID_MID => (Position::Mid, state.is_mid),
            BUTTON_ID_OFFLANE => (Position::Offlane, state.is_offlane),
            BUTTON_ID_SUPPORT => (Position::Support, state.is_support),
            _ => {
                continue;
            }
        };

        let new_value = !current_value;

        if !new_value && state.active_count() <= 1 {
            let (content, components) = build_hero_panel(&state);
            let blocked_content = format!(
                "{}\n\n{} Cannot disable — hero must have at least one position.",
                content, Emoji::SILENCE
            );
            interaction
                .create_response(
                    &ctx.discord_ctx.serenity_context().http,
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::default()
                            .content(blocked_content)
                            .components(components),
                    ),
                )
                .await?;
            continue;
        }

        heroes_db::update_hero_position(state.hero_id, &position, new_value).await?;

        match &position {
            Position::Carry => state.is_carry = new_value,
            Position::Mid => state.is_mid = new_value,
            Position::Offlane => state.is_offlane = new_value,
            Position::Support => state.is_support = new_value,
        }

        info!(
            hero_id = state.hero_id,
            hero_name = state.name,
            position = ?position,
            enabled = new_value,
            "Hero position updated"
        );

        let (new_content, new_components) = build_hero_panel(&state);

        interaction
            .create_response(
                &ctx.discord_ctx.serenity_context().http,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::default()
                        .content(new_content)
                        .components(new_components),
                ),
            )
            .await?;
    }

    reply
        .edit(
            ctx.discord_ctx,
            poise::CreateReply::default()
                .content("*Hero panel closed*")
                .components(vec![]),
        )
        .await?;

    Ok(())
}

fn build_hero_panel(state: &HeroState) -> (String, Vec<CreateComponent<'static>>) {
    let content = format!(
        "## {} **{}** {}\n> Toggle position flags for this hero.",
        Emoji::DOUBLEDAMAGE, state.name, Emoji::DOUBLEDAMAGE
    );

    let carry_btn = build_position_button(BUTTON_ID_CARRY, "Carry", state.is_carry);
    let mid_btn = build_position_button(BUTTON_ID_MID, "Mid", state.is_mid);
    let offlane_btn = build_position_button(BUTTON_ID_OFFLANE, "Offlane", state.is_offlane);
    let support_btn = build_position_button(BUTTON_ID_SUPPORT, "Support", state.is_support);

    let button_row =
        CreateActionRow::Buttons(vec![carry_btn, mid_btn, offlane_btn, support_btn].into());

    let components = vec![CreateComponent::ActionRow(button_row)];

    (content, components)
}

fn build_position_button(
    custom_id: &str,
    label: &str,
    is_enabled: bool,
) -> CreateButton<'static> {
    let (emoji_str, style) = if is_enabled {
        (Emoji::GOODJOB, ButtonStyle::Success)
    } else {
        (Emoji::SILENCE, ButtonStyle::Secondary)
    };

    let mut btn = CreateButton::new(custom_id.to_string())
        .style(style)
        .label(label.to_string());

    if let Some(emoji) = discord_helper::parse_custom_emoji(emoji_str) {
        btn = btn.emoji(emoji);
    }

    btn
}
