use dashmap::DashMap;
use regex::Regex;
use std::collections::HashMap;
use strsim::jaro_winkler;
use tracing::{debug, error, info, warn};

const SUGGESTION_LIMIT: usize = 3;
const SUGGESTION_THRESHOLD: f64 = 0.6;

use crate::{
    error::{AppError, RouteError},
    models::{match_key::MatchKey, route::RouteDefinition},
    routing::regex_gen::{compile_pattern, try_match_route},
};

struct StoredRoute {
    definition: RouteDefinition,
    compiled_regex: Regex,
    normalized_pattern: String,
    param_count: usize,
    constraint_specificity: usize,
}

fn is_narrowing_constraint(constraint: &str) -> bool {
    matches!(
        constraint,
        "int" | "long" | "double" | "decimal" | "float" | "bool" | "guid" | "alpha" | "datetime"
    )
}

/// Counts parameters in `pattern` that carry a recognized narrowing constraint
/// (matching what `regex_gen::constraint_pattern` actually restricts), so the
/// router can prefer e.g. `{id:int}` over `{id:string}` when both patterns are
/// otherwise equally ranked. Unknown constraints fall back to `[^/]+` just like
/// `:string`, so they are not counted.
fn constraint_specificity(pattern: &str) -> usize {
    let mut count = 0;
    let mut remaining = pattern;
    while let Some(start) = remaining.find('{') {
        let rest = &remaining[start..];
        let end = match rest.find('}') {
            Some(i) => i,
            None => break,
        };
        let inner = rest[1..end].trim_end_matches('?');
        if let Some(colon) = inner.find(':') {
            let constraint = inner[colon + 1..].to_ascii_lowercase();
            if is_narrowing_constraint(&constraint) {
                count += 1;
            }
        }
        remaining = &remaining[start + end + 1..];
    }
    count
}

/// Counts `{param}` tokens in `pattern`, used to sort static routes before
/// parameterized ones. Cached in `StoredRoute` to avoid re-scanning on every
/// sort during `match_route`.
fn param_count(pattern: &str) -> usize {
    pattern.matches('{').count()
}

pub struct RouteStore {
    // outer key = uppercase HTTP verb ("GET"), inner key = fully-lowercase normalized pattern
    routes: DashMap<String, DashMap<String, StoredRoute>>,
    max_routes: usize,
    max_content_size: usize,
    route_count: std::sync::atomic::AtomicUsize,
}

impl Default for RouteStore {
    fn default() -> Self {
        Self::new(1000, 10 * 1024 * 1024)
    }
}

impl RouteStore {
    pub fn new(max_routes: usize, max_content_size: usize) -> Self {
        Self {
            routes: DashMap::new(),
            max_routes,
            max_content_size,
            route_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    pub fn set_route(&self, def: RouteDefinition) -> Result<Vec<RouteDefinition>, AppError> {
        let body_strategy_count = [
            def.response.content.is_some(),
            def.response.content_file.is_some(),
            def.response.binary_file.is_some(),
            def.response.loopback,
        ]
        .iter()
        .filter(|&&x| x)
        .count();
        if body_strategy_count > 1 {
            return Err(AppError::BadRequest(
                "Only one of 'content', 'contentFile', 'binaryFile', or 'loopback' may be specified".to_string(),
            ));
        }

        if let Some(content) = &def.response.content {
            if content.len() > self.max_content_size {
                return Err(AppError::BadRequest(format!(
                    "Content size {} bytes exceeds maximum allowed {} bytes",
                    content.len(),
                    self.max_content_size
                )));
            }
        }

        if def.max_calls == Some(0) {
            return Err(AppError::BadRequest(
                "'maxCalls' must be greater than 0".to_string(),
            ));
        }

        let verbs = Self::parse_http_verbs(&def.match_key.verb)?;

        // Compile pattern once — identical for every verb in the list.
        // `pattern` preserves case inside {braces} so param names match Liquid template
        // access (e.g. `{{routeValues.thingName}}`), but lowercases literal segments.
        // `lookup_key` is fully lowercase for case-insensitive deduplication in the map.
        let pattern = MatchKey::normalize_pattern(&def.match_key.pattern);
        let lookup_key = pattern.to_ascii_lowercase();
        let compiled = match compile_pattern(&pattern) {
            Ok(c) => c,
            Err(e) => {
                error!(pattern = %pattern, error = %e, "Failed to compile route pattern");
                return Err(AppError::BadRequest(format!(
                    "Invalid pattern '{}': {}",
                    pattern, e
                )));
            }
        };
        let specificity = constraint_specificity(&pattern);
        let params = param_count(&pattern);

        // Phase 1: validate all preconditions before any writes so a failure
        // mid-list cannot leave the store partially modified.
        if self.route_count.load(std::sync::atomic::Ordering::Relaxed) + verbs.len()
            > self.max_routes
        {
            return Err(AppError::BadRequest(format!(
                "Maximum number of routes ({}) reached",
                self.max_routes
            )));
        }
        for verb in &verbs {
            if self
                .routes
                .get(verb)
                .is_some_and(|m| m.contains_key(&lookup_key))
            {
                let key_str = format!("{}|{}", verb, lookup_key);
                warn!(route = %key_str, "Route already exists, skipping");
                return Err(AppError::AlreadyExists(key_str));
            }
        }

        // Phase 2: all insertions — preconditions satisfied, no early returns below.
        let mut results = Vec::with_capacity(verbs.len());
        for verb in verbs {
            let mut working = def.clone();
            working.match_key.verb = verb.clone();
            working.match_key.pattern = pattern.clone();
            let key_str = working.match_key.to_string();

            let verb_entry = self.routes.entry(verb).or_default();
            verb_entry.value().insert(
                lookup_key.clone(),
                StoredRoute {
                    definition: working.clone(),
                    compiled_regex: compiled.clone(),
                    param_count: params,
                    constraint_specificity: specificity,
                    normalized_pattern: pattern.clone(),
                },
            );
            self.route_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            info!(route = %key_str, "Route set");
            results.push(working);
        }
        Ok(results)
    }

    pub fn delete_route(&self, key: &MatchKey) -> Result<(), RouteError> {
        let verb = key.verb.trim().to_ascii_uppercase();
        let lookup_key = MatchKey::normalize_pattern(&key.pattern).to_ascii_lowercase();
        let key_str = format!("{}|{}", verb, lookup_key);

        let verb_map = match self.routes.get(&verb) {
            Some(m) => m,
            None => {
                warn!(route = %key_str, "Route not found for deletion");
                return Err(RouteError::NotFound(key_str));
            }
        };

        if verb_map.remove(&lookup_key).is_none() {
            warn!(route = %key_str, "Route not found for deletion");
            return Err(RouteError::NotFound(key_str));
        }

        self.route_count
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        info!(route = %key_str, "Route deleted");
        Ok(())
    }

    /// Finds the first stored route whose pattern matches the given URL.
    /// Returns the matched RouteDefinition and the extracted route values.
    pub fn match_route(
        &self,
        verb: &str,
        path: &str,
    ) -> Option<(RouteDefinition, HashMap<String, String>)> {
        let result = self.routes.get(verb).and_then(|verb_map| {
            let mut entries: Vec<_> = verb_map.iter().collect();
            // Static routes before parameterized; among equal param counts, more
            // specific constraints (e.g. :int) before catch-all ones (e.g. :string);
            // then longer patterns before shorter.
            entries.sort_unstable_by(|a, b| {
                a.value()
                    .param_count
                    .cmp(&b.value().param_count)
                    .then_with(|| {
                        b.value()
                            .constraint_specificity
                            .cmp(&a.value().constraint_specificity)
                    })
                    .then_with(|| {
                        b.value()
                            .normalized_pattern
                            .len()
                            .cmp(&a.value().normalized_pattern.len())
                    })
            });
            entries.iter().find_map(|entry| {
                let stored = entry.value();
                try_match_route(&stored.compiled_regex, &stored.normalized_pattern, path)
                    .map(|route_values| (stored.definition.clone(), route_values))
            })
        });

        match &result {
            Some((def, _)) => debug!(verb, path, route = %def.match_key, "Request matched route"),
            None => debug!(verb, path, "No route matched request"),
        }
        result
    }

    /// Returns up to `SUGGESTION_LIMIT` route labels (e.g. "GET|api/users/{id}") whose
    /// patterns are most similar to the requested path, sorted by similarity descending.
    pub fn suggest_routes(&self, verb: &str, path: &str) -> Vec<String> {
        let normalized = MatchKey::normalize_pattern(path);
        let verb_upper = verb.to_ascii_uppercase();

        let mut scored: Vec<(f64, String)> = self
            .routes
            .iter()
            .flat_map(|outer| {
                let stored_verb = outer.key().clone();
                let normalized = normalized.clone();
                let verb_upper = verb_upper.clone();
                outer
                    .value()
                    .iter()
                    .map(move |inner| {
                        let pattern = &inner.value().normalized_pattern;
                        let mut score = jaro_winkler(normalized.as_str(), pattern.as_str());
                        if stored_verb == verb_upper {
                            score += 0.05;
                        }
                        (score, format!("{}|{}", stored_verb, pattern))
                    })
                    .collect::<Vec<_>>()
            })
            .filter(|(score, _)| *score >= SUGGESTION_THRESHOLD)
            .collect();

        scored.sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(SUGGESTION_LIMIT);
        scored.into_iter().map(|(_, label)| label).collect()
    }

    pub fn get_all_routes(&self) -> Vec<RouteDefinition> {
        self.routes
            .iter()
            .flat_map(|verb_entry| {
                verb_entry
                    .value()
                    .iter()
                    .map(|e| e.value().definition.clone())
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    pub fn reset(&self) {
        self.routes.clear();
        self.route_count
            .store(0, std::sync::atomic::Ordering::Relaxed);
        info!("Route store cleared");
    }

    fn parse_http_verbs(input: &str) -> Result<Vec<String>, AppError> {
        const VALID_VERBS: &[&str] = &[
            "GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS", "CONNECT", "TRACE",
        ];

        let verbs: Vec<String> = input
            .split(',')
            .map(|v| v.trim().to_ascii_uppercase())
            .collect();

        let mut seen = std::collections::HashSet::new();
        for verb in &verbs {
            if !VALID_VERBS.contains(&verb.as_str()) {
                return Err(AppError::BadRequest(format!(
                    "'{}' is not a valid HTTP verb",
                    verb
                )));
            }
            if !seen.insert(verb.as_str()) {
                return Err(AppError::BadRequest(format!(
                    "Duplicate verb '{}' in verb list",
                    verb
                )));
            }
        }

        Ok(verbs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        error::RouteError,
        models::{
            match_key::MatchKey,
            route::{ResponseDefinition, RouteDefinition},
        },
    };

    fn make_route(verb: &str, pattern: &str) -> RouteDefinition {
        RouteDefinition {
            match_key: MatchKey::new(verb, pattern),
            set_state: None,
            simulation: None,
            max_calls: None,
            response: ResponseDefinition::default(),
        }
    }

    #[test]
    fn set_route_and_match_it() {
        let store = RouteStore::default();
        store.set_route(make_route("GET", "/api/users")).unwrap();
        assert!(store.match_route("GET", "/api/users").is_some());
    }

    #[test]
    fn set_routes_and_match_it() {
        let store = RouteStore::default();
        store
            .set_route(make_route("GET,POST", "/api/users"))
            .unwrap();
        assert!(store.match_route("GET", "/api/users").is_some());
        assert!(store.match_route("POST", "/api/users").is_some());
    }

    #[test]
    fn malformed_verb_throws() {
        let store = RouteStore::default();
        let err = store
            .set_route(make_route("GET,PfOST", "/api/users"))
            .unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn duplicate_set_route_returns_already_exists() {
        let store = RouteStore::default();
        store.set_route(make_route("GET", "/api/users")).unwrap();
        let err = store
            .set_route(make_route("GET", "/api/users"))
            .unwrap_err();
        assert!(matches!(err, AppError::AlreadyExists(_)));
    }

    #[test]
    fn delete_route_removes_it() {
        let store = RouteStore::default();
        store
            .set_route(make_route("DELETE", "/items/{id}"))
            .unwrap();
        let key = MatchKey::new("DELETE", "/items/{id}");
        store.delete_route(&key).unwrap();
        assert!(store.match_route("DELETE", "/items/{id}").is_none());
    }

    #[test]
    fn delete_nonexistent_route_returns_not_found() {
        let store = RouteStore::default();
        let err = store
            .delete_route(&MatchKey::new("GET", "/missing"))
            .unwrap_err();
        assert!(matches!(err, RouteError::NotFound(_)));
    }

    #[test]
    fn match_static_route() {
        let store = RouteStore::default();
        store.set_route(make_route("GET", "/hello")).unwrap();
        let (def, values) = store.match_route("GET", "/hello").unwrap();
        assert_eq!(def.match_key.pattern, "hello");
        assert!(values.is_empty());
    }

    #[test]
    fn match_parameterized_route_extracts_values() {
        let store = RouteStore::default();
        store.set_route(make_route("GET", "/users/{id}")).unwrap();
        let (_, values) = store.match_route("GET", "/users/42").unwrap();
        assert_eq!(values["id"], "42");
    }

    #[test]
    fn match_route_returns_none_for_no_match() {
        let store = RouteStore::default();
        store.set_route(make_route("GET", "/api/users")).unwrap();
        assert!(store.match_route("GET", "/api/orders").is_none());
        assert!(store.match_route("POST", "/api/users").is_none());
    }

    #[test]
    fn get_all_routes_returns_all() {
        let store = RouteStore::default();
        store.set_route(make_route("GET", "/a")).unwrap();
        store.set_route(make_route("POST", "/b")).unwrap();
        assert_eq!(store.get_all_routes().len(), 2);
    }

    #[test]
    fn reset_clears_all_routes() {
        let store = RouteStore::default();
        store.set_route(make_route("GET", "/a")).unwrap();
        store.reset();
        assert!(store.get_all_routes().is_empty());
        assert!(store.match_route("GET", "/a").is_none());
    }

    #[test]
    fn typed_int_param_wins_over_string_when_value_is_numeric() {
        let store = RouteStore::default();
        store
            .set_route(make_route("GET", "/items/{id:int}"))
            .unwrap();
        store
            .set_route(make_route("GET", "/items/{id:string}"))
            .unwrap();

        let (int_def, _) = store.match_route("GET", "/items/42").unwrap();
        assert!(
            int_def.match_key.pattern.contains(":int"),
            "expected :int route to win for numeric value, got {:?}",
            int_def.match_key.pattern
        );

        let (str_def, _) = store.match_route("GET", "/items/hello").unwrap();
        assert!(
            str_def.match_key.pattern.contains(":string"),
            "expected :string route to win for non-numeric value, got {:?}",
            str_def.match_key.pattern
        );
    }

    #[test]
    fn typed_int_param_rejects_non_numeric_with_no_string_fallback() {
        let store = RouteStore::default();
        store
            .set_route(make_route("GET", "/items/{id:int}"))
            .unwrap();
        assert!(store.match_route("GET", "/items/abc").is_none());
    }

    #[test]
    fn suggest_routes_returns_similar_route() {
        let store = RouteStore::default();
        store.set_route(make_route("GET", "/api/users")).unwrap();
        store.set_route(make_route("GET", "/api/orders")).unwrap();
        let suggestions = store.suggest_routes("GET", "/api/users");
        assert!(!suggestions.is_empty(), "expected at least one suggestion");
        assert!(
            suggestions[0].contains("api/users"),
            "closest match should be api/users, got {:?}",
            suggestions
        );
    }

    #[test]
    fn suggest_routes_boosts_same_verb() {
        let store = RouteStore::default();
        store.set_route(make_route("GET", "/api/users")).unwrap();
        store.set_route(make_route("POST", "/api/users")).unwrap();
        // Same path requested with GET — GET|api/users should rank first due to verb boost.
        let suggestions = store.suggest_routes("GET", "/api/users");
        assert!(!suggestions.is_empty());
        assert!(
            suggestions[0].starts_with("GET|"),
            "GET route should rank above POST due to verb boost, got {:?}",
            suggestions
        );
    }

    #[test]
    fn suggest_routes_returns_empty_when_no_similar_routes() {
        let store = RouteStore::default();
        store
            .set_route(make_route("GET", "/completely/different/path"))
            .unwrap();
        // A totally unrelated short path should fall below the similarity threshold.
        let suggestions = store.suggest_routes("POST", "/xyz");
        assert!(
            suggestions.is_empty(),
            "expected no suggestions, got {:?}",
            suggestions
        );
    }

    #[test]
    fn suggest_routes_caps_results_at_limit() {
        let store = RouteStore::default();
        store.set_route(make_route("GET", "/api/users")).unwrap();
        store.set_route(make_route("GET", "/api/user")).unwrap();
        store
            .set_route(make_route("GET", "/api/users/list"))
            .unwrap();
        store
            .set_route(make_route("GET", "/api/users/search"))
            .unwrap();
        let suggestions = store.suggest_routes("GET", "/api/users");
        assert!(
            suggestions.len() <= SUGGESTION_LIMIT,
            "expected at most {} suggestions, got {}",
            SUGGESTION_LIMIT,
            suggestions.len()
        );
    }

    #[test]
    fn suggest_routes_returns_empty_when_store_is_empty() {
        let store = RouteStore::default();
        let suggestions = store.suggest_routes("GET", "/api/users");
        assert!(suggestions.is_empty());
    }

    fn make_route_with_response(
        verb: &str,
        pattern: &str,
        response: ResponseDefinition,
    ) -> RouteDefinition {
        RouteDefinition {
            match_key: MatchKey::new(verb, pattern),
            set_state: None,
            simulation: None,
            max_calls: None,
            response,
        }
    }

    #[test]
    fn single_content_is_accepted() {
        let store = RouteStore::default();
        let result = store.set_route(make_route_with_response(
            "GET",
            "/a",
            ResponseDefinition {
                content: Some("hello".to_string()),
                ..Default::default()
            },
        ));
        assert!(result.is_ok());
    }

    #[test]
    fn single_content_file_is_accepted() {
        let store = RouteStore::default();
        let result = store.set_route(make_route_with_response(
            "GET",
            "/b",
            ResponseDefinition {
                content_file: Some("/some/file.txt".to_string()),
                ..Default::default()
            },
        ));
        assert!(result.is_ok());
    }

    #[test]
    fn single_binary_file_is_accepted() {
        let store = RouteStore::default();
        let result = store.set_route(make_route_with_response(
            "GET",
            "/c",
            ResponseDefinition {
                binary_file: Some("/some/file.bin".to_string()),
                ..Default::default()
            },
        ));
        assert!(result.is_ok());
    }

    #[test]
    fn single_loopback_is_accepted() {
        let store = RouteStore::default();
        let result = store.set_route(make_route_with_response(
            "GET",
            "/d",
            ResponseDefinition {
                loopback: true,
                ..Default::default()
            },
        ));
        assert!(result.is_ok());
    }

    #[test]
    fn content_and_content_file_together_is_rejected() {
        let store = RouteStore::default();
        let err = store
            .set_route(make_route_with_response(
                "GET",
                "/e",
                ResponseDefinition {
                    content: Some("hello".to_string()),
                    content_file: Some("/some/file.txt".to_string()),
                    ..Default::default()
                },
            ))
            .unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn content_and_binary_file_together_is_rejected() {
        let store = RouteStore::default();
        let err = store
            .set_route(make_route_with_response(
                "GET",
                "/f",
                ResponseDefinition {
                    content: Some("hello".to_string()),
                    binary_file: Some("/some/file.bin".to_string()),
                    ..Default::default()
                },
            ))
            .unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn content_file_and_binary_file_together_is_rejected() {
        let store = RouteStore::default();
        let err = store
            .set_route(make_route_with_response(
                "GET",
                "/g",
                ResponseDefinition {
                    content_file: Some("/some/file.txt".to_string()),
                    binary_file: Some("/some/file.bin".to_string()),
                    ..Default::default()
                },
            ))
            .unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn loopback_and_content_together_is_rejected() {
        let store = RouteStore::default();
        let err = store
            .set_route(make_route_with_response(
                "GET",
                "/h",
                ResponseDefinition {
                    loopback: true,
                    content: Some("hello".to_string()),
                    ..Default::default()
                },
            ))
            .unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn loopback_and_content_file_together_is_rejected() {
        let store = RouteStore::default();
        let err = store
            .set_route(make_route_with_response(
                "GET",
                "/i",
                ResponseDefinition {
                    loopback: true,
                    content_file: Some("/some/file.txt".to_string()),
                    ..Default::default()
                },
            ))
            .unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn max_routes_limit_is_enforced() {
        let store = RouteStore::new(1, 10 * 1024 * 1024);
        store.set_route(make_route("GET", "/first")).unwrap();
        let err = store.set_route(make_route("GET", "/second")).unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[test]
    fn max_content_size_limit_is_enforced() {
        let store = RouteStore::new(1000, 5);
        let err = store
            .set_route(make_route_with_response(
                "GET",
                "/too-big",
                ResponseDefinition {
                    content: Some("123456".to_string()),
                    ..Default::default()
                },
            ))
            .unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));

        let ok = store.set_route(make_route_with_response(
            "GET",
            "/just-right",
            ResponseDefinition {
                content: Some("12345".to_string()),
                ..Default::default()
            },
        ));
        assert!(ok.is_ok());
    }
}
