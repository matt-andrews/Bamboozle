# Simulated Fault Demo

Spin up Bamboozle in Docker, point a small TypeScript HTTP client at it, and watch the failure modes from the post — TCP connection resets, empty 200s, latency spikes, and transient flakes — actually take the client down. Then watch a more carefully written client survive them.

## What's in here

```json
.
├── docker-compose.yml          # boots Bamboozle with the routes mounted in
├── routes/
│   └── routes.yaml             # one route per fault type from the post
├── src/
│   └── payment-client.ts       # naiveFetch (buggy) and robustFetch (fixed)
└── tests/
    └── payment-client.test.ts  # one test per fault, naive vs. robust contrast
```

The two clients are deliberately, instructively different:

- **`naiveFetch`** retries only on 5xx status codes. This is what most teams ship after writing unit tests. It looks correct because every test in the unit suite is shaped like an HTTP error.
- **`robustFetch`** also catches `ECONNRESET` / `ETIMEDOUT` / `EPIPE`, validates that the response body is actually usable, and only then returns. This is what teams write *after* an incident.

The integration tests run both against the same Bamboozle faults. The contrast is the point.

## Run it

You need Docker and Node 20+.

```bash
npm install
npm run bamboozle:up      # start Bamboozle (mock listener on :8080, control on :9090)
npm test                  # run the integration tests
npm run bamboozle:down    # tear down
```

If something looks off, `npm run bamboozle:logs` will tail the container.

## The route map

Each route in `routes/routes.yaml` corresponds to a section of the blog post. Hit them with curl to see for yourself:

| Path                  | What Bamboozle does                                      |
| -------------------   | -------------------------------------------------------- |
| `GET /happy-path`     | Plain 200 with a JSON body                               |
| `GET /server-error`   | Plain 503                                                |
| `POST /payments`      | Sends headers, then resets the TCP connection            |
| `GET /empty-response` | Returns `200 OK` with an empty body                      |
| `GET /slow`           | Gaussian-distributed latency, mean 300ms, stddev 80ms    |
| `GET /flaky`          | 30% of calls reset the connection; the other 70% succeed |

Try this with the container running:

```bash
# Healthy
curl -i http://localhost:8080/happy-path

# 503
curl -i http://localhost:8080/server-error

# Connection reset — curl will report 'Empty reply from server' or 'Recv failure'
curl -i -X POST http://localhost:8080/payments

# Empty body
curl -i http://localhost:8080/empty-response

# Slow
time curl -s http://localhost:8080/slow

# Flaky — run it a bunch of times and watch some calls fail
for i in {1..20}; do curl -s -o /dev/null -w "%{http_code} %{errormsg}\n" http://localhost:8080/flaky; done
```

## What the tests demonstrate

Read `tests/payment-client.test.ts` top to bottom — each `describe` block names the post section it's exercising. The interesting moments:

1. **Both clients pass the 503 test.** This is the part your existing unit tests already cover.
2. **`naiveFetch` leaks a raw, non-FetchError exception on `/reset-me`.** The exception isn't an HTTP error — it comes from the socket layer, so the retry loop never sees it. This is the production fire.
3. **`naiveFetch` happily returns a non-PaymentResult value on `/empty-response`.** The status was 200, so the loop returned. Whatever bad data is downstream of this client just got fed garbage. This is the *silent* failure mode — no exception raised at all.
4. **On the flaky route, `robustFetch` succeeds ~97% of the time** (3 attempts × 30% fault rate ≈ 2.7% all-fail probability) **while `naiveFetch` succeeds about 70%** — the raw success rate of the underlying fault. The retry policy that *looked* correct in unit tests does nothing in the real world.

## How the tests stay honest

A test that only checks "did something throw?" isn't really testing the failure mode it claims to test. If Bamboozle weren't running, the `connection reset` test would still pass — the request would just throw a different network error for a different reason. To prevent that whole class of false confidence:

- **Each fault has a probe.** Before testing the client against `/reset-me`, the test hits the route directly with raw `fetch` and asserts the connection actually fails at the transport layer. If Bamboozle's fault isn't applied, the probe fails *there*, with a useful message — instead of misleading you downstream.
- **Errors carry a `kind` discriminator.** Both `naiveFetch` and `robustFetch` throw `FetchError` with `kind: "network" | "timeout" | "invalid_body" | "server_error" | ...`. Tests assert on the kind, not on regex matches against `message`. A timeout cannot be confused with an empty body cannot be confused with a connection reset.
- **The naive-client tests assert what type of error escapes.** The `/reset-me` test for `naiveFetch` proves that the thrown error is *not* a `FetchError` — i.e., naiveFetch wasn't handling network errors at all. That's the bug, expressed as an assertion.

If a test passes, the cause matched the claim.

## How Bamboozle is wired up here

Bamboozle exposes two ports:

- **`:8080`** — the mock HTTP listener. The TypeScript client (`payment-client.ts`) talks to this. From the client's perspective, this is just an upstream service.
- **`:9090`** — the control API. The test file talks to this directly via `fetch()` to register runtime routes, clear per-route call history, and assert on call counts.

Routes are defined two ways. Most of this demo uses **static config**: `routes/routes.yaml` is mounted into the container, and Bamboozle loads every YAML/JSON file in `/etc/bamboozle/routes` at startup. This is the right choice for routes that don't change between tests.

For routes that *do* change between tests — e.g., "first call returns success, second call returns failure" — register them at runtime. The connection-reset test in `tests/payment-client.test.ts` shows the pattern:

```ts
await fetch(`${CONTROL_URL}/control/routes`, {
  method: "PUT", // PUT is idempotent; safe to call across reruns
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({
    match: { verb: "GET", pattern: "/reset-me" },
    response: { status: "200" },
    simulation: { fault: { type: "connectionReset" } },
  }),
});
```

See the [Bamboozle docs](https://github.com/matt-andrews/Bamboozle/tree/main/docs) for the full route schema.

### A note on `reset()`

The control API has `POST /control/reset`, which removes **all** routes and **all** call history — including routes loaded from `routes.yaml` at startup. Use it sparingly; once you reset, anything statically configured is gone for the lifetime of the container. See [#61 for more on the current usecase](https://github.com/matt-andrews/Bamboozle/issues/61)

For per-test cleanup, prefer the narrower endpoint:

```
DELETE /control/routes/{verb}/{pattern}/calls
```

It clears call history for one route without removing the route itself. The assert-calls test uses this in `beforeEach`.

## Caveats

- The probability-based test (`/flaky`) uses statistical bounds (>85%, <85%). On 50 samples that's loose enough to be reliable, but if you crank `N` down you'll start to see flakes. Probabilistic tests need probabilistic assertions.
- Don't run Bamboozle anywhere reachable from the public internet. It's a fault-injecting mock server with no auth — it's for testing only.
