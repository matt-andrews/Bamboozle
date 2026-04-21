use axum::{
    body::{Body, Bytes},
    extract::State,
    http::{HeaderMap, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    Router,
};
use futures::stream;
use std::{collections::HashMap, time::Duration};

use tracing::warn;

use crate::{
    app_state::AppState,
    liquid_render::Renderer,
    models::{
        context::ContextModel,
        match_key::MatchKey,
        route::{ResponseDefinition, RouteDefinition},
        simulation::{FaultKind, SimulationConfig},
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
    let query_params = extract_query_params(&uri);
    let header_map = extract_headers(&headers);
    let (body_raw, body) = parse_body(&body_bytes, &headers);

    match state.store.match_route(&verb, &path) {
        None => {
            let suggestions = state.store.suggest_routes(&verb, &path);
            if suggestions.is_empty() {
                warn!(
                    verb = %verb,
                    path = %path,
                    "Unmatched request — no similar routes registered"
                );
            } else {
                warn!(
                    verb = %verb,
                    path = %path,
                    suggestions = %suggestions.join(", "),
                    "Unmatched request — did you mean one of these routes?"
                );
            }
            let ctx = ContextModel {
                query_params,
                headers: header_map,
                route_values: HashMap::new(),
                body,
                body_raw,
                route_model: RouteDefinition {
                    match_key: MatchKey::new(verb, path),
                    set_state: None,
                    simulation: None,
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
                body_raw,
                route_model: route_def.clone(),
                previous_context,
                state: String::new(),
            };

            if let Some(set_state) = &route_def.set_state {
                ctx.state = state.renderer.render_or_fallback(set_state, &ctx, "");
            }

            state.tracker.record_matched(ctx.clone());

            if let Some(sim) = &route_def.simulation {
                if let Some(fault_response) = apply_simulation(sim).await {
                    return fault_response;
                }
            }

            build_response(&route_def, &ctx, &state.renderer)
        }
    }
}

fn extract_query_params(uri: &Uri) -> HashMap<String, String> {
    uri.query()
        .map(|q| {
            form_urlencoded::parse(q.as_bytes())
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect()
        })
        .unwrap_or_default()
}

fn extract_headers(headers: &HeaderMap) -> HashMap<String, String> {
    headers
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
        .collect()
}

fn parse_body(body_bytes: &Bytes, headers: &HeaderMap) -> (String, serde_json::Value) {
    let body_raw = String::from_utf8_lossy(body_bytes).into_owned();
    let is_json = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("application/json"))
        .unwrap_or(false);
    let body = if is_json {
        serde_json::from_str(&body_raw).unwrap_or(serde_json::Value::String(body_raw.clone()))
    } else {
        serde_json::Value::String(body_raw.clone())
    };
    (body_raw, body)
}

/// Applies delay and fault simulation. Returns `Some(Response)` if a fault fired
/// (caller should short-circuit), or `None` if normal response processing should continue.
async fn apply_simulation(sim: &SimulationConfig) -> Option<Response> {
    if let Some(delay) = &sim.delay {
        tokio::time::sleep(Duration::from_millis(delay.sample_ms())).await;
    }
    if let Some(fault) = &sim.fault {
        if fault.should_trigger() {
            return Some(fault_response(&fault.kind));
        }
    }
    None
}

fn fault_response(kind: &FaultKind) -> Response {
    match kind {
        FaultKind::ConnectionReset => {
            let body =
                Body::from_stream(stream::once(std::future::ready(
                    Err::<Bytes, std::io::Error>(std::io::Error::new(
                        std::io::ErrorKind::ConnectionReset,
                        "simulated connection reset",
                    )),
                )));
            Response::builder()
                .status(200)
                .body(body)
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        FaultKind::EmptyResponse => Response::builder()
            .status(200)
            .body(Body::empty())
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()),
    }
}

fn build_response(
    route_def: &RouteDefinition,
    ctx: &ContextModel,
    renderer: &Renderer,
) -> Response {
    let status_str = renderer.render_or_fallback(&route_def.response.status, ctx, "200");
    let status_code: u16 = status_str.trim().parse().unwrap_or(200);

    let body = if route_def.response.loopback {
        ctx.body_raw.clone()
    } else {
        route_def
            .response
            .content
            .as_deref()
            .map(|t| renderer.render_or_fallback(t, ctx, ""))
            .unwrap_or_default()
    };

    let mut builder = Response::builder().status(status_code);
    for (key, val_template) in &route_def.response.headers {
        let val = renderer.render_or_fallback(val_template, ctx, "");
        builder = builder.header(key.as_str(), val.as_str());
    }

    builder
        .body(Body::from(body))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
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
            simulation: None,
            response: ResponseDefinition {
                status: status.to_string(),
                content: content.map(|s| s.to_string()),
                ..Default::default()
            },
        }
    }

    async fn make_request(state: AppState, uri: &str) {
        router(state)
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
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
                simulation: None,
                response: ResponseDefinition {
                    status: "200".to_string(),
                    loopback: true,
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
                simulation: None,
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
        let call_with_prev = calls.iter().find(|c| c.previous_context.is_some()).unwrap();
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
                simulation: None,
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
                simulation: None,
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

    #[tokio::test]
    async fn simulation_none_has_no_effect() {
        let state = AppState::new();
        state
            .store
            .set_route(make_route("GET", "/plain", Some("ok"), "200"))
            .unwrap();
        let response = router(state)
            .oneshot(
                Request::builder()
                    .uri("/plain")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body_string(response.into_body()).await, "ok");
    }

    #[tokio::test]
    async fn fixed_delay_applies() {
        use crate::models::simulation::{DelayConfig, SimulationConfig};
        use std::time::Instant;

        let state = AppState::new();
        state
            .store
            .set_route(RouteDefinition {
                match_key: MatchKey::new("GET", "/slow"),
                set_state: None,
                simulation: Some(SimulationConfig {
                    delay: Some(DelayConfig::Fixed { ms: 50 }),
                    fault: None,
                }),
                response: ResponseDefinition {
                    status: "200".to_string(),
                    content: Some("ok".to_string()),
                    ..Default::default()
                },
            })
            .unwrap();
        let start = Instant::now();
        router(state)
            .oneshot(Request::builder().uri("/slow").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert!(start.elapsed() >= Duration::from_millis(50));
    }

    #[tokio::test]
    async fn fault_empty_response_returns_200_empty_body() {
        use crate::models::simulation::{FaultConfig, FaultKind, SimulationConfig};

        let state = AppState::new();
        state
            .store
            .set_route(RouteDefinition {
                match_key: MatchKey::new("GET", "/empty-fault"),
                set_state: None,
                simulation: Some(SimulationConfig {
                    delay: None,
                    fault: Some(FaultConfig {
                        kind: FaultKind::EmptyResponse,
                        probability: 1.0,
                    }),
                }),
                response: ResponseDefinition {
                    status: "200".to_string(),
                    content: Some("should not appear".to_string()),
                    ..Default::default()
                },
            })
            .unwrap();
        let response = router(state)
            .oneshot(
                Request::builder()
                    .uri("/empty-fault")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body_string(response.into_body()).await, "");
    }

    #[tokio::test]
    async fn fault_connection_reset_errors_on_body_read() {
        use crate::models::simulation::{FaultConfig, FaultKind, SimulationConfig};

        let state = AppState::new();
        state
            .store
            .set_route(RouteDefinition {
                match_key: MatchKey::new("GET", "/reset-fault"),
                set_state: None,
                simulation: Some(SimulationConfig {
                    delay: None,
                    fault: Some(FaultConfig {
                        kind: FaultKind::ConnectionReset,
                        probability: 1.0,
                    }),
                }),
                response: ResponseDefinition {
                    status: "200".to_string(),
                    content: Some("should not appear".to_string()),
                    ..Default::default()
                },
            })
            .unwrap();
        let response = router(state)
            .oneshot(
                Request::builder()
                    .uri("/reset-fault")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let result = axum::body::to_bytes(response.into_body(), usize::MAX).await;
        assert!(
            result.is_err(),
            "body read should error for connection reset"
        );
    }

    #[tokio::test]
    async fn fault_probability_zero_never_triggers() {
        use crate::models::simulation::{FaultConfig, FaultKind, SimulationConfig};

        let state = AppState::new();
        state
            .store
            .set_route(RouteDefinition {
                match_key: MatchKey::new("GET", "/no-fault"),
                set_state: None,
                simulation: Some(SimulationConfig {
                    delay: None,
                    fault: Some(FaultConfig {
                        kind: FaultKind::EmptyResponse,
                        probability: 0.0,
                    }),
                }),
                response: ResponseDefinition {
                    status: "200".to_string(),
                    content: Some("normal".to_string()),
                    ..Default::default()
                },
            })
            .unwrap();
        for _ in 0..10 {
            let response = router(state.clone())
                .oneshot(
                    Request::builder()
                        .uri("/no-fault")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(body_string(response.into_body()).await, "normal");
        }
    }

    #[tokio::test]
    async fn fault_probability_one_always_triggers() {
        use crate::models::simulation::{FaultConfig, FaultKind, SimulationConfig};

        let state = AppState::new();
        state
            .store
            .set_route(RouteDefinition {
                match_key: MatchKey::new("GET", "/always-fault"),
                set_state: None,
                simulation: Some(SimulationConfig {
                    delay: None,
                    fault: Some(FaultConfig {
                        kind: FaultKind::EmptyResponse,
                        probability: 1.0,
                    }),
                }),
                response: ResponseDefinition {
                    status: "200".to_string(),
                    content: Some("should not appear".to_string()),
                    ..Default::default()
                },
            })
            .unwrap();
        for _ in 0..5 {
            let response = router(state.clone())
                .oneshot(
                    Request::builder()
                        .uri("/always-fault")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(body_string(response.into_body()).await, "");
        }
    }
}
