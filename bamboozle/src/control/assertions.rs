use serde::Deserialize;
use thiserror::Error;
use utoipa::ToSchema;

use crate::{
    cel::{CelError, CompiledExpression},
    models::context::ContextModel,
};

#[derive(Deserialize, ToSchema)]
pub struct AssertRequest {
    /// CEL expression evaluated against each recorded call; it must return a boolean.
    ///
    /// Variables: `verb`, `pattern`, `query`, `headers`, `route`, `body`, and `state`.
    /// The raw request body is exposed as `body`; call `body.json()` for JSON access.
    ///
    /// Example: `query.status == "active" && verb == "POST"`
    pub expression: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct AssertQuery {
    pub called_exactly: Option<i64>,
    pub called_at_least: Option<i64>,
    pub called_at_most: Option<i64>,
    #[serde(default)]
    pub never_called: bool,
}

#[derive(Debug, Error)]
pub enum AssertionError {
    #[error("{name} must be >= 0")]
    InvalidQualifier { name: &'static str },
    #[error("Invalid CEL expression: {0}")]
    InvalidExpression(#[from] CelError),
}

#[derive(Debug, PartialEq, Eq)]
pub struct AssertionResult {
    pub passed: bool,
    pub matched_count: i64,
    pub condition: String,
}

pub fn normalize_expression(expression: Option<&str>) -> Option<&str> {
    expression.map(str::trim).filter(|value| !value.is_empty())
}

pub fn evaluate(
    calls: &[ContextModel],
    expression: Option<&str>,
    query: &AssertQuery,
) -> Result<AssertionResult, AssertionError> {
    let expression = normalize_expression(expression);
    let compiled = expression.map(CompiledExpression::compile).transpose()?;

    validate_qualifiers(query)?;

    let matched_count = if let Some(compiled) = compiled {
        calls.iter().try_fold(0_i64, |count, call| {
            compiled
                .evaluate(call)
                .map(|matched| count + i64::from(matched))
        })?
    } else {
        calls.len() as i64
    };

    let any_qualifier = query.never_called
        || query.called_exactly.is_some()
        || query.called_at_least.is_some()
        || query.called_at_most.is_some();
    let mut failing = Vec::new();

    if query.never_called && matched_count != 0 {
        failing.push(format!(
            "expected 0 calls (never_called), got {matched_count}"
        ));
    }
    if let Some(expected) = query.called_exactly
        && matched_count != expected
    {
        failing.push(format!("expected exactly {expected}, got {matched_count}"));
    }
    if let Some(expected) = query.called_at_least
        && matched_count < expected
    {
        failing.push(format!("expected at least {expected}, got {matched_count}"));
    }
    if let Some(expected) = query.called_at_most
        && matched_count > expected
    {
        failing.push(format!("expected at most {expected}, got {matched_count}"));
    }

    let passed = if any_qualifier {
        failing.is_empty()
    } else if expression.is_some() {
        matched_count >= 1
    } else {
        true
    };
    let condition = if !passed && !any_qualifier {
        format!("expected >= 1 match for expression, got {matched_count}")
    } else {
        failing.join("; ")
    };

    Ok(AssertionResult {
        passed,
        matched_count,
        condition,
    })
}

fn validate_qualifiers(query: &AssertQuery) -> Result<(), AssertionError> {
    for (name, value) in [
        ("called_exactly", query.called_exactly),
        ("called_at_least", query.called_at_least),
        ("called_at_most", query.called_at_most),
    ] {
        if value.is_some_and(|number| number < 0) {
            return Err(AssertionError::InvalidQualifier { name });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        match_key::MatchKey,
        route::{ResponseDefinition, RouteDefinition},
    };
    use std::collections::HashMap;

    fn make_call(verb: &str) -> ContextModel {
        ContextModel {
            query_params: HashMap::new(),
            headers: HashMap::new(),
            route_values: HashMap::new(),
            route_model: RouteDefinition {
                match_key: MatchKey::new(verb, "/test"),
                set_state: None,
                simulation: None,
                max_calls: None,
                response: ResponseDefinition::default(),
            },
            body: serde_json::Value::Null,
            body_raw: String::new(),
            state: String::new(),
            previous_context: None,
        }
    }

    #[test]
    fn no_expression_or_qualifier_is_a_no_op_pass() {
        let result = evaluate(&[], None, &AssertQuery::default()).unwrap();
        assert_eq!(
            result,
            AssertionResult {
                passed: true,
                matched_count: 0,
                condition: String::new(),
            }
        );
    }

    #[test]
    fn expression_filters_calls_before_counting() {
        let calls = vec![make_call("GET"), make_call("POST"), make_call("GET")];
        let query = AssertQuery {
            called_exactly: Some(2),
            ..Default::default()
        };

        let result = evaluate(&calls, Some("verb == 'GET'"), &query).unwrap();
        assert!(result.passed);
        assert_eq!(result.matched_count, 2);
    }

    #[test]
    fn expression_without_qualifier_requires_one_match() {
        let calls = vec![make_call("GET")];
        let result = evaluate(&calls, Some("verb == 'POST'"), &AssertQuery::default()).unwrap();

        assert!(!result.passed);
        assert_eq!(
            result.condition,
            "expected >= 1 match for expression, got 0"
        );
    }

    #[test]
    fn compatible_qualifiers_all_pass() {
        let calls = vec![make_call("GET")];
        let query = AssertQuery {
            called_at_least: Some(1),
            called_at_most: Some(1),
            ..Default::default()
        };

        assert!(evaluate(&calls, None, &query).unwrap().passed);
    }

    #[test]
    fn never_called_counts_filtered_calls() {
        let calls = vec![make_call("GET")];
        let query = AssertQuery {
            never_called: true,
            ..Default::default()
        };

        assert!(
            evaluate(&calls, Some("verb == 'POST'"), &query)
                .unwrap()
                .passed
        );
    }

    #[test]
    fn negative_qualifier_is_rejected() {
        let query = AssertQuery {
            called_exactly: Some(-1),
            ..Default::default()
        };

        assert!(matches!(
            evaluate(&[], None, &query),
            Err(AssertionError::InvalidQualifier {
                name: "called_exactly"
            })
        ));
    }

    #[test]
    fn malformed_expression_is_rejected_with_no_calls() {
        assert!(matches!(
            evaluate(&[], Some("this is not valid =="), &AssertQuery::default()),
            Err(AssertionError::InvalidExpression(CelError::Compile(_)))
        ));
    }
}
