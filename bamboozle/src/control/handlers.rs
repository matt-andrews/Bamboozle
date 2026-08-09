use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use tracing::{debug, warn};

use crate::{
    app_state::AppState,
    error::AppError,
    models::{context::ContextModel, match_key::MatchKey, route::RouteDefinition},
};

use super::assertions::{self, AssertQuery, AssertRequest};

// ── POST /control/routes ────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/control/routes",
    request_body = RouteDefinition,
    responses(
        (status = 201, description = "Route created", body = Vec<RouteDefinition>),
        (status = 409, description = "Route already exists"),
    ),
    tag = "Routes"
)]
pub async fn post_routes(
    State(state): State<AppState>,
    Json(route): Json<RouteDefinition>,
) -> Result<(StatusCode, Json<Vec<RouteDefinition>>), AppError> {
    let response = state.store.set_route(route)?;
    for def in &response {
        state.tracker.delete_calls_for_route(&def.match_key);
    }
    Ok((StatusCode::CREATED, Json(response)))
}

// ── PUT /control/routes ─────────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/control/routes",
    request_body = RouteDefinition,
    responses(
        (status = 200, description = "Route updated", body = Vec<RouteDefinition>),
        (status = 201, description = "Route created", body = Vec<RouteDefinition>),
    ),
    tag = "Routes"
)]
pub async fn put_routes(
    State(state): State<AppState>,
    Json(route): Json<RouteDefinition>,
) -> Result<(StatusCode, Json<Vec<RouteDefinition>>), AppError> {
    // Delete each verb individually. With a multi-verb string like "GET,POST"
    // the store keys routes by single verb, so we must fan out the deletes.
    let mut any_replaced = false;
    for v in route.match_key.verb.split(',') {
        if state
            .store
            .delete_route(&MatchKey::new(v.trim(), &route.match_key.pattern))
            .is_ok()
        {
            any_replaced = true;
        }
    }

    let return_status = if any_replaced {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };

    let response = state.store.set_route(route)?;
    for def in &response {
        state.tracker.delete_calls_for_route(&def.match_key);
    }
    Ok((return_status, Json(response)))
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
        (status = 204, description = "Route deleted"),
        (status = 404, description = "Route not found"),
    ),
    tag = "Routes"
)]
pub async fn delete_route(
    State(state): State<AppState>,
    Path((verb, pattern)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    state.store.delete_route(&MatchKey::new(verb, pattern))?;
    Ok(StatusCode::NO_CONTENT)
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
        (status = 204, description = "Call history cleared"),
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
    StatusCode::NO_CONTENT
}

// ── POST /control/routes/:verb/:pattern/assert ────────────────────────────────

#[utoipa::path(
    post,
    path = "/control/routes/{verb}/{pattern}/assert",
    params(
        ("verb" = String, Path, description = "HTTP verb"),
        ("pattern" = String, Path, description = "Route pattern (URL-encode slashes as %2F)"),
        ("called_exactly" = Option<i64>, Query, description = "Assert the filtered call count equals exactly n."),
        ("called_at_least" = Option<i64>, Query, description = "Assert the filtered call count is at least n."),
        ("called_at_most" = Option<i64>, Query, description = "Assert the filtered call count is at most n."),
        ("never_called" = Option<bool>, Query, description = "Assert the route was never called (equivalent to called_exactly=0)."),
    ),
    request_body = AssertRequest,
    responses(
        (status = 200, description = "Assertion passed"),
        (status = 400, description = "Invalid CEL syntax, execution, result type, or count qualifier"),
        (status = 406, description = "Assertion failed — filtered call count did not match expect"),
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
    let expression = assertions::normalize_expression(body.expression.as_deref());
    let result = assertions::evaluate(&calls, expression, &q).map_err(|error| {
        warn!(
            verb = %match_key.verb,
            pattern = %match_key.pattern,
            expression = expression.unwrap_or("<none>"),
            error = %error,
            "Invalid assertion request"
        );
        AppError::BadRequest(error.to_string())
    })?;

    if result.passed {
        debug!(
            verb = %match_key.verb,
            pattern = %match_key.pattern,
            matched_count = result.matched_count,
            expression = expression.unwrap_or("<none>"),
            "Assertion passed"
        );
        Ok(StatusCode::OK)
    } else {
        warn!(
            verb = %match_key.verb,
            pattern = %match_key.pattern,
            matched_count = result.matched_count,
            total_calls = calls.len(),
            expression = expression.unwrap_or("<none>"),
            condition = %result.condition,
            "Assertion failed"
        );
        Ok(StatusCode::NOT_ACCEPTABLE)
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
        (status = 204, description = "All routes and call history cleared"),
    ),
    tag = "Control"
)]
pub async fn reset(State(state): State<AppState>) -> StatusCode {
    state.store.reset();
    state.tracker.reset();
    StatusCode::NO_CONTENT
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
