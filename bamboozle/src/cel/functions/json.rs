use cel_interpreter::extractors::This;
use cel_interpreter::{Context, FunctionContext, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type JsonValues = HashMap<Arc<String>, Result<Value, String>>;

pub fn register(context: &mut Context) {
    let cache = Mutex::new(JsonValues::new());

    context.add_function(
        "json",
        move |function: &FunctionContext, This(source): This<Arc<String>>| {
            cache
                .lock()
                .map_err(|_| function.error("cached JSON values lock was poisoned"))?
                .entry(source.clone())
                .or_insert_with(|| {
                    let parsed = serde_json::from_str::<serde_json::Value>(&source)
                        .map_err(|error| error.to_string())?;
                    cel_interpreter::to_value(parsed).map_err(|error| error.to_string())
                })
                .clone()
                .map_err(|error| function.error(error))
        },
    );
}
