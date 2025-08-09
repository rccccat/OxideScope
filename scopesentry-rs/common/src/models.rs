use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAddRequest {
    pub name: String,
    pub target: String,
    #[serde(default)]
    pub ignore: String,
    #[serde(default)]
    pub node: Vec<String>,
    #[serde(default)]
    pub allNode: bool,
    #[serde(default)]
    pub scheduledTasks: bool,
    pub template: String,
    #[serde(default)]
    pub duplicates: bool,
    #[serde(default)]
    pub cycleType: Option<String>,
    #[serde(default)]
    pub hour: Option<u32>,
    #[serde(default)]
    pub minute: Option<u32>,
    #[serde(default)]
    pub day: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateDoc {
    #[serde(rename = "_id")]
    pub id: bson::oid::ObjectId,
    #[serde(default)]
    pub Parameters: HashMap<String, HashMap<String, String>>, // module -> plugin -> args
    #[serde(default)]
    pub vullist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchTemplate {
    pub Parameters: HashMap<String, HashMap<String, String>>, // resolved
    pub TaskName: String,
    pub ignore: String,
    pub duplicates: bool,
    pub ID: String,
    pub r#type: String,
    #[serde(default)]
    pub IsStart: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeLogPayload<'a> {
    pub name: &'a str,
    pub log: &'a str,
}