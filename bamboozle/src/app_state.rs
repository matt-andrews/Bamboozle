use std::sync::Arc;

use crate::{liquid_render::Renderer, routing::store::RouteStore, tracking::tracker::CallTracker};

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<RouteStore>,
    pub tracker: Arc<CallTracker>,
    pub renderer: Arc<Renderer>,
}

impl AppState {
    pub fn new(max_routes: usize, max_content_size: usize) -> Self {
        Self {
            store: Arc::new(RouteStore::new(max_routes, max_content_size)),
            tracker: Arc::new(CallTracker::new()),
            renderer: Arc::new(Renderer::new()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(1000, 10 * 1024 * 1024)
    }
}
