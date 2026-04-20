use axum::{response::Html, routing::{delete, get, post}, Router};
use utoipa::OpenApi;

use crate::{
    app_state::AppState,
    models::{
        context::ContextModel,
        match_key::MatchKey,
        route::{ResponseDefinition, RouteDefinition},
    },
};

pub mod handlers;

#[derive(OpenApi)]
#[openapi(
    info(title = "Bamboozle Control API", version = env!("CARGO_PKG_VERSION")),
    paths(
        handlers::post_routes,
        handlers::put_routes,
        handlers::delete_route,
        handlers::get_routes,
        handlers::get_route_calls,
        handlers::delete_route_calls,
        handlers::assert_route,
        handlers::get_unmatched,
        handlers::reset,
        handlers::health,
        handlers::version,
    ),
    components(
        schemas(
            MatchKey,
            RouteDefinition,
            ResponseDefinition,
            ContextModel,
            handlers::AssertRequest,
        )
    ),
    tags(
        (name = "Routes", description = "Create, update, and delete mock routes"),
        (name = "Calls",  description = "Inspect and assert recorded HTTP calls"),
        (name = "Control", description = "Health, version, and reset"),
    )
)]
struct ApiDoc;

async fn scalar_ui() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html>
  <head>
    <title>Bamboozle Control API</title>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
  </head>
  <body>
    <script id="api-reference" data-url="/api-docs/openapi.json" data-configuration='{"theme":"elysiajs"}'></script>
    <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
  </body>
</html>"#,
    )
}

async fn openapi_json() -> axum::Json<utoipa::openapi::OpenApi> {
    axum::Json(ApiDoc::openapi())
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(
            "/control/routes",
            post(handlers::post_routes)
                .put(handlers::put_routes)
                .get(handlers::get_routes),
        )
        .route(
            "/control/routes/:verb/:pattern",
            delete(handlers::delete_route),
        )
        .route(
            "/control/routes/:verb/:pattern/calls",
            get(handlers::get_route_calls).delete(handlers::delete_route_calls),
        )
        .route(
            "/control/routes/:verb/:pattern/assert",
            post(handlers::assert_route),
        )
        .route("/control/unmatched", get(handlers::get_unmatched))
        .route("/control/reset", post(handlers::reset))
        .route("/control/health", get(handlers::health))
        .route("/control/version", get(handlers::version))
        .route("/", get(scalar_ui))
        .route("/api-docs/openapi.json", get(openapi_json))
        .with_state(state)
}
