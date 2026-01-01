use std::time::Duration;

use serenity::all::{
    ButtonStyle, Component, ComponentInteractionCollector, ComponentInteractionDataKind,
    CreateActionRow, CreateButton, CreateComponent, CreateInputText, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateLabel, CreateModal, CreateModalComponent,
    CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption, InputTextStyle,
    LabelComponent, ModalInteractionCollector,
};
use tracing::info;

use crate::database::{database_access, player_servers_db};
use crate::database::player_servers_db::PlayerServerModel;
use crate::discord::discord_helper::{self, CmdCtx};
use crate::leaderboard::emoji::Emoji;
use crate::{Context, Error};

const SELECT_ID_PLAYER: &str = "dotacord_manage_player_select";
const BUTTON_ID_SET_ID: &str = "dotacord_manage_set_id";
const BUTTON_ID_SET_NAME: &str = "dotacord_manage_set_name";
const BUTTON_ID_SET_DISCORD: &str = "dotacord_manage_set_discord";
const BUTTON_ID_REMOVE: &str = "dotacord_manage_remove";
const BUTTON_ID_BACK: &str = "dotacord_manage_back";
const SELECT_ID_DISCORD_USER: &str = "dotacord_manage_discord_user";
const BUTTON_ID_ADD: &str = "dotacord_manage_add";
const SELECT_ID_ADD_DISCORD_USER: &str = "dotacord_manage_add_discord_user";
const MODAL_ID_SET_ID: &str = "dotacord_modal_set_id";
const MODAL_ID_SET_NAME: &str = "dotacord_modal_set_name";
const MODAL_ID_ADD_PLAYER: &str = "dotacord_modal_add_player";

#[derive(Clone, Copy, PartialEq)]
enum PanelView {
    Main,
    SelectDiscordUser,
    SelectDiscordUserForAdd,
}

struct PanelState {
    selected_player_id: Option<i64>,
    players: Vec<PlayerServerModel>,
    current_view: PanelView,
    pending_add_discord_user: Option<(i64, String)>,
}

#[poise::command(slash_command, guild_only)]
pub async fn players(ctx: Context<'_>) -> Result<(), Error> {
    let cmd_ctx = discord_helper::get_command_ctx(ctx).await?;
    if !discord_helper::ensure_admin(&cmd_ctx).await? {
        return Ok(());
    }
    manage_players_panel(&cmd_ctx).await?;
    Ok(())
}

async fn manage_players_panel(ctx: &CmdCtx<'_>) -> Result<(), Error> {
    let players = player_servers_db::query_server_players(ctx.guild_id).await?;

    let mut state = PanelState {
        selected_player_id: None,
        players,
        current_view: PanelView::Main,
        pending_add_discord_user: None,
    };

    let (content, components) = build_panel(&state);
    let reply = ctx
        .discord_ctx
        .send(
            poise::CreateReply::default()
                .content(content)
                .components(components)
                .ephemeral(true),
        )
        .await?;

    while let Some(interaction) =
        ComponentInteractionCollector::new(ctx.discord_ctx.serenity_context())
            .author_id(ctx.discord_ctx.author().id)
            .channel_id(ctx.discord_ctx.channel_id())
            .timeout(Duration::from_secs(120))
            .await
    {
        let custom_id = interaction.data.custom_id.as_str();

        match custom_id {
            SELECT_ID_PLAYER => {
                if let ComponentInteractionDataKind::StringSelect { values } = &interaction.data.kind
                {
                    if let Some(value) = values.first() {
                        if let Ok(player_id) = value.parse::<i64>() {
                            state.selected_player_id = Some(player_id);
                        }
                    }
                }
            }
            BUTTON_ID_SET_ID => {
                if let Some(player_id) = state.selected_player_id {
                    let modal = create_set_id_modal(player_id);
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
                            .filter(move |m| m.data.custom_id == MODAL_ID_SET_ID)
                            .await
                    {
                        let new_id_str = extract_modal_value(&modal_interaction.data.components);
                        if let Some(id_str) = new_id_str {
                            if let Ok(new_player_id) = id_str.parse::<i64>() {
                                let txn = database_access::get_transaction().await?;
                                player_servers_db::update_player_id(
                                    &txn,
                                    ctx.guild_id,
                                    player_id,
                                    new_player_id,
                                )
                                .await?;
                                txn.commit().await?;

                                state.players =
                                    player_servers_db::query_server_players(ctx.guild_id).await?;
                                state.selected_player_id = Some(new_player_id);

                                info!(
                                    server_id = ctx.guild_id,
                                    old_id = player_id,
                                    new_id = new_player_id,
                                    "Player ID updated via manage panel"
                                );
                            }
                        }

                        let (new_content, new_components) = build_panel(&state);
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
            }
            BUTTON_ID_SET_NAME => {
                if let Some(player_id) = state.selected_player_id {
                    let current_name = state
                        .players
                        .iter()
                        .find(|p| p.player_id == player_id)
                        .map(|p| {
                            p.player_name
                                .as_ref()
                                .unwrap_or(&p.discord_name)
                                .as_str()
                        })
                        .unwrap_or("");

                    let modal = create_set_name_modal(current_name);
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
                            .filter(move |m| m.data.custom_id == MODAL_ID_SET_NAME)
                            .await
                    {
                        let new_name = extract_modal_value(&modal_interaction.data.components);
                        if let Some(name) = new_name {
                            let txn = database_access::get_transaction().await?;
                            player_servers_db::rename_server_player_by_user_id(
                                &txn,
                                ctx.guild_id,
                                player_id,
                                &name,
                            )
                            .await?;
                            txn.commit().await?;

                            state.players =
                                player_servers_db::query_server_players(ctx.guild_id).await?;

                            info!(
                                server_id = ctx.guild_id,
                                player_id,
                                new_name = name,
                                "Player name updated via manage panel"
                            );
                        }

                        let (new_content, new_components) = build_panel(&state);
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
            }
            BUTTON_ID_SET_DISCORD => {
                state.current_view = PanelView::SelectDiscordUser;
            }
            BUTTON_ID_ADD => {
                state.current_view = PanelView::SelectDiscordUserForAdd;
            }
            BUTTON_ID_BACK => {
                state.current_view = PanelView::Main;
                state.pending_add_discord_user = None;
            }
            SELECT_ID_DISCORD_USER => {
                if let ComponentInteractionDataKind::UserSelect { values } = &interaction.data.kind
                {
                    if let Some(user_id) = values.first() {
                        if let Some(player_id) = state.selected_player_id {
                            let user = user_id.to_user(&ctx.discord_ctx.serenity_context().http).await?;
                            let discord_user_id = user.id.get() as i64;
                            let discord_name = user.name.to_string();

                            let txn = database_access::get_transaction().await?;
                            player_servers_db::update_discord_user(
                                &txn,
                                ctx.guild_id,
                                player_id,
                                discord_user_id,
                                discord_name.clone(),
                            )
                            .await?;
                            txn.commit().await?;

                            state.players =
                                player_servers_db::query_server_players(ctx.guild_id).await?;
                            state.current_view = PanelView::Main;

                            info!(
                                server_id = ctx.guild_id,
                                player_id,
                                discord_user_id,
                                discord_name,
                                "Discord user updated via manage panel"
                            );
                        }
                    }
                }
            }
            SELECT_ID_ADD_DISCORD_USER => {
                if let ComponentInteractionDataKind::UserSelect { values } = &interaction.data.kind
                {
                    if let Some(user_id) = values.first() {
                        let user = user_id.to_user(&ctx.discord_ctx.serenity_context().http).await?;
                        let discord_user_id = user.id.get() as i64;
                        let discord_name = user.name.to_string();

                        state.pending_add_discord_user = Some((discord_user_id, discord_name.clone()));

                        let modal = create_add_player_modal(&discord_name);
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
                                .filter(move |m| m.data.custom_id == MODAL_ID_ADD_PLAYER)
                                .await
                        {
                            let (player_id_str, nickname) =
                                extract_add_player_modal_values(&modal_interaction.data.components);

                            if let Some(id_str) = player_id_str {
                                if let Ok(player_id) = id_str.parse::<i64>() {
                                    if let Some((discord_id, discord_name)) =
                                        state.pending_add_discord_user.take()
                                    {
                                        let txn = database_access::get_transaction().await?;
                                        crate::database::players_db::ensure_player_exists(&txn, player_id).await?;
                                        player_servers_db::insert_player_server(
                                            &txn,
                                            ctx.guild_id,
                                            player_id,
                                            nickname.clone(),
                                            Some(discord_id),
                                            discord_name.clone(),
                                        )
                                        .await?;
                                        txn.commit().await?;

                                        state.players =
                                            player_servers_db::query_server_players(ctx.guild_id).await?;
                                        state.current_view = PanelView::Main;

                                        info!(
                                            server_id = ctx.guild_id,
                                            player_id,
                                            discord_id,
                                            discord_name,
                                            ?nickname,
                                            "Player added via manage panel"
                                        );
                                    }
                                }
                            }

                            let (new_content, new_components) = build_panel(&state);
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
                }
            }
            BUTTON_ID_REMOVE => {
                if let Some(player_id) = state.selected_player_id {
                    let txn = database_access::get_transaction().await?;
                    player_servers_db::remove_server_player_by_user_id(&txn, ctx.guild_id, player_id)
                        .await?;
                    txn.commit().await?;

                    state.players = player_servers_db::query_server_players(ctx.guild_id).await?;
                    state.selected_player_id = None;

                    info!(
                        server_id = ctx.guild_id,
                        player_id, "Player removed via manage panel"
                    );

                    if state.players.is_empty() {
                        interaction
                            .create_response(
                                &ctx.discord_ctx.serenity_context().http,
                                CreateInteractionResponse::UpdateMessage(
                                    CreateInteractionResponseMessage::default()
                                        .content("*All players removed. Panel closed.*")
                                        .components(vec![]),
                                ),
                            )
                            .await?;
                        return Ok(());
                    }
                }
            }
            _ => {}
        }

        let (new_content, new_components) = build_panel(&state);
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
                .content("*Manage players panel closed*")
                .components(vec![]),
        )
        .await?;

    Ok(())
}

fn build_panel(state: &PanelState) -> (String, Vec<CreateComponent<'static>>) {
    match state.current_view {
        PanelView::Main => build_main_panel(state),
        PanelView::SelectDiscordUser => build_discord_select_panel(state),
        PanelView::SelectDiscordUserForAdd => build_add_discord_select_panel(),
    }
}

fn build_main_panel(state: &PanelState) -> (String, Vec<CreateComponent<'static>>) {
    let content = format!("# {} **Manage Players** {}", Emoji::NERD, Emoji::SENTRY_WARD);

    let has_selection = state.selected_player_id.is_some();
    let has_players = !state.players.is_empty();

    let mut add_btn = CreateButton::new(BUTTON_ID_ADD)
        .style(ButtonStyle::Success)
        .label("Add New Player");
    if let Some(emoji) = discord_helper::parse_custom_emoji(Emoji::GOODJOB) {
        add_btn = add_btn.emoji(emoji);
    }
    let add_row = CreateActionRow::Buttons(vec![add_btn].into());

    if !has_players {
        return (content, vec![CreateComponent::ActionRow(add_row)]);
    }

    let player_select = build_player_select(&state.players, state.selected_player_id);
    let player_row = CreateActionRow::SelectMenu(player_select);

    let set_id_btn = build_action_button(BUTTON_ID_SET_ID, "Set ID", Emoji::MIDAS, !has_selection);
    let set_name_btn = build_action_button(BUTTON_ID_SET_NAME, "Set Name", Emoji::TP, !has_selection);
    let set_discord_btn =
        build_action_button(BUTTON_ID_SET_DISCORD, "Set Discord", Emoji::GUILD, !has_selection);
    let mut remove_btn = CreateButton::new(BUTTON_ID_REMOVE)
        .style(ButtonStyle::Danger)
        .label("Remove")
        .disabled(!has_selection);
    if let Some(emoji) = discord_helper::parse_custom_emoji(Emoji::SILENCE) {
        remove_btn = remove_btn.emoji(emoji);
    }
    let action_row = CreateActionRow::Buttons(vec![set_id_btn, set_name_btn, set_discord_btn, remove_btn].into());

    let components = vec![
        CreateComponent::ActionRow(player_row),
        CreateComponent::ActionRow(action_row),
        CreateComponent::ActionRow(add_row),
    ];

    (content, components)
}

fn build_discord_select_panel(state: &PanelState) -> (String, Vec<CreateComponent<'static>>) {
    let player_name = state
        .selected_player_id
        .and_then(|id| state.players.iter().find(|p| p.player_id == id))
        .map(|p| p.player_name.as_ref().unwrap_or(&p.discord_name).as_str())
        .unwrap_or("Unknown");

    let content = format!(
        "## {} Select Discord User for: **{}**",
        Emoji::GUILD, player_name
    );

    let user_select = CreateSelectMenu::new(
        SELECT_ID_DISCORD_USER.to_string(),
        CreateSelectMenuKind::User { default_users: None },
    )
    .placeholder("Select Discord user".to_string());
    let user_row = CreateActionRow::SelectMenu(user_select);

    let back_btn = CreateButton::new(BUTTON_ID_BACK)
        .style(ButtonStyle::Secondary)
        .label("Back".to_string());
    let back_row = CreateActionRow::Buttons(vec![back_btn].into());

    let components = vec![
        CreateComponent::ActionRow(user_row),
        CreateComponent::ActionRow(back_row),
    ];

    (content, components)
}

fn build_player_select(
    players: &[PlayerServerModel],
    selected: Option<i64>,
) -> CreateSelectMenu<'static> {
    let options: Vec<CreateSelectMenuOption> = players
        .iter()
        .map(|p| {
            let label = match &p.player_name {
                Some(nickname) => format!("@{} ({}) - {}", p.discord_name, nickname, p.player_id),
                None => format!("@{} - {}", p.discord_name, p.player_id),
            };
            CreateSelectMenuOption::new(label, p.player_id.to_string())
                .default_selection(selected == Some(p.player_id))
        })
        .collect();

    CreateSelectMenu::new(
        SELECT_ID_PLAYER.to_string(),
        CreateSelectMenuKind::String {
            options: options.into(),
        },
    )
    .placeholder("Select a player to manage".to_string())
}

fn build_action_button(
    id: &str,
    label: &str,
    emoji_str: &str,
    disabled: bool,
) -> CreateButton<'static> {
    let mut btn = CreateButton::new(id.to_string())
        .style(ButtonStyle::Primary)
        .label(label.to_string())
        .disabled(disabled);

    if let Some(emoji) = discord_helper::parse_custom_emoji(emoji_str) {
        btn = btn.emoji(emoji);
    }

    btn
}

fn create_set_id_modal(current_id: i64) -> CreateModal<'static> {
    let input = CreateInputText::new(InputTextStyle::Short, "new_player_id")
        .placeholder(current_id.to_string())
        .required(true)
        .min_length(1)
        .max_length(15);

    CreateModal::new(MODAL_ID_SET_ID, "Change Dota Player ID").components(vec![
        CreateModalComponent::Label(CreateLabel::input_text("New Dota Player ID", input)),
    ])
}

fn create_set_name_modal(current_name: &str) -> CreateModal<'static> {
    let input = CreateInputText::new(InputTextStyle::Short, "new_name")
        .placeholder(current_name.to_string())
        .required(true)
        .min_length(1)
        .max_length(32);

    CreateModal::new(MODAL_ID_SET_NAME, "Change Player Nickname").components(vec![
        CreateModalComponent::Label(CreateLabel::input_text("New Nickname", input)),
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

fn build_add_discord_select_panel() -> (String, Vec<CreateComponent<'static>>) {
    let content = format!("## {} Select Discord User for new player", Emoji::GUILD);

    let user_select = CreateSelectMenu::new(
        SELECT_ID_ADD_DISCORD_USER.to_string(),
        CreateSelectMenuKind::User { default_users: None },
    )
    .placeholder("Select Discord user".to_string());
    let user_row = CreateActionRow::SelectMenu(user_select);

    let back_btn = CreateButton::new(BUTTON_ID_BACK)
        .style(ButtonStyle::Secondary)
        .label("Back".to_string());
    let back_row = CreateActionRow::Buttons(vec![back_btn].into());

    let components = vec![
        CreateComponent::ActionRow(user_row),
        CreateComponent::ActionRow(back_row),
    ];

    (content, components)
}

fn create_add_player_modal(discord_name: &str) -> CreateModal<'static> {
    let id_input = CreateInputText::new(InputTextStyle::Short, "player_id")
        .placeholder("e.g. 123456789")
        .required(true)
        .min_length(1)
        .max_length(15);

    let name_input = CreateInputText::new(InputTextStyle::Short, "nickname")
        .placeholder(discord_name.to_string())
        .required(false)
        .max_length(32);

    CreateModal::new(MODAL_ID_ADD_PLAYER, "Add Player").components(vec![
        CreateModalComponent::Label(CreateLabel::input_text("Dota Player ID", id_input)),
        CreateModalComponent::Label(CreateLabel::input_text("Nickname (optional)", name_input)),
    ])
}

fn extract_add_player_modal_values(components: &[Component]) -> (Option<String>, Option<String>) {
    let mut player_id = None;
    let mut nickname = None;

    for component in components {
        if let Component::Label(label) = component {
            if let LabelComponent::InputText(input) = &label.component {
                let custom_id = input.custom_id.as_str();
                let value = input.value.as_ref().map(|v| v.to_string());

                match custom_id {
                    "player_id" => player_id = value,
                    "nickname" => {
                        nickname = value.filter(|s| !s.is_empty());
                    }
                    _ => {}
                }
            }
        }
    }

    (player_id, nickname)
}

