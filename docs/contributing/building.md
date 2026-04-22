# Building and running locally

## Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- Docker (for image builds and E2E tests)

## Cargo

```bash
# debug build
cargo build

# release build (LTO, stripped symbols)
cargo build --release

# run with defaults (no startup routes, info logging)
cargo run

# run with a static config folder
ROUTE_CONFIG_FOLDERS='["/path/to/routes"]' cargo run

# run with debug logging
RUST_LOG=debug cargo run
```

The mock surface starts on `:8080` and the control API on `:9090`.

## Docker image

```bash
docker build -t bamboozle bamboozle/
docker run -p 8080:8080 -p 9090:9090 bamboozle
```

The Dockerfile in `bamboozle/` uses a multi-stage build. The final image is based on `debian:bookworm-slim` and contains only the compiled binary.

A Docker Compose file for local development is also available in `./scripts/`, with pre-configured routes and environment variables.

## OpenAPI / Scalar UI

While the server is running, open `http://localhost:9090/` for the interactive Scalar UI. The raw OpenAPI JSON is at `http://localhost:9090/api-docs/openapi.json`.

The spec is generated at compile time from `#[utoipa::path(...)]` attributes on each handler and `#[derive(ToSchema)]` on each model. Adding a new endpoint or changing a model field automatically updates the spec.

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `ROUTE_CONFIG_FOLDERS` | `[]` | JSON array of folder paths to scan at startup |
| `ROUTE_CONFIG_THROW_ON_ERROR` | `false` | Fail startup if any config file is invalid |
| `RUST_LOG` | `info` | Log filter |
| `RUST_LOG_FORMAT` | `compact` | `compact`, `pretty`, or `json` |

See [environment variables reference](../reference/environment-variables.md) for the full list.
