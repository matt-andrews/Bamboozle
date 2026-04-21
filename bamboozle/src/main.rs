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

    #[cfg(feature = "otel")]
    {
        // OTEL layer only activates when the endpoint env var is set.
        // On build error, fall back to console-only and print a warning — don't crash.
        let otel_layer = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok().and_then(|_| {
            match opentelemetry_otlp::SpanExporter::builder().with_http().build() {
                Ok(exporter) => {
                    let provider = opentelemetry_sdk::trace::TracerProvider::builder()
                        .with_simple_exporter(exporter)
                        .build();
                    use opentelemetry::trace::TracerProvider as _;
                    let tracer = provider.tracer("bamboozle");
                    opentelemetry::global::set_tracer_provider(provider);
                    Some(tracing_opentelemetry::layer().with_tracer(tracer))
                }
                Err(e) => {
                    // Tracing isn't initialized yet so we can't use warn! here.
                    eprintln!("warning: failed to initialize OTLP exporter ({e}), falling back to console-only logging");
                    None
                }
            }
        });

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .with(otel_layer)
            .init();

        return;
    }

    #[cfg(not(feature = "otel"))]
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
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

    // Flush any buffered OTLP spans before exiting.
    #[cfg(feature = "otel")]
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
