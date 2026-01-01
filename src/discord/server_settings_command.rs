use std::time::Duration;

use serenity::all::{
    ButtonStyle, ChannelType, Component, ComponentInteractionCollector,
    ComponentInteractionDataKind, CreateActionRow, CreateButton, CreateComponent,
    CreateInputText, CreateInteractionResponse, CreateInteractionResponseMessage, CreateLabel,
    CreateModal, CreateModalComponent, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption, GenericChannelId, InputTextStyle, LabelComponent,
    ModalInteractionCollector,
};
use tracing::info;

use crate::database::player_servers_db::PlayerServerModel;
use crate::database::{database_access, player_servers_db, servers_db};
use crate::discord::discord_helper::{self, CmdCtx};
use crate::leaderboard::emoji::Emoji;
use crate::{Context, Error};

const BUTTON_ID_WEEK: &str = "dotacord_settings_week";
const BUTTON_ID_MONTH: &str = "dotacord_settings_month";
const BUTTON_ID_RELOAD: &str = "dotacord_settings_reload";

const SELECT_ID_CHANNEL: &str = "dotacord_settings_channel";
const SELECT_ID_WEEKLY_DAY: &str = "dotacord_settings_weekly_day";
const SELECT_ID_WEEKLY_HOUR: &str = "dotacord_settings_weekly_hour";
const SELECT_ID_MONTHLY_WEEK: &str = "dotacord_settings_monthly_week";
const SELECT_ID_MONTHLY_WEEKDAY: &str = "dotacord_settings_monthly_weekday";
const SELECT_ID_MONTHLY_HOUR: &str = "dotacord_settings_monthly_hour";

const BUTTON_ID_CONFIG_WEEKLY: &str = "dotacord_config_weekly";
const BUTTON_ID_CONFIG_MONTHLY: &str = "dotacord_config_monthly";
const BUTTON_ID_PLAYERS: &str = "dotacord_config_players";
const BUTTON_ID_BACK: &str = "dotacord_back";

const SELECT_ID_PLAYER: &str = "dotacord_player_select";
const BUTTON_ID_SET_ID: &str = "dotacord_player_set_id";
const BUTTON_ID_SET_NAME: &str = "dotacord_player_set_name";
const BUTTON_ID_SET_DISCORD: &str = "dotacord_player_set_discord";
const BUTTON_ID_REMOVE: &str = "dotacord_player_remove";
const BUTTON_ID_ADD: &str = "dotacord_player_add";
const SELECT_ID_DISCORD_USER: &str = "dotacord_player_discord_user";
const SELECT_ID_ADD_DISCORD_USER: &str = "dotacord_player_add_discord_user";
const MODAL_ID_SET_ID: &str = "dotacord_modal_set_id";
const MODAL_ID_SET_NAME: &str = "dotacord_modal_set_name";
const MODAL_ID_ADD_PLAYER: &str = "dotacord_modal_add_player";

#[derive(Clone, Copy, PartialEq)]
enum Panel {
    Main,
    Weekly,
    Monthly,
    Players,
    PlayersDiscord,
    PlayersAddDiscord,
}

struct ServerState {
    channel_id: Option<i64>,
    is_sub_week: i32,
    is_sub_month: i32,
    is_sub_reload: i32,
    weekly_day: Option<i32>,
    weekly_hour: Option<i32>,
    monthly_week: Option<i32>,
    monthly_weekday: Option<i32>,
    monthly_hour: Option<i32>,
    selected_player_id: Option<i64>,
    players: Vec<PlayerServerModel>,
    pending_add_discord_user: Option<(i64, String)>,
}

#[poise::command(slash_command, guild_only)]
pub async fn server_settings(ctx: Context<'_>) -> Result<(), Error> {
    let cmd_ctx = discord_helper::get_command_ctx(ctx).await?;
    if !discord_helper::ensure_admin(&cmd_ctx).await? {
        return Ok(());
    }
    server_settings_panel(&cmd_ctx).await?;
    Ok(())
}

async fn server_settings_panel(ctx: &CmdCtx<'_>) -> Result<(), Error> {
    let server = servers_db::query_server_by_id(ctx.guild_id)
        .await?
        .ok_or_else(|| Error::from("Server not found in database"))?;

    let players = player_servers_db::query_server_players(ctx.guild_id).await?;

    let mut state = ServerState {
        channel_id: server.channel_id,
        is_sub_week: server.is_sub_week,
        is_sub_month: server.is_sub_month,
        is_sub_reload: server.is_sub_reload,
        weekly_day: server.weekly_day,
        weekly_hour: server.weekly_hour,
        monthly_week: server.monthly_week,
        monthly_weekday: server.monthly_weekday,
        monthly_hour: server.monthly_hour,
        selected_player_id: None,
        players,
        pending_add_discord_user: None,
    };

    let mut current_panel = Panel::Main;
    let (content, components) = build_panel(current_panel, &state);

    let reply = ctx
        .discord_ctx
        .send(
            poise::CreateReply::default()
                .content(content.clone())
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
            BUTTON_ID_WEEK => {
                state.is_sub_week = 1 - state.is_sub_week;
                servers_db::update_server_sub_week(ctx.guild_id, state.is_sub_week).await?;
                let status = if state.is_sub_week != 0 { "enabled" } else { "disabled" };
                info!(server_id = ctx.guild_id, status, "Weekly leaderboard subscription updated");
            }
            BUTTON_ID_MONTH => {
                state.is_sub_month = 1 - state.is_sub_month;
                servers_db::update_server_sub_month(ctx.guild_id, state.is_sub_month).await?;
                let status = if state.is_sub_month != 0 { "enabled" } else { "disabled" };
                info!(server_id = ctx.guild_id, status, "Monthly leaderboard subscription updated");
            }
            BUTTON_ID_RELOAD => {
                state.is_sub_reload = 1 - state.is_sub_reload;
                servers_db::update_server_sub_reload(ctx.guild_id, state.is_sub_reload).await?;
                let status = if state.is_sub_reload != 0 { "enabled" } else { "disabled" };
                info!(server_id = ctx.guild_id, status, "Auto-reload subscription updated");
            }
            BUTTON_ID_CONFIG_WEEKLY => {
                current_panel = Panel::Weekly;
            }
            BUTTON_ID_CONFIG_MONTHLY => {
                current_panel = Panel::Monthly;
            }
            BUTTON_ID_BACK => {
                match current_panel {
                    Panel::PlayersDiscord | Panel::PlayersAddDiscord => {
                        current_panel = Panel::Players;
                        state.pending_add_discord_user = None;
                    }
                    _ => {
                        current_panel = Panel::Main;
                    }
                }
            }
            BUTTON_ID_PLAYERS => {
                current_panel = Panel::Players;
            }
            SELECT_ID_CHANNEL => {
                if let ComponentInteractionDataKind::ChannelSelect { values } = &interaction.data.kind {
                    if let Some(channel_id) = values.first() {
                        let id = channel_id.get() as i64;
                        state.channel_id = Some(id);
                        servers_db::update_server_channel(ctx.guild_id, id).await?;
                        info!(server_id = ctx.guild_id, channel_id = id, "Leaderboard channel updated");
                    }
                }
            }
            SELECT_ID_WEEKLY_DAY => {
                if let ComponentInteractionDataKind::StringSelect { values } = &interaction.data.kind {
                    if let Some(value) = values.first() {
                        if let Ok(day) = value.parse::<i32>() {
                            state.weekly_day = Some(day);
                            servers_db::update_server_weekly_day(ctx.guild_id, day).await?;
                            info!(server_id = ctx.guild_id, day, "Weekly schedule day updated");
                        }
                    }
                }
            }
            SELECT_ID_WEEKLY_HOUR => {
                if let ComponentInteractionDataKind::StringSelect { values } = &interaction.data.kind {
                    if let Some(value) = values.first() {
                        if let Ok(hour) = value.parse::<i32>() {
                            state.weekly_hour = Some(hour);
                            servers_db::update_server_weekly_hour(ctx.guild_id, hour).await?;
                            info!(server_id = ctx.guild_id, hour, "Weekly schedule hour updated");
                        }
                    }
                }
            }
            SELECT_ID_MONTHLY_WEEK => {
                if let ComponentInteractionDataKind::StringSelect { values } = &interaction.data.kind {
                    if let Some(value) = values.first() {
                        if let Ok(week) = value.parse::<i32>() {
                            state.monthly_week = Some(week);
                            servers_db::update_server_monthly_week(ctx.guild_id, week).await?;
                            info!(server_id = ctx.guild_id, week, "Monthly schedule week updated");
                        }
                    }
                }
            }
            SELECT_ID_MONTHLY_WEEKDAY => {
                if let ComponentInteractionDataKind::StringSelect { values } = &interaction.data.kind {
                    if let Some(value) = values.first() {
                        if let Ok(weekday) = value.parse::<i32>() {
                            state.monthly_weekday = Some(weekday);
                            servers_db::update_server_monthly_weekday(ctx.guild_id, weekday).await?;
                            info!(server_id = ctx.guild_id, weekday, "Monthly schedule weekday updated");
                        }
                    }
                }
            }
            SELECT_ID_MONTHLY_HOUR => {
                if let ComponentInteractionDataKind::StringSelect { values } = &interaction.data.kind {
                    if let Some(value) = values.first() {
                        if let Ok(hour) = value.parse::<i32>() {
                            state.monthly_hour = Some(hour);
                            servers_db::update_server_monthly_hour(ctx.guild_id, hour).await?;
                            info!(server_id = ctx.guild_id, hour, "Monthly schedule hour updated");
                        }
                    }
                }
            }
            SELECT_ID_PLAYER => {
                if let ComponentInteractionDataKind::StringSelect { values } = &interaction.data.kind {
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
                                    "Player ID updated via settings panel"
                                );
                            }
                        }

                        let (new_content, new_components) = build_panel(current_panel, &state);
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
                        let new_name =
                            extract_modal_value(&modal_interaction.data.components).unwrap_or_default();

                        let txn = database_access::get_transaction().await?;
                        player_servers_db::rename_server_player_by_user_id(
                            &txn,
                            ctx.guild_id,
                            player_id,
                            &new_name,
                        )
                        .await?;
                        txn.commit().await?;

                        state.players =
                            player_servers_db::query_server_players(ctx.guild_id).await?;

                        info!(
                            server_id = ctx.guild_id,
                            player_id,
                            new_name = if new_name.is_empty() { "cleared" } else { &new_name },
                            "Player name updated via settings panel"
                        );

                        let (new_content, new_components) = build_panel(current_panel, &state);
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
                current_panel = Panel::PlayersDiscord;
            }
            BUTTON_ID_ADD => {
                current_panel = Panel::PlayersAddDiscord;
            }
            SELECT_ID_DISCORD_USER => {
                if let ComponentInteractionDataKind::UserSelect { values } = &interaction.data.kind {
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
                            current_panel = Panel::Players;

                            info!(
                                server_id = ctx.guild_id,
                                player_id,
                                discord_user_id,
                                discord_name,
                                "Discord user updated via settings panel"
                            );
                        }
                    }
                }
            }
            SELECT_ID_ADD_DISCORD_USER => {
                if let ComponentInteractionDataKind::UserSelect { values } = &interaction.data.kind {
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
                                        crate::database::players_db::ensure_player_exists(&txn, player_id)
                                            .await?;
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
                                        current_panel = Panel::Players;

                                        info!(
                                            server_id = ctx.guild_id,
                                            player_id,
                                            discord_id,
                                            discord_name,
                                            ?nickname,
                                            "Player added via settings panel"
                                        );
                                    }
                                }
                            }

                            let (new_content, new_components) = build_panel(current_panel, &state);
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
                        player_id,
                        "Player removed via settings panel"
                    );
                }
            }
            _ => {}
        }

        let (new_content, new_components) = build_panel(current_panel, &state);

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
                .content("*Settings panel closed*")
                .components(vec![]),
        )
        .await?;

    Ok(())
}

fn build_panel(panel: Panel, state: &ServerState) -> (String, Vec<CreateComponent<'static>>) {
    match panel {
        Panel::Main => build_main_panel(state),
        Panel::Weekly => build_weekly_panel(state),
        Panel::Monthly => build_monthly_panel(state),
        Panel::Players => build_players_panel(state),
        Panel::PlayersDiscord => build_players_discord_select_panel(state),
        Panel::PlayersAddDiscord => build_players_add_discord_select_panel(),
    }
}

fn build_main_panel(state: &ServerState) -> (String, Vec<CreateComponent<'static>>) {
    let content = format!(
        "# {} **Dotacord Server Settings** {}",
        Emoji::NERD, Emoji::ORACLE_BURN
    );

    let mut channel_label = CreateButton::new("dotacord_channel_label")
        .style(ButtonStyle::Primary)
        .label("Leaderboard Channel".to_string())
        .disabled(true);
    if let Some(emoji) = discord_helper::parse_custom_emoji(Emoji::IMMORTAL) {
        channel_label = channel_label.emoji(emoji);
    }
    let channel_label_row = CreateActionRow::Buttons(vec![channel_label].into());

    let config_weekly =
        build_toggle_button(BUTTON_ID_CONFIG_WEEKLY, "Weekly Leaderboard", state.is_sub_week);
    let config_monthly =
        build_toggle_button(BUTTON_ID_CONFIG_MONTHLY, "Monthly Leaderboard", state.is_sub_month);
    let reload_toggle =
        build_toggle_button(BUTTON_ID_RELOAD, "Auto Reload", state.is_sub_reload);
    let config_row =
        CreateActionRow::Buttons(vec![config_weekly, config_monthly, reload_toggle].into());

    let channel_select = build_channel_select(state.channel_id);
    let channel_row = CreateActionRow::SelectMenu(channel_select);

    let mut sub_label = CreateButton::new("dotacord_sub_label")
        .style(ButtonStyle::Primary)
        .label("Subscriptions".to_string())
        .disabled(true);
    if let Some(emoji) = discord_helper::parse_custom_emoji(Emoji::GUILD) {
        sub_label = sub_label.emoji(emoji);
    }
    let sub_label_row = CreateActionRow::Buttons(vec![sub_label].into());

    let mut players_btn = CreateButton::new(BUTTON_ID_PLAYERS)
        .style(ButtonStyle::Primary)
        .label("Manage Players".to_string());
    if let Some(emoji) = discord_helper::parse_custom_emoji(Emoji::SENTRY_WARD) {
        players_btn = players_btn.emoji(emoji);
    }
    let players_row = CreateActionRow::Buttons(vec![players_btn].into());

    let components = vec![
        CreateComponent::ActionRow(sub_label_row),
        CreateComponent::ActionRow(config_row),
        CreateComponent::ActionRow(channel_label_row),
        CreateComponent::ActionRow(channel_row),
        CreateComponent::ActionRow(players_row),
    ];

    (content, components)
}

fn build_weekly_panel(state: &ServerState) -> (String, Vec<CreateComponent<'static>>) {
    let content = format!(
        "## {} **Weekly Leaderboard** {}\n> Which day of the week to post the Weekly Leaderboard?",
        Emoji::GUILD, Emoji::IMMORTAL
    );

    let day_select = build_weekly_day_select(state.weekly_day);
    let day_row = CreateActionRow::SelectMenu(day_select);

    let hour_select = build_hour_select(SELECT_ID_WEEKLY_HOUR, state.weekly_hour, "Select hour");
    let hour_row = CreateActionRow::SelectMenu(hour_select);

    let toggle_row = build_subpanel_toggle_row(BUTTON_ID_WEEK, state.is_sub_week);

    let back_row = build_back_button_row();

    let components = vec![
        CreateComponent::ActionRow(day_row),
        CreateComponent::ActionRow(hour_row),
        CreateComponent::ActionRow(toggle_row),
        CreateComponent::ActionRow(back_row),
    ];

    (content, components)
}

fn build_monthly_panel(state: &ServerState) -> (String, Vec<CreateComponent<'static>>) {
    let content = format!(
        "## {} **Monthly Leaderboard** {}\n> Which day of the month to post the Monthly Leaderboard?",
        Emoji::GUILD, Emoji::TOP1
    );

    let week_select = build_monthly_week_select(state.monthly_week);
    let week_row = CreateActionRow::SelectMenu(week_select);

    let weekday_select = build_monthly_weekday_select(state.monthly_weekday);
    let weekday_row = CreateActionRow::SelectMenu(weekday_select);

    let hour_select = build_hour_select(SELECT_ID_MONTHLY_HOUR, state.monthly_hour, "Select hour");
    let hour_row = CreateActionRow::SelectMenu(hour_select);

    let toggle_row = build_subpanel_toggle_row(BUTTON_ID_MONTH, state.is_sub_month);

    let back_row = build_back_button_row();

    let components = vec![
        CreateComponent::ActionRow(week_row),
        CreateComponent::ActionRow(weekday_row),
        CreateComponent::ActionRow(hour_row),
        CreateComponent::ActionRow(toggle_row),
        CreateComponent::ActionRow(back_row),
    ];

    (content, components)
}

fn build_toggle_button(custom_id: &str, label: &str, is_enabled: i32) -> CreateButton<'static> {
    let (emoji_str, style, status) = if is_enabled != 0 {
        (Emoji::GOODJOB, ButtonStyle::Success, "Enabled")
    } else {
        (Emoji::SILENCE, ButtonStyle::Secondary, "Disabled")
    };

    let text = if label.is_empty() {
        status.to_string()
    } else {
        label.to_string()
    };

    let mut btn = CreateButton::new(custom_id.to_string())
        .style(style)
        .label(text);

    if let Some(emoji) = discord_helper::parse_custom_emoji(emoji_str) {
        btn = btn.emoji(emoji);
    }

    btn
}

fn build_subpanel_toggle_row(toggle_id: &str, is_enabled: i32) -> CreateActionRow<'static> {
    let toggle_btn = build_toggle_button(toggle_id, "", is_enabled);
    CreateActionRow::Buttons(vec![toggle_btn].into())
}

fn build_back_button_row() -> CreateActionRow<'static> {
    let back_btn = CreateButton::new(BUTTON_ID_BACK)
        .style(ButtonStyle::Secondary)
        .label("Back".to_string());
    CreateActionRow::Buttons(vec![back_btn].into())
}

fn build_channel_select(current: Option<i64>) -> CreateSelectMenu<'static> {
    let default_channels = current.map(|id| vec![GenericChannelId::new(id as u64)]);

    CreateSelectMenu::new(
        SELECT_ID_CHANNEL.to_string(),
        CreateSelectMenuKind::Channel {
            channel_types: Some(vec![ChannelType::Text].into()),
            default_channels: default_channels.map(|v| v.into()),
        },
    )
    .placeholder("Select leaderboard channel".to_string())
}

fn build_weekly_day_select(current: Option<i32>) -> CreateSelectMenu<'static> {
    let days = [
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ];

    let options: Vec<CreateSelectMenuOption> = days
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let day_num = (i + 1) as i32;
            CreateSelectMenuOption::new(*name, day_num.to_string())
                .default_selection(current == Some(day_num))
        })
        .collect();

    CreateSelectMenu::new(
        SELECT_ID_WEEKLY_DAY.to_string(),
        CreateSelectMenuKind::String {
            options: options.into(),
        },
    )
    .placeholder("Select day of week".to_string())
}

fn build_monthly_week_select(current: Option<i32>) -> CreateSelectMenu<'static> {
    let weeks = [
        (1, "First"),
        (2, "Second"),
        (3, "Third"),
        (4, "Fourth"),
        (5, "Last"),
    ];

    let options: Vec<CreateSelectMenuOption> = weeks
        .iter()
        .map(|(value, name)| {
            CreateSelectMenuOption::new(*name, value.to_string())
                .default_selection(current == Some(*value))
        })
        .collect();

    CreateSelectMenu::new(
        SELECT_ID_MONTHLY_WEEK.to_string(),
        CreateSelectMenuKind::String {
            options: options.into(),
        },
    )
    .placeholder("Select week of month".to_string())
}

fn build_monthly_weekday_select(current: Option<i32>) -> CreateSelectMenu<'static> {
    let days = [
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ];

    let options: Vec<CreateSelectMenuOption> = days
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let day_num = (i + 1) as i32;
            CreateSelectMenuOption::new(*name, day_num.to_string())
                .default_selection(current == Some(day_num))
        })
        .collect();

    CreateSelectMenu::new(
        SELECT_ID_MONTHLY_WEEKDAY.to_string(),
        CreateSelectMenuKind::String {
            options: options.into(),
        },
    )
    .placeholder("Select day of week".to_string())
}

fn build_hour_select(
    custom_id: &str,
    current: Option<i32>,
    placeholder: &str,
) -> CreateSelectMenu<'static> {
    let options: Vec<CreateSelectMenuOption> = (0..24)
        .map(|hour| {
            CreateSelectMenuOption::new(format!("{:02}:00", hour), hour.to_string())
                .default_selection(current == Some(hour))
        })
        .collect();

    CreateSelectMenu::new(
        custom_id.to_string(),
        CreateSelectMenuKind::String {
            options: options.into(),
        },
    )
    .placeholder(placeholder.to_string())
}

fn day_to_name(day: i32) -> &'static str {
    match day {
        1 => "Monday",
        2 => "Tuesday",
        3 => "Wednesday",
        4 => "Thursday",
        5 => "Friday",
        6 => "Saturday",
        7 => "Sunday",
        _ => "Unknown",
    }
}

fn week_to_name(week: i32) -> &'static str {
    match week {
        1 => "First",
        2 => "Second",
        3 => "Third",
        4 => "Fourth",
        5 => "Last",
        _ => "Unknown",
    }
}

fn build_players_panel(state: &ServerState) -> (String, Vec<CreateComponent<'static>>) {
    let content = format!("## {} **Manage Players** {}", Emoji::NERD, Emoji::SENTRY_WARD);

    let has_selection = state.selected_player_id.is_some();
    let has_players = !state.players.is_empty();

    let mut add_btn = CreateButton::new(BUTTON_ID_ADD)
        .style(ButtonStyle::Success)
        .label("Add New Player".to_string());
    if let Some(emoji) = discord_helper::parse_custom_emoji(Emoji::GOODJOB) {
        add_btn = add_btn.emoji(emoji);
    }

    let back_btn = CreateButton::new(BUTTON_ID_BACK)
        .style(ButtonStyle::Secondary)
        .label("Back".to_string());

    let add_back_row = CreateActionRow::Buttons(vec![add_btn, back_btn].into());

    if !has_players {
        return (content, vec![CreateComponent::ActionRow(add_back_row)]);
    }

    let player_select = build_player_select(&state.players, state.selected_player_id);
    let player_row = CreateActionRow::SelectMenu(player_select);

    let set_discord_btn = build_player_action_button(
        BUTTON_ID_SET_DISCORD,
        "Set Discord User",
        Emoji::GUILD,
        !has_selection,
    );
    let set_name_btn =
        build_player_action_button(BUTTON_ID_SET_NAME, "Set Nickname", Emoji::TP, !has_selection);
    let set_id_btn =
        build_player_action_button(BUTTON_ID_SET_ID, "Set Player ID", Emoji::MIDAS, !has_selection);

    let mut remove_btn = CreateButton::new(BUTTON_ID_REMOVE)
        .style(ButtonStyle::Danger)
        .label("Remove".to_string())
        .disabled(!has_selection);
    if let Some(emoji) = discord_helper::parse_custom_emoji(Emoji::SILENCE) {
        remove_btn = remove_btn.emoji(emoji);
    }

    let action_row =
        CreateActionRow::Buttons(vec![set_discord_btn, set_name_btn, set_id_btn, remove_btn].into());

    let components = vec![
        CreateComponent::ActionRow(player_row),
        CreateComponent::ActionRow(action_row),
        CreateComponent::ActionRow(add_back_row),
    ];

    (content, components)
}

fn build_players_discord_select_panel(state: &ServerState) -> (String, Vec<CreateComponent<'static>>) {
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

fn build_players_add_discord_select_panel() -> (String, Vec<CreateComponent<'static>>) {
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

fn build_player_action_button(
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
        .required(false)
        .max_length(32);

    CreateModal::new(MODAL_ID_SET_NAME, "Change Player Nickname").components(vec![
        CreateModalComponent::Label(CreateLabel::input_text("New Nickname (empty to clear)", input)),
    ])
}

fn create_add_player_modal(discord_name: &str) -> CreateModal<'static> {
    let id_input = CreateInputText::new(InputTextStyle::Short, "player_id")
        .placeholder("e.g. 123456789".to_string())
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

