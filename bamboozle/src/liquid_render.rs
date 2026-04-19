use liquid::model::Value;
use std::collections::HashMap;

use crate::models::context::ContextModel;
use serde_json::Value as JsonValue;

pub struct Renderer {
    parser: liquid::Parser,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            parser: liquid::ParserBuilder::with_stdlib()
                .build()
                .expect("Failed to build liquid parser"),
        }
    }

    pub fn render(&self, template_str: &str, ctx: &ContextModel) -> anyhow::Result<String> {
        let template = self
            .parser
            .parse(template_str)
            .map_err(|e| anyhow::anyhow!("Template parse error: {}", e))?;

        let globals = build_globals(ctx);

        template
            .render(&globals)
            .map_err(|e| anyhow::anyhow!("Template render error: {}", e))
    }

    /// Renders a template, returning `fallback` on any error.
    pub fn render_or_fallback(&self, template_str: &str, ctx: &ContextModel, fallback: &str) -> String {
        self.render(template_str, ctx)
            .unwrap_or_else(|_| fallback.to_string())
    }
}

/// Builds the liquid template globals from a ContextModel.
///
/// Maps are stored as liquid Objects so that dot-access ({{queryParams.status}})
/// works. When iterating an Object in a Liquid for-loop, the `liquid` crate yields
/// [key, value] pair arrays, enabling the {{kvp[0]}}={{kvp[1]}} pattern used in
/// the test configs.
fn build_globals(ctx: &ContextModel) -> liquid::Object {
    let mut globals = liquid::Object::new();
    globals.insert("queryParams".into(), map_to_value(&ctx.query_params));
    globals.insert("headers".into(), map_to_value(&ctx.headers));
    globals.insert("routeValues".into(), map_to_value(&ctx.route_values));
    globals.insert("body".into(), json_to_liquid(&ctx.body));
    globals.insert("bodyRaw".into(), Value::scalar(ctx.body_raw.clone()));
    globals
}

fn map_to_value(map: &HashMap<String, String>) -> Value {
    let obj: liquid::Object = map
        .iter()
        .map(|(k, v)| (k.clone().into(), Value::scalar(v.clone())))
        .collect();
    Value::Object(obj)
}

fn json_to_liquid(value: &JsonValue) -> Value {
    match value {
        JsonValue::Null => Value::Nil,
        JsonValue::Bool(b) => Value::scalar(*b),
        JsonValue::Number(n) => n
            .as_i64()
            .map(Value::scalar)
            .or_else(|| n.as_f64().map(Value::scalar))
            .unwrap_or_else(|| Value::scalar(n.to_string())),
        JsonValue::String(s) => Value::scalar(s.clone()),
        JsonValue::Array(arr) => Value::Array(arr.iter().map(json_to_liquid).collect()),
        JsonValue::Object(obj) => {
            let liquid_obj: liquid::Object = obj
                .iter()
                .map(|(k, v)| (k.clone().into(), json_to_liquid(v)))
                .collect();
            Value::Object(liquid_obj)
        }
    }
}
