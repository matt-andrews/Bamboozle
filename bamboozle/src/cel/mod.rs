use cel_interpreter::{Program, Value};
use thiserror::Error;

use crate::models::context::ContextModel;

pub mod context;
pub mod functions;

#[derive(Debug, Error)]
pub enum CelError {
    #[error("could not compile CEL expression: {0}")]
    Compile(String),
    #[error("could not build CEL context: {0}")]
    Context(String),
    #[error("could not execute CEL expression: {0}")]
    Execute(String),
    #[error("CEL expression must return a boolean, got {0}")]
    NonBoolean(String),
}

/// A CEL expression compiled once and reusable against multiple recorded calls.
pub struct CompiledExpression {
    program: Program,
}

impl CompiledExpression {
    pub fn compile(expression: &str) -> Result<Self, CelError> {
        let program =
            Program::compile(expression).map_err(|error| CelError::Compile(error.to_string()))?;
        Ok(Self { program })
    }

    pub fn evaluate(&self, call: &ContextModel) -> Result<bool, CelError> {
        let context = context::for_call(call)?;
        let value = self
            .program
            .execute(&context)
            .map_err(|error| CelError::Execute(error.to_string()))?;

        match value {
            Value::Bool(result) => Ok(result),
            other => Err(CelError::NonBoolean(format!("{other:?}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        match_key::MatchKey,
        route::{ResponseDefinition, RouteDefinition},
    };
    use std::collections::HashMap;

    fn make_call() -> ContextModel {
        ContextModel {
            query_params: HashMap::new(),
            headers: HashMap::new(),
            route_values: HashMap::new(),
            body: serde_json::Value::Null,
            body_raw: String::new(),
            state: String::new(),
            route_model: RouteDefinition {
                match_key: MatchKey::new("GET", "/test"),
                set_state: None,
                simulation: None,
                max_calls: None,
                response: ResponseDefinition::default(),
            },
            previous_context: None,
        }
    }

    fn evaluate(expression: &str, call: &ContextModel) -> Result<bool, CelError> {
        CompiledExpression::compile(expression)?.evaluate(call)
    }

    #[test]
    fn exposes_method_and_pattern() {
        let call = make_call();
        assert!(evaluate(r#"verb == "GET" && pattern == "test""#, &call).unwrap());
        assert!(!evaluate(r#"verb == "POST""#, &call).unwrap());
    }

    #[test]
    fn exposes_query_route_and_headers_as_maps() {
        let mut call = make_call();
        call.query_params
            .insert("status".to_string(), "active".to_string());
        call.route_values.insert("id".to_string(), "42".to_string());
        call.headers
            .insert("X-Request-ID".to_string(), "abc-123".to_string());

        let expression = r#"query.status == "active" && route.id == "42" && headers["x-request-id"] == "abc-123""#;
        assert!(evaluate(expression, &call).unwrap());
    }

    #[test]
    fn body_is_raw_text_with_json_method() {
        let mut call = make_call();
        call.body_raw = r#"{"owner":{"name":"Alice"},"count":30,"active":true}"#.to_string();
        call.body = serde_json::from_str(&call.body_raw).unwrap();

        let expression = r#"body.contains("Alice") && body.json().owner.name == "Alice" && body.json().count == 30 && body.json().active"#;
        assert!(evaluate(expression, &call).unwrap());
    }

    #[test]
    fn supports_cel_string_methods_regex_and_collection_macros() {
        let mut call = make_call();
        call.body_raw = r#"[{"id":1,"name":"alpha"},{"id":2,"name":"beta"}]"#.to_string();
        call.body = serde_json::from_str(&call.body_raw).unwrap();

        let expression = r#"body.json().size() == 2 && body.json().all(item, item.id > 0) && body.json().exists(item, item.name.startsWith("a")) && body.json().map(item, item.id) == [1, 2] && "abc-123".matches("^[a-z]+-[0-9]+$")"#;
        assert!(evaluate(expression, &call).unwrap());
    }

    #[test]
    fn exposes_state() {
        let mut call = make_call();
        call.state = "active".to_string();
        assert!(evaluate(r#"state == "active""#, &call).unwrap());
    }

    #[test]
    fn json_method_parses_arbitrary_strings() {
        let call = make_call();
        assert!(evaluate(r#"'{"value":42}'.json().value == 42"#, &call).unwrap());
    }

    #[test]
    fn malformed_expression_fails_to_compile() {
        assert!(matches!(
            CompiledExpression::compile("this is not valid =="),
            Err(CelError::Compile(_))
        ));
    }

    #[test]
    fn invalid_json_is_an_execution_error() {
        let mut call = make_call();
        call.body_raw = "not json".to_string();
        assert!(matches!(
            evaluate("body.json().name == 'Alice'", &call),
            Err(CelError::Execute(_))
        ));
    }

    #[test]
    fn missing_map_value_is_an_execution_error() {
        let call = make_call();
        assert!(matches!(
            evaluate("query.missing == ''", &call),
            Err(CelError::Execute(_))
        ));
    }

    #[test]
    fn map_membership_supports_optional_values() {
        let call = make_call();
        assert!(evaluate(r#"!("missing" in query)"#, &call).unwrap());
    }

    #[test]
    fn large_json_integers_use_unsigned_literals() {
        let mut call = make_call();
        call.body_raw = r#"{"small":30,"large":9223372036854775808}"#.to_string();
        call.body = serde_json::from_str(&call.body_raw).unwrap();

        assert!(
            evaluate(
                "body.json().small == 30 && body.json().large == 9223372036854775808u",
                &call
            )
            .unwrap()
        );
    }

    #[test]
    fn non_boolean_result_is_rejected() {
        let call = make_call();
        assert!(matches!(
            evaluate("body", &call),
            Err(CelError::NonBoolean(_))
        ));
    }
}
