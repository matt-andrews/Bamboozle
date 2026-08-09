# Contributing to Bamboozle

Bamboozle is a Rust application built on Axum. A stable Rust toolchain is enough for local development; Docker is also required for image and integration testing.

## Build and run

From the repository root:

```bash
cargo build --manifest-path bamboozle/Cargo.toml
cargo run --manifest-path bamboozle/Cargo.toml
```

The mock surface listens on `:8080` and the control API on `:9090`. TLS and OpenTelemetry are optional Cargo features; the published Docker image enables TLS.

Build and run the image with:

```bash
docker build -t bamboozle bamboozle/
docker run -p 8080:8080 -p 9090:9090 bamboozle
```

## Test

Run the Rust checks:

```bash
cargo clippy --manifest-path bamboozle/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path bamboozle/Cargo.toml
```

Run the HTTP integration and regression suite:

```bash
bash tests/run.sh
```

The integration suite shares route state and call history, so it runs serially. See [`tests/README.md`](./tests/README.md) for targeted runs and suite layout.

## Architecture

- `main.rs` starts the mock and control listeners and configures tracing.
- `control/` owns route management, assertions, and the generated OpenAPI document.
- `routing/` compiles and matches route patterns.
- `mock_server.rs` records requests, renders responses, and applies simulations.
- `models/`, `tracking/`, and `liquid_render.rs` hold the shared request and response behavior.

Keep endpoint and schema details in code annotations so the interactive API reference remains the source of truth. Add unit tests near the changed module and integration coverage when behavior crosses the HTTP boundary.
