# Assertion expressions

An assertion can include an expression that filters recorded calls before applying a count condition:

```json
{
  "expression": "body(\"userId\") == \"abc123\" && header(\"x-tenant\") == \"acme\""
}
```

## Values

| Value | Description |
|---|---|
| `verb` | HTTP method |
| `pattern` | Matched route pattern |
| `state` | Route state recorded with the call |
| `body` | Parsed request body serialized as a string |
| `body_raw` | Original request body |
| `query("key")` | Query-string value |
| `header("key")` | Header value; keys are case-insensitive |
| `route("key")` | Captured route parameter |
| `body("key")` | Top-level JSON field |

Missing function values return an empty string.

## Functions and operators

| Syntax | Meaning |
|---|---|
| `contains(value, part)` | Value contains part |
| `starts_with(value, prefix)` | Value starts with prefix |
| `ends_with(value, suffix)` | Value ends with suffix |
| `==`, `!=` | Equality or inequality |
| `>`, `>=`, `<`, `<=` | Numeric or string comparison |
| `&&`, `\|\|` | Logical AND or OR |

Invalid expressions return `400 Bad Request`. Use `RUST_LOG=debug` to log evaluation errors.
