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
    constraint_specificity: usize,
}

/// Counts parameters in `pattern` that carry a specific constraint (anything other
/// than `:string` or untyped), so the router can prefer e.g. `{id:int}` over
/// `{id:string}` when both patterns are otherwise equally ranked.
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
            let constraint = &inner[colon + 1..];
            if constraint != "string" {
                count += 1;
            }
        }
        remaining = &remaining[start + end + 1..];
    }
    count
}

pub struct RouteStore {
    // outer key = uppercase HTTP verb ("GET"), inner key = fully-lowercase normalized pattern
    routes: DashMap<String, DashMap<String, StoredRoute>>,
}

impl Default for RouteStore {
    fn default() -> Self {
        Self::new()
    }
}

impl RouteStore {
    pub fn new() -> Self {
        Self {
            routes: DashMap::new(),
        }
    }

    pub fn set_route(&self, mut def: RouteDefinition) -> Result<RouteDefinition, AppError> {
        let verb = def.match_key.verb.trim().to_ascii_uppercase();
        // `pattern` preserves case inside {braces} so param names match Liquid template
        // access (e.g. `{{routeValues.thingName}}`), but lowercases literal segments.
        // `lookup_key` is fully lowercase for case-insensitive deduplication in the map.
        let pattern = MatchKey::normalize_pattern(&def.match_key.pattern);
        let lookup_key = pattern.to_ascii_lowercase();
        def.match_key.verb = verb.clone();
        def.match_key.pattern = pattern.clone();
        let key_str = def.match_key.to_string();
        let compiled = match compile_pattern(&pattern) {
            Ok(c) => c,
            Err(e) => {
                error!(route = %key_str, error = %e, "Failed to compile route pattern");
                return Err(AppError::BadRequest(format!(
                    "Invalid pattern '{}': {}",
                    pattern, e
                )));
            }
        };

        let verb_entry = self.routes.entry(verb.clone()).or_default();
        let verb_map = verb_entry.value();

        if verb_map.contains_key(&lookup_key) {
            warn!(route = %key_str, "Route already exists, skipping");
            return Err(AppError::AlreadyExists(key_str));
        }

        let specificity = constraint_specificity(&pattern);
        verb_map.insert(
            lookup_key,
            StoredRoute {
                definition: def.clone(),
                compiled_regex: compiled,
                constraint_specificity: specificity,
                normalized_pattern: pattern,
            },
        );

        info!(route = %key_str, "Route set");
        Ok(def)
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
                let a_params = a.value().normalized_pattern.matches('{').count();
                let b_params = b.value().normalized_pattern.matches('{').count();
                a_params.cmp(&b_params)
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
                outer.value().iter().map(move |inner| {
                    let pattern = &inner.value().normalized_pattern;
                    let mut score = jaro_winkler(normalized.as_str(), pattern.as_str());
                    if stored_verb == verb_upper {
                        score += 0.05;
                    }
                    (score, format!("{}|{}", stored_verb, pattern))
                }).collect::<Vec<_>>()
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
        info!("Route store cleared");
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
            response: ResponseDefinition::default(),
        }
    }

    #[test]
    fn set_route_and_match_it() {
        let store = RouteStore::new();
        store.set_route(make_route("GET", "/api/users")).unwrap();
        assert!(store.match_route("GET", "/api/users").is_some());
    }

    #[test]
    fn duplicate_set_route_returns_already_exists() {
        let store = RouteStore::new();
        store.set_route(make_route("GET", "/api/users")).unwrap();
        let err = store
            .set_route(make_route("GET", "/api/users"))
            .unwrap_err();
        assert!(matches!(err, AppError::AlreadyExists(_)));
    }

    #[test]
    fn delete_route_removes_it() {
        let store = RouteStore::new();
        store
            .set_route(make_route("DELETE", "/items/{id}"))
            .unwrap();
        let key = MatchKey::new("DELETE", "/items/{id}");
        store.delete_route(&key).unwrap();
        assert!(store.match_route("DELETE", "/items/{id}").is_none());
    }

    #[test]
    fn delete_nonexistent_route_returns_not_found() {
        let store = RouteStore::new();
        let err = store
            .delete_route(&MatchKey::new("GET", "/missing"))
            .unwrap_err();
        assert!(matches!(err, RouteError::NotFound(_)));
    }

    #[test]
    fn match_static_route() {
        let store = RouteStore::new();
        store.set_route(make_route("GET", "/hello")).unwrap();
        let (def, values) = store.match_route("GET", "/hello").unwrap();
        assert_eq!(def.match_key.pattern, "hello");
        assert!(values.is_empty());
    }

    #[test]
    fn match_parameterized_route_extracts_values() {
        let store = RouteStore::new();
        store.set_route(make_route("GET", "/users/{id}")).unwrap();
        let (_, values) = store.match_route("GET", "/users/42").unwrap();
        assert_eq!(values["id"], "42");
    }

    #[test]
    fn match_route_returns_none_for_no_match() {
        let store = RouteStore::new();
        store.set_route(make_route("GET", "/api/users")).unwrap();
        assert!(store.match_route("GET", "/api/orders").is_none());
        assert!(store.match_route("POST", "/api/users").is_none());
    }

    #[test]
    fn get_all_routes_returns_all() {
        let store = RouteStore::new();
        store.set_route(make_route("GET", "/a")).unwrap();
        store.set_route(make_route("POST", "/b")).unwrap();
        assert_eq!(store.get_all_routes().len(), 2);
    }

    #[test]
    fn reset_clears_all_routes() {
        let store = RouteStore::new();
        store.set_route(make_route("GET", "/a")).unwrap();
        store.reset();
        assert!(store.get_all_routes().is_empty());
        assert!(store.match_route("GET", "/a").is_none());
    }

    #[test]
    fn typed_int_param_wins_over_string_when_value_is_numeric() {
        let store = RouteStore::new();
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
        let store = RouteStore::new();
        store
            .set_route(make_route("GET", "/items/{id:int}"))
            .unwrap();
        assert!(store.match_route("GET", "/items/abc").is_none());
    }

    #[test]
    fn suggest_routes_returns_similar_route() {
        let store = RouteStore::new();
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
        let store = RouteStore::new();
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
        let store = RouteStore::new();
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
        let store = RouteStore::new();
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
        let store = RouteStore::new();
        let suggestions = store.suggest_routes("GET", "/api/users");
        assert!(suggestions.is_empty());
    }
}
