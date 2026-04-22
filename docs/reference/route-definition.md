# Route definition

The JSON/YAML schema for a route. Used in `POST /control/routes`, `PUT /control/routes`, and static config files.

## `RouteDefinition`

| Field | Type | Required | Description |
|---|---|---|---|
| `match` | `MatchKey` | yes | Identifies the route |
| `response` | `ResponseDefinition` | yes | What to return |
| `setState` | string | no | Liquid template — evaluated after each match, result stored as `state` |
| `simulation` | `SimulationConfig` | no | Latency or fault injection |

## `MatchKey`

| Field | Type | Description |
|---|---|---|
| `verb` | string | HTTP method, case-insensitive (`GET`, `POST`, etc.) |
| `pattern` | string | URL path pattern, optionally with typed parameters |

### Pattern syntax

| Syntax | Matches |
|---|---|
| `/orders` | Exact path |
| `/orders/{id}` | Any single path segment, captured as `id` |
| `/orders/{id:int}` | Integer segments only |
| `/orders/{id:guid}` | Valid GUIDs only |
| `/orders/{slug?}` | Optional segment — matches with or without it |

Type constraints: `int`, `long`, `double`, `decimal`, `float`, `bool`, `guid`, `alpha`, `datetime`.

## `ResponseDefinition`

| Field | Type | Default | Description |
|---|---|---|---|
| `status` | string | `"200"` | HTTP status code — supports Liquid templates |
| `headers` | object | `{}` | Response headers — values support Liquid templates |
| `content` | string | — | Inline response body (Liquid template) |
| `contentFile` | string | — | Path to a text file inside the container (Liquid template applied) |
| `binaryFile` | string | — | Path to a binary file inside the container (no templating) |
| `loopback` | bool | `false` | Echo the request body back as the response body |

Exactly one of `content`, `contentFile`, `binaryFile`, or `loopback` may be set. Specifying more than one returns `400 Bad Request`.

### Template variables

Available in `content`, `contentFile`, `status`, response `headers`, and `setState`:

| Variable | Type | Value |
|---|---|---|
| `{{ routeValues.key }}` | string | Captured route parameter |
| `{{ queryParams.key }}` | string | Query string parameter |
| `{{ headers.key }}` | string | Request header (case-insensitive key) |
| `{{ body.key }}` | any | Top-level JSON body field |
| `{{ bodyRaw }}` | string | Raw request body |
| `{{ state }}` | string | Result of `setState` from the previous matched request |
| `{{ previousContext }}` | object | Full context snapshot of the previous matched request |
| `{{ previousContext.state }}` | string | State from the request before that |

## `SimulationConfig`

| Field | Type | Description |
|---|---|---|
| `delay` | `DelayConfig` | Introduces latency before the response |
| `fault` | `FaultConfig` | Injects a connection failure |

Both fields are optional. When both are set, delay fires first, then the fault check.

## `DelayConfig`

| `type` | Required fields | Description |
|---|---|---|
| `fixed` | `ms: number` | Delays exactly `ms` milliseconds |
| `random` | `minMs: number`, `maxMs: number` | Uniform random delay in `[minMs, maxMs]` |
| `gaussian` | `meanMs: number`, `stdDevMs: number` | Normally-distributed delay, clamped to 0 |

## `FaultConfig`

| Field | Type | Default | Description |
|---|---|---|---|
| `type` | string | — | `connectionReset` or `emptyResponse` |
| `probability` | float | `1.0` | Fraction of requests that trigger the fault (0.0–1.0) |

`connectionReset` — sends headers then abruptly closes the connection.  
`emptyResponse` — returns `200 OK` with an empty body.
