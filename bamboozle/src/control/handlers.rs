use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::{debug, warn};
use utoipa::ToSchema;

use crate::{
    app_state::AppState,
    error::AppError,
    expression,
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
    let response = state.store.set_route(route)?;
    Ok(Json(response))
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
    // Ignore NotFound — PUT is idempotent. delete_route normalizes internally.
    let _ = state.store.delete_route(&route.match_key);
    let response = state.store.set_route(route)?;
    Ok(Json(response))
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
    let calls = state
        .tracker
        .get_calls_for_route(&MatchKey::new(verb, pattern));
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
    /// Boolean expression evaluated against each recorded call.
    ///
    /// Variables: `verb`, `pattern`
    /// Functions: `query("key")`, `header("key")`, `route("key")`,
    ///   `contains(s, sub)`, `starts_with(s, prefix)`, `ends_with(s, suffix)`
    ///
    /// Example: `query("status") == "active" && verb == "POST"`
    pub expression: Option<String>,
}

#[derive(Deserialize)]
pub struct AssertQuery {
    #[serde(default = "AssertQuery::default_expect")]
    pub expect: i64,
}

impl AssertQuery {
    fn default_expect() -> i64 {
        -1
    }
}

#[utoipa::path(
    post,
    path = "/control/routes/{verb}/{pattern}/assert",
    params(
        ("verb" = String, Path, description = "HTTP verb"),
        ("pattern" = String, Path, description = "Route pattern (URL-encode slashes as %2F)"),
        ("expect" = Option<i64>, Query, description = "Expected call count after filtering. -1 (default) accepts any count ≥ 1 when an expression is given, or any count otherwise."),
    ),
    request_body = AssertRequest,
    responses(
        (status = 200, description = "Assertion passed"),
        (status = 400, description = "Invalid expression syntax"),
        (status = 418, description = "Assertion failed — filtered call count did not match expect"),
    ),
    tag = "Calls"
)]
pub async fn assert_route(
    State(state): State<AppState>,
    Path((verb, pattern)): Path<(String, String)>,
    Query(q): Query<AssertQuery>,
    Json(body): Json<AssertRequest>,
) -> Result<StatusCode, AppError> {
    let match_key = MatchKey::new(verb, pattern);
    let calls = state.tracker.get_calls_for_route(&match_key);
    let expr = body
        .expression
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let filtered: Vec<_> = if let Some(expr) = expr {
        let mut result = Vec::with_capacity(calls.len());
        for ctx in &calls {
            match expression::eval_expression(expr, ctx) {
                Ok(true) => result.push(ctx),
                Ok(false) => {}
                Err(e) => {
                    warn!(
                        verb = %match_key.verb,
                        pattern = %match_key.pattern,
                        expression = expr,
                        error = %e,
                        "Expression error during assertion filtering"
                    );
                    return Err(AppError::BadRequest(format!("Invalid expression: {e}")));
                }
            }
        }
        result
    } else {
        calls.iter().collect()
    };

    let count = filtered.len() as i64;
    let passed = if q.expect >= 0 {
        count == q.expect
    } else if expr.is_some() {
        // expression given but no explicit expect → require at least one match
        count >= 1
    } else {
        true
    };
    if passed {
        debug!(
            verb = %match_key.verb,
            pattern = %match_key.pattern,
            matched_count = count,
            expected = q.expect,
            expression = expr.unwrap_or("<none>"),
            "Assertion passed"
        );
        Ok(StatusCode::OK)
    } else {
        let condition = if q.expect >= 0 {
            format!("expected exactly {}, got {}", q.expect, count)
        } else {
            format!("expected >= 1 match for expression, got {}", count)
        };
        warn!(
            verb = %match_key.verb,
            pattern = %match_key.pattern,
            matched_count = count,
            total_calls = calls.len(),
            expected = q.expect,
            expression = expr.unwrap_or("<none>"),
            condition = %condition,
            "Assertion failed"
        );
        Ok(StatusCode::IM_A_TEAPOT)
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
