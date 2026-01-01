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
const BUTTON_ID_REMOVE: &str = "dotacord_player_remove";
const MODAL_ID_SET_ID: &str = "dotacord_modal_set_id";
const MODAL_ID_SET_NAME: &str = "dotacord_modal_set_name";

#[derive(Clone, Copy, PartialEq)]
enum Panel {
    Main,
    Weekly,
    Monthly,
    Players,
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
    selected_discord_user: Option<(i64, String)>,
    players: Vec<PlayerServerModel>,
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
        selected_discord_user: None,
        players,
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

    let message = reply.message().await?;
    let message_id = message.id;

    while let Some(interaction) =
        ComponentInteractionCollector::new(ctx.discord_ctx.serenity_context())
            .author_id(ctx.discord_ctx.author().id)
            .message_id(message_id)
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
                current_panel = Panel::Main;
                state.selected_discord_user = None;
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
                if let ComponentInteractionDataKind::UserSelect { values } = &interaction.data.kind {
                    if let Some(user_id) = values.first() {
                        if let Some(user) = interaction.data.resolved.users.get(user_id) {
                            let discord_user_id = user.id.get() as i64;
                            let discord_name = user.name.to_string();
                            state.selected_discord_user = Some((discord_user_id, discord_name));
                        }
                    }
                }
            }
            BUTTON_ID_SET_ID => {
                if let Some((discord_user_id, ref discord_name)) = state.selected_discord_user {
                    let existing_player = state
                        .players
                        .iter()
                        .find(|p| p.discord_user_id == Some(discord_user_id));

                    let current_player_id = existing_player.map(|p| p.player_id);
                    let modal = create_set_id_modal(current_player_id);

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
                        let mut error_message: Option<String> = None;

                        if let Some(id_str) = new_id_str {
                            if let Ok(new_player_id) = id_str.parse::<i64>() {
                                let already_exists = state.players.iter().any(|p| {
                                    p.player_id == new_player_id
                                        && p.discord_user_id != Some(discord_user_id)
                                });

                                if already_exists {
                                    error_message = Some(format!(
                                        "{} Player ID `{}` is already added to this server.",
                                        Emoji::SILENCE, new_player_id
                                    ));
                                } else {
                                    let txn = database_access::get_transaction().await?;

                                    if let Some(old_player_id) = current_player_id {
                                        player_servers_db::update_player_id(
                                            &txn,
                                            ctx.guild_id,
                                            old_player_id,
                                            new_player_id,
                                        )
                                        .await?;
                                        info!(
                                            server_id = ctx.guild_id,
                                            old_id = old_player_id,
                                            new_id = new_player_id,
                                            "Player ID updated via settings panel"
                                        );
                                    } else {
                                        crate::database::players_db::ensure_player_exists(&txn, new_player_id)
                                            .await?;
                                        player_servers_db::insert_player_server(
                                            &txn,
                                            ctx.guild_id,
                                            new_player_id,
                                            None,
                                            Some(discord_user_id),
                                            discord_name.clone(),
                                        )
                                        .await?;
                                        info!(
                                            server_id = ctx.guild_id,
                                            player_id = new_player_id,
                                            discord_user_id,
                                            discord_name,
                                            "Player added via settings panel"
                                        );
                                    }

                                    txn.commit().await?;
                                    state.players =
                                        player_servers_db::query_server_players(ctx.guild_id).await?;
                                }
                            }
                        }

                        let (panel_content, new_components) = build_panel(current_panel, &state);
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
            }
            BUTTON_ID_SET_NAME => {
                if let Some((discord_user_id, _)) = state.selected_discord_user {
                    let existing_player = state
                        .players
                        .iter()
                        .find(|p| p.discord_user_id == Some(discord_user_id));

                    if let Some(player) = existing_player {
                        let player_id = player.player_id;
                        let current_name = player
                            .player_name
                            .as_ref()
                            .unwrap_or(&player.discord_name)
                            .as_str();

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
            }
            BUTTON_ID_REMOVE => {
                if let Some((discord_user_id, _)) = state.selected_discord_user {
                    let txn = database_access::get_transaction().await?;
                    player_servers_db::remove_server_player_by_discord_id(
                        &txn,
                        ctx.guild_id,
                        discord_user_id,
                    )
                    .await?;
                    txn.commit().await?;

                    state.players = player_servers_db::query_server_players(ctx.guild_id).await?;
                    state.selected_discord_user = None;

                    info!(
                        server_id = ctx.guild_id,
                        discord_user_id,
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
    }
}

fn build_main_panel(state: &ServerState) -> (String, Vec<CreateComponent<'static>>) {
    let content = format!(
        "# {} **Dotacord Server Settings** {}\n> Select a leaderboard channel and access server settings",
        Emoji::NERD, Emoji::ORACLE_BURN
    );

    let config_weekly =
        build_toggle_button(BUTTON_ID_CONFIG_WEEKLY, "Weekly Leaderboard Settings", state.is_sub_week);
    let config_monthly =
        build_toggle_button(BUTTON_ID_CONFIG_MONTHLY, "Monthly Leaderboard Settings", state.is_sub_month);
    let config_row =
        CreateActionRow::Buttons(vec![config_weekly, config_monthly].into());

    let mut players_btn = CreateButton::new(BUTTON_ID_PLAYERS)
        .style(ButtonStyle::Primary)
        .label("Manage Players".to_string());
    if let Some(emoji) = discord_helper::parse_custom_emoji(Emoji::MEEP_MOP) {
        players_btn = players_btn.emoji(emoji);
    }
    let players_row = CreateActionRow::Buttons(vec![players_btn].into());

    let components = vec![
        CreateComponent::ActionRow(CreateActionRow::SelectMenu(build_channel_select(state.channel_id))),
        CreateComponent::ActionRow(config_row),
        CreateComponent::ActionRow(CreateActionRow::Buttons(vec![build_toggle_button(BUTTON_ID_RELOAD, "Auto Reload Toggle", state.is_sub_reload)].into())),
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
    let existing_player = state.selected_discord_user.as_ref().and_then(|(discord_id, _)| {
        state.players.iter().find(|p| p.discord_user_id == Some(*discord_id))
    });

    let content = match (&state.selected_discord_user, existing_player) {
        (Some((_, discord_name)), Some(player)) => {
            let nickname_info = player
                .player_name
                .as_ref()
                .map(|n| format!(" ({})", n))
                .unwrap_or_default();
            format!(
                "## {} **Manage Players** {}\n### **@{}**{} - Player ID: `{}`",
                Emoji::ILLUSION_RUNE, Emoji::THROWGAME, discord_name, nickname_info, player.player_id
            )
        }
        (Some((_, discord_name)), None) => {
            format!(
                "## {} **Manage Players** {}\n> {} **@{}** is not added to this server",
                Emoji::ILLUSION_RUNE, Emoji::THROWGAME, Emoji::SILENCE, discord_name
            )
        }
        _ => {
            format!("## {} **Manage Players** {}", Emoji::ILLUSION_RUNE, Emoji::THROWGAME)
        }
    };

    let has_selection = state.selected_discord_user.is_some();
    let is_existing_player = existing_player.is_some();

    let user_select = CreateSelectMenu::new(
        SELECT_ID_PLAYER.to_string(),
        CreateSelectMenuKind::User { default_users: None },
    )
    .placeholder("Select Discord user".to_string());
    let player_row = CreateActionRow::SelectMenu(user_select);

    let set_id_btn = build_player_action_button(
        BUTTON_ID_SET_ID,
        "Set Player ID",
        Emoji::SENTRY_WARD,
        !has_selection,
    );
    let set_name_btn = build_player_action_button(
        BUTTON_ID_SET_NAME,
        "Set Nickname",
        Emoji::JUGG,
        !has_selection || !is_existing_player,
    );

    let mut remove_btn = CreateButton::new(BUTTON_ID_REMOVE)
        .style(ButtonStyle::Danger)
        .label("Remove".to_string())
        .disabled(!has_selection || !is_existing_player);
    if let Some(emoji) = discord_helper::parse_custom_emoji(Emoji::GRAVE) {
        remove_btn = remove_btn.emoji(emoji);
    }

    let action_row = CreateActionRow::Buttons(vec![set_id_btn, set_name_btn, remove_btn].into());

    let back_btn = CreateButton::new(BUTTON_ID_BACK)
        .style(ButtonStyle::Secondary)
        .label("Back".to_string());
    let back_row = CreateActionRow::Buttons(vec![back_btn].into());

    let components = vec![
        CreateComponent::ActionRow(player_row),
        CreateComponent::ActionRow(action_row),
        CreateComponent::ActionRow(back_row),
    ];

    (content, components)
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

fn create_set_id_modal(current_id: Option<i64>) -> CreateModal<'static> {
    let placeholder = current_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| "e.g. 123456789".to_string());

    let title = if current_id.is_some() {
        "Change Dota Player ID"
    } else {
        "Set Dota Player ID"
    };

    let input = CreateInputText::new(InputTextStyle::Short, "new_player_id")
        .placeholder(placeholder)
        .required(true)
        .min_length(1)
        .max_length(15);

    CreateModal::new(MODAL_ID_SET_ID, title).components(vec![
        CreateModalComponent::Label(CreateLabel::input_text("Dota Player ID", input)),
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


