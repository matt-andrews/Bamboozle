use dashmap::DashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::models::{context::ContextModel, match_key::MatchKey};

pub struct CallTracker {
    matched: DashMap<Uuid, ContextModel>,
    unmatched: DashMap<Uuid, ContextModel>,
    last_matched: DashMap<MatchKey, ContextModel>,
}

impl Default for CallTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl CallTracker {
    pub fn new() -> Self {
        Self {
            matched: DashMap::new(),
            unmatched: DashMap::new(),
            last_matched: DashMap::new(),
        }
    }

    pub fn record_matched(&self, ctx: ContextModel) {
        debug!(route = %ctx.route_model.match_key, "Recorded matched call");
        self.last_matched
            .insert(ctx.route_model.match_key.clone(), ctx.clone());
        self.matched.insert(Uuid::new_v4(), ctx);
    }

    pub fn get_last_matched_for_route(&self, key: &MatchKey) -> Option<Box<ContextModel>> {
        self.last_matched.get(key).map(|entry| {
            let mut ctx = entry.value().clone();
            ctx.previous_context = None;
            Box::new(ctx)
        })
    }

    pub fn record_unmatched(&self, ctx: ContextModel) {
        warn!(route = %ctx.route_model.match_key, "Recorded unmatched call");
        self.unmatched.insert(Uuid::new_v4(), ctx);
    }

    pub fn get_calls_for_route(&self, key: &MatchKey) -> Vec<ContextModel> {
        let calls: Vec<ContextModel> = self
            .matched
            .iter()
            .filter(|entry| entry.value().route_model.match_key == *key)
            .map(|entry| entry.value().clone())
            .collect();
        debug!(route = %key, count = calls.len(), "Retrieved calls for route");
        calls
    }

    pub fn delete_calls_for_route(&self, key: &MatchKey) {
        self.matched
            .retain(|_, ctx| ctx.route_model.match_key != *key);
        self.last_matched.remove(key);
        info!(route = %key, "Deleted calls for route");
    }

    /// Returns the MatchKey for each unmatched request (mirrors C# GetUnmatchedRouteCalls).
    pub fn get_unmatched(&self) -> Vec<MatchKey> {
        let keys: Vec<MatchKey> = self
            .unmatched
            .iter()
            .map(|entry| entry.value().route_model.match_key.clone())
            .collect();
        debug!(count = keys.len(), "Retrieved unmatched calls");
        keys
    }

    pub fn reset(&self) {
        self.matched.clear();
        self.unmatched.clear();
        self.last_matched.clear();
        info!("Call tracker cleared");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        context::ContextModel,
        match_key::MatchKey,
        route::{ResponseDefinition, RouteDefinition},
    };
    use std::collections::HashMap;

    fn make_ctx(verb: &str, pattern: &str) -> ContextModel {
        ContextModel {
            query_params: HashMap::new(),
            headers: HashMap::new(),
            route_values: HashMap::new(),
            body: serde_json::Value::Null,
            body_raw: String::new(),
            state: String::new(),
            route_model: RouteDefinition {
                match_key: MatchKey::new(verb, pattern),
                set_state: None,
                simulation: None,
                max_calls: None,
                response: ResponseDefinition::default(),
            },
            previous_context: None,
        }
    }

    #[test]
    fn new_tracker_has_no_calls() {
        let tracker = CallTracker::new();
        let key = MatchKey::new("GET", "/test");
        assert!(tracker.get_calls_for_route(&key).is_empty());
        assert!(tracker.get_unmatched().is_empty());
    }

    #[test]
    fn record_matched_and_retrieve() {
        let tracker = CallTracker::new();
        tracker.record_matched(make_ctx("GET", "/api/users"));
        let key = MatchKey::new("GET", "/api/users");
        let calls = tracker.get_calls_for_route(&key);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].route_model.match_key.verb, "GET");
    }

    #[test]
    fn record_unmatched_and_retrieve() {
        let tracker = CallTracker::new();
        tracker.record_unmatched(make_ctx("POST", "/unknown"));
        let keys = tracker.get_unmatched();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].pattern, "unknown");
    }

    #[test]
    fn get_calls_filters_by_route() {
        let tracker = CallTracker::new();
        tracker.record_matched(make_ctx("GET", "/a"));
        tracker.record_matched(make_ctx("GET", "/a"));
        tracker.record_matched(make_ctx("GET", "/b"));
        assert_eq!(
            tracker
                .get_calls_for_route(&MatchKey::new("GET", "/a"))
                .len(),
            2
        );
        assert_eq!(
            tracker
                .get_calls_for_route(&MatchKey::new("GET", "/b"))
                .len(),
            1
        );
    }

    #[test]
    fn delete_calls_removes_only_matching_route() {
        let tracker = CallTracker::new();
        tracker.record_matched(make_ctx("GET", "/a"));
        tracker.record_matched(make_ctx("GET", "/b"));
        tracker.delete_calls_for_route(&MatchKey::new("GET", "/a"));
        assert!(tracker
            .get_calls_for_route(&MatchKey::new("GET", "/a"))
            .is_empty());
        assert_eq!(
            tracker
                .get_calls_for_route(&MatchKey::new("GET", "/b"))
                .len(),
            1
        );
    }

    #[test]
    fn reset_clears_all_calls() {
        let tracker = CallTracker::new();
        tracker.record_matched(make_ctx("GET", "/a"));
        tracker.record_unmatched(make_ctx("POST", "/missing"));
        tracker.reset();
        assert!(tracker
            .get_calls_for_route(&MatchKey::new("GET", "/a"))
            .is_empty());
        assert!(tracker.get_unmatched().is_empty());
    }

    #[test]
    fn get_last_matched_returns_none_when_no_calls() {
        let tracker = CallTracker::new();
        assert!(tracker
            .get_last_matched_for_route(&MatchKey::new("GET", "/a"))
            .is_none());
    }

    #[test]
    fn get_last_matched_returns_context_after_first_call() {
        let tracker = CallTracker::new();
        tracker.record_matched(make_ctx("GET", "/a"));
        assert!(tracker
            .get_last_matched_for_route(&MatchKey::new("GET", "/a"))
            .is_some());
    }

    #[test]
    fn get_last_matched_nulls_out_previous_context() {
        let tracker = CallTracker::new();
        let mut ctx = make_ctx("GET", "/a");
        ctx.previous_context = Some(Box::new(make_ctx("GET", "/a")));
        tracker.record_matched(ctx);
        let result = tracker
            .get_last_matched_for_route(&MatchKey::new("GET", "/a"))
            .unwrap();
        assert!(result.previous_context.is_none());
    }

    #[test]
    fn delete_calls_clears_last_matched() {
        let tracker = CallTracker::new();
        tracker.record_matched(make_ctx("GET", "/a"));
        tracker.delete_calls_for_route(&MatchKey::new("GET", "/a"));
        assert!(tracker
            .get_last_matched_for_route(&MatchKey::new("GET", "/a"))
            .is_none());
    }

    #[test]
    fn reset_clears_last_matched() {
        let tracker = CallTracker::new();
        tracker.record_matched(make_ctx("GET", "/a"));
        tracker.reset();
        assert!(tracker
            .get_last_matched_for_route(&MatchKey::new("GET", "/a"))
            .is_none());
    }
}
