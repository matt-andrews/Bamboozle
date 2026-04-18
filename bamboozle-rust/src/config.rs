use anyhow::Context;

pub struct AppConfig {
    pub route_config_folders: Vec<String>,
    pub throw_on_error: bool,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let route_config_folders = match std::env::var("ROUTE_CONFIG_FOLDERS") {
            Ok(val) => serde_json::from_str::<Vec<String>>(&val)
                .context("ROUTE_CONFIG_FOLDERS must be a JSON array of strings")?,
            Err(_) => vec![],
        };

        let throw_on_error = std::env::var("ROUTE_CONFIG_THROW_ON_ERROR")
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        Ok(Self {
            route_config_folders,
            throw_on_error,
        })
    }
}
