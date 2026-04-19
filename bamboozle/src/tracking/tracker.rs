use dashmap::DashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::models::{context::ContextModel, match_key::MatchKey};

pub struct CallTracker {
    matched: DashMap<Uuid, ContextModel>,
    unmatched: DashMap<Uuid, ContextModel>,
}

impl CallTracker {
    pub fn new() -> Self {
        Self {
            matched: DashMap::new(),
            unmatched: DashMap::new(),
        }
    }

    pub fn record_matched(&self, ctx: ContextModel) {
        debug!(route = %ctx.route_model.match_key, "Recorded matched call");
        self.matched.insert(Uuid::new_v4(), ctx);
    }

    pub fn record_unmatched(&self, ctx: ContextModel) {
        warn!(route = %ctx.route_model.match_key, "Recorded unmatched call");
        self.unmatched.insert(Uuid::new_v4(), ctx);
    }

    pub fn get_calls_for_route(&self, key: &MatchKey) -> Vec<ContextModel> {
        let calls: Vec<ContextModel> = self.matched
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
        info!(route = %key, "Deleted calls for route");
    }

    /// Returns the MatchKey for each unmatched request (mirrors C# GetUnmatchedRouteCalls).
    pub fn get_unmatched(&self) -> Vec<MatchKey> {
        let keys: Vec<MatchKey> = self.unmatched
            .iter()
            .map(|entry| entry.value().route_model.match_key.clone())
            .collect();
        debug!(count = keys.len(), "Retrieved unmatched calls");
        keys
    }

    pub fn reset(&self) {
        self.matched.clear();
        self.unmatched.clear();
        info!("Call tracker cleared");
    }
}
