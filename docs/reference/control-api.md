# Control API reference

The control API runs on `:9090`. An interactive Scalar UI is available at `http://localhost:9090/` when the container is running. The raw OpenAPI spec is at `http://localhost:9090/api-docs/openapi.json`.

## URL encoding

When a verb or pattern appears as a path segment, URL-encode any special characters. Pattern `/orders/{id}` becomes `orders%2F%7Bid%7D`.

---

## Routes

### Create a route

```
POST /control/routes
```

Body: [`RouteDefinition`](route-definition.md)

| Response | Meaning |
|---|---|
| `201 Created` | Route registered |
| `400 Bad Request` | Invalid definition (e.g. multiple body strategies) |
| `409 Conflict` | Verb + pattern already exists — use `PUT` to replace |

### Create or replace a route

```
PUT /control/routes
```

Body: [`RouteDefinition`](route-definition.md)

Idempotent. If the route exists it is deleted and recreated. Returns `201`.

### List all routes

```
GET /control/routes
```

Returns an array of `RouteDefinition` objects.

### Delete a route

```
DELETE /control/routes/{verb}/{pattern}
```

| Response | Meaning |
|---|---|
| `204 No Content` | Deleted |
| `404 Not Found` | Route doesn't exist |

---

## Call history

### Get calls for a route

```
GET /control/routes/{verb}/{pattern}/calls
```

Returns an array of `ContextModel` objects in order of arrival.

### Clear calls for a route

```
DELETE /control/routes/{verb}/{pattern}/calls
```

Removes call history without deleting the route. Returns `204`.

### Get unmatched calls

```
GET /control/unmatched
```

Returns calls that reached the mock surface without matching any route. Check this when your system under test is receiving unexpected `404` responses.

---

## Assertions

### Assert on a route

```
POST /control/routes/{verb}/{pattern}/assert
```

Query parameters (all optional):

| Parameter | Description |
|---|---|
| `called_exactly=n` | Pass only if call count equals `n` |
| `called_at_least=n` | Pass only if call count ≥ `n` |
| `called_at_most=n` | Pass only if call count ≤ `n` |
| `never_called=true` | Pass only if route was never called |

Request body (optional):

```json
{ "expression": "body(\"userId\") == \"abc123\"" }
```

When an expression is provided, count qualifiers apply to the filtered set. When no count qualifier is given with an expression, at least one matching call must exist.

| Response | Meaning |
|---|---|
| `200 OK` | Assertion passes |
| `418 I'm a Teapot` | Assertion fails |
| `400 Bad Request` | Expression is invalid |

---

## Lifecycle

### Reset

```
POST /control/reset
```

Removes all routes and clears all call history. Returns `204`.

### Health

```
GET /control/health
```

Returns `200 OK` when the server is ready.

### Version

```
GET /control/version
```

Returns the current version string as plain text.
