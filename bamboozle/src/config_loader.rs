use std::path::Path;
use tokio::fs;
use tracing::{error, warn};

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
                "json" => load_file(&file_path, state, parse_json, config.throw_on_error).await,
                "yml" | "yaml" => load_file(&file_path, state, parse_yaml, config.throw_on_error).await,
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
    throw_on_error: bool,
) -> anyhow::Result<()> {
    let content = fs::read_to_string(path).await?;
    let model = parser(&content)?;

    for route in model.routes {
        if let Err(err) = state.store.set_route(route) {
            error!(error = %err, "Failed to insert route");
            if throw_on_error {
                return Err(err.into());
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_json_empty_routes() {
        let model = parse_json(r#"{"routes": []}"#).unwrap();
        assert!(model.routes.is_empty());
    }

    #[test]
    fn parse_json_with_route() {
        let json = r#"{
            "routes": [{
                "match": { "verb": "GET", "pattern": "/api/users" },
                "response": { "status": "200", "content": "[]" }
            }]
        }"#;
        let model = parse_json(json).unwrap();
        assert_eq!(model.routes.len(), 1);
        assert_eq!(model.routes[0].match_key.verb, "GET");
        assert_eq!(model.routes[0].match_key.pattern, "/api/users");
        assert_eq!(model.routes[0].response.content.as_deref(), Some("[]"));
    }

    #[test]
    fn parse_json_invalid_returns_err() {
        assert!(parse_json("not valid json {{{{").is_err());
    }

    #[test]
    fn parse_yaml_empty_routes() {
        let model = parse_yaml("routes: []").unwrap();
        assert!(model.routes.is_empty());
    }

    #[test]
    fn parse_yaml_with_route() {
        let yaml = "
routes:
  - match:
      verb: POST
      pattern: /api/items
    response:
      status: \"201\"
      content: created
";
        let model = parse_yaml(yaml).unwrap();
        assert_eq!(model.routes.len(), 1);
        assert_eq!(model.routes[0].match_key.verb, "POST");
        assert_eq!(model.routes[0].match_key.pattern, "/api/items");
        assert_eq!(model.routes[0].response.status, "201");
    }

    #[test]
    fn parse_yaml_wrong_type_returns_err() {
        assert!(parse_yaml("routes: not_a_list").is_err());
    }
}
