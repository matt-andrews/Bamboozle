<div align="center">
  <img src="./assets/logo_full_19apr26.png" width=256 alt="Bamboozle Logo" >
  <h1>Bamboozle</h1>

  [![Startup Time](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/matt-andrews/Bamboozle/badges/badge-startup.json?v=1&cacheSeconds=3600)](https://github.com/matt-andrews/Bamboozle/actions/workflows/startup-time.yml)
  [![Image Size](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/matt-andrews/Bamboozle/badges/badge-docker-size.json?v=1&cacheSeconds=3600)](https://github.com/matt-andrews/Bamboozle/pkgs/container/bamboozle)
</div>

> [!WARNING]
> This project is still in a super-pre-release state. Use at your own risk.

Bamboozle is a container-native HTTP mock server for integration testing. It intercepts real HTTP traffic, allowing you to test your full client stack — serialisation, authentication headers, retry logic, circuit breakers — without any live dependency.

Unlike in-process mocking libraries, Bamboozle runs as a self-contained Docker container that is fully programmable at runtime from your test code, or configurable from startup files.

This readme discusses the app; we are working on a collection of SDKs to help you leverage the app in your language of choice.

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

## Logging

Bamboozle is designed as a developer tool, so its logs are intentionally rich. By default it writes compact, coloured output to stdout.

### Log level

Control verbosity with `RUST_LOG`, which accepts the standard `env_logger` filter syntax:

```bash
# Info and above (default)
RUST_LOG=info

# Show debug output for route matching and expression evaluation
RUST_LOG=debug

# Silence everything except errors
RUST_LOG=error
```

### Log format

| Variable | Default | Description |
| ---------- | --------- | ------------- |
| `RUST_LOG` | `info` | Log level / filter — standard `env_logger` syntax |
| `RUST_LOG_FORMAT` | `compact` | Output format: `compact`, `pretty`, or `json` |
| `NO_COLOR` | unset | Set to any value to disable ANSI colour codes |

`json` format emits one JSON object per line, suitable for ingestion by log shippers such as [Vector](https://vector.dev), [Promtail](https://grafana.com/docs/loki/latest/send-data/promtail/), or [Fluent Bit](https://fluentbit.io):

```bash
docker run -e RUST_LOG_FORMAT=json -p 8080:8080 -p 9090:9090 mattisthegreatest/bamboozle
```

### Forwarding to an external backend

Bamboozle supports [OpenTelemetry OTLP](https://opentelemetry.io/docs/specs/otlp/) natively. Set `OTEL_EXPORTER_OTLP_ENDPOINT` to send traces to any OTLP-compatible backend — no additional libraries or sidecars required.

| Variable | Description |
| ---------- | ------------- |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OTLP HTTP endpoint. Setting this variable activates the exporter. |
| `OTEL_EXPORTER_OTLP_HEADERS` | Comma-separated `key=value` pairs sent as HTTP headers — used for authentication. |

**Grafana Cloud**

```bash
docker run \
  -e OTEL_EXPORTER_OTLP_ENDPOINT=https://otlp-gateway-<zone>.grafana.net/otlp \
  -e 'OTEL_EXPORTER_OTLP_HEADERS=Authorization=Basic <base64-token>' \
  -p 8080:8080 -p 9090:9090 \
  mattisthegreatest/bamboozle
```

**New Relic**

```bash
docker run \
  -e OTEL_EXPORTER_OTLP_ENDPOINT=https://otlp.nr-data.net \
  -e 'OTEL_EXPORTER_OTLP_HEADERS=Api-Key=<your-license-key>' \
  -p 8080:8080 -p 9090:9090 \
  mattisthegreatest/bamboozle
```

Console and OTLP output are active simultaneously when the endpoint is set, so you keep local visibility while shipping to a backend.

### What gets logged

| Event | Level | Details |
| ------- | ----- | ------- |
| Unmatched request | `warn` | Verb, path, and up to 3 fuzzy-matched route suggestions |
| Assertion failed | `warn` | Expected count, matched count, total calls, expression |
| Assertion passed | `debug` | Same fields as above |
| Expression error | `debug` | Expression string and the specific evaluation failure |
| Route registered / deleted | `info` | Route key |
| Startup | `info` | Listening addresses |

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
| `{{ previousContext }}` | The context from the previous request (if it exists) |

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

## Fault & latency simulation

Add a `simulation` object to any route to inject artificial delay or failure. Routes without a `simulation` field behave normally.

### Delay

Three distributions are available:

| Type | Fields | Behaviour |
| ------ | ------- | --------- |
| `fixed` | `ms` | Always delays by exactly `ms` milliseconds — fully deterministic |
| `random` | `minMs`, `maxMs` | Uniform random delay in the range `[minMs, maxMs]` |
| `gaussian` | `meanMs`, `stdDevMs` | Normally-distributed delay centred on `meanMs`; clamped to 0 |

```json
{
  "match": { "verb": "GET", "pattern": "/orders" },
  "response": { "status": "200", "content": "[]" },
  "simulation": {
    "delay": { "type": "random", "minMs": 100, "maxMs": 800 }
  }
}
```

Delay is implemented with `tokio::time::sleep` — the waiting task yields back to the executor, so other routes are served concurrently and no threads are blocked.

### Faults

Two fault modes are supported:

| Type | Behaviour |
| ------ | --------- |
| `connectionReset` | Sends response headers then abruptly closes the connection — client sees a broken-pipe / connection-reset error |
| `emptyResponse` | Returns `200 OK` with an empty body |

```json
{
  "match": { "verb": "POST", "pattern": "/payments" },
  "response": { "status": "200" },
  "simulation": {
    "fault": { "type": "connectionReset", "probability": 0.1 }
  }
}
```

`probability` is a float from `0.0` (never) to `1.0` (always, the default). Values less than 1.0 make the fault **transient** — useful for chaos-style testing where only a fraction of calls should fail.

### Combining delay and fault

Both fields may be set together. When both are present, the delay is always applied first, then the fault check runs — so you can simulate "slow then broken" scenarios:

```json
{
  "match": { "verb": "GET", "pattern": "/slow-and-flaky" },
  "response": { "status": "200", "content": "ok" },
  "simulation": {
    "delay": { "type": "gaussian", "meanMs": 300, "stdDevMs": 80 },
    "fault": { "type": "emptyResponse", "probability": 0.25 }
  }
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
| `state` | The state value set through RouteDefinition.setState |
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

Scalar API reference is available at `http://localhost:9090/` when the container is running.

## Roadmap

Bamboozle is built against a detailed design specification. The following capabilities are planned but not yet implemented:

- **Request matching on content** — match routes based on headers, query parameters, and request body (JSON path conditions and schema validation), in addition to verb and path
- **Session isolation** — per-test namespace via a session header, enabling safe parallel test execution against a shared Bamboozle instance
- **Route lifecycle options** — `times` to auto-deactivate a route after N matches, and `ttl` to auto-deactivate after N seconds
- **Richer assertions** — structured assertion types including `calledAtLeast`, `calledAtMost`, `neverCalled`, and per-call body and header verification
- **TestContainers modules** — first-class companion libraries to integrate Bamboozle into test suites with minimal boilerplate
