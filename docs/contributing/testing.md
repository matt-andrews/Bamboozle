# Testing

## Unit tests

Tests live in the same file as the code they test, inside a `#[cfg(test)]` block. The test binary is only compiled when running `cargo test`.

```bash
cargo test                      # run all tests
cargo test tracker              # run tests whose names contain "tracker"
cargo test -- --nocapture       # show println! output
```

Most modules have a `make_ctx()` or `make_route()` helper at the top of the test block. Use these instead of spelling out all fields — they'll stay valid as the types evolve.

## Handler tests

Handler tests in `mock_server.rs` go through the real axum router using `tower::ServiceExt::oneshot`. The full middleware stack, body parsing, and state extraction run — no network socket required.

```rust
let response = router(AppState::new())
    .oneshot(Request::builder().uri("/things").body(Body::empty()).unwrap())
    .await
    .unwrap();
assert_eq!(response.status(), StatusCode::OK);
```

To test a route that needs a pre-registered mock route, insert into the store before calling `oneshot`:

```rust
let state = AppState::new();
state.store.set_route(make_route()).unwrap();

let response = router(state)
    .oneshot(Request::builder().uri("/things/42").body(Body::empty()).unwrap())
    .await
    .unwrap();
```

## Playwright E2E tests

End-to-end tests in `/playwright/` start a real container and drive it over HTTP.

```bash
cd playwright
npm install
npx playwright test
```

Tests are grouped by feature area:

| File | Covers |
|---|---|
| `assert.spec.ts` | Assertion expressions and count qualifiers |
| `file-response.spec.ts` | `contentFile`, `binaryFile` |
| `previous-context.spec.ts` | `setState`, `previousContext` state chaining |
| `response.spec.ts` | Liquid templating, loopback |
| `simulation.spec.ts` | Delay distributions, fault injection |

The tests use `@bamboozle/sdk` for the control API calls. See `playwright/tests/*.spec.ts` for patterns.
