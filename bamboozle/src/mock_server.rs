use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    Router,
};
use std::collections::HashMap;

use crate::{
    app_state::AppState,
    models::{
        context::ContextModel,
        match_key::MatchKey,
        route::{ResponseDefinition, RouteDefinition},
    },
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .fallback(catch_all)
        .with_state(state)
}

async fn catch_all(
    State(state): State<AppState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body_bytes: Bytes,
) -> impl IntoResponse {
    let verb = method.as_str().to_string();
    let path = uri.path().to_string();

    let query_params: HashMap<String, String> = uri
        .query()
        .map(|q| {
            form_urlencoded::parse(q.as_bytes())
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect()
        })
        .unwrap_or_default();

    let header_map: HashMap<String, String> = headers
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().to_string(),
                v.to_str().unwrap_or("").to_string(),
            )
        })
        .collect();

    let body_raw = String::from_utf8_lossy(&body_bytes).into_owned();
    let is_json = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("application/json"))
        .unwrap_or(false);
    let body: serde_json::Value = if is_json {
        serde_json::from_str(&body_raw).unwrap_or(serde_json::Value::String(body_raw.clone()))
    } else {
        serde_json::Value::String(body_raw.clone())
    };

    match state.store.match_route(&verb, &path) {
        None => {
            let ctx = ContextModel {
                query_params: HashMap::new(),
                headers: HashMap::new(),
                route_values: HashMap::new(),
                body: serde_json::Value::Null,
                body_raw: String::new(),
                route_model: RouteDefinition {
                    match_key: MatchKey::new(verb, path),
                    response: ResponseDefinition::default(),
                },
            };
            state.tracker.record_unmatched(ctx);
            StatusCode::NOT_FOUND.into_response()
        }

        Some((route_def, route_values)) => {
            let ctx = ContextModel {
                query_params,
                headers: header_map,
                route_values,
                body,
                body_raw: body_raw.clone(),
                route_model: route_def.clone(),
            };
            state.tracker.record_matched(ctx.clone());

            let status_str = state
                .renderer
                .render_or_fallback(&route_def.response.status, &ctx, "200");
            let status_code: u16 = status_str.trim().parse().unwrap_or(200);

            let body = if route_def.response.loopback == Some(true) {
                body_raw
            } else {
                route_def
                    .response
                    .content
                    .as_deref()
                    .map(|t| state.renderer.render_or_fallback(t, &ctx, ""))
                    .unwrap_or_default()
            };

            let mut builder = Response::builder().status(status_code);

            for (key, val_template) in &route_def.response.headers {
                let val = state.renderer.render_or_fallback(val_template, &ctx, "");
                builder = builder.header(key.as_str(), val.as_str());
            }

            builder
                .body(axum::body::Body::from(body))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}
