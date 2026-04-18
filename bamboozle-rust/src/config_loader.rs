use std::path::Path;
use tokio::fs;
use tracing::{error, info, warn};

use crate::{
    app_state::AppState,
    config::AppConfig,
    models::config_file::ConfigLoaderModel,
};

pub async fn load(config: &AppConfig, state: &AppState) -> anyhow::Result<()> {
    for folder in &config.route_config_folders {
        let path = Path::new(folder);

        if !path.exists() {
            warn!(folder = %folder, "Config folder not found, skipping");
            if config.throw_on_error {
                return Err(anyhow::anyhow!("Config folder not found: {}", folder));
            }
            continue;
        }

        let mut entries = match fs::read_dir(path).await {
            Ok(e) => e,
            Err(err) => {
                error!(folder = %folder, error = %err, "Failed to read config folder");
                if config.throw_on_error {
                    return Err(err.into());
                }
                continue;
            }
        };

        while let Some(entry) = entries.next_entry().await? {
            let file_path = entry.path();
            if !file_path.is_file() {
                continue;
            }

            let ext = file_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();

            let result = match ext.as_str() {
                "json" => load_file(&file_path, state, parse_json).await,
                "yml" | "yaml" => load_file(&file_path, state, parse_yaml).await,
                _ => continue,
            };

            if let Err(err) = result {
                error!(path = %file_path.display(), error = %err, "Failed to load config file");
                if config.throw_on_error {
                    return Err(err);
                }
            }
        }
    }
    Ok(())
}

async fn load_file(
    path: &Path,
    state: &AppState,
    parser: fn(&str) -> anyhow::Result<ConfigLoaderModel>,
) -> anyhow::Result<()> {
    let content = fs::read_to_string(path).await?;
    let model = parser(&content)?;

    for route in model.routes {
        let key = route.match_key.to_string();
        match state.store.set_route(route) {
            Ok(_) => info!(route = %key, "Loaded route from config"),
            Err(e) => error!(route = %key, error = %e, "Failed to load route from config"),
        }
    }

    Ok(())
}

fn parse_json(content: &str) -> anyhow::Result<ConfigLoaderModel> {
    serde_json::from_str(content).map_err(Into::into)
}

fn parse_yaml(content: &str) -> anyhow::Result<ConfigLoaderModel> {
    serde_yaml::from_str(content).map_err(Into::into)
}
