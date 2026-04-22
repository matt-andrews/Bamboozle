# Expression syntax

Expressions are used in the `POST /control/routes/{verb}/{pattern}/assert` body to filter calls before counting. An assertion with an expression passes only on calls where the expression evaluates to `true`.

```json
{ "expression": "body(\"userId\") == \"abc123\" && header(\"x-tenant\") == \"acme\"" }
```

## Value tokens

These extract a value from the recorded request:

| Token | Returns | Notes |
|---|---|---|
| `verb` | string | HTTP method of the call |
| `state` | string | Value stored by `setState` at the time of the call |
| `query("key")` | string | Query string parameter |
| `header("key")` | string | Request header — key match is case-insensitive |
| `route("key")` | string | Captured route parameter |
| `body("key")` | string \| number | Top-level JSON body field |

## String functions

| Function | Description |
|---|---|
| `contains(s, sub)` | `true` if `s` contains `sub` |
| `starts_with(s, prefix)` | `true` if `s` starts with `prefix` |
| `ends_with(s, suffix)` | `true` if `s` ends with `suffix` |

## Operators

| Operator | Description |
|---|---|
| `==`, `!=` | Equality / inequality |
| `>`, `>=`, `<`, `<=` | Numeric or lexicographic comparison |
| `&&` | Logical AND |
| `\|\|` | Logical OR |

## Examples

Assert a specific user and tenant:

```json
{
  "expression": "body(\"userId\") == \"abc123\" && header(\"x-tenant\") == \"acme\""
}
```

Assert a minimum order value was sent:

```json
{
  "expression": "body(\"amount\") >= 100"
}
```

Assert the request path captured an integer in a specific range (via route param):

```json
{
  "expression": "route(\"id\") >= 1 && route(\"id\") <= 999"
}
```

Assert a header starts with a known prefix:

```json
{
  "expression": "starts_with(header(\"authorization\"), \"Bearer \")"
}
```

## Error handling

An invalid expression returns `400 Bad Request`. The specific parse or evaluation error is logged at `debug` level — set `RUST_LOG=debug` if you need to diagnose an expression that isn't behaving as expected.
