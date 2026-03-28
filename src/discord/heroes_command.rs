use std::time::Duration;

use serenity::all::{
    ButtonStyle, Component, ComponentInteractionCollector, CreateActionRow, CreateButton,
    CreateComponent, CreateInputText, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateLabel, CreateModal, CreateModalComponent,
    InputTextStyle, LabelComponent, ModalInteractionCollector,
};
use tracing::info;

use crate::database::heroes_db::{self, HeroLookup, HeroModel};
use crate::database::heroes_db::Position;
use crate::discord::discord_helper::{self, CmdCtx};
use crate::leaderboard::emoji::Emoji;
use crate::{Context, Error};

const BUTTON_ID_CARRY: &str = "dotacord_hero_carry";
const BUTTON_ID_MID: &str = "dotacord_hero_mid";
const BUTTON_ID_OFFLANE: &str = "dotacord_hero_offlane";
const BUTTON_ID_SUPPORT: &str = "dotacord_hero_support";
const BUTTON_ID_ADD_NICKNAME: &str = "dotacord_hero_add_nick";
const BUTTON_ID_REMOVE_NICKNAME: &str = "dotacord_hero_remove_nick";
const MODAL_ID_ADD_NICKNAME: &str = "dotacord_modal_add_nick";
const MODAL_ID_REMOVE_NICKNAME: &str = "dotacord_modal_remove_nick";

/// View and edit hero position flags and nicknames
#[poise::command(slash_command, guild_only)]
pub async fn heroes(
    ctx: Context<'_>,
    #[description = "Hero name or nickname (e.g. Storm Spirit or stormspirit)"] hero_name: String,
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

    let nicknames = hero_lookup.get_nicknames(hero.hero_id).to_vec();
    generate_hero_panel(&cmd_ctx, hero, nicknames).await?;
    Ok(())
}

struct HeroState {
    hero_id: i32,
    name: String,
    is_carry: bool,
    is_mid: bool,
    is_offlane: bool,
    is_support: bool,
    nicknames: Vec<String>,
}

impl HeroState {
    fn from_model(model: &HeroModel, nicknames: Vec<String>) -> Self {
        Self {
            hero_id: model.hero_id,
            name: model.name.clone(),
            is_carry: model.is_carry,
            is_mid: model.is_mid,
            is_offlane: model.is_offlane,
            is_support: model.is_support,
            nicknames,
        }
    }

    fn active_count(&self) -> u8 {
        self.is_carry as u8 + self.is_mid as u8 + self.is_offlane as u8 + self.is_support as u8
    }
}

async fn generate_hero_panel(
    ctx: &CmdCtx<'_>,
    hero: HeroModel,
    nicknames: Vec<String>,
) -> Result<(), Error> {
    let mut state = HeroState::from_model(&hero, nicknames);

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

        match custom_id {
            BUTTON_ID_CARRY | BUTTON_ID_MID | BUTTON_ID_OFFLANE | BUTTON_ID_SUPPORT => {
                let (position, current_value) = match custom_id {
                    BUTTON_ID_CARRY => (Position::Carry, state.is_carry),
                    BUTTON_ID_MID => (Position::Mid, state.is_mid),
                    BUTTON_ID_OFFLANE => (Position::Offlane, state.is_offlane),
                    BUTTON_ID_SUPPORT => (Position::Support, state.is_support),
                    _ => unreachable!(),
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
            }
            BUTTON_ID_ADD_NICKNAME => {
                let modal = create_add_nickname_modal();
                interaction
                    .create_response(
                        &ctx.discord_ctx.serenity_context().http,
                        CreateInteractionResponse::Modal(modal),
                    )
                    .await?;

                if let Some(modal_interaction) =
                    ModalInteractionCollector::new(ctx.discord_ctx.serenity_context())
                        .author_id(ctx.discord_ctx.author().id)
                        .timeout(Duration::from_secs(60))
                        .filter(move |m| m.data.custom_id == MODAL_ID_ADD_NICKNAME)
                        .await
                {
                    let nickname = extract_modal_value(&modal_interaction.data.components)
                        .unwrap_or_default();
                    let nickname = nickname.trim().to_string();

                    let mut error_message: Option<String> = None;

                    if nickname.is_empty() {
                        error_message = Some(format!("{} Nickname cannot be empty.", Emoji::SILENCE));
                    } else if state.nicknames.iter().any(|n| n.to_lowercase() == nickname.to_lowercase()) {
                        error_message = Some(format!(
                            "{} Nickname **{}** already exists.",
                            Emoji::SILENCE, nickname
                        ));
                    } else {
                        heroes_db::insert_nickname(state.hero_id, &nickname).await?;
                        state.nicknames.push(nickname.clone());
                        info!(
                            hero_id = state.hero_id,
                            hero_name = state.name,
                            nickname,
                            "Hero nickname added"
                        );
                    }

                    let (panel_content, new_components) = build_hero_panel(&state);
                    let new_content = match error_message {
                        Some(err) => format!("{}\n\n{}", panel_content, err),
                        None => panel_content,
                    };
                    modal_interaction
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
                continue;
            }
            BUTTON_ID_REMOVE_NICKNAME => {
                let current_nicks = state.nicknames.join(", ");
                let modal = create_remove_nickname_modal(&current_nicks);
                interaction
                    .create_response(
                        &ctx.discord_ctx.serenity_context().http,
                        CreateInteractionResponse::Modal(modal),
                    )
                    .await?;

                if let Some(modal_interaction) =
                    ModalInteractionCollector::new(ctx.discord_ctx.serenity_context())
                        .author_id(ctx.discord_ctx.author().id)
                        .timeout(Duration::from_secs(60))
                        .filter(move |m| m.data.custom_id == MODAL_ID_REMOVE_NICKNAME)
                        .await
                {
                    let nickname = extract_modal_value(&modal_interaction.data.components)
                        .unwrap_or_default();
                    let nickname = nickname.trim().to_string();

                    let mut error_message: Option<String> = None;

                    let found = state
                        .nicknames
                        .iter()
                        .position(|n| n.to_lowercase() == nickname.to_lowercase());

                    if let Some(idx) = found {
                        let exact_nick = state.nicknames.remove(idx);
                        heroes_db::delete_nickname(state.hero_id, &exact_nick).await?;
                        info!(
                            hero_id = state.hero_id,
                            hero_name = state.name,
                            nickname = exact_nick,
                            "Hero nickname removed"
                        );
                    } else {
                        error_message = Some(format!(
                            "{} Nickname **{}** not found.",
                            Emoji::SILENCE, nickname
                        ));
                    }

                    let (panel_content, new_components) = build_hero_panel(&state);
                    let new_content = match error_message {
                        Some(err) => format!("{}\n\n{}", panel_content, err),
                        None => panel_content,
                    };
                    modal_interaction
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
                continue;
            }
            _ => {}
        }

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
    let nicknames_display = if state.nicknames.is_empty() {
        "None".to_string()
    } else {
        state.nicknames.join(", ")
    };

    let content = format!(
        "## {} **{}** {}\n> **Nicknames:** {}\n> Toggle position flags or manage nicknames.",
        Emoji::DOUBLEDAMAGE, state.name, Emoji::DOUBLEDAMAGE, nicknames_display
    );

    let carry_btn = build_position_button(BUTTON_ID_CARRY, "Carry", state.is_carry);
    let mid_btn = build_position_button(BUTTON_ID_MID, "Mid", state.is_mid);
    let offlane_btn = build_position_button(BUTTON_ID_OFFLANE, "Offlane", state.is_offlane);
    let support_btn = build_position_button(BUTTON_ID_SUPPORT, "Support", state.is_support);

    let position_row =
        CreateActionRow::Buttons(vec![carry_btn, mid_btn, offlane_btn, support_btn].into());

    let mut add_nick_btn = CreateButton::new(BUTTON_ID_ADD_NICKNAME)
        .style(ButtonStyle::Primary)
        .label("Add Nickname".to_string());
    if let Some(emoji) = discord_helper::parse_custom_emoji(Emoji::BOUNTYRUNE) {
        add_nick_btn = add_nick_btn.emoji(emoji);
    }

    let mut remove_nick_btn = CreateButton::new(BUTTON_ID_REMOVE_NICKNAME)
        .style(ButtonStyle::Danger)
        .label("Remove Nickname".to_string())
        .disabled(state.nicknames.is_empty());
    if let Some(emoji) = discord_helper::parse_custom_emoji(Emoji::GRAVE) {
        remove_nick_btn = remove_nick_btn.emoji(emoji);
    }

    let nickname_row =
        CreateActionRow::Buttons(vec![add_nick_btn, remove_nick_btn].into());

    let components = vec![
        CreateComponent::ActionRow(position_row),
        CreateComponent::ActionRow(nickname_row),
    ];

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

fn create_add_nickname_modal() -> CreateModal<'static> {
    let input = CreateInputText::new(InputTextStyle::Short, "nickname")
        .placeholder("e.g. Tree".to_string())
        .required(true)
        .min_length(1)
        .max_length(50);

    CreateModal::new(MODAL_ID_ADD_NICKNAME, "Add Hero Nickname").components(vec![
        CreateModalComponent::Label(CreateLabel::input_text("Nickname", input)),
    ])
}

fn create_remove_nickname_modal(current_nicknames: &str) -> CreateModal<'static> {
    let input = CreateInputText::new(InputTextStyle::Short, "nickname")
        .placeholder(current_nicknames.to_string())
        .required(true)
        .min_length(1)
        .max_length(50);

    CreateModal::new(MODAL_ID_REMOVE_NICKNAME, "Remove Hero Nickname").components(vec![
        CreateModalComponent::Label(CreateLabel::input_text("Nickname to remove", input)),
    ])
}

fn extract_modal_value(components: &[Component]) -> Option<String> {
    for component in components {
        if let Component::Label(label) = component {
            if let LabelComponent::InputText(input) = &label.component {
                return input.value.as_ref().map(|v| v.to_string());
            }
        }
    }
    None
}
