use cel_interpreter::Context;
use std::collections::HashMap;

use crate::models::context::ContextModel;

use super::{CelError, functions};

pub fn for_call<'a>(call: &ContextModel) -> Result<Context<'a>, CelError> {
    let mut context = Context::default();
    let headers = call
        .headers
        .iter()
        .map(|(name, value)| (name.to_ascii_lowercase(), value.clone()))
        .collect::<HashMap<_, _>>();

    context
        .add_variable("verb", &call.route_model.match_key.verb)
        .map_err(context_error)?;
    context
        .add_variable("pattern", &call.route_model.match_key.pattern)
        .map_err(context_error)?;
    context
        .add_variable("query", &call.query_params)
        .map_err(context_error)?;
    context
        .add_variable("headers", headers)
        .map_err(context_error)?;
    context
        .add_variable("route", &call.route_values)
        .map_err(context_error)?;
    context
        .add_variable("body", &call.body_raw)
        .map_err(context_error)?;
    context
        .add_variable("state", &call.state)
        .map_err(context_error)?;

    functions::register_all(&mut context);
    Ok(context)
}

fn context_error(error: cel_interpreter::SerializationError) -> CelError {
    CelError::Context(error.to_string())
}
