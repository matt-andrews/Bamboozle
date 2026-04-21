use dashmap::DashMap;
use regex::Regex;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

use crate::{
    error::{AppError, RouteError},
    models::{match_key::MatchKey, route::RouteDefinition},
    routing::regex_gen::{compile_pattern, try_match_route},
};

struct StoredRoute {
    definition: RouteDefinition,
    compiled_regex: Regex,
    normalized_pattern: String,
}

pub struct RouteStore {
    // outer key = HTTP verb ("GET"), inner key = normalized pattern string
    routes: DashMap<String, DashMap<String, StoredRoute>>,
}

impl RouteStore {
    pub fn new() -> Self {
        Self {
            routes: DashMap::new(),
        }
    }

    pub fn set_route(&self, mut def: RouteDefinition) -> Result<RouteDefinition, AppError> {
        let verb = def.match_key.verb.trim().to_ascii_uppercase();
        // Normalise: lowercase literal segments but preserve case inside {braces} so
        // param names match Liquid template access ({{routeValues.thingName}}).
        // The storage key is fully lowercase for case-insensitive deduplication.
        let normalized = MatchKey::normalize_pattern(&def.match_key.pattern);
        let storage_key = normalized.to_ascii_lowercase();
        def.match_key.verb = verb.clone();
        def.match_key.pattern = normalized.clone();
        let key_str = def.match_key.to_string();
        let compiled = match compile_pattern(&normalized) {
            Ok(c) => c,
            Err(e) => {
                error!(route = %key_str, error = %e, "Failed to compile route pattern");
                return Err(AppError::BadRequest(format!(
                    "Invalid pattern '{}': {}",
                    normalized, e
                )));
            }
        };

        let verb_entry = self.routes.entry(verb.clone()).or_default();
        let verb_map = verb_entry.value();

        if verb_map.contains_key(&storage_key) {
            warn!(route = %key_str, "Route already exists, skipping");
            return Err(AppError::AlreadyExists(key_str));
        }

        verb_map.insert(
            storage_key,
            StoredRoute {
                definition: def.clone(),
                compiled_regex: compiled,
                normalized_pattern: normalized,
            },
        );

        info!(route = %key_str, "Route set");
        Ok(def)
    }

    pub fn delete_route(&self, key: &MatchKey) -> Result<(), RouteError> {
        let key_str = key.to_string();
        let normalized = MatchKey::normalize_pattern(&key.pattern).to_ascii_lowercase();

        let verb_map = match self.routes.get(&key.verb) {
            Some(m) => m,
            None => {
                warn!(route = %key_str, "Route not found for deletion");
                return Err(RouteError::NotFound(key_str));
            }
        };

        if verb_map.remove(&normalized).is_none() {
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
            // Static routes before parameterized; longer patterns before shorter.
            entries.sort_by(|a, b| {
                let a_params = a.value().normalized_pattern.matches('{').count();
                let b_params = b.value().normalized_pattern.matches('{').count();
                a_params.cmp(&b_params).then_with(|| {
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
}
