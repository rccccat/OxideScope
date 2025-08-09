use std::{collections::HashMap, sync::Arc, time::Duration};

use mongodb::{bson::{doc, Document}, options::IndexOptions, IndexModel};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tracing_subscriber::{EnvFilter, fmt};
use tracing_subscriber::prelude::*;

use scopesentry_common::{settings::AppConfig, mongo, rds, models::DispatchTemplate, util::now_string};

#[derive(Debug, Clone)]
struct Ctx {
    cfg: Arc<AppConfig>,
    mongo: mongodb::Client,
    node_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProgressEntry {
    node: String,
    scan_start: String,
    scan_end: String,
    TargetHandler_start: String,
    SubdomainScan_start: String,
    SubdomainScan_end: String,
    AssetMapping_start: String,
    AssetMapping_end: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer())
        .init();

    let cfg = Arc::new(AppConfig::load()?);
    let mongo = mongo::connect_mongo(&cfg).await?;
    let node_name = std::env::var("NODE_NAME").ok().unwrap_or_else(|| hostname::get().unwrap_or_default().to_string_lossy().to_string());
    let ctx = Ctx { cfg: cfg.clone(), mongo, node_name };

    ensure_indexes(&ctx).await?;

    let mut con = rds::connect_redis(&ctx.cfg).await?;

    // initial register
    register_node(&mut con, &ctx.node_name).await?;
    publish_log(&mut con, &ctx.node_name, "Register Success").await.ok();

    // spawn heartbeat task
    {
        let node = ctx.node_name.clone();
        let mut con_hb = rds::connect_redis(&ctx.cfg).await?;
        tokio::spawn(async move {
            loop {
                let _ : redis::RedisResult<()> = con_hb.hset(format!("node:{}", node), "state", "1").await;
                let _ : redis::RedisResult<()> = con_hb.hset(format!("node:{}", node), "updateTime", now_string()).await;
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        });
    }

    // main loop: consume NodeTask and process tasks
    loop {
        let key = format!("NodeTask:{}", ctx.node_name);
        let res: redis::RedisResult<(String, String)> = con.blpop(&key, 5.0).await; // (key, payload)
        match res {
            Ok((_k, payload)) => {
                match serde_json::from_str::<DispatchTemplate>(&payload) {
                    Ok(tmpl) => {
                        if let Err(e) = handle_task(&ctx, &mut con, tmpl).await {
                            tracing::error!("task error: {}", e);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("invalid tmpl: {}", e);
                    }
                }
            }
            Err(_timeout) => {
                // idle
            }
        }
    }
}

async fn ensure_indexes(ctx: &Ctx) -> anyhow::Result<()> {
    let db = scopesentry_common::mongo::db(&ctx.mongo, &ctx.cfg);
    // asset unique (host, port)
    let asset = db.collection::<Document>("asset");
    let keys = doc!{"host": 1, "port": 1};
    let opts = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder().keys(keys).options(opts).build();
    let _ = asset.create_index(model).await;
    Ok(())
}

async fn register_node(con: &mut redis::aio::Connection, name: &str) -> redis::RedisResult<()> {
    let key = format!("node:{}", name);
    let _: () = con.hset(&key, "state", "1").await?;
    let _: () = con.hset(&key, "name", name).await?;
    let _: () = con.hset(&key, "updateTime", now_string()).await?;
    Ok(())
}

async fn publish_log(con: &mut redis::aio::Connection, name: &str, log: &str) -> redis::RedisResult<i64> {
    let payload = serde_json::json!({"name": name, "log": log});
    con.publish("logs", payload.to_string()).await
}

async fn handle_task(ctx: &Ctx, con: &mut redis::aio::Connection, tmpl: DispatchTemplate) -> anyhow::Result<()> {
    let id = tmpl.ID.clone();
    let mut progress = ProgressEntry{
        node: ctx.node_name.clone(),
        scan_start: now_string(),
        scan_end: String::new(),
        TargetHandler_start: now_string(),
        SubdomainScan_start: String::new(),
        SubdomainScan_end: String::new(),
        AssetMapping_start: String::new(),
        AssetMapping_end: String::new(),
    };

    // consume targets list
    let list_key = format!("TaskInfo:{}", id);
    loop {
        let r: redis::RedisResult<String> = con.rpop(&list_key, None).await; // pop from tail
        let Some(target) = r.ok() else { break; };
        let t = target.clone();
        // mark per-target progress hash
        let pkey = format!("TaskInfo:progress:{}:{}", id, t);
        let _: () = con.hset(&pkey, "node", &ctx.node_name).await?;
        let _: () = con.hset(&pkey, "TargetHandler_start", &progress.TargetHandler_start).await?;

        // Subdomain scan
        progress.SubdomainScan_start = now_string();
        let subs = subdomain_basic(&t).await;
        progress.SubdomainScan_end = now_string();
        if !subs.is_empty() { save_subdomains(ctx, &tmpl.TaskName, &subs).await.ok(); }
        let _: () = con.hset(&pkey, "SubdomainScan_start", &progress.SubdomainScan_start).await?;
        let _: () = con.hset(&pkey, "SubdomainScan_end", &progress.SubdomainScan_end).await?;

        // Asset liveness
        progress.AssetMapping_start = now_string();
        if let Some(asset) = asset_probe(&t).await { save_asset(ctx, &tmpl.TaskName, &asset).await.ok(); }
        progress.AssetMapping_end = now_string();
        let _: () = con.hset(&pkey, "AssetMapping_start", &progress.AssetMapping_start).await?;
        let _: () = con.hset(&pkey, "AssetMapping_end", &progress.AssetMapping_end).await?;

        // add to tmp set for progress counting
        let _: () = con.sadd(format!("TaskInfo:tmp:{}", id), &t).await?;
    }

    progress.scan_end = now_string();
    let _: () = con.set(format!("TaskInfo:time:{}", id), &progress.scan_end).await?;
    publish_log(con, &ctx.node_name, &format!("Task {} completed", id)).await.ok();

    Ok(())
}

async fn subdomain_basic(target: &str) -> Vec<String> {
    // very basic: if target looks like domain (no scheme, not IP), brute small list
    if target.contains("://") { return vec![]; }
    if target.parse::<std::net::IpAddr>().is_ok() { return vec![]; }
    let wordlist = ["www", "test", "dev", "admin", "api"];
    let mut out = vec![];
    for w in wordlist.iter() {
        let sub = format!("{}.{}", w, target);
        if resolve_a(&sub).await { out.push(sub); }
    }
    out
}

async fn resolve_a(name: &str) -> bool {
    use trust_dns_resolver::{TokioAsyncResolver, config::{ResolverConfig, ResolverOpts}};
    let resolver = match TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default()) { Ok(r) => r, Err(_) => return false };
    resolver.lookup_ip(name).await.is_ok()
}

#[derive(Debug, Clone)]
struct AssetRec { url: String, host: String, port: i32, service: String, typ: String }

async fn asset_probe(target: &str) -> Option<AssetRec> {
    // If already URL, try request; else attempt http://target
    let (url, host, port, svc, typ);
    if target.contains("://") {
        let parsed = url::Url::parse(target).ok()?;
        host = parsed.host_str()?.to_string();
        port = parsed.port().unwrap_or_else(|| if parsed.scheme() == "https" { 443 } else { 80 }) as i32;
        svc = parsed.scheme().to_string();
        typ = "http".to_string();
        url = target.to_string();
    } else {
        host = target.to_string();
        port = 80;
        svc = "http".to_string();
        typ = "http".to_string();
        url = format!("http://{}", target);
    }
    let client = reqwest::Client::builder().timeout(Duration::from_secs(3)).build().ok()?;
    if let Ok(resp) = client.get(&url).send().await { let _ = resp.status(); } else { return None; }
    Some(AssetRec{ url, host, port, service: svc, typ })
}

async fn save_subdomains(ctx: &Ctx, task_name: &str, subs: &[String]) -> anyhow::Result<()> {
    let db = scopesentry_common::mongo::db(&ctx.mongo, &ctx.cfg);
    let coll = db.collection::<Document>("subdomain");
    let now = now_string();
    let docs: Vec<Document> = subs.iter().map(|h| doc!{"host": h, "time": &now, "taskName": task_name}).collect();
    if !docs.is_empty() { let _ = coll.insert_many(docs).await; }
    Ok(())
}

async fn save_asset(ctx: &Ctx, task_name: &str, a: &AssetRec) -> anyhow::Result<()> {
    let db = scopesentry_common::mongo::db(&ctx.mongo, &ctx.cfg);
    let coll = db.collection::<Document>("asset");
    let now = now_string();
    let filter = doc!{"host": &a.host, "port": a.port};
    let update = doc!{"$set": {"url": &a.url, "host": &a.host, "port": a.port, "service": &a.service, "type": &a.typ, "time": &now, "taskName": task_name}};
    let _ = coll.update_one(filter, update).await;
    Ok(())
}