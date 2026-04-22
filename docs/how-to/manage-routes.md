# Manage routes

## Register a route

`POST /control/routes` creates a route. Returns `409 Conflict` if the same verb + pattern already exists.

```http
POST http://localhost:9090/control/routes
Content-Type: application/json

{
  "match": { "verb": "POST", "pattern": "/payments" },
  "response": { "status": "200", "content": "{\"id\": \"abc\"}" }
}
```

## Idempotent registration

`PUT /control/routes` creates or replaces. Use this from test setup code that may run more than once — it won't 409 on repeat runs.

```http
PUT http://localhost:9090/control/routes
Content-Type: application/json

{
  "match": { "verb": "POST", "pattern": "/payments" },
  "response": { "status": "200", "content": "{\"id\": \"abc\"}" }
}
```

## List all routes

```http
GET http://localhost:9090/control/routes
```

Returns the full `RouteDefinition` for every active route.

## Delete a route

```http
DELETE http://localhost:9090/control/routes/POST/payments
```

Returns `404` if the route doesn't exist.

## URL encoding

When a pattern contains `{`, `}`, or `/`, URL-encode those characters in the path:

| Pattern | Encoded |
|---|---|
| `/orders/{id}` | `/orders%2F%7Bid%7D` |
| `/users/{id}/orders` | `/users%2F%7Bid%7D%2Forders` |

## Reset everything

```http
POST http://localhost:9090/control/reset
```

Removes all routes and clears all call history. Use this in `afterEach` / `tearDown`.

---

**See also:** [Route definition reference](../reference/route-definition.md)
