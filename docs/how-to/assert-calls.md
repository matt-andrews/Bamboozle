# Assert on calls

Bamboozle records every request to the mock surface. The control API lets you inspect those records and assert on their count and content from your tests.

## Retrieve call history

```http
GET http://localhost:9090/control/routes/GET/version/calls
```

Returns an array of `ContextModel` objects — one per matched request, in order.

URL-encode the pattern if it contains path characters. `/orders/{id}` becomes `orders%2F%7Bid%7D`:

```http
GET http://localhost:9090/control/routes/GET/orders%2F%7Bid%7D/calls
```

## Count assertions

Append a count qualifier as a query parameter:

| Parameter | Passes when |
|---|---|
| `called_exactly=n` | matched call count equals `n` |
| `called_at_least=n` | matched call count ≥ `n` |
| `called_at_most=n` | matched call count ≤ `n` |
| `never_called=true` | route was never called (equivalent to `called_exactly=0`) |

```http
POST http://localhost:9090/control/routes/POST/payments/assert?called_exactly=3
Content-Type: application/json

{}
```

`200 OK` on pass. `406 Not Acceptable` on failure.

When no count parameter is given with an empty body, the assertion always passes.

## Filter with expressions

Pass an `expression` to assert only on calls that match a condition. The count qualifier then applies to the filtered set.

```http
POST http://localhost:9090/control/routes/POST/payments/assert?called_at_least=1
Content-Type: application/json

{
  "expression": "body(\"userId\") == \"abc123\" && header(\"x-tenant\") == \"acme\""
}
```

Passes if at least one call had `userId = abc123` in the body and `x-tenant: acme` in the headers.

When an `expression` is provided but no count qualifier, the assertion passes if at least one call matches the expression.

`400 Bad Request` is returned for an invalid expression. See [expression syntax reference](../reference/expression-syntax.md) for all available tokens.

## Unmatched calls

Any request that reaches the mock surface without matching a route is recorded separately:

```http
GET http://localhost:9090/control/unmatched
```

Check this first when your system under test is getting unexpected `404` responses.

## Clear history for one route

Removes call history for a single route without deleting the route itself:

```http
DELETE http://localhost:9090/control/routes/GET/version/calls
```

---

**See also:** [Expression syntax reference](../reference/expression-syntax.md) · [Control API reference](../reference/control-api.md)
