mod app_state;
mod config;
mod config_loader;
mod control;
mod error;
mod expression;
mod liquid_render;
mod mock_server;
mod models;
mod routing;
mod tracking;

use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_ansi(false)
        .init();

    let config = config::AppConfig::from_env()?;
    let state = app_state::AppState::new();

    config_loader::load(&config, &state).await?;

    let mock_listener = TcpListener::bind("0.0.0.0:8080").await?;
    let control_listener = TcpListener::bind("0.0.0.0:9090").await?;

    info!("Mock server listening on :8080");
    info!("Control server listening on :9090");

    tokio::try_join!(
        axum::serve(mock_listener, mock_server::router(state.clone())),
        axum::serve(control_listener, control::router(state.clone())),
    )?;

    Ok(())
}
