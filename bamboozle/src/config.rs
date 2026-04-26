use anyhow::Context;

pub struct AppConfig {
    pub route_config_folders: Vec<String>,
    pub throw_on_error: bool,
    pub max_routes: usize,
    pub max_content_size: usize,
    #[cfg(feature = "tls")]
    pub tls_cert_file: Option<String>,
    #[cfg(feature = "tls")]
    pub tls_key_file: Option<String>,
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

        let max_routes = std::env::var("MAX_ROUTES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1000);

        let max_content_size = std::env::var("MAX_CONTENT_SIZE_BYTES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10 * 1024 * 1024);

        #[cfg(feature = "tls")]
        let tls_cert_file = std::env::var("TLS_CERT_FILE").ok();
        #[cfg(feature = "tls")]
        let tls_key_file = std::env::var("TLS_KEY_FILE").ok();

        #[cfg(feature = "tls")]
        if tls_cert_file.is_some() != tls_key_file.is_some() {
            anyhow::bail!(
                "TLS_CERT_FILE and TLS_KEY_FILE must both be set or both be unset"
            );
        }

        Ok(Self {
            route_config_folders,
            throw_on_error,
            max_routes,
            max_content_size,
            #[cfg(feature = "tls")]
            tls_cert_file,
            #[cfg(feature = "tls")]
            tls_key_file,
        })
    }
}
