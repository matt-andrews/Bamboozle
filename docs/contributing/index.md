# Contributing to Bamboozle

Bamboozle is a Rust application built on [Axum](https://github.com/tokio-rs/axum). This section covers what you need to know to work on it.

- [Building and running locally](building.md)
- [Request lifecycle](request-lifecycle.md)
- [Adding a feature](adding-a-feature.md)
- [Testing](testing.md)

---

## Module map

```
bamboozle/src/
├── main.rs            — entry point; binds two Tokio listeners
├── app_state.rs       — shared state (Arc-wrapped store + tracker + renderer)
├── config.rs          — reads env vars into AppConfig
├── config_loader.rs   — loads JSON/YAML route files at startup
├── error.rs           — typed error enum; maps to HTTP status codes
├── expression.rs      — boolean expression evaluator for /assert filtering
├── liquid_render.rs   — Liquid template engine; converts ContextModel → globals
├── mock_server.rs     — catch-all handler: match → record → simulate → respond
│
├── models/
│   ├── match_key.rs   — verb + pattern pair; normalizes on construction
│   ├── route.rs       — RouteDefinition and ResponseDefinition
│   ├── context.rs     — ContextModel: snapshot of one request
│   └── simulation.rs  — delay and fault configuration
│
├── routing/
│   ├── store.rs       — DashMap-backed route store; compiles patterns at insert time
│   └── regex_gen.rs   — converts patterns like {id:int} into compiled Regex
│
├── tracking/
│   └── tracker.rs     — records matched + unmatched calls; exposes per-route history
│
└── control/
    ├── mod.rs         — OpenAPI spec, Scalar UI, router wiring
    └── handlers.rs    — one async fn per control endpoint
```

---

## Key Rust patterns

### Shared state — `Arc<T>` instead of singletons

Shared state is wrapped in `Arc<T>` (atomic reference count) and cloned cheaply across request handlers — the clone increments a counter; the data is never copied.

```rust
// app_state.rs
#[derive(Clone)]
pub struct AppState {
    pub store: Arc<RouteStore>,
    pub tracker: Arc<CallTracker>,
    pub renderer: Arc<Renderer>,
}
```

Axum extracts state in handlers via `State(state): State<AppState>`.

### Concurrent mutation — `DashMap`

`RouteStore` and `CallTracker` need to be mutated from concurrent requests through a shared `Arc`. `DashMap` is a concurrent hash map with internal fine-grained locking, so call sites look like ordinary HashMap reads and writes without explicit lock guards.

```rust
// routing/store.rs — no &mut self needed
pub fn set_route(&self, def: RouteDefinition) -> Result<...> { ... }
pub fn match_route(&self, verb: &str, path: &str) -> Option<...> { ... }
```

### Error handling — `thiserror` + `anyhow`

`thiserror` defines typed errors with known variants (`NotFound`, `AlreadyExists`) that callers can match on. `anyhow` wraps arbitrary errors for the "just propagate it" path. `AppError` implements axum's `IntoResponse`, so returning `Err(AppError::NotFound(...))` from a handler automatically sends a `404` JSON response.

### Compile-once regex

Route patterns like `/orders/{id:int}` are compiled to `Regex` once at insert time. The compiled regex is stored alongside the definition in `StoredRoute`. `match_route` runs the pre-compiled regex — no parsing on the hot path.
