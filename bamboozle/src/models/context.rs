use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use utoipa::ToSchema;

use super::route::RouteDefinition;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ContextModel {
    #[serde(rename = "queryParams")]
    pub query_params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    #[serde(rename = "routeValues")]
    pub route_values: HashMap<String, String>,
    #[serde(rename = "routeModel")]
    pub route_model: RouteDefinition,
    pub body: JsonValue,
}
