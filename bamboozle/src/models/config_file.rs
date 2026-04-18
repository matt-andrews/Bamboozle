use serde::Deserialize;

use super::route::RouteDefinition;

#[derive(Debug, Deserialize, Default)]
pub struct ConfigLoaderModel {
    #[serde(default)]
    pub routes: Vec<RouteDefinition>,
}
