use anyhow::Result;
use serde::Deserialize;
use std::{env, fs};

#[derive(Debug, Clone, Deserialize)]
pub struct SystemSettings {
    pub timezone: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MongoSettings {
    pub ip: String,
    pub port: u16,
    pub mongodb_database: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisSettings {
    pub ip: String,
    pub port: u16,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogsSettings {
    pub total_logs: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub system: SystemSettings,
    pub mongodb: MongoSettings,
    pub redis: RedisSettings,
    pub logs: Option<LogsSettings>,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let path = env::var("SCOPESENTRY_CONFIG")
            .ok()
            .unwrap_or_else(|| "../ScopeSentry/config.yaml".to_string());
        let content = fs::read_to_string(&path)?;
        let cfg: AppConfig = serde_yaml::from_str(&content)?;
        Ok(cfg)
    }
}