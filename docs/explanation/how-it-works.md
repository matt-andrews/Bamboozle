# How Bamboozle works

## Two servers, one process

Bamboozle runs two HTTP listeners in a single process:

- **`:8080` — mock surface.** This is what your system under test calls. It looks and behaves like the real service you're replacing. Routes defined here can use real domain paths, return realistic payloads, and inject realistic failures.
- **`:9090` — control API.** This is what your test code calls. It's where you register routes, inspect recorded calls, run assertions, and reset state between tests.

Keeping them on separate ports means the SUT never accidentally hits the control API, and the control API is never visible to whatever network the SUT is bound to.

## Test lifecycle

A typical test runs through five phases:

| Phase | Your action | What Bamboozle does |
|---|---|---|
| Setup | `POST /control/routes` | Stores the route, activates immediately |
| Exercise | SUT calls `:8080` | Matches route, returns response, records call |
| Verify | `GET /control/routes/{verb}/{pattern}/calls` | Returns full call history |
| Assert | `POST /control/routes/{verb}/{pattern}/assert` | Evaluates conditions, returns pass or fail |
| Teardown | `POST /control/reset` | Removes all routes and clears all history |

## Why container-native matters

In-process mocking libraries replace the HTTP client or stub interfaces inside your application. That means your tests only exercise the code that calls the mock — not serialisation, not auth header generation, not retry logic, not circuit breakers. A stubbed method passes by definition.

Bamboozle intercepts real HTTP traffic. Your full client stack runs. If your application double-encodes a JSON field, omits a required header, or retries idempotent requests incorrectly, Bamboozle will surface that — the same way a real service would.

## Route matching

When a request arrives at `:8080`, Bamboozle resolves it in order of specificity:

1. **Static routes first** — `/orders/active` before `/orders/{id}`
2. **Longer patterns preferred** — `/orders/{id}/items` before `/orders/{id}`
3. **Typed constraints preferred** — `{id:int}` before `{id}`

Routes are compiled to regex once at registration time, not on each request.

## Call recording

Every request to the mock surface is recorded, whether it matches a route or not. Matched calls are stored per-route and queryable by path. Unmatched calls go to a separate bucket, accessible at `GET /control/unmatched`. Recorded data persists until `POST /control/reset` or `DELETE /control/routes/{verb}/{pattern}/calls`.
