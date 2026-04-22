# Request lifecycle

Trace of a request through the mock surface (`mock_server.rs`):

```
HTTP request arrives at :8080
       │
       ▼
catch_all (mock_server.rs)
       │
       ├─ extract verb, path, query params, headers, body
       │
       ├─ RouteStore::match_route(verb, path)
       │    ├─ look up verb in outer DashMap
       │    ├─ sort stored routes: static first, longer first, more type constraints first
       │    └─ run compiled Regex against path; extract named captures into routeValues
       │
       ├─ [no match]
       │    ├─ record_unmatched(verb, path)
       │    ├─ log warn with up to 3 fuzzy-matched route suggestions (Jaro-Winkler)
       │    └─ return 404
       │
       └─ [match] → build ContextModel (queryParams, headers, routeValues, body, bodyRaw)
                   → render setState template (liquid_render.rs), store result per MatchKey
                   → record_matched(match_key, context)
                   → apply_simulation:
                   │    ├─ delay? → tokio::time::sleep (async, non-blocking)
                   │    └─ fault? → roll probability → connectionReset or emptyResponse
                   └─ build_response:
                        ├─ render status template
                        ├─ render header templates
                        └─ render body: content | contentFile | binaryFile | loopback
```

## Key files

| File | Role |
|---|---|
| `mock_server.rs` | `catch_all` handler — the entry point for all mock requests |
| `routing/store.rs` | `match_route` — sorting and regex matching |
| `routing/regex_gen.rs` | `compile_pattern` — pattern → `Regex` |
| `liquid_render.rs` | `render` / `render_or_fallback` — Liquid evaluation |
| `tracking/tracker.rs` | `record_matched` / `record_unmatched` |
| `models/simulation.rs` | Delay sampling (rand/rand_distr), fault probability |

## ContextModel

`ContextModel` is the snapshot passed to both the template engine and the call tracker. It contains:

- `queryParams` — parsed query string (key → value map)
- `headers` — request headers (lowercased keys)
- `routeValues` — named captures from the regex match
- `body` — parsed JSON body (serde_json `Value`)
- `bodyRaw` — raw body string
- `state` — result of `setState` from the previous matched request on this route
- `previousContext` — full `ContextModel` of the previous matched request (nested `previousContext` stripped to prevent unbounded depth)
- `routeModel` — the matched `RouteDefinition`
