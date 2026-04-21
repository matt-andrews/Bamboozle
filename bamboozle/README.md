# Bamboozle — Contributor Guide

This document is for developers working on the Rust application itself. If you're
looking for usage documentation, see the [project README](../README.md).

---

## What the app does

Bamboozle is a programmable HTTP mock server. It runs two listeners:

| Port | Purpose |
| ------ | --------- |
| `:8080` | **Mock surface** — receives traffic from the system under test |
| `:9090` | **Control API** — receives commands from test code |

Test code registers routes on the control API. The mock surface matches incoming
requests against those routes, records every call, and returns configured
responses. At the end of a test the control API is used to assert what was called
and then reset state.

---

## Module map

```json
src/
├── main.rs            — entry point; binds two Tokio listeners and starts both servers
├── app_state.rs       — shared state (Arc-wrapped store + tracker + renderer)
├── config.rs          — reads env vars into AppConfig
├── config_loader.rs   — loads JSON/YAML route files at startup
├── error.rs           — typed error enum; maps to HTTP status codes via IntoResponse
├── expression.rs      — boolean expression evaluator for /assert filtering
├── liquid_render.rs   — Liquid template engine; converts ContextModel → globals
├── mock_server.rs     — catch-all handler: match → record → simulate → respond
│
├── models/
│   ├── match_key.rs   — verb + pattern pair; normalizes on construction
│   ├── route.rs       — RouteDefinition and ResponseDefinition (the JSON schema)
│   ├── context.rs     — ContextModel: snapshot of one request (used for templates + tracking)
│   └── simulation.rs  — delay and fault configuration with sampling logic
│
├── routing/
│   ├── store.rs       — DashMap-backed route store; compiles patterns at insert time
│   └── regex_gen.rs   — converts route patterns like `{id:int}` into compiled Regex
│
├── tracking/
│   └── tracker.rs     — records matched + unmatched calls; exposes per-route history
│
└── control/
    ├── mod.rs         — OpenAPI spec, Scalar UI, and router wiring
    └── handlers.rs    — one async fn per control endpoint
```

---

## Key patterns (C# → Rust translation)

### Shared mutable state — `Arc<T>` instead of `static` / DI

In C# you'd register a singleton in the DI container. In Rust, shared state is
wrapped in `Arc<T>` (atomic reference-counted pointer) and cloned cheaply — the
clone just bumps a reference count; the data itself is never copied.

```rust
// app_state.rs — cloning AppState is free; all three fields point at the same heap data
#[derive(Clone)]
pub struct AppState {
    pub store: Arc<RouteStore>,
    pub tracker: Arc<CallTracker>,
    pub renderer: Arc<Renderer>,
}
```

Axum extracts state in handlers via `State(state): State<AppState>`.

### Interior mutability without locks — `DashMap`

`RouteStore` and `CallTracker` need to be mutated from concurrent requests but
are accessed through a shared `Arc`. Rust would normally require `Mutex<HashMap>`
to allow mutation through a shared reference. `DashMap` is a concurrent hash map
that handles its own fine-grained locking internally, so call sites look like
ordinary HashMap reads and writes without explicit lock guards.

```rust
// routing/store.rs
pub struct RouteStore {
    routes: DashMap<String, DashMap<String, StoredRoute>>,
}

// No &mut self — shared ref is enough:
pub fn set_route(&self, def: RouteDefinition) -> Result<...> { ... }
pub fn match_route(&self, verb: &str, path: &str) -> Option<...> { ... }
```

### Error handling — `thiserror` + `anyhow`

This is the standard two-layer pattern:

- **`thiserror`** defines typed errors with known variants (domain errors that
  callers may want to match on, like `NotFound` or `AlreadyExists`).
- **`anyhow`** wraps arbitrary errors into a single opaque type for the
  "something went wrong, just propagate it" path.
- The `?` operator works like C#'s `throw` on error — it short-circuits the
  function and returns the error to the caller.

`AppError` implements `IntoResponse` (axum's equivalent of an exception filter),
so returning `Err(AppError::NotFound(...))` from a handler automatically sends
a `404` JSON response.

### Compile-once, match-many — regex at insert time

Route patterns like `/orders/{id:int}` are compiled into a `Regex` exactly
once when the route is stored. The compiled `Regex` is kept alongside the
definition in `StoredRoute`. `match_route` just runs the already-compiled
regex — no parsing on the hot path.

### Owned data in closures — `move` closures and `.clone()` before capture

Rust closures that are stored (e.g. inside `evalexpr::Function::new`) must not
borrow from the enclosing scope. In `expression.rs`, query params / headers / etc.
are cloned into local variables before being captured:

```rust
let query_params = ctx.query_params.clone();
context.set_function("query", Function::new(move |arg| {
    // `query_params` is owned by this closure, not borrowed from ctx
    ...
}));
```

This is the Rust equivalent of capturing by value in a C# lambda.

---

## Request lifecycle (mock surface)

```json
HTTP request arrives at :8080
       │
       ▼
catch_all (mock_server.rs)
       │
       ├─ extract verb, path, query params, headers, body
       │
       ├─ RouteStore::match_route
       │    ├─ looks up verb map in DashMap
       │    ├─ sorts stored routes (static before parameterized, longer first)
       │    └─ runs compiled Regex against normalized URL; extracts named captures
       │
       ├─ [no match] → record_unmatched → 404
       │
       └─ [match] → build ContextModel
                   → render setState template (if set)
                   → record_matched
                   → apply_simulation (delay then fault check)
                   → build_response (render status, headers, body templates)
```

---

## Adding a new feature — end-to-end walkthrough

The typical flow for adding new behaviour is:

1. **Add a field to a model** (`models/`) — add it to `RouteDefinition`,
   `ResponseDefinition`, or `SimulationConfig` with `#[serde(default)]` so
   existing config files keep working.

2. **Update `ContextModel` if the field needs to be template-accessible** — add
   it to `context_to_object` in `liquid_render.rs` and expose it under a logical
   name so Liquid templates can reference it.

3. **Use the field in the handler** — `catch_all` in `mock_server.rs` is the
   right place for per-request behaviour changes.

4. **Expose via control API if test code needs to configure it** — add a handler
   in `control/handlers.rs` and wire it into the router in `control/mod.rs`.
   Add the type to the `#[openapi(components(schemas(...)))]` list so it appears
   in the Scalar UI.

5. **Write tests** — each module has `#[cfg(test)]` inline. Integration-style
   tests in `mock_server.rs` use `tower::ServiceExt::oneshot` to send a fake
   request through the full axum stack without binding a port.

---

## Testing approach

Tests live in the same file as the code they test, inside a `#[cfg(test)]` block.
The test binary is only compiled when running `cargo test`.

```bash
cargo test              # run all tests
cargo test tracker      # run tests whose names contain "tracker"
cargo test -- --nocapture   # show println! output during tests
```

Most modules have a `make_ctx()` or `make_route()` helper at the top of the test
block that constructs a minimal valid value — use these instead of spelling out
all fields when adding new tests.

Handler tests in `mock_server.rs` go through the real axum router using
`tower::ServiceExt::oneshot`. This means the full middleware stack, body parsing,
and state extraction are exercised without opening a network socket:

```rust
let response = router(AppState::new())
    .oneshot(Request::builder().uri("/things").body(Body::empty()).unwrap())
    .await
    .unwrap();
assert_eq!(response.status(), StatusCode::OK);
```

---

## Building and running locally

```bash
# Build (debug)
cargo build

# Build (release — uses LTO, strips symbols)
cargo build --release

# Run with default settings (no startup routes)
cargo run

# Run with a config folder
ROUTE_CONFIG_FOLDERS='["/path/to/routes"]' cargo run

# Build the Docker image
docker build -t bamboozle .
```

### Environment variables

| Variable | Default | Description |
| ---------- | --------- | ------------- |
| `ROUTE_CONFIG_FOLDERS` | `[]` | JSON array of folder paths to load at startup |
| `ROUTE_CONFIG_THROW_ON_ERROR` | `false` | Fail startup if any config file is invalid |
| `RUST_LOG` | `info` | Log filter (`trace`, `debug`, `info`, `warn`, `error`) |

---

## OpenAPI / Scalar UI

The control API is fully documented. While the server is running, open
`http://localhost:9090/` for the interactive Scalar UI. The raw OpenAPI JSON is
at `http://localhost:9090/api-docs/openapi.json`.

The spec is generated at compile time from `#[utoipa::path(...)]` attributes on
each handler and `#[derive(ToSchema)]` on each model type. Adding a new endpoint
or changing a model field automatically updates the spec — no manual JSON editing.
