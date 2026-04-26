use evalexpr::{
    context_map, eval_boolean_with_context, ContextWithMutableFunctions, EvalexprError, Function,
    HashMapContext, Value,
};

use crate::models::context::ContextModel;

/// Evaluate a boolean expression against a single call context.
///
/// Available variables and functions:
///   verb                     – HTTP method of the recorded call ("GET", "POST", …)
///   pattern                  – route pattern matched by the call
///   state                    – computed state string from the setState template
///   query("key")             – value of a query-string parameter (empty string if absent)
///   header("key")            – value of a request header (empty string if absent)
///   route("key")             – value of a route-template capture (empty string if absent)
///   contains(haystack, needle)
///   starts_with(str, prefix)
///   ends_with(str, suffix)
pub fn eval_expression(expr: &str, ctx: &ContextModel) -> Result<bool, EvalexprError> {
    if expr.trim().is_empty() {
        return Ok(true);
    }
    let query_params = ctx.query_params.clone();
    let headers = ctx.headers.clone();
    let route_values = ctx.route_values.clone();
    let body_json = ctx.body.clone();

    let body_str = match &ctx.body {
        serde_json::Value::String(s) => s.clone(),
        v => serde_json::to_string(v).unwrap_or_default(),
    };

    let mut context: HashMapContext = context_map! {
        "verb"     => Value::String(ctx.route_model.match_key.verb.clone()),
        "pattern"  => Value::String(ctx.route_model.match_key.pattern.clone()),
        "body"     => Value::String(body_str),
        "body_raw" => Value::String(ctx.body_raw.clone()),
        "state"    => Value::String(ctx.state.clone()),
    }?;

    context.set_function(
        "query".to_string(),
        Function::new(move |arg| {
            let key = arg.as_string()?;
            Ok(Value::String(
                query_params.get(&key).cloned().unwrap_or_default(),
            ))
        }),
    )?;

    context.set_function(
        "header".to_string(),
        Function::new(move |arg| {
            let key = arg.as_string()?.to_ascii_lowercase();
            Ok(Value::String(
                headers.get(&key).cloned().unwrap_or_default(),
            ))
        }),
    )?;

    context.set_function(
        "route".to_string(),
        Function::new(move |arg| {
            let key = arg.as_string()?;
            Ok(Value::String(
                route_values.get(&key).cloned().unwrap_or_default(),
            ))
        }),
    )?;

    context.set_function(
        "body".to_string(),
        Function::new(move |arg| {
            let key = arg.as_string()?;
            match body_json.get(&key) {
                Some(serde_json::Value::String(s)) => Ok(Value::String(s.clone())),
                Some(serde_json::Value::Number(n)) => {
                    if let Some(i) = n.as_i64() {
                        Ok(Value::Int(i))
                    } else if let Some(f) = n.as_f64() {
                        Ok(Value::Float(f))
                    } else {
                        Ok(Value::String(n.to_string()))
                    }
                }
                Some(serde_json::Value::Bool(b)) => Ok(Value::Boolean(*b)),
                _ => Ok(Value::String(String::new())),
            }
        }),
    )?;

    context.set_function(
        "contains".to_string(),
        Function::new(|arg| {
            let (haystack, needle) = two_string_args(arg)?;
            Ok(Value::Boolean(haystack.contains(needle.as_str())))
        }),
    )?;

    context.set_function(
        "starts_with".to_string(),
        Function::new(|arg| {
            let (s, prefix) = two_string_args(arg)?;
            Ok(Value::Boolean(s.starts_with(prefix.as_str())))
        }),
    )?;

    context.set_function(
        "ends_with".to_string(),
        Function::new(|arg| {
            let (s, suffix) = two_string_args(arg)?;
            Ok(Value::Boolean(s.ends_with(suffix.as_str())))
        }),
    )?;

    let result = eval_boolean_with_context(expr, &context);
    if let Err(ref e) = result {
        match e {
            EvalexprError::ExpectedBoolean { .. } => {
                tracing::debug!(
                    expression = %expr,
                    error = %e,
                    "Expression did not evaluate to a boolean — must be a true/false expression"
                );
            }
            _ => {
                tracing::debug!(
                    expression = %expr,
                    error = %e,
                    "Expression evaluation error"
                );
            }
        }
    }
    result
}

fn two_string_args(arg: &Value) -> Result<(String, String), EvalexprError> {
    let args = arg.as_tuple()?;
    if args.len() != 2 {
        return Err(EvalexprError::WrongOperatorArgumentAmount {
            expected: 2,
            actual: args.len(),
        });
    }
    Ok((args[0].as_string()?, args[1].as_string()?))
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

    fn make_ctx() -> ContextModel {
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

    #[test]
    fn empty_expression_returns_true() {
        let ctx = make_ctx();
        assert!(eval_expression("", &ctx).unwrap());
        assert!(eval_expression("   ", &ctx).unwrap());
    }

    #[test]
    fn verb_variable() {
        let ctx = make_ctx();
        assert!(eval_expression(r#"verb == "GET""#, &ctx).unwrap());
        assert!(!eval_expression(r#"verb == "POST""#, &ctx).unwrap());
    }

    #[test]
    fn pattern_variable() {
        let ctx = make_ctx();
        assert!(eval_expression(r#"pattern == "test""#, &ctx).unwrap());
    }

    #[test]
    fn query_function_present_and_absent() {
        let mut ctx = make_ctx();
        ctx.query_params
            .insert("status".to_string(), "active".to_string());
        assert!(eval_expression(r#"query("status") == "active""#, &ctx).unwrap());
        assert!(eval_expression(r#"query("missing") == """#, &ctx).unwrap());
    }

    #[test]
    fn header_function_present_and_absent() {
        let mut ctx = make_ctx();
        ctx.headers
            .insert("x-request-id".to_string(), "abc123".to_string());
        assert!(eval_expression(r#"header("x-Request-id") == "abc123""#, &ctx).unwrap());
        assert!(eval_expression(r#"header("missing") == """#, &ctx).unwrap());
    }

    #[test]
    fn route_function() {
        let mut ctx = make_ctx();
        ctx.route_values.insert("id".to_string(), "42".to_string());
        assert!(eval_expression(r#"route("id") == "42""#, &ctx).unwrap());
    }

    #[test]
    fn body_variable_as_string() {
        let mut ctx = make_ctx();
        ctx.body = serde_json::Value::String("hello world".to_string());
        ctx.body_raw = "hello world".to_string();
        assert!(eval_expression(r#"contains(body, "hello")"#, &ctx).unwrap());
    }

    #[test]
    fn body_function_json_string_field() {
        let mut ctx = make_ctx();
        ctx.body = serde_json::json!({"name": "Alice"});
        assert!(eval_expression(r#"body("name") == "Alice""#, &ctx).unwrap());
    }

    #[test]
    fn body_function_json_number_field() {
        let mut ctx = make_ctx();
        ctx.body = serde_json::json!({"count": 30});
        assert!(eval_expression(r#"body("count") == 30"#, &ctx).unwrap());
    }

    #[test]
    fn contains_function() {
        let ctx = make_ctx();
        assert!(eval_expression(r#"contains("hello world", "world")"#, &ctx).unwrap());
        assert!(!eval_expression(r#"contains("hello world", "xyz")"#, &ctx).unwrap());
    }

    #[test]
    fn starts_with_function() {
        let ctx = make_ctx();
        assert!(eval_expression(r#"starts_with("hello world", "hello")"#, &ctx).unwrap());
        assert!(!eval_expression(r#"starts_with("hello world", "world")"#, &ctx).unwrap());
    }

    #[test]
    fn ends_with_function() {
        let ctx = make_ctx();
        assert!(eval_expression(r#"ends_with("hello world", "world")"#, &ctx).unwrap());
        assert!(!eval_expression(r#"ends_with("hello world", "hello")"#, &ctx).unwrap());
    }

    #[test]
    fn combined_and_expression() {
        let mut ctx = make_ctx();
        ctx.query_params
            .insert("env".to_string(), "prod".to_string());
        assert!(eval_expression(r#"verb == "GET" && query("env") == "prod""#, &ctx).unwrap());
        assert!(!eval_expression(r#"verb == "POST" && query("env") == "prod""#, &ctx).unwrap());
    }

    #[test]
    fn state_variable() {
        let mut ctx = make_ctx();
        ctx.state = "active".to_string();
        assert!(eval_expression(r#"state == "active""#, &ctx).unwrap());
        assert!(!eval_expression(r#"state == "inactive""#, &ctx).unwrap());
    }

    #[test]
    fn invalid_expression_returns_err() {
        let ctx = make_ctx();
        assert!(eval_expression("@@@", &ctx).is_err());
    }
}
