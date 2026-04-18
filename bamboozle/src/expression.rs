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
    let query_params = ctx.query_params.clone();
    let headers = ctx.headers.clone();
    let route_values = ctx.route_values.clone();

    let mut context: HashMapContext = context_map! {
        "verb"    => Value::String(ctx.route_model.match_key.verb.clone()),
        "pattern" => Value::String(ctx.route_model.match_key.pattern.clone()),
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
