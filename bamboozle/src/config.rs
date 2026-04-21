use anyhow::Context;

pub struct AppConfig {
    pub route_config_folders: Vec<String>,
    pub throw_on_error: bool,
    pub mock_ports: Vec<u16>,
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

        let mock_ports = match std::env::var("EXPOSE_PORTS") {
            Ok(val) => parse_ports(&val)
                .context("EXPOSE_PORTS must be a single port (8080), a range (8000-9000), or a comma-separated list (8080,8081)")?,
            Err(_) => vec![8080],
        };

        Ok(Self {
            route_config_folders,
            throw_on_error,
            mock_ports,
        })
    }
}

fn parse_ports(val: &str) -> anyhow::Result<Vec<u16>> {
    let val = val.trim();
    if val.contains(',') {
        val.split(',')
            .map(|s| s.trim().parse::<u16>().context("invalid port number"))
            .collect()
    } else if let Some((start, end)) = val.split_once('-') {
        let start: u16 = start.trim().parse().context("invalid range start")?;
        let end: u16 = end.trim().parse().context("invalid range end")?;
        anyhow::ensure!(start <= end, "range start must be <= end");
        Ok((start..=end).collect())
    } else {
        let port: u16 = val.parse().context("invalid port number")?;
        Ok(vec![port])
    }
}
