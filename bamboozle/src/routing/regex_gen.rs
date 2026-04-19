use regex::Regex;
use std::collections::HashMap;

pub fn normalize_url(url: &str) -> String {
    let trimmed = url.trim_matches('/');
    let mut result = String::with_capacity(trimmed.len());
    let mut prev_slash = false;
    for c in trimmed.chars() {
        if c == '/' {
            if !prev_slash {
                result.push(c);
            }
            prev_slash = true;
        } else {
            result.push(c);
            prev_slash = false;
        }
    }
    result
}

fn constraint_pattern(constraint: &str) -> &'static str {
    match constraint {
        "int" | "long" => r"-?\d+",
        "double" | "decimal" | "float" => r"-?\d+(\.\d+)?",
        "bool" => r"true|false",
        "guid" => r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}",
        "alpha" => r"[a-zA-Z]+",
        "datetime" => r"\d{4}-\d{2}-\d{2}(T\d{2}:\d{2}:\d{2})?",
        _ => r"[^/]+",
    }
}

/// Compiles a route pattern (e.g. "api/{id:int}/{slug?}") into a Regex.
/// Called once at route-insert time; the compiled Regex is stored with the route.
pub fn compile_pattern(pattern: &str) -> Result<Regex, regex::Error> {
    let normalized = normalize_url(pattern);
    let mut regex_body = String::new();
    let mut remaining = normalized.as_str();

    while !remaining.is_empty() {
        if let Some(brace_start) = remaining.find('{') {
            // Append escaped literal text before this token
            regex_body.push_str(&regex::escape(&remaining[..brace_start]));
            remaining = &remaining[brace_start..];

            if let Some(brace_end) = remaining.find('}') {
                let inner = &remaining[1..brace_end]; // content between { }
                remaining = &remaining[brace_end + 1..];

                let optional = inner.ends_with('?');
                let inner = if optional {
                    &inner[..inner.len() - 1]
                } else {
                    inner
                };

                let (param_name, constraint) = match inner.find(':') {
                    Some(colon) => (&inner[..colon], &inner[colon + 1..]),
                    None => (inner, ""),
                };

                let value_pat = constraint_pattern(constraint);

                if optional {
                    // Pull the preceding slash into the optional group:
                    // "blog/{slug?}" → "blog(?:/(?P<slug>[^/]+))?"
                    if regex_body.ends_with('/') {
                        regex_body.pop();
                    }
                    regex_body.push_str(&format!("(?:/(?P<{}>{}))? ", param_name, value_pat));
                    // Remove the trailing space we accidentally wrote
                    let len = regex_body.len();
                    regex_body.truncate(len - 1);
                } else {
                    regex_body.push_str(&format!("(?P<{}>{})", param_name, value_pat));
                }
            } else {
                // No closing brace — treat the rest as literal
                regex_body.push_str(&regex::escape(remaining));
                break;
            }
        } else {
            regex_body.push_str(&regex::escape(remaining));
            break;
        }
    }

    Regex::new(&format!("(?i)^{}$", regex_body))
}

/// Attempts to match a URL against a compiled route pattern.
/// Returns Some(route_values) on success, None on miss.
/// Tries exact match first (fast path), then regex.
pub fn try_match_route(
    compiled: &Regex,
    normalized_pattern: &str,
    url: &str,
) -> Option<HashMap<String, String>> {
    let normalized_url = normalize_url(url);

    // Fast path: exact case-insensitive match (no route values to extract)
    if normalized_pattern.eq_ignore_ascii_case(&normalized_url) {
        return Some(HashMap::new());
    }

    // Regex match with named group extraction
    let caps = compiled.captures(&normalized_url)?;

    let route_values = compiled
        .capture_names()
        .flatten()
        .map(|name| {
            (
                name.to_string(),
                caps.name(name).map_or("", |m| m.as_str()).to_string(),
            )
        })
        .collect();

    Some(route_values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_route_exact_match() {
        let regex = compile_pattern("test/hello").unwrap();
        let result = try_match_route(&regex, "test/hello", "test/hello");
        assert!(result.is_some());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn static_route_case_insensitive() {
        let regex = compile_pattern("test/hello").unwrap();
        let result = try_match_route(&regex, "test/hello", "TEST/HELLO");
        assert!(result.is_some());
    }

    #[test]
    fn static_route_no_match() {
        let regex = compile_pattern("test/hello").unwrap();
        let result = try_match_route(&regex, "test/hello", "test/world");
        assert!(result.is_none());
    }

    #[test]
    fn parameterized_route() {
        let regex = compile_pattern("secrets/{name}/{version}").unwrap();
        let result = try_match_route(&regex, "secrets/{name}/{version}", "secrets/mysecret/v1");
        assert!(result.is_some());
        let values = result.unwrap();
        assert_eq!(values["name"], "mysecret");
        assert_eq!(values["version"], "v1");
    }

    #[test]
    fn optional_param_present() {
        let regex = compile_pattern("blog/{slug?}").unwrap();
        let result = try_match_route(&regex, "blog/{slug?}", "blog/my-post");
        assert!(result.is_some());
        assert_eq!(result.unwrap()["slug"], "my-post");
    }

    #[test]
    fn optional_param_absent() {
        let regex = compile_pattern("blog/{slug?}").unwrap();
        let result = try_match_route(&regex, "blog/{slug?}", "blog");
        assert!(result.is_some());
        assert_eq!(
            result
                .unwrap()
                .get("slug")
                .map(|s| s.as_str())
                .unwrap_or(""),
            ""
        );
    }

    #[test]
    fn int_constraint() {
        let regex = compile_pattern("items/{id:int}").unwrap();
        assert!(try_match_route(&regex, "items/{id:int}", "items/42").is_some());
        assert!(try_match_route(&regex, "items/{id:int}", "items/abc").is_none());
    }

    #[test]
    fn guid_constraint() {
        let regex = compile_pattern("items/{id:guid}").unwrap();
        assert!(try_match_route(
            &regex,
            "items/{id:guid}",
            "items/550e8400-e29b-41d4-a716-446655440000"
        )
        .is_some());
        assert!(try_match_route(&regex, "items/{id:guid}", "items/not-a-guid").is_none());
    }

    #[test]
    fn normalize_strips_slashes() {
        assert_eq!(normalize_url("/test/hello/"), "test/hello");
        assert_eq!(normalize_url("//test//hello//"), "test/hello");
    }

    #[test]
    fn leading_slash_pattern_matches_no_slash_url() {
        let regex = compile_pattern("/secrets/{name}").unwrap();
        let result = try_match_route(&regex, "secrets/{name}", "secrets/mysecret");
        assert!(result.is_some());
    }
}
