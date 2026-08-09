# Bamboozle integration and regression tests

This suite uses [Tempest](https://github.com/matt-andrews/Tempest) to test the
running Bamboozle binary entirely through HTTP. The suite covers both the
example mock APIs and Bamboozle's control plane; there is no language-specific
client or generated SDK to maintain.

The Compose runner builds the current Bamboozle source, loads every route in
[`../examples/routes`](../examples/routes), runs the specs serially, and tears
the stack down afterward.

## Run everything

Bash:

```bash
bash tests/run.sh
```

The suite intentionally ends with `suites/zz-destructive/reset.spec.yml`, which
validates `POST /control/reset`. Keep destructive global-state tests in that
directory so they execute after the ordinary suites.

## Run Tempest against an existing Bamboozle instance

The defaults in `local.env` target the example Compose ports (`18080` and
`19090`):

```bash
tempest test --path ./tests
```

Override the endpoints through Tempest's environment arguments when needed:

```bash
tempest test --path ./tests \
  -e BAMBOOZLE_MOCK_URI=http://localhost:8080 \
  -e BAMBOOZLE_CONTROL_URI=http://localhost:9090
```

To run only the non-destructive portion:

```bash
tempest test --path ./tests --run \
  suites/00-startup.spec.yml \
  suites/control \
  suites/examples
```

Specs within one file are sequential, and file concurrency is disabled in
`tempest.config.yml`. This is intentional because state chaining, `maxCalls`,
call history, and reset are shared server state.

## Layout

| Path | Purpose |
|---|---|
| `suites/00-startup.spec.yml` | Readiness, route loading, docs, and control metadata |
| `suites/examples/` | Executable contracts for every example API |
| `suites/control/` | Route lifecycle, validation, call history, and CEL assertions |
| `suites/zz-destructive/` | Global reset checks that must run last |
