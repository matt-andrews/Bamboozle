<div align="center">
  <img src="https://raw.githubusercontent.com/matt-andrews/Bamboozle/main/assets/logo_full_19apr26.png" width=256 alt="Bamboozle Logo">
  <h1>Bamboozle</h1>

  [![Docker Image Size](https://img.shields.io/docker/image-size/mattisthegreatest/bamboozle?style=for-the-badge)](https://hub.docker.com/r/mattisthegreatest/bamboozle)
  [![Startup Time](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/matt-andrews/Bamboozle/badges/badge-startup.json?v=1&cacheSeconds=3600&style=for-the-badge)](https://github.com/matt-andrews/Bamboozle/actions/workflows/startup-time.yml)
  [![Docker Image Version](https://img.shields.io/docker/v/mattisthegreatest/bamboozle?style=for-the-badge&sort=semver)](https://hub.docker.com/r/mattisthegreatest/bamboozle)
  ![GitHub License](https://img.shields.io/github/license/matt-andrews/Bamboozle?style=for-the-badge)
</div>

Bamboozle is a fast, lightweight out-of-process HTTP mock server for integration testing. Run it as a self-contained Docker container so your tests talk to it over real HTTP — exercising connection handling, timeouts, TLS, and request serialization exactly as they would against a real service.

## Why Bamboozle

- **Fast startup.** Sub-second cold start means CI runs that don't burn money waiting for mocks to warm up.
- **Tiny image.** ~5 MB vs hundreds of MB for alternatives.
- **Language-agnostic.** Drive it via HTTP from any language. Optional Node and .NET SDKs included for control interactions if needed.
- **Real HTTP boundaries — not in-process fakes.** Catches bugs that in-process mocking can't — connection handling, timeouts, TLS, request serialization.

## Who is this for?

- Engineers writing integration tests against external APIs
- Teams running CI pipelines where startup time matters
- Developers who want realistic HTTP behavior (timeouts, TLS, retries)

---

## Tutorial: Your first mock

```bash
docker run -p 8080:8080 -p 9090:9090 mattisthegreatest/bamboozle
```

Bamboozle runs two servers. Your system under test calls `:8080` (the mock surface). Your test code talks to `:9090` (the control API) to configure routes and assert behaviour.

### Register a route

```http
POST http://localhost:9090/control/routes
Content-Type: application/json

{
  "match": { "verb": "GET", "pattern": "/version" },
  "response": { "status": "200", "content": "1.0.0", "headers": { "Content-Type": "text/plain" } }
}
```

### Call the mock

```bash
curl http://localhost:8080/version
# 1.0.0
```

### Assert it was called

```http
POST http://localhost:9090/control/routes/GET/version/assert?called_exactly=1
Content-Type: application/json

{}
```

Returns `200 OK` on pass, `406 Not Acceptable` on failure.

### Tear down

```http
POST http://localhost:9090/control/reset
```

See the **[full tutorial](https://github.com/matt-andrews/Bamboozle/blob/main/docs/tutorials/first-mock.md)** for more detail.

---

## Tags

| Tag | Description |
|-----|-------------|
| `latest` | Most recent stable release |
| `vX.Y.Z` | Specific release version |
| `nightly` | Latest commit from main (may be unstable) |

---

## Documentation

| | |
|---|---|
| **[How-to guides](https://github.com/matt-andrews/Bamboozle/blob/main/docs/how-to/)** | Task-focused recipes for common testing scenarios. |
| **[Reference](https://github.com/matt-andrews/Bamboozle/blob/main/docs/reference/)** | Route schema, API endpoints, expression syntax, environment variables. |
| **[Explanation](https://github.com/matt-andrews/Bamboozle/blob/main/docs/explanation/)** | How the two-server model works, state chaining, matching priority. |

**How-to guides**

- [Manage routes](https://github.com/matt-andrews/Bamboozle/blob/main/docs/how-to/manage-routes.md) — register, replace, list, delete
- [Write responses](https://github.com/matt-andrews/Bamboozle/blob/main/docs/how-to/write-responses.md) — inline content, file responses, Liquid templates, loopback
- [Simulate faults](https://github.com/matt-andrews/Bamboozle/blob/main/docs/how-to/simulate-faults.md) — latency injection, connection resets, transient failures
- [Assert on calls](https://github.com/matt-andrews/Bamboozle/blob/main/docs/how-to/assert-calls.md) — count assertions, expression filters, call history
- [Load static config](https://github.com/matt-andrews/Bamboozle/blob/main/docs/how-to/load-static-config.md) — JSON/YAML route files at startup
- [Configure logging](https://github.com/matt-andrews/Bamboozle/blob/main/docs/how-to/configure-logging.md) — log levels, formats, OpenTelemetry export

---

## SDKs

| Download | Notes |
| - | - |
| `npm install @matt-andrews/bamboozle-sdk` | TypeScript/JavaScript client for the control API |
| `dotnet add package Bamboozle.Core` | dotnet client for the control API |

---

> **Warning:** Bamboozle was not intended to be used in any uncontrolled environment such as production or an environment that needs to be secure in any way. For testing purposes only.

## Source

[github.com/matt-andrews/Bamboozle](https://github.com/matt-andrews/Bamboozle)
