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

fn init_tracing() {
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let format = std::env::var("RUST_LOG_FORMAT")
        .unwrap_or_default()
        .to_ascii_lowercase();

    // ANSI colors on by default; opt out with NO_COLOR=1 (https://no-color.org)
    let ansi = std::env::var("NO_COLOR").is_err();

    // Each format variant is a different concrete type; box to unify them.
    let fmt_layer: Box<dyn tracing_subscriber::Layer<_> + Send + Sync> = match format.as_str() {
        "json"   => Box::new(fmt::layer().json()),
        "pretty" => Box::new(fmt::layer().pretty().with_ansi(ansi)),
        _        => Box::new(fmt::layer().compact().with_ansi(ansi)),
    };

    // OTEL layer is always compiled; only activates when the endpoint env var is set.
    let otel_layer = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok().map(|endpoint| {
        use opentelemetry_otlp::WithExportConfig;
        let exporter = opentelemetry_otlp::new_exporter()
            .http()
            .with_endpoint(endpoint);
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(exporter)
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .expect("Failed to install OpenTelemetry OTLP tracer");
        tracing_opentelemetry::layer().with_tracer(tracer)
    });

    // Option<L> is a no-op layer when None — no branching needed.
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(otel_layer)
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

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
