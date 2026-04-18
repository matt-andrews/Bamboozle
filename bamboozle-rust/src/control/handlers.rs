use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    app_state::AppState,
    error::{AppError, RouteError},
    models::{context::ContextModel, match_key::MatchKey, route::RouteDefinition},
};

// ── POST /control/routes ────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/control/routes",
    request_body = RouteDefinition,
    responses(
        (status = 200, description = "Route created", body = RouteDefinition),
        (status = 409, description = "Route already exists"),
    ),
    tag = "Routes"
)]
pub async fn post_routes(
    State(state): State<AppState>,
    Json(route): Json<RouteDefinition>,
) -> Result<Json<RouteDefinition>, AppError> {
    state.store.set_route(route.clone())?;
    Ok(Json(route))
}

// ── PUT /control/routes ─────────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/control/routes",
    request_body = RouteDefinition,
    responses(
        (status = 200, description = "Route upserted", body = RouteDefinition),
    ),
    tag = "Routes"
)]
pub async fn put_routes(
    State(state): State<AppState>,
    Json(route): Json<RouteDefinition>,
) -> Result<Json<RouteDefinition>, AppError> {
    // Ignore NotFound — PUT is idempotent
    let _ = state.store.delete_route(&route.match_key);
    state.store.set_route(route.clone())?;
    Ok(Json(route))
}

// ── DELETE /control/routes/:verb/:pattern ────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/control/routes/{verb}/{pattern}",
    params(
        ("verb" = String, Path, description = "HTTP verb (e.g. GET, POST)"),
        ("pattern" = String, Path, description = "Route pattern — URL-encode slashes as %2F (e.g. api%2Fusers%2F%7Bid%7D)"),
    ),
    responses(
        (status = 200, description = "Route deleted"),
        (status = 404, description = "Route not found"),
    ),
    tag = "Routes"
)]
pub async fn delete_route(
    State(state): State<AppState>,
    Path((verb, pattern)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    state.store.delete_route(&MatchKey::new(verb, pattern))?;
    Ok(StatusCode::OK)
}

// ── GET /control/routes ──────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/control/routes",
    responses(
        (status = 200, description = "All stored routes", body = Vec<RouteDefinition>),
    ),
    tag = "Routes"
)]
pub async fn get_routes(State(state): State<AppState>) -> Json<Vec<RouteDefinition>> {
    Json(state.store.get_all_routes())
}

// ── GET /control/routes/:verb/:pattern/calls ─────────────────────────────────

#[utoipa::path(
    get,
    path = "/control/routes/{verb}/{pattern}/calls",
    params(
        ("verb" = String, Path, description = "HTTP verb"),
        ("pattern" = String, Path, description = "Route pattern (URL-encode slashes as %2F)"),
    ),
    responses(
        (status = 200, description = "All recorded calls to this route", body = Vec<ContextModel>),
    ),
    tag = "Calls"
)]
pub async fn get_route_calls(
    State(state): State<AppState>,
    Path((verb, pattern)): Path<(String, String)>,
) -> impl IntoResponse {
    let calls = state.tracker.get_calls_for_route(&MatchKey::new(verb, pattern));
    Json(calls)
}

// ── DELETE /control/routes/:verb/:pattern/calls ───────────────────────────────

#[utoipa::path(
    delete,
    path = "/control/routes/{verb}/{pattern}/calls",
    params(
        ("verb" = String, Path, description = "HTTP verb"),
        ("pattern" = String, Path, description = "Route pattern (URL-encode slashes as %2F)"),
    ),
    responses(
        (status = 200, description = "Call history cleared"),
    ),
    tag = "Calls"
)]
pub async fn delete_route_calls(
    State(state): State<AppState>,
    Path((verb, pattern)): Path<(String, String)>,
) -> StatusCode {
    state
        .tracker
        .delete_calls_for_route(&MatchKey::new(verb, pattern));
    StatusCode::OK
}

// ── POST /control/routes/:verb/:pattern/assert ────────────────────────────────

#[derive(Deserialize, ToSchema)]
pub struct AssertRequest {
    /// LINQ-style filter expression (not yet evaluated — reserved for future use).
    pub expression: Option<String>,
}

#[derive(Deserialize)]
pub struct AssertQuery {
    #[serde(default = "default_expect")]
    pub expect: i64,
}

fn default_expect() -> i64 {
    -1
}

#[utoipa::path(
    post,
    path = "/control/routes/{verb}/{pattern}/assert",
    params(
        ("verb" = String, Path, description = "HTTP verb"),
        ("pattern" = String, Path, description = "Route pattern (URL-encode slashes as %2F)"),
        ("expect" = Option<i64>, Query, description = "Expected call count. -1 (default) accepts any count."),
    ),
    request_body = AssertRequest,
    responses(
        (status = 200, description = "Assertion passed"),
        (status = 418, description = "Assertion failed — call count did not match expect"),
    ),
    tag = "Calls"
)]
pub async fn assert_route(
    State(state): State<AppState>,
    Path((verb, pattern)): Path<(String, String)>,
    Query(q): Query<AssertQuery>,
    Json(_body): Json<AssertRequest>,
) -> StatusCode {
    let calls = state.tracker.get_calls_for_route(&MatchKey::new(verb, pattern));
    let count = calls.len() as i64;

    if q.expect < 0 || count == q.expect {
        StatusCode::OK
    } else {
        StatusCode::IM_A_TEAPOT
    }
}

// ── GET /control/unmatched ───────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/control/unmatched",
    responses(
        (status = 200, description = "All requests that did not match any route", body = Vec<MatchKey>),
    ),
    tag = "Calls"
)]
pub async fn get_unmatched(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.tracker.get_unmatched())
}

// ── POST /control/reset ──────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/control/reset",
    responses(
        (status = 200, description = "All routes and call history cleared"),
    ),
    tag = "Control"
)]
pub async fn reset(State(state): State<AppState>) -> StatusCode {
    state.store.reset();
    state.tracker.reset();
    StatusCode::OK
}

// ── GET /control/health ──────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/control/health",
    responses(
        (status = 200, description = "Service is healthy"),
    ),
    tag = "Control"
)]
pub async fn health() -> StatusCode {
    StatusCode::OK
}

// ── GET /control/version ─────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/control/version",
    responses(
        (status = 200, description = "Bamboozle version string", body = String),
    ),
    tag = "Control"
)]
pub async fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
