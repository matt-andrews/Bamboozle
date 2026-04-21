use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

use super::match_key::MatchKey;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RouteDefinition {
    #[serde(rename = "match")]
    pub match_key: MatchKey,
    #[serde(default)]
    pub response: ResponseDefinition,
    #[serde(rename = "setState", default, skip_serializing_if = "Option::is_none")]
    pub set_state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct ResponseDefinition {
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    pub content: Option<String>,
    pub loopback: Option<bool>,
}

fn default_status() -> String {
    "200".to_string()
}
