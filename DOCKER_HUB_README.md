<div align="center">
  <img src="https://raw.githubusercontent.com/matt-andrews/Bamboozle/main/assets/logo_full_19apr26.png" width=256 alt="Bamboozle Logo">
  <h1>Bamboozle</h1>

  [![Startup Time](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/matt-andrews/Bamboozle/badges/badge-startup.json?v=1&cacheSeconds=3600&style=for-the-badge)](https://github.com/matt-andrews/Bamboozle/actions/workflows/startup-time.yml)
  [![Docker Image Size](https://img.shields.io/docker/image-size/mattisthegreatest/bamboozle?style=for-the-badge)](https://hub.docker.com/r/mattisthegreatest/bamboozle)
  ![Docker Image Version](https://img.shields.io/docker/v/mattisthegreatest/bamboozle?style=for-the-badge)
  ![GitHub License](https://img.shields.io/github/license/matt-andrews/Bamboozle?style=for-the-badge)
</div>

Bamboozle is a container-native HTTP mock server for integration testing. It runs as a self-contained Docker image so that your tests talk to it over real HTTP, and you can assert with confidence that your application is making the expected HTTP requests.

```bash
docker run -p 8080:8080 -p 9090:9090 mattisthegreatest/bamboozle
```

Your system under test calls `:8080`. Your test code configures and asserts via `:9090`.

> **Warning:** Do NOT use this in any uncontrolled or production environment. This is for testing purposes only.

---

## Quick start

1. Start the container
2. Register a route on the control port (`:9090`)
3. Make your application call the mock port (`:8080`)
4. Assert the call was received

See the **[Tutorial: your first mock](https://github.com/matt-andrews/Bamboozle/blob/main/docs/tutorials/first-mock.md)** for a full walkthrough.

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
| **[Tutorial: your first mock](https://github.com/matt-andrews/Bamboozle/blob/main/docs/tutorials/first-mock.md)** | Start here. Register a route, call it, assert it was called. |
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

| Language | Notes |
|----------|-------|
| [Node.js / TypeScript](https://github.com/matt-andrews/Bamboozle/blob/main/sdks/npm/README.md) | TypeScript/JavaScript client for the control API *(WIP)* |

---

## Source

[github.com/matt-andrews/Bamboozle](https://github.com/matt-andrews/Bamboozle)
