use cel_interpreter::Context;

pub mod json;

pub fn register_all(context: &mut Context) {
    json::register(context);
}
