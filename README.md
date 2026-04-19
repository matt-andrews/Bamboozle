<div align="center">
  <img src="./assets/logo_full_19apr26.png" width=256 alt="Bamboozle Logo" >
  <h1>Bamboozle</h1>
</div>

> ![WARNING]
> This project is still in a pre-release state. Use at your own risk.

Bamboozle is a container-native HTTP mock server for integration testing. It intercepts real HTTP traffic, allowing you to test your full client stack — serialisation, authentication headers, retry logic, circuit breakers — without any live dependency.

Unlike in-process mocking libraries, Bamboozle runs as a self-contained Docker container that is fully programmable at runtime from your test code., or configurable from startup files.

This readme discusses the app; we have a collection of sdk's to help you leverage the app in your language of choice.

## How it works

Bamboozle runs two HTTP servers:

- **Mock server** (`:8080`) — The surface your system under test calls. Matches requests against configured routes and returns configured responses.
- **Control API** (`:9090`) — The surface your test code calls to configure routes, inspect recorded calls, and assert behaviour.

A typical test lifecycle:

| Phase | Your action | What Bamboozle does |
| ------- | ------------- | --------------------- |
| Setup | `POST /control/routes` | Stores the route, activates immediately |
| Exercise | SUT calls `:8080` | Matches route, returns response, records call |
| Verify | `GET /control/routes/{verb}/{pattern}/calls` | Returns full call history |
| Assert | `POST /control/routes/{verb}/{pattern}/assert` | Evaluates conditions, returns pass or fail |
| Teardown | `POST /control/reset` | Removes all routes and clears all history |

## Getting started

```bash
docker run -p 8080:8080 -p 9090:9090 mattisthegreatest/bamboozle
```

Define a route via the control API:

```http
POST http://localhost:9090/control/routes
Content-Type: application/json

{
  "match": {
    "verb": "GET",
    "pattern": "/version"
  },
  "response": {
    "status": "200",
    "content": "1.0.0",
    "headers": { "Content-Type": "text/plain" }
  }
}
```

Your system under test can now call `http://localhost:8080/version` and receive `1.0.0`.

## Static configuration

Routes can be loaded from JSON or YAML files at startup — useful for routes that are constant across all tests, such as a health check or a version endpoint.

```bash
docker run \
  -e 'ROUTE_CONFIG_FOLDERS=["/etc/bamboozle/routes"]' \
  -e 'ROUTE_CONFIG_THROW_ON_ERROR=true' \
  -v ./routes:/etc/bamboozle/routes \
  -p 8080:8080 -p 9090:9090 \
  mattisthegreatest/bamboozle
```

| Variable | Default | Description |
| ---------- | --------- | ------------- |
| `ROUTE_CONFIG_FOLDERS` | `[]` | JSON array of directory paths to load route files from |
| `ROUTE_CONFIG_THROW_ON_ERROR` | `false` | Fail startup if any route file cannot be parsed |

Files with `.json`, `.yaml`, or `.yml` extensions are loaded automatically. The mock listener does not start until all files are loaded, so routes defined in static config are always active before the container is reachable.

Example route file:

```yaml
routes:
  - match:
      verb: GET
      pattern: /version
    response:
      status: "200"
      content: "1.0.0"
      headers:
        Content-Type: text/plain
```

## Route patterns

Paths can include typed parameters that are captured and made available in response templates.

| Syntax | Matches |
| -------- | --------- |
| `{id}` | Any path segment, captured as `id` |
| `{id:int}` | Integer segments only |
| `{id:guid}` | Valid GUIDs only |
| `{slug?}` | Optional — route matches with or without the segment |

Full list of type constraints: `int`, `long`, `double`, `decimal`, `float`, `bool`, `guid`, `alpha`, `datetime`.

The pattern `orders/{id:int}` matches `/orders/42` but not `/orders/abc`.

## Response templating

Response bodies, headers, and status codes support [Liquid](https://shopify.github.io/liquid/) templates. The following variables are available from the matched request:

| Variable | Value |
| ---------- | ------- |
| `{{ routeValues }}` | Captured route parameter |
| `{{ queryParams }}` | Query string parameter |
| `{{ headers }}` | Request header |
| `{{ body }}` | Parsed JSON body |
| `{{ bodyRaw }}` | Raw request body as a string |

Example — echo the captured ID back in a JSON response:

```json
{
  "routes": [
    {
      "match":{
        "verb": "GET",
        "pattern": "/orders/{id}"
      },
      "response": {
        "status": "200",
        "content": "{\"orderId\": \"{{ routeValues.id }}\"}",
        "headers": { "Content-Type": "application/json" }
      }
    }
  ]
}
```

## Loopback mode

Set `"loopback": true` on a response to echo the full request body back as the response body. Useful for verifying that your client sends the expected payload.

```json
{
  "routes": [
    {
      "match":{
        "verb": "POST",
        "pattern": "/echo"
      },
      "response": {
        "status": "200",
        "loopback": true
      }
    }
  ]
}
```

## Route management

| Method | Path | Description |
| -------- | ------ | ------------- |
| `POST` | `/control/routes` | Create a route — returns `409` if verb+pattern already exists |
| `PUT` | `/control/routes` | Create or replace a route (idempotent) |
| `GET` | `/control/routes` | List all active routes |
| `DELETE` | `/control/routes/{verb}/{pattern}` | Remove a route |

Routes are identified by their `verb`+`pattern` combination. Registering the same combination twice via `POST` returns a `409 Conflict`; use `PUT` when you need idempotent registration (e.g. from test setup code that may run more than once).

## Call history and assertions

All requests to the mock surface are recorded. Use the control API to inspect them and assert on their content from your tests.

### Retrieve calls

```http
GET http://localhost:9090/control/routes/GET/version/calls
```

URL-encode the pattern if it contains path characters — e.g. `orders/{id}` becomes `orders%2F%7Bid%7D`.

### Assert on calls

```http
POST http://localhost:9090/control/routes/GET/version/assert?expect=1
Content-Type: application/json

{}
```

`expect` is the required call count. Use `-1` to assert that at least one call was made regardless of count.

Returns `200 OK` on pass, `418 I'm a Teapot` on failure, `400 Bad Request` for an invalid expression.

### Filter with expressions

Pass an `expression` in the request body to assert only on calls that match a condition:

```json
{ "expression": "body(\"userId\") == \"abc123\" && header(\"x-tenant\") == \"acme\"" }
```

Available expression tokens:

| Token | Description |
| ------- | ------------- |
| `verb` | HTTP method of the call |
| `query("key")` | Query string parameter |
| `header("key")` | Request header (case-insensitive) |
| `route("key")` | Captured route parameter |
| `body("key")` | Top-level JSON body field |
| `contains(s, sub)` | Substring match |
| `starts_with(s, prefix)` | Prefix match |
| `ends_with(s, suffix)` | Suffix match |

> The assertion model is intentionally minimal in the current release. Richer structured conditions — `calledAtLeast`, `calledAtMost`, `neverCalled`, body shape verification, and per-call header checks — are planned.

### Unmatched calls

Any request that reaches the mock surface without matching a route is recorded separately:

```http
GET http://localhost:9090/control/unmatched
```

This is the first place to check when your system under test is unexpectedly receiving 404 responses.

## Lifecycle management

| Method | Path | Description |
| -------- | ------ | ------------- |
| `POST` | `/control/reset` | Remove all routes and clear all call history |
| `DELETE` | `/control/routes/{verb}/{pattern}/calls` | Clear history for one route without removing the route |
| `GET` | `/control/health` | Returns `200 OK` when the server is ready |
| `GET` | `/control/version` | Returns the current version string |

## API reference

Swagger UI is available at `http://localhost:9090/swagger-ui` when the container is running.

## Roadmap

Bamboozle is built against a detailed design specification. The following capabilities are planned but not yet implemented:

- **Request matching on content** — match routes based on headers, query parameters, and request body (JSON path conditions and schema validation), in addition to verb and path
- **Response sequences** — return different responses on successive calls to the same route, enabling stateful mock scenarios (e.g. first call returns `202`, second returns `409`)
- **Delay simulation** — configurable fixed, random, and gaussian latency to test client timeout handling and retry behaviour
- **Fault simulation** — connection reset and empty response injection to test client resilience against network failures
- **Session isolation** — per-test namespace via a session header, enabling safe parallel test execution against a shared Bamboozle instance
- **Route lifecycle options** — `times` to auto-deactivate a route after N matches, and `ttl` to auto-deactivate after N seconds
- **Richer assertions** — structured assertion types including `calledAtLeast`, `calledAtMost`, `neverCalled`, and per-call body and header verification
- **TestContainers modules** — first-class companion libraries to integrate Bamboozle into test suites with minimal boilerplate
