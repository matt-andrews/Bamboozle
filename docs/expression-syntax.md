# CEL assertion expressions

An assertion can include a [Common Expression Language (CEL)](https://cel.dev/)
expression that filters recorded calls before applying a count condition:

```json
{
  "expression": "body.json().userId == \"abc123\" && headers[\"x-tenant\"] == \"acme\""
}
```

Every expression must evaluate to a boolean. A `true` result includes the call
in the filtered count; a `false` result excludes it.

## Variables

| Variable | CEL type | Description |
|---|---|---|
| `verb` | `string` | HTTP method |
| `pattern` | `string` | Matched route pattern |
| `query` | `map<string, string>` | Query-string parameters |
| `headers` | `map<string, string>` | Request headers; names are lowercase |
| `route` | `map<string, string>` | Captured route parameters |
| `body` | `string` | Original request body |
| `state` | `string` | Route state recorded with the call |

Identifier-like map keys support dotted access:

```cel
query.mode == "sync" && route.tenant == "acme"
```

Use bracket access for keys containing punctuation:

```cel
headers["x-request-id"].startsWith("callback-")
```

Missing map keys are evaluation errors; they do not become empty strings. Test
optional values before reading them:

```cel
"mode" in query && query.mode == "sync"
```

## JSON request bodies

Call `.json()` on the body to parse it into CEL-compatible JSON data:

```cel
body.json().user.name == "Alice"
body.json()["display-name"] != ""
body.json().items.all(item, item.quantity > 0)
```

Invalid JSON produces an evaluation error. The raw string remains available for
text assertions such as `body.contains("order-42")`.

## CEL methods, macros, and literals

Normal CEL operators, methods, macros, and literals supported by the bundled
interpreter are available. Common examples include:

| Syntax | Meaning |
|---|---|
| `value.contains(part)` | String, list, or map contains a value |
| `value.startsWith(prefix)` | String starts with a prefix |
| `value.endsWith(suffix)` | String ends with a suffix |
| `value.matches(pattern)` | String matches a regular expression |
| `value.size()` | String, list, or map size |
| `items.all(item, condition)` | Every list item matches |
| `items.exists(item, condition)` | At least one list item matches |
| `items.exists_one(item, condition)` | Exactly one list item matches |
| `items.filter(item, condition)` | Filter a list |
| `items.map(item, expression)` | Transform a list |
| `==`, `!=`, `>`, `>=`, `<`, `<=` | Comparison |
| `&&`, `\|\|`, `!` | Boolean operators |

Ordinary JSON integers use CEL signed integer literals such as `100`. Values
larger than `i64::MAX` are unsigned and require the `u` suffix, for example
`9223372036854775808u`.

## Errors and count conditions

Malformed CEL, execution errors, missing values, type errors, invalid JSON, and
non-boolean results return `400 Bad Request`. A valid expression that matches
the wrong number of calls returns `406 Not Acceptable`.

Count conditions are applied after CEL filtering. Without a count condition, an
expression passes when at least one recorded call evaluates to `true`.

Use `RUST_LOG=debug` for assertion diagnostics.
