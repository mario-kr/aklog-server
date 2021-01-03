// This file was copied from:
// https://raw.githubusercontent.com/nickbabcock/rrinlog/master/rrinlog-server/src/api.rs
// on the 10th December 2017


use chrono::prelude::*;
use serde_json;
use getset::{Getters, MutGetters};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct Range {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Target {
    pub target: String,
    #[serde(rename = "refId")] pub ref_id: String,
    #[serde(rename = "type")] pub _type: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Query {
    pub range: Range,
    #[serde(rename = "intervalMs")] pub interval_ms: i32,
    #[serde(rename = "maxDataPoints")] pub max_data_points: i32,
    pub format: Option<String>,
    pub targets: Vec<Target>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum TargetData {
    Series(Series),
    Table(Table),
}

#[derive(Getters, MutGetters, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Series {
    target: String,

    #[getset(get = "pub", get_mut = "pub")]
    datapoints: Vec<[f64; 2]>,
}

impl Series {
    pub fn new(target: String) -> Self {
        Series { target, datapoints: Vec::new() }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Column {
    text: String,
    #[serde(rename = "type")] _type: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Table {
    columns: Vec<Column>,
    #[serde(rename = "type")] _type: String,
    rows: Vec<Vec<serde_json::Value>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Search {
    target: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SearchResponse(pub Vec<String>);

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct QueryResponse(pub Vec<TargetData>);

