use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct MatchKey {
    pub verb: String,
    pub pattern: String,
}

impl MatchKey {
    pub fn new(verb: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self {
            verb: verb.into().to_uppercase(),
            pattern: Self::normalize_pattern(pattern),
        }
    }

    fn normalize_pattern(pattern: impl Into<String>) -> String {
        let pattern = pattern.into().to_ascii_lowercase();
        let trimmed = pattern.trim_matches('/');
        if trimmed.is_empty() {
            return String::new();
        }
        trimmed
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>()
            .join("/")
    }
}

impl fmt::Display for MatchKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}|{}", self.verb, self.pattern)
    }
}
