<div align="center">
  <img src="./assets/logo_full_19apr26.png" width=256 alt="Bamboozle Logo" >
  <h1>Bamboozle</h1>

  [![Docker Image Size](https://img.shields.io/docker/image-size/mattisthegreatest/bamboozle?style=for-the-badge)](https://hub.docker.com/r/mattisthegreatest/bamboozle)
  [![Startup Time](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/matt-andrews/Bamboozle/badges/badge-startup.json?v=1&cacheSeconds=3600&style=for-the-badge)](https://github.com/matt-andrews/Bamboozle/actions/workflows/startup-time.yml)
  [![Docker Image Version](https://img.shields.io/docker/v/mattisthegreatest/bamboozle?style=for-the-badge&sort=semver)](https://hub.docker.com/r/mattisthegreatest/bamboozle)
  ![GitHub License](https://img.shields.io/github/license/matt-andrews/Bamboozle?style=for-the-badge)

</div>

Bamboozle is a fast, lightweight out-of-process HTTP mock server designed for realistic mocking in CI and local testing where it can be cumbersome to load up entire ecosystems.

## Why Bamboozle

- Fast startup. Sub-second cold start means CI runs that don't burn money waiting for mocks to warm up.
- Tiny image. ~5MB vs hundreds of MB for alternatives.
- Language-agnostic. Drive it via HTTP from any language. Optional Node and .NET SDKs included for control interactions if needed.
- Test against real HTTP boundaries — not in-process fakes. Catches bugs that in-process mocking can't — connection handling, timeouts, TLS, request serialization.

### Who is this for?

- Engineers writing integration tests against external APIs
- Teams running CI pipelines where startup time matters
- Developers who want realistic HTTP behavior (timeouts, TLS, retries)

---

## Tutorial: Your first mock

```bash
docker run -p 8080:8080 -p 9090:9090 mattisthegreatest/bamboozle
```

Bamboozle runs two servers. Your system under test calls `:8080` (the mock surface). Your test code talks to `:9090` (the control API) to configure routes and assert behaviour.

Wait for the log line:

```log
INFO bamboozle: mock listening on 0.0.0.0:8080
INFO bamboozle: control listening on 0.0.0.0:9090
```

---

### Register a route

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

The route is active immediately.

Depending on your workflow, you may want to use [static route configuration files](docs/how-to/load-static-config.md).

Routes use the [Liquid Template Engine](https://shopify.github.io/liquid/) for dynamic rendering for any string in the `response` section.

---

### Call the mock

```bash
curl http://localhost:8080/version
```

Because of the route definition above, you will get the following response:

```curl
1.0.0
```

---

### Assert it was called

You can assert on any verb + route pattern combination, and there are various options to configure your assertions.

```http
POST http://localhost:9090/control/routes/GET/version/assert?called_exactly=1
Content-Type: application/json

{}
```

In this case there are two expected results:

- `200 OK` - the assertion passes. The route pattern matched the incoming requests; `called_exactly=1` means it was only recorded once.
- `406 Not Acceptable` - the assertion fails. The route pattern *did not* match the incoming requests **or** it was recorded more than once.

---

### Tear down

```http
POST http://localhost:9090/control/reset
```

All routes and call history are cleared.

---

## Documentation

| | |
| --- | --- |
| **[How-to guides](docs/how-to/)** | Task-focused recipes for common testing scenarios. |
| **[Reference](docs/reference/)** | Route schema, API endpoints, expression syntax, environment variables. |
| **[Explanation](docs/explanation/)** | How the two-server model works, state chaining, matching priority. |

### How-to guides

- [Manage routes](docs/how-to/manage-routes.md) — register, replace, list, delete
- [Write responses](docs/how-to/write-responses.md) — inline content, file responses, Liquid templates, loopback
- [Simulate faults](docs/how-to/simulate-faults.md) — latency injection, connection resets, transient failures
- [Assert on calls](docs/how-to/assert-calls.md) — count assertions, expression filters, call history
- [Load static config](docs/how-to/load-static-config.md) — JSON/YAML route files at startup
- [Configure logging](docs/how-to/configure-logging.md) — log levels, formats, OpenTelemetry export
- [Enable TLS](docs/how-to/enable-tls.md) — HTTPS on the mock server, certificate generation
- [Fault Tolerance Example](examples/fault-demo/README.md) — Example: simulated faults including latency injection, connection resets, and transient failures

---

### SDKs

| Download | Code | Notes |
| - | - | - |
| `npm install @matt-andrews/bamboozle-sdk` | [`Node`](sdks/npm/README.md) | TypeScript/JavaScript client for the control API |
| `dotnet add package Bamboozle.Core` | [`dotnet`](sdks/dotnet/Bamboozle/) | dotnet client for the control API |

---

## Disclaimers

Bamboozle is currently in an `alpha` state for as long as the major version is `0`. We are making best effort to ensure the major functionality and API's remain consistent, but are leaving us room to make major refactors if absolutely necessary before `1.0`.

Bamboozle was **not** intended to be used in any uncontrolled environment such as production or an environment that needs to be secure in any way. This was only intended to be used for testing purposes.

## Try it in your project

- Run the example above in your local environment
- Check out the [Fault Tolerance Example](examples/fault-demo/README.md) for advanced scenarios
- See [docs/contributing/](docs/contributing/) for architecture, request lifecycle, and how to add a feature.

If it clicks, ⭐ star the repo — it helps others find it.
