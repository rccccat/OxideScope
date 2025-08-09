use anyhow::Result;
use mongodb::{options::ClientOptions, Client, Database};
use crate::settings::AppConfig;

pub async fn connect_mongo(cfg: &AppConfig) -> Result<Client> {
    let uri = format!(
        "mongodb://{}:{}@{}:{}",
        urlencoding::encode(&cfg.mongodb.username),
        urlencoding::encode(&cfg.mongodb.password),
        cfg.mongodb.ip,
        cfg.mongodb.port
    );
    let mut opts = ClientOptions::parse(uri).await?;
    opts.app_name = Some("scopesentry-rs".into());
    let client = Client::with_options(opts)?;
    Ok(client)
}

pub fn db(client: &Client, cfg: &AppConfig) -> Database {
    client.database(&cfg.mongodb.mongodb_database)
}