<div align="center">
  <img src="https://raw.githubusercontent.com/matt-andrews/Bamboozle/main/.assets/logo_full_19apr26.png" width=256 alt="Bamboozle Logo" >
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
- Language-agnostic. Drive it directly via HTTP from any language.
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

Bamboozle runs two servers. Your system under test calls `:8080` (the mock surface). Your test code talks to `:9090` (the control API) to configure routes and assert behavior.

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

> [!NOTE]
> Slashes in the `match.pattern` are automatically trimmed from the ends when setting the route to simplify matching and assertions.

The route is active immediately.

Depending on your workflow, you may want to use [static route configuration files](https://github.com/matt-andrews/Bamboozle/blob/main/docs/how-to/load-static-config.md).

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

> [!NOTE]
> Your route pattern must be url encoded.

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
| **[How-to guides](https://github.com/matt-andrews/Bamboozle/tree/main/docs/how-to)** | Task-focused recipes for common testing scenarios. |
| **[Reference](https://github.com/matt-andrews/Bamboozle/tree/main/docs/reference)** | Route schema, API endpoints, expression syntax, environment variables. |
| **[Explanation](https://github.com/matt-andrews/Bamboozle/tree/main/docs/explanation)** | How the two-server model works, state chaining, matching priority. |

### How-to guides

- [Manage routes](https://github.com/matt-andrews/Bamboozle/blob/main/docs/how-to/manage-routes.md) — register, replace, list, delete
- [Write responses](https://github.com/matt-andrews/Bamboozle/blob/main/docs/how-to/write-responses.md) — inline content, file responses, Liquid templates, loopback
- [Simulate faults](https://github.com/matt-andrews/Bamboozle/blob/main/docs/how-to/simulate-faults.md) — latency injection, connection resets, transient failures
- [Assert on calls](https://github.com/matt-andrews/Bamboozle/blob/main/docs/how-to/assert-calls.md) — count assertions, expression filters, call history
- [Load static config](https://github.com/matt-andrews/Bamboozle/blob/main/docs/how-to/load-static-config.md) — JSON/YAML route files at startup
- [Configure logging](https://github.com/matt-andrews/Bamboozle/blob/main/docs/how-to/configure-logging.md) — log levels, formats, OpenTelemetry export
- [Enable TLS](https://github.com/matt-andrews/Bamboozle/blob/main/docs/how-to/enable-tls.md) — HTTPS on the mock server, certificate generation

---

## Disclaimers

Bamboozle is currently in an `alpha` state for as long as the major version is `0`. We are making a best effort to ensure the major functionality and APIs remain consistent, while leaving room for major refactors if absolutely necessary before `1.0`.

Bamboozle was **not** intended to be used in any uncontrolled environment such as production, or in any environment that needs to be secure in any way. It is intended for testing purposes only.

## Try it in your project

- Run the example above in your local environment
- See [docs/contributing/](https://github.com/matt-andrews/Bamboozle/tree/main/docs/contributing) for architecture, request lifecycle, and how to add a feature.

If it clicks, ⭐ star the repo — it helps others find it.
