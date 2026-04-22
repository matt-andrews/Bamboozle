# Adding a feature

Most new behaviour follows the same five steps.

## 1. Add a field to a model

New configuration lives in `models/`. Add the field with `#[serde(default)]` so existing config files keep working without the new field.

```rust
// models/route.rs
#[derive(Deserialize, Serialize, ToSchema)]
pub struct ResponseDefinition {
    // existing fields...
    #[serde(default)]
    pub my_new_field: Option<String>,
}
```

## 2. Expose it in templates (if needed)

If test code or templates should be able to read the new value, add it to `context_to_object` in `liquid_render.rs`:

```rust
globals.insert("myNewField".into(), liquid::model::Value::scalar(value));
```

## 3. Use it in the handler

`catch_all` in `mock_server.rs` is the right place for per-request behaviour. It has the matched `RouteDefinition`, the built `ContextModel`, and the rendered response — add your logic where it fits in the flow.

## 4. Expose via the control API (if needed)

If test code needs to configure or query the new behaviour:

1. Add a handler function in `control/handlers.rs`
2. Wire it into the router in `control/mod.rs`
3. Add `#[utoipa::path(...)]` to the handler so the OpenAPI spec updates automatically
4. Add the new type to the `#[openapi(components(schemas(...)))]` list in `control/mod.rs`

## 5. Write tests

Each module has a `#[cfg(test)]` block at the bottom. Use the existing `make_ctx()` or `make_route()` helpers rather than constructing full values from scratch.

For request-path behaviour, add a handler test in `mock_server.rs` using `tower::ServiceExt::oneshot` — this sends a fake request through the full axum stack without binding a port. See [Testing](testing.md) for examples.
