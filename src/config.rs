use std::path::PathBuf;
use std::{env, fs};

use poise::serenity_prelude as serenity;
use serde::Deserialize;

use crate::util::dates;

#[derive(Debug, Deserialize, Clone)]
struct FileConfig {
    pub database_path: String,
    pub heroes_path: String,
    pub api_key_var: String,
    pub test_guild: Option<u64>,
    pub test_channel: Option<u64>,
    pub online_status: serenity::model::user::OnlineStatus,
    pub log_level: String,
    pub log_path: String,
    pub log_json_path: String,
    pub clear_commands_on_startup: bool,
    pub max_message_length: usize,
    pub max_players_per_server: usize,
    pub seq_endpoint: Option<String>,
    pub scheduler: SchedulerConfig,
    pub countdown_duration_ms: u64,
    pub countdown_offset_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SchedulerConfig {
    pub enabled: bool,
    pub heartbeat_interval_minutes: u64,

    pub timer_check_mins: u64,

    /// Reloads matches between the start and end hour (eg. 3, 15, 23, ...)
    pub auto_reload_start_hour: u8,
    pub auto_reload_end_hour: u8,
    /// Interval between reloads
    pub auto_reload_interval_minutes: u64,

    pub weekly_leaderboard_day: Option<u8>,
    pub weekly_leaderboard_hour: Option<u8>,

    pub monthly_leaderboard_day: Option<u8>,
    pub monthly_leaderboard_hour: Option<u8>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct AppConfig {
    pub database_path: PathBuf,
    pub heroes_path: PathBuf,
    pub discord_api_key: String,
    pub log_level: String,
    pub log_path: PathBuf,
    pub log_json_path: PathBuf,
    pub test_guild: Option<u64>,
    pub test_channel: Option<u64>,
    pub online_status: serenity::model::user::OnlineStatus,
    pub max_message_length: usize,
    pub max_players_per_server: usize,
    pub clear_commands_on_startup: bool,
    pub seq_endpoint: Option<String>,
    pub scheduler: SchedulerConfig,
    pub countdown_duration_ms: u64,
    pub countdown_offset_ms: u64,
}

fn expand_tilde(path: &str) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    if path.starts_with("~/") {
        let home = env::var("HOME")?;
        Ok(PathBuf::from(path.replacen("~", &home, 1)))
    } else {
        Ok(PathBuf::from(path))
    }
}

pub fn load_config() -> Result<AppConfig, Box<dyn std::error::Error + Send + Sync>> {
    let exe_path = env::current_exe()?;
    let config_path = match exe_path.parent() {
        Some(dir) => dir.join("dotacord.toml"),
        _ => return Err("failed to determine executable directory".into()),
    };

    if !config_path.exists() || !config_path.is_file() {
        return Err(format!(
            "Config file does not exist or is not a file: {}",
            config_path.display()
        )
        .into());
    }
    let s = fs::read_to_string(&config_path)?;
    let cfg: FileConfig = toml::from_str(&s)?;

    let database_path = expand_tilde(&cfg.database_path)?;
    if !database_path.exists() || !database_path.is_file() {
        return Err(format!("Database file does not exist: {}", &cfg.database_path).into());
    }

    let heroes_path = expand_tilde(&cfg.heroes_path)?;
    if !heroes_path.exists() || !heroes_path.is_file() {
        return Err(format!("Heroes file does not exist: {}", &cfg.heroes_path).into());
    }

    // let api_key = env::var(&cfg.api_key_var).map_err(|e| {
    //     fmt!(
    //         "Failed to read API key from env var {}: {}",
    //         &cfg.api_key_var, e
    //     )
    // })?;
    let api_key = cfg.api_key_var;

    let log_path = log_file_replacements(&cfg.log_path)?;
    if let Some(parent) = log_path.parent() {
        if !parent.exists() {
            return Err(format!("Log file directory does not exist: {}", parent.display()).into());
        }
    }
    if log_path.exists() && !log_path.is_file() {
        return Err(format!("Log path exists but is not a file: {}", &cfg.log_path).into());
    }

    let log_json_path = log_file_replacements(&cfg.log_json_path)?;
    if let Some(parent) = log_json_path.parent() {
        if !parent.exists() {
            return Err(format!("Log file directory does not exist: {}", parent.display()).into());
        }
    }
    if log_json_path.exists() && !log_json_path.is_file() {
        return Err(format!("Log path exists but is not a file: {}", &cfg.log_json_path).into());
    }

    Ok(AppConfig {
        database_path,
        heroes_path,
        discord_api_key: api_key,
        log_level: cfg.log_level,
        log_path,
        log_json_path,
        test_guild: cfg.test_guild,
        test_channel: cfg.test_channel,
        online_status: cfg.online_status,
        max_message_length: cfg.max_message_length,
        max_players_per_server: cfg.max_players_per_server,
        clear_commands_on_startup: cfg.clear_commands_on_startup,
        seq_endpoint: cfg.seq_endpoint,
        scheduler: cfg.scheduler,
        countdown_duration_ms: cfg.countdown_duration_ms,
        countdown_offset_ms: cfg.countdown_offset_ms,
    })
}

fn log_file_replacements(cfg_path: &str) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let date_str = dates::local_date_yyyy_mm_dd();
    let replaced = cfg_path.replace("{DATE}", &date_str);
    expand_tilde(&replaced)
}
