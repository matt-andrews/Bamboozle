use dashmap::DashMap;
use regex::Regex;
use std::collections::HashMap;

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

        let compiled = compile_pattern(&def.match_key.pattern)
            .map_err(|e| RouteError::AlreadyExists(format!("Invalid pattern '{}': {}", normalized, e)))?;

        // Ensure the verb bucket exists, then drop the entry ref before borrowing again.
        self.routes.entry(verb.clone()).or_insert_with(DashMap::new);

        // Now borrow the inner map via get() — separate from the entry above.
        let verb_ref = self.routes.get(&verb).unwrap();
        let verb_map = verb_ref.value();

        if verb_map.contains_key(&normalized) {
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

        Ok(())
    }

    pub fn delete_route(&self, key: &MatchKey) -> Result<(), RouteError> {
        let normalized = normalize_url(&key.pattern);
        let verb_map = self
            .routes
            .get(&key.verb)
            .ok_or_else(|| RouteError::NotFound(key.to_string()))?;

        if verb_map.remove(&normalized).is_none() {
            return Err(RouteError::NotFound(key.to_string()));
        }
        Ok(())
    }

    pub fn get_route(&self, key: &MatchKey) -> Option<RouteDefinition> {
        let normalized = normalize_url(&key.pattern);
        self.routes
            .get(&key.verb)?
            .get(&normalized)
            .map(|r| r.definition.clone())
    }

    /// Finds the first stored route whose pattern matches the given URL.
    /// Returns the matched RouteDefinition and the extracted route values.
    pub fn match_route(
        &self,
        verb: &str,
        path: &str,
    ) -> Option<(RouteDefinition, HashMap<String, String>)> {
        let verb_map = self.routes.get(verb)?;

        for entry in verb_map.iter() {
            let stored = entry.value();
            if let Some(route_values) =
                try_match_route(&stored.compiled_regex, &stored.normalized_pattern, path)
            {
                return Some((stored.definition.clone(), route_values));
            }
        }
        None
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
    }
}
