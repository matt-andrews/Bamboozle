use liquid::model::Value;
use std::collections::HashMap;
use tracing::warn;

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
    pub fn render_or_fallback(
        &self,
        template_str: &str,
        ctx: &ContextModel,
        fallback: &str,
    ) -> String {
        self.render(template_str, ctx).unwrap_or_else(|e| {
            warn!(template = template_str, error = %e, "Template rendering failed, using fallback");
            fallback.to_string()
        })
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
    globals.insert("routeModel".into(), route_model_to_value(&ctx.route_model));
    let prev = ctx
        .previous_context
        .as_deref()
        .map(|p| Value::Object(context_to_object(p)))
        .unwrap_or(Value::Nil);
    globals.insert("previousContext".into(), prev);
    globals
}

fn context_to_object(ctx: &ContextModel) -> liquid::Object {
    let mut obj = liquid::Object::new();
    obj.insert("queryParams".into(), map_to_value(&ctx.query_params));
    obj.insert("headers".into(), map_to_value(&ctx.headers));
    obj.insert("routeValues".into(), map_to_value(&ctx.route_values));
    obj.insert("body".into(), json_to_liquid(&ctx.body));
    obj.insert("bodyRaw".into(), Value::scalar(ctx.body_raw.clone()));
    obj.insert("routeModel".into(), route_model_to_value(&ctx.route_model));
    obj
}

fn route_model_to_value(route_model: &crate::models::route::RouteDefinition) -> Value {
    let json = serde_json::to_value(route_model).unwrap_or(JsonValue::Null);
    json_to_liquid(&json)
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
            route_model: RouteDefinition {
                match_key: MatchKey::new("GET", "/test"),
                response: ResponseDefinition::default(),
            },
            previous_context: None,
        }
    }

    #[test]
    fn static_template() {
        let r = Renderer::new();
        assert_eq!(r.render("hello world", &make_ctx()).unwrap(), "hello world");
    }

    #[test]
    fn query_param_interpolation() {
        let r = Renderer::new();
        let mut ctx = make_ctx();
        ctx.query_params
            .insert("name".to_string(), "Alice".to_string());
        assert_eq!(
            r.render("Hello {{ queryParams.name }}", &ctx).unwrap(),
            "Hello Alice"
        );
    }

    #[test]
    fn header_interpolation() {
        let r = Renderer::new();
        let mut ctx = make_ctx();
        ctx.headers
            .insert("token".to_string(), "tok123".to_string());
        assert_eq!(r.render("{{ headers.token }}", &ctx).unwrap(), "tok123");
    }

    #[test]
    fn route_value_interpolation() {
        let r = Renderer::new();
        let mut ctx = make_ctx();
        ctx.route_values.insert("id".to_string(), "99".to_string());
        assert_eq!(r.render("id={{ routeValues.id }}", &ctx).unwrap(), "id=99");
    }

    #[test]
    fn body_raw_interpolation() {
        let r = Renderer::new();
        let mut ctx = make_ctx();
        ctx.body_raw = "raw content".to_string();
        assert_eq!(r.render("{{ bodyRaw }}", &ctx).unwrap(), "raw content");
    }

    #[test]
    fn body_json_field_interpolation() {
        let r = Renderer::new();
        let mut ctx = make_ctx();
        ctx.body = serde_json::json!({"status": "ok"});
        assert_eq!(r.render("{{ body.status }}", &ctx).unwrap(), "ok");
    }

    #[test]
    fn invalid_template_returns_err() {
        let r = Renderer::new();
        assert!(r.render("{%", &make_ctx()).is_err());
    }

    #[test]
    fn render_or_fallback_uses_fallback_on_error() {
        let r = Renderer::new();
        assert_eq!(
            r.render_or_fallback("{%", &make_ctx(), "fallback"),
            "fallback"
        );
    }

    #[test]
    fn render_or_fallback_returns_rendered_output() {
        let r = Renderer::new();
        let mut ctx = make_ctx();
        ctx.query_params.insert("x".to_string(), "1".to_string());
        assert_eq!(
            r.render_or_fallback("{{ queryParams.x }}", &ctx, "fallback"),
            "1"
        );
    }

    #[test]
    fn route_model_verb_interpolation() {
        let r = Renderer::new();
        let ctx = make_ctx();
        assert_eq!(
            r.render("{{ routeModel.match.verb }}", &ctx).unwrap(),
            "GET"
        );
    }

    #[test]
    fn route_model_pattern_interpolation() {
        let r = Renderer::new();
        let ctx = make_ctx();
        assert_eq!(
            r.render("{{ routeModel.match.pattern }}", &ctx).unwrap(),
            "test"
        );
    }

    #[test]
    fn previous_context_interpolation() {
        let r = Renderer::new();
        let mut prev = make_ctx();
        prev.route_values.insert("id".to_string(), "42".to_string());
        let mut ctx = make_ctx();
        ctx.previous_context = Some(Box::new(prev));
        assert_eq!(
            r.render("prev={{ previousContext.routeValues.id }}", &ctx)
                .unwrap(),
            "prev=42"
        );
    }

    #[test]
    fn previous_context_nil_when_absent() {
        let r = Renderer::new();
        let ctx = make_ctx();
        assert_eq!(r.render("{{ previousContext }}", &ctx).unwrap(), "");
    }
}
