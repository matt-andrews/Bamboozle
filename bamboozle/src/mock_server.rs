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
    Router::new().fallback(catch_all).with_state(state)
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
        .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
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
                query_params,
                headers: header_map,
                route_values: HashMap::new(),
                body,
                body_raw,
                route_model: RouteDefinition {
                    match_key: MatchKey::new(verb, path),
                    set_state: None,
                    response: ResponseDefinition::default(),
                },
                state: String::new(),
                previous_context: None,
            };
            state.tracker.record_unmatched(ctx);
            StatusCode::NOT_FOUND.into_response()
        }

        Some((route_def, route_values)) => {
            let previous_context = state
                .tracker
                .get_last_matched_for_route(&route_def.match_key);
            let mut ctx = ContextModel {
                query_params,
                headers: header_map,
                route_values,
                body,
                body_raw: body_raw.clone(),
                route_model: route_def.clone(),
                previous_context,
                state: String::new(),
            };

            if let Some(set_state) = &route_def.set_state {
                ctx.state = state.renderer.render_or_fallback(set_state, &ctx, "");
            }

            state.tracker.record_matched(ctx.clone());

            let status_str =
                state
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app_state::AppState,
        models::{
            match_key::MatchKey,
            route::{ResponseDefinition, RouteDefinition},
        },
    };
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    fn make_route(
        verb: &str,
        pattern: &str,
        content: Option<&str>,
        status: &str,
    ) -> RouteDefinition {
        RouteDefinition {
            match_key: MatchKey::new(verb, pattern),
            set_state: None,
            response: ResponseDefinition {
                status: status.to_string(),
                content: content.map(|s| s.to_string()),
                ..Default::default()
            },
        }
    }

    async fn make_request(state: AppState, uri: &str) {
        router(state)
            .oneshot(
                Request::builder()
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    async fn body_string(body: axum::body::Body) -> String {
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    #[tokio::test]
    async fn unmatched_route_returns_404() {
        let app = router(AppState::new());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn matched_static_route_returns_200() {
        let state = AppState::new();
        state
            .store
            .set_route(make_route("GET", "/hello", Some("world"), "200"))
            .unwrap();
        let response = router(state)
            .oneshot(
                Request::builder()
                    .uri("/hello")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn response_body_matches_configured_content() {
        let state = AppState::new();
        state
            .store
            .set_route(make_route("GET", "/greet", Some("hello there"), "200"))
            .unwrap();
        let response = router(state)
            .oneshot(
                Request::builder()
                    .uri("/greet")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(body_string(response.into_body()).await, "hello there");
    }

    #[tokio::test]
    async fn custom_status_code_is_returned() {
        let state = AppState::new();
        state
            .store
            .set_route(make_route("POST", "/things", None, "201"))
            .unwrap();
        let response = router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/things")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn loopback_echoes_request_body() {
        let state = AppState::new();
        state
            .store
            .set_route(RouteDefinition {
                match_key: MatchKey::new("POST", "/echo"),
                set_state: None,
                response: ResponseDefinition {
                    status: "200".to_string(),
                    loopback: Some(true),
                    ..Default::default()
                },
            })
            .unwrap();
        let response = router(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/echo")
                    .body(Body::from("ping"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(body_string(response.into_body()).await, "ping");
    }

    #[tokio::test]
    async fn route_value_rendered_in_response_body() {
        let state = AppState::new();
        state
            .store
            .set_route(make_route(
                "GET",
                "/greet/{name}",
                Some("Hello {{ routeValues.name }}"),
                "200",
            ))
            .unwrap();
        let response = router(state)
            .oneshot(
                Request::builder()
                    .uri("/greet/Alice")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(body_string(response.into_body()).await, "Hello Alice");
    }

    #[tokio::test]
    async fn camel_case_route_param_rendered_in_response_body() {
        // Construct via MatchKey fields directly (not MatchKey::new) to simulate the
        // serde deserialization path used by config file loading, where param names
        // retain their original case.
        let state = AppState::new();
        state
            .store
            .set_route(RouteDefinition {
                match_key: MatchKey {
                    verb: "GET".to_string(),
                    pattern: "/thing/{thingName}/{thingVersion}".to_string(),
                },
                set_state: None,
                response: ResponseDefinition {
                    status: "200".to_string(),
                    content: Some(r#"{"id":"{{ routeValues.thingName }}"}"#.to_string()),
                    ..Default::default()
                },
            })
            .unwrap();
        let response = router(state)
            .oneshot(
                Request::builder()
                    .uri("/thing/widget/v1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            body_string(response.into_body()).await,
            r#"{"id":"widget"}"#
        );
    }

    #[tokio::test]
    async fn unmatched_request_is_tracked() {
        let state = AppState::new();
        let tracker = state.tracker.clone();
        router(state)
            .oneshot(
                Request::builder()
                    .uri("/missing")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(tracker.get_unmatched().len(), 1);
    }

    #[tokio::test]
    async fn matched_request_is_tracked() {
        let state = AppState::new();
        state
            .store
            .set_route(make_route("GET", "/tracked", Some("ok"), "200"))
            .unwrap();
        let tracker = state.tracker.clone();
        router(state)
            .oneshot(
                Request::builder()
                    .uri("/tracked")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            tracker
                .get_calls_for_route(&MatchKey::new("GET", "/tracked"))
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn previous_context_is_null_on_first_call() {
        let state = AppState::new();
        state
            .store
            .set_route(make_route("GET", "/thing", Some("ok"), "200"))
            .unwrap();
        let tracker = state.tracker.clone();
        make_request(state, "/thing").await;
        let calls = tracker.get_calls_for_route(&MatchKey::new("GET", "/thing"));
        assert_eq!(calls.len(), 1);
        assert!(calls[0].previous_context.is_none());
    }

    #[tokio::test]
    async fn previous_context_is_set_on_second_call() {
        let state = AppState::new();
        state
            .store
            .set_route(make_route("GET", "/thing", Some("ok"), "200"))
            .unwrap();
        let tracker = state.tracker.clone();
        make_request(state.clone(), "/thing").await;
        make_request(state, "/thing").await;
        let calls = tracker.get_calls_for_route(&MatchKey::new("GET", "/thing"));
        assert_eq!(calls.len(), 2);
        assert!(calls.iter().any(|c| c.previous_context.is_some()));
    }

    #[tokio::test]
    async fn previous_context_does_not_nest() {
        let state = AppState::new();
        state
            .store
            .set_route(make_route("GET", "/thing", Some("ok"), "200"))
            .unwrap();
        let tracker = state.tracker.clone();
        make_request(state.clone(), "/thing").await;
        make_request(state.clone(), "/thing").await;
        make_request(state, "/thing").await;
        let calls = tracker.get_calls_for_route(&MatchKey::new("GET", "/thing"));
        let call_with_prev = calls
            .iter()
            .find(|c| c.previous_context.is_some())
            .unwrap();
        assert!(call_with_prev
            .previous_context
            .as_ref()
            .unwrap()
            .previous_context
            .is_none());
    }

    #[tokio::test]
    async fn set_state_template_is_rendered_and_stored() {
        let state = AppState::new();
        state
            .store
            .set_route(RouteDefinition {
                match_key: MatchKey::new("GET", "/stateful"),
                set_state: Some("hello-state".to_string()),
                response: ResponseDefinition {
                    status: "200".to_string(),
                    content: Some("{{ state }}".to_string()),
                    ..Default::default()
                },
            })
            .unwrap();
        let tracker = state.tracker.clone();
        let response = router(state)
            .oneshot(
                Request::builder()
                    .uri("/stateful")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(body_string(response.into_body()).await, "hello-state");
        let calls = tracker.get_calls_for_route(&MatchKey::new("GET", "/stateful"));
        assert_eq!(calls[0].state, "hello-state");
    }

    #[tokio::test]
    async fn previous_context_state_accessible_on_second_call() {
        let state = AppState::new();
        state
            .store
            .set_route(RouteDefinition {
                match_key: MatchKey::new("GET", "/counter"),
                set_state: Some(
                    "{% if previousContext == nil %}0{% else %}{{ previousContext.state }}{% endif %}"
                        .to_string(),
                ),
                response: ResponseDefinition {
                    status: "200".to_string(),
                    content: Some("{{ state }}".to_string()),
                    ..Default::default()
                },
            })
            .unwrap();
        make_request(state.clone(), "/counter").await;
        let response = router(state)
            .oneshot(
                Request::builder()
                    .uri("/counter")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(body_string(response.into_body()).await, "0");
    }
}
