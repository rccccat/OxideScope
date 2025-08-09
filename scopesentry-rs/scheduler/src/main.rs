use axum::{routing::{get, post}, Json, Router};
use axum::extract::State;
use serde_json::json;
use std::sync::Arc;
use tracing_subscriber::{EnvFilter, fmt};
use tracing_subscriber::prelude::*;

use scopesentry_common::{settings::AppConfig, mongo, rds, models::{TaskAddRequest, TemplateDoc, DispatchTemplate}, util::{now_string, expand_targets}};
use mongodb::{bson::{self, doc, oid::ObjectId}, Collection};
use redis::AsyncCommands;

#[derive(Clone)]
struct AppState {
    cfg: Arc<AppConfig>,
    mongo: mongodb::Client,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer())
        .init();

    let cfg = Arc::new(AppConfig::load()?);
    let mongo = mongo::connect_mongo(&cfg).await?;

    let state = AppState { cfg: cfg.clone(), mongo };

    let app = Router::new()
        .route("/api/node/data/online", get(node_online))
        .route("/api/task/add", post(add_task))
        .with_state(state);

    let port: u16 = std::env::var("SCHEDULER_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8083);
    let addr = std::net::SocketAddr::from(([0,0,0,0], port));
    tracing::info!("scheduler listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

async fn node_online(State(state): State<AppState>) -> Json<serde_json::Value> {
    let mut con = rds::connect_redis(&state.cfg).await.expect("redis");
    let keys: Vec<String> = con.keys("node:*").await.unwrap_or_default();
    let mut result = vec![];
    for key in keys {
        let name = key.split(':').nth(1).unwrap_or("").to_string();
        let hash: std::collections::HashMap<String, String> = con.hgetall(&key).await.unwrap_or_default();
        if hash.get("state").map(|s| s == "1").unwrap_or(false) {
            result.push(name);
        }
    }
    Json(json!({"code":200, "data": {"list": result}}))
}

async fn add_task(State(state): State<AppState>, Json(mut req): Json<TaskAddRequest>) -> Json<serde_json::Value> {
    // validate
    if req.name.trim().is_empty() || (req.node.is_empty() && !req.allNode) {
        return Json(json!({"code":400, "message":"invalid args"}));
    }

    // expand targets
    let targets = expand_targets(&req.target, &req.ignore);
    let task_num = targets.len() as i32;

    // resolve all online nodes if allNode
    if req.allNode {
        let mut con = rds::connect_redis(&state.cfg).await.expect("redis");
        let keys: Vec<String> = con.keys("node:*").await.unwrap_or_default();
        for key in keys {
            let name = key.split(':').nth(1).unwrap_or("");
            let h: std::collections::HashMap<String, String> = con.hgetall(&key).await.unwrap_or_default();
            if h.get("state").map(|s| s == "1").unwrap_or(false) {
                if !req.node.contains(&name.to_string()) { req.node.push(name.to_string()); }
            }
        }
    }

    // insert task doc
    let db = scopesentry_common::mongo::db(&state.mongo, &state.cfg);
    let task_coll: Collection<bson::Document> = db.collection("task");
    let now = now_string();
    let doc = doc!{
        "name": &req.name,
        "target": targets.join("\n"),
        "ignore": &req.ignore,
        "node": bson::to_bson(&req.node).unwrap(),
        "allNode": req.allNode,
        "scheduledTasks": req.scheduledTasks,
        "template": &req.template,
        "duplicates": req.duplicates,
        "taskNum": task_num,
        "progress": 0.0_f64,
        "creatTime": &now,
        "endTime": "",
        "status": 1_i32,
        "type": "scan",
    };
    let ins = task_coll.insert_one(doc).await;
    let Ok(ins_res) = ins else { return Json(json!({"code":500, "message":"db insert failed"})); };
    let task_id = ins_res.inserted_id.as_object_id().unwrap_or(ObjectId::new());
    let task_id_str = task_id.to_hex();

    // enqueue targets to redis
    let mut con = rds::connect_redis(&state.cfg).await.expect("redis");
    if !targets.is_empty() {
        let key = format!("TaskInfo:{}", task_id_str);
        let _: i64 = con.lpush(key, targets).await.unwrap_or(0);
    }

    // load template and resolve
    let tmpl_coll: Collection<TemplateDoc> = db.collection("ScanTemplates");
    let tmpl_oid = ObjectId::parse_str(&req.template).unwrap_or_else(|_| task_id.clone());
    let mut dispatch = if let Ok(Some(tmpl)) = tmpl_coll.find_one(doc!{"_id": tmpl_oid}).await {
        DispatchTemplate{ Parameters: tmpl.Parameters, TaskName: req.name.clone(), ignore: req.ignore.clone(), duplicates: req.duplicates, ID: task_id_str.clone(), r#type: "scan".to_string(), IsStart: false }
    } else {
        DispatchTemplate{ Parameters: Default::default(), TaskName: req.name.clone(), ignore: req.ignore.clone(), duplicates: req.duplicates, ID: task_id_str.clone(), r#type: "scan".to_string(), IsStart: false }
    };

    // dispatch to each node
    for name in &req.node {
        let key = format!("NodeTask:{}", name);
        let _ = scopesentry_common::rds::rpush_json(&mut con, &key, &dispatch).await;
    }

    Json(json!({"code":200, "message":"Task added successfully"}))
}