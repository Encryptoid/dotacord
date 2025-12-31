use std::time::Duration;

use serenity::all::{
    ButtonStyle, ChannelType, ComponentInteractionCollector, ComponentInteractionDataKind,
    CreateActionRow, CreateButton, CreateComponent, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption, GenericChannelId,
};

use crate::database::servers_db;
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
const SELECT_ID_RELOAD_START: &str = "dotacord_settings_reload_start";
const SELECT_ID_RELOAD_END: &str = "dotacord_settings_reload_end";

const BUTTON_ID_CONFIG_WEEKLY: &str = "dotacord_config_weekly";
const BUTTON_ID_CONFIG_MONTHLY: &str = "dotacord_config_monthly";
const BUTTON_ID_CONFIG_RELOAD: &str = "dotacord_config_reload";
const BUTTON_ID_BACK: &str = "dotacord_back";

#[derive(Clone, Copy, PartialEq)]
enum Panel {
    Main,
    Weekly,
    Monthly,
    Reload,
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
    reload_start: Option<i32>,
    reload_end: Option<i32>,
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
        reload_start: server.reload_start,
        reload_end: server.reload_end,
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
            }
            BUTTON_ID_MONTH => {
                state.is_sub_month = 1 - state.is_sub_month;
                servers_db::update_server_sub_month(ctx.guild_id, state.is_sub_month).await?;
            }
            BUTTON_ID_RELOAD => {
                state.is_sub_reload = 1 - state.is_sub_reload;
                servers_db::update_server_sub_reload(ctx.guild_id, state.is_sub_reload).await?;
            }
            BUTTON_ID_CONFIG_WEEKLY => {
                current_panel = Panel::Weekly;
            }
            BUTTON_ID_CONFIG_MONTHLY => {
                current_panel = Panel::Monthly;
            }
            BUTTON_ID_CONFIG_RELOAD => {
                current_panel = Panel::Reload;
            }
            BUTTON_ID_BACK => {
                current_panel = Panel::Main;
            }
            SELECT_ID_CHANNEL => {
                if let ComponentInteractionDataKind::ChannelSelect { values } = &interaction.data.kind {
                    if let Some(channel_id) = values.first() {
                        let id = channel_id.get() as i64;
                        state.channel_id = Some(id);
                        servers_db::update_server_channel(ctx.guild_id, id).await?;
                    }
                }
            }
            SELECT_ID_WEEKLY_DAY => {
                if let ComponentInteractionDataKind::StringSelect { values } = &interaction.data.kind {
                    if let Some(value) = values.first() {
                        if let Ok(day) = value.parse::<i32>() {
                            state.weekly_day = Some(day);
                            servers_db::update_server_weekly_day(ctx.guild_id, day).await?;
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
                        }
                    }
                }
            }
            SELECT_ID_RELOAD_START => {
                if let ComponentInteractionDataKind::StringSelect { values } = &interaction.data.kind {
                    if let Some(value) = values.first() {
                        if let Ok(hour) = value.parse::<i32>() {
                            state.reload_start = Some(hour);
                            servers_db::update_server_reload_start(ctx.guild_id, hour).await?;
                        }
                    }
                }
            }
            SELECT_ID_RELOAD_END => {
                if let ComponentInteractionDataKind::StringSelect { values } = &interaction.data.kind {
                    if let Some(value) = values.first() {
                        if let Ok(hour) = value.parse::<i32>() {
                            state.reload_end = Some(hour);
                            servers_db::update_server_reload_end(ctx.guild_id, hour).await?;
                        }
                    }
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
        Panel::Reload => build_reload_panel(state),
    }
}

fn build_main_panel(state: &ServerState) -> (String, Vec<CreateComponent<'static>>) {
    let channel_info = match state.channel_id {
        Some(id) => format!("Subscription channel: <#{}>", id),
        None => String::from("No subscription channel configured"),
    };

    let content = format!("**Server Settings**\n\n{}", channel_info);

    let channel_select = build_channel_select(state.channel_id);
    let channel_row = CreateActionRow::SelectMenu(channel_select);

    let config_weekly =
        build_toggle_button(BUTTON_ID_CONFIG_WEEKLY, "Weekly Leaderboard", state.is_sub_week);
    let config_monthly =
        build_toggle_button(BUTTON_ID_CONFIG_MONTHLY, "Monthly Leaderboard", state.is_sub_month);
    let config_reload =
        build_toggle_button(BUTTON_ID_CONFIG_RELOAD, "Auto Reload", state.is_sub_reload);
    let config_row =
        CreateActionRow::Buttons(vec![config_weekly, config_monthly, config_reload].into());

    let components = vec![
        CreateComponent::ActionRow(channel_row),
        CreateComponent::ActionRow(config_row),
    ];

    (content, components)
}

fn build_weekly_panel(state: &ServerState) -> (String, Vec<CreateComponent<'static>>) {
    let day_name = state.weekly_day.map(day_to_name).unwrap_or("Not set");
    let hour_str = state
        .weekly_hour
        .map(|h| format!("{:02}:00 UTC", h))
        .unwrap_or_else(|| "Not set".to_string());

    let content = format!(
        "**Weekly Leaderboard Schedule**\n\nCurrent: {} at {}",
        day_name, hour_str
    );

    let toggle_btn = build_toggle_button(BUTTON_ID_WEEK, "", state.is_sub_week);
    let back_btn = CreateButton::new(BUTTON_ID_BACK)
        .style(ButtonStyle::Secondary)
        .label("Back".to_string());
    let button_row = CreateActionRow::Buttons(vec![toggle_btn, back_btn].into());

    let day_select = build_weekly_day_select(state.weekly_day);
    let day_row = CreateActionRow::SelectMenu(day_select);

    let hour_select = build_hour_select(SELECT_ID_WEEKLY_HOUR, state.weekly_hour, "Select hour");
    let hour_row = CreateActionRow::SelectMenu(hour_select);

    let components = vec![
        CreateComponent::ActionRow(button_row),
        CreateComponent::ActionRow(day_row),
        CreateComponent::ActionRow(hour_row),
    ];

    (content, components)
}

fn build_monthly_panel(state: &ServerState) -> (String, Vec<CreateComponent<'static>>) {
    let week_str = state.monthly_week.map(week_to_name).unwrap_or("Not set");
    let weekday_str = state.monthly_weekday.map(day_to_name).unwrap_or("Not set");
    let hour_str = state
        .monthly_hour
        .map(|h| format!("{:02}:00 UTC", h))
        .unwrap_or_else(|| "Not set".to_string());

    let content = format!(
        "**Monthly Leaderboard Schedule**\n\nCurrent: {} {} at {}",
        week_str, weekday_str, hour_str
    );

    let toggle_btn = build_toggle_button(BUTTON_ID_MONTH, "", state.is_sub_month);
    let back_btn = CreateButton::new(BUTTON_ID_BACK)
        .style(ButtonStyle::Secondary)
        .label("Back".to_string());
    let button_row = CreateActionRow::Buttons(vec![toggle_btn, back_btn].into());

    let week_select = build_monthly_week_select(state.monthly_week);
    let week_row = CreateActionRow::SelectMenu(week_select);

    let weekday_select = build_monthly_weekday_select(state.monthly_weekday);
    let weekday_row = CreateActionRow::SelectMenu(weekday_select);

    let hour_select = build_hour_select(SELECT_ID_MONTHLY_HOUR, state.monthly_hour, "Select hour");
    let hour_row = CreateActionRow::SelectMenu(hour_select);

    let components = vec![
        CreateComponent::ActionRow(button_row),
        CreateComponent::ActionRow(week_row),
        CreateComponent::ActionRow(weekday_row),
        CreateComponent::ActionRow(hour_row),
    ];

    (content, components)
}

fn build_reload_panel(state: &ServerState) -> (String, Vec<CreateComponent<'static>>) {
    let start_str = state
        .reload_start
        .map(|h| format!("{:02}:00", h))
        .unwrap_or_else(|| "Not set".to_string());
    let end_str = state
        .reload_end
        .map(|h| format!("{:02}:00", h))
        .unwrap_or_else(|| "Not set".to_string());

    let content = format!(
        "**Auto-Reload Time Window**\n\nCurrent: {} to {} (local time)",
        start_str, end_str
    );

    let toggle_btn = build_toggle_button(BUTTON_ID_RELOAD, "", state.is_sub_reload);
    let back_btn = CreateButton::new(BUTTON_ID_BACK)
        .style(ButtonStyle::Secondary)
        .label("Back".to_string());
    let button_row = CreateActionRow::Buttons(vec![toggle_btn, back_btn].into());

    let start_select =
        build_hour_select(SELECT_ID_RELOAD_START, state.reload_start, "Start hour");
    let start_row = CreateActionRow::SelectMenu(start_select);

    let end_select = build_hour_select(SELECT_ID_RELOAD_END, state.reload_end, "End hour");
    let end_row = CreateActionRow::SelectMenu(end_select);

    let components = vec![
        CreateComponent::ActionRow(button_row),
        CreateComponent::ActionRow(start_row),
        CreateComponent::ActionRow(end_row),
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

fn build_channel_select(current: Option<i64>) -> CreateSelectMenu<'static> {
    let default_channels = current.map(|id| vec![GenericChannelId::new(id as u64)]);

    CreateSelectMenu::new(
        SELECT_ID_CHANNEL.to_string(),
        CreateSelectMenuKind::Channel {
            channel_types: Some(vec![ChannelType::Text].into()),
            default_channels: default_channels.map(|v| v.into()),
        },
    )
    .placeholder("Select subscription channel".to_string())
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
