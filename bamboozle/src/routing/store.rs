use dashmap::DashMap;
use regex::Regex;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

use crate::{
    error::RouteError,
    models::{match_key::MatchKey, route::RouteDefinition},
    routing::regex_gen::{compile_pattern, normalize_url, try_match_route},
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

    pub fn set_route(&self, def: RouteDefinition) -> Result<(), RouteError> {
        let verb = def.match_key.verb.clone();
        let key_str = def.match_key.to_string();
        let normalized = normalize_url(&def.match_key.pattern);

        let compiled = match compile_pattern(&def.match_key.pattern) {
            Ok(c) => c,
            Err(e) => {
                error!(route = %key_str, error = %e, "Failed to compile route pattern");
                return Err(RouteError::AlreadyExists(format!("Invalid pattern '{}': {}", normalized, e)));
            }
        };

        // Ensure the verb bucket exists, then drop the entry ref before borrowing again.
        self.routes.entry(verb.clone()).or_insert_with(DashMap::new);

        // Now borrow the inner map via get() — separate from the entry above.
        let verb_ref = self.routes.get(&verb).unwrap();
        let verb_map = verb_ref.value();

        if verb_map.contains_key(&normalized) {
            warn!(route = %key_str, "Route already exists, skipping");
            return Err(RouteError::AlreadyExists(key_str));
        }

        verb_map.insert(
            normalized.clone(),
            StoredRoute {
                definition: def,
                compiled_regex: compiled,
                normalized_pattern: normalized,
            },
        );

        info!(route = %key_str, "Route set");
        Ok(())
    }

    pub fn delete_route(&self, key: &MatchKey) -> Result<(), RouteError> {
        let key_str = key.to_string();
        let normalized = normalize_url(&key.pattern);

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

    pub fn get_route(&self, key: &MatchKey) -> Option<RouteDefinition> {
        let normalized = normalize_url(&key.pattern);
        let result = self.routes
            .get(&key.verb)
            .and_then(|m| m.get(&normalized).map(|r| r.definition.clone()));

        if result.is_some() {
            debug!(route = %key, "Route retrieved");
        } else {
            debug!(route = %key, "Route not found");
        }
        result
    }

    /// Finds the first stored route whose pattern matches the given URL.
    /// Returns the matched RouteDefinition and the extracted route values.
    pub fn match_route(
        &self,
        verb: &str,
        path: &str,
    ) -> Option<(RouteDefinition, HashMap<String, String>)> {
        let result = self.routes.get(verb).and_then(|verb_map| {
            verb_map.iter().find_map(|entry| {
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
