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
        "verb"    => Value::String(ctx.route_model.match_key.verb.clone()),
        "pattern" => Value::String(ctx.route_model.match_key.pattern.clone()),
        "body"    => Value::String(body_str),
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
            let key = arg.as_string()?;
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
            let args = arg.as_tuple()?;
            if args.len() != 2 {
                return Err(EvalexprError::WrongOperatorArgumentAmount {
                    expected: 2,
                    actual: args.len(),
                });
            }
            let haystack = args[0].as_string()?;
            let needle = args[1].as_string()?;
            Ok(Value::Boolean(haystack.contains(needle.as_str())))
        }),
    )?;

    context.set_function(
        "starts_with".to_string(),
        Function::new(|arg| {
            let args = arg.as_tuple()?;
            if args.len() != 2 {
                return Err(EvalexprError::WrongOperatorArgumentAmount {
                    expected: 2,
                    actual: args.len(),
                });
            }
            let s = args[0].as_string()?;
            let prefix = args[1].as_string()?;
            Ok(Value::Boolean(s.starts_with(prefix.as_str())))
        }),
    )?;

    context.set_function(
        "ends_with".to_string(),
        Function::new(|arg| {
            let args = arg.as_tuple()?;
            if args.len() != 2 {
                return Err(EvalexprError::WrongOperatorArgumentAmount {
                    expected: 2,
                    actual: args.len(),
                });
            }
            let s = args[0].as_string()?;
            let suffix = args[1].as_string()?;
            Ok(Value::Boolean(s.ends_with(suffix.as_str())))
        }),
    )?;

    eval_boolean_with_context(expr, &context)
}
