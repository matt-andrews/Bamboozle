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

    /// Normalises a route pattern for canonical storage and comparison.
    /// Literal URL segments are lowercased (case-insensitive routing), but the
    /// content inside `{braces}` is preserved so param names match what callers
    /// use in Liquid templates (e.g. `{{routeValues.thingName}}`).
    pub(crate) fn normalize_pattern(pattern: impl Into<String>) -> String {
        let pattern = pattern.into();
        let trimmed = pattern.trim_matches('/');
        if trimmed.is_empty() {
            return String::new();
        }
        let mut result = String::with_capacity(trimmed.len());
        let mut in_braces = false;
        for c in trimmed.chars() {
            match c {
                '{' => { in_braces = true; result.push(c); }
                '}' => { in_braces = false; result.push(c); }
                _ if in_braces => result.push(c),
                _ => result.push(c.to_ascii_lowercase()),
            }
        }
        result
            .split('/')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("/")
    }
}

impl fmt::Display for MatchKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}|{}", self.verb, self.pattern)
    }
}
