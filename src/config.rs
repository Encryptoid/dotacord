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
    pub max_message_length: usize,
    pub max_players_per_server: usize,
    pub log: FileLogConfig,
    pub scheduler: SchedulerConfig,
    pub cooldowns: CooldownsConfig,
    pub roll_countdown_duration_sec: u64,
    pub flip_countdown_duration_sec: u64,
    pub countdown_offset_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
struct FileLogConfig {
    pub level: String,
    pub path: String,
    pub json_path: String,
    pub seq_endpoint: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SchedulerConfig {
    pub enabled: bool,
    pub heartbeat_interval_minutes: u64,
    pub timer_check_mins: u64,
    pub auto_reload: AutoReloadConfig,
    pub weekly_leaderboard: WeeklyLeaderboardConfig,
    pub monthly_leaderboard: MonthlyLeaderboardConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AutoReloadConfig {
    pub enabled: bool,
    pub start_hour: u8,
    pub end_hour: u8,
    pub interval_minutes: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WeeklyLeaderboardConfig {
    pub enabled: bool,
    pub minute: u8,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MonthlyLeaderboardConfig {
    pub enabled: bool,
    pub minute: u8,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CooldownsConfig {
    pub user_refresh_min: u64,
    pub admin_refresh_min: u64,
}

#[derive(Clone, Debug)]
pub struct LogConfig {
    pub level: String,
    pub path: PathBuf,
    pub json_path: PathBuf,
    pub seq_endpoint: Option<String>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct AppConfig {
    pub database_path: PathBuf,
    pub heroes_path: PathBuf,
    pub discord_api_key: String,
    pub test_guild: Option<u64>,
    pub test_channel: Option<u64>,
    pub online_status: serenity::model::user::OnlineStatus,
    pub max_message_length: usize,
    pub max_players_per_server: usize,
    pub log: LogConfig,
    pub scheduler: SchedulerConfig,
    pub cooldowns: CooldownsConfig,
    pub roll_countdown_duration_sec: u64,
    pub flip_countdown_duration_sec: u64,
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

    Ok(AppConfig {
        database_path,
        heroes_path,
        discord_api_key: api_key,
        test_guild: cfg.test_guild,
        test_channel: cfg.test_channel,
        online_status: cfg.online_status,
        max_message_length: cfg.max_message_length,
        max_players_per_server: cfg.max_players_per_server,
        log: build_log_config(cfg.log)?,
        scheduler: cfg.scheduler,
        cooldowns: cfg.cooldowns,
        roll_countdown_duration_sec: cfg.roll_countdown_duration_sec,
        flip_countdown_duration_sec: cfg.flip_countdown_duration_sec,
        countdown_offset_ms: cfg.countdown_offset_ms,
    })
}

fn build_log_config(file_log: FileLogConfig) -> Result<LogConfig, Box<dyn std::error::Error + Send + Sync>> {
    let path = log_file_replacements(&file_log.path)?;
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            return Err(format!("Log file directory does not exist: {}", parent.display()).into());
        }
    }
    if path.exists() && !path.is_file() {
        return Err(format!("Log path exists but is not a file: {}", &file_log.path).into());
    }

    let json_path = log_file_replacements(&file_log.json_path)?;
    if let Some(parent) = json_path.parent() {
        if !parent.exists() {
            return Err(format!("Log file directory does not exist: {}", parent.display()).into());
        }
    }
    if json_path.exists() && !json_path.is_file() {
        return Err(format!("Log path exists but is not a file: {}", &file_log.json_path).into());
    }

    Ok(LogConfig {
        level: file_log.level,
        path,
        json_path,
        seq_endpoint: file_log.seq_endpoint,
    })
}

fn log_file_replacements(cfg_path: &str) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let date_str = dates::local_date_yyyy_mm_dd();
    let replaced = cfg_path.replace("{DATE}", &date_str);
    expand_tilde(&replaced)
}
