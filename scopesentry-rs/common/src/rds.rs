use anyhow::Result;
use redis::{aio::Connection, AsyncCommands, Client, RedisResult};
use tokio::time::{sleep, Duration};
use crate::settings::AppConfig;

pub async fn connect_redis(cfg: &AppConfig) -> Result<Connection> {
    let url = format!(
        "redis://:{}@{}:{}",
        urlencoding::encode(&cfg.redis.password),
        cfg.redis.ip,
        cfg.redis.port
    );
    let client = Client::open(url)?;
    let conn = client.get_tokio_connection().await?;
    Ok(conn)
}

pub async fn rpush_json<T: serde::Serialize>(con: &mut Connection, key: &str, value: &T) -> RedisResult<i64> {
    let payload = serde_json::to_string(value).unwrap();
    con.rpush(key, payload).await
}

pub async fn publish_json<T: serde::Serialize>(con: &mut Connection, channel: &str, value: &T) -> RedisResult<i64> {
    let payload = serde_json::to_string(value).unwrap();
    con.publish(channel, payload).await
}

pub async fn keep_try_redis<F, Fut, T>(mut f: F, retries: usize, delay_ms: u64) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, redis::RedisError>>,
{
    let mut last: Option<redis::RedisError> = None;
    for _ in 0..retries {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                last = Some(e);
                sleep(Duration::from_millis(delay_ms)).await;
            }
        }
    }
    Err(anyhow::anyhow!(last.unwrap_or_else(|| redis::RedisError::from((redis::ErrorKind::IoError, "unknown")))) )
}