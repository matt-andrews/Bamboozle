use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

use super::{match_key::MatchKey, simulation::SimulationConfig};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RouteDefinition {
    #[serde(rename = "match")]
    pub match_key: MatchKey,
    #[serde(default)]
    pub response: ResponseDefinition,
    #[serde(rename = "setState", default, skip_serializing_if = "Option::is_none")]
    pub set_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub simulation: Option<SimulationConfig>,
    #[serde(rename = "maxCalls", default, skip_serializing_if = "Option::is_none")]
    pub max_calls: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct ResponseDefinition {
    #[serde(default = "ResponseDefinition::default_status")]
    pub status: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(
        rename = "contentFile",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub content_file: Option<String>,
    #[serde(
        rename = "binaryFile",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub binary_file: Option<String>,
    #[serde(default)]
    pub loopback: bool,
}

impl ResponseDefinition {
    fn default_status() -> String {
        "200".to_string()
    }
}
