<div align="center">
  <img src="./assets/logo_full_19apr26.png" width=256 alt="Bamboozle Logo" >
  <h1>Bamboozle</h1>

  [![Startup Time](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/matt-andrews/Bamboozle/badges/badge-startup.json?v=1&cacheSeconds=3600&style=for-the-badge)](https://github.com/matt-andrews/Bamboozle/actions/workflows/startup-time.yml)
  [![Docker Image Size](https://img.shields.io/docker/image-size/mattisthegreatest/bamboozle?style=for-the-badge)](https://hub.docker.com/r/mattisthegreatest/bamboozle)
  ![Docker Image Version](https://img.shields.io/docker/v/mattisthegreatest/bamboozle?style=for-the-badge)
  ![GitHub License](https://img.shields.io/github/license/matt-andrews/Bamboozle?style=for-the-badge)

</div>

Bamboozle is a container-native HTTP mock server for integration testing. It runs as a self-contained Docker image your tests talk to over real HTTP — no in-process mocking, no monkey-patching, no SDK dependency in production code.

```bash
docker run -p 8080:8080 -p 9090:9090 mattisthegreatest/bamboozle
```

Your system under test calls `:8080`. Your test code configures and asserts via `:9090`.

---

## Documentation

| | |
|---|---|
| **[Tutorial — your first mock](docs/tutorials/first-mock.md)** | Start here. Register a route, call it, assert it was called. |
| **[How-to guides](docs/how-to/)** | Task-focused recipes for common testing scenarios. |
| **[Reference](docs/reference/)** | Route schema, API endpoints, expression syntax, environment variables. |
| **[Explanation](docs/explanation/)** | How the two-server model works, state chaining, matching priority. |

**How-to guides**

- [Manage routes](docs/how-to/manage-routes.md) — register, replace, list, delete
- [Write responses](docs/how-to/write-responses.md) — inline content, file responses, Liquid templates, loopback
- [Simulate faults](docs/how-to/simulate-faults.md) — latency injection, connection resets, transient failures
- [Assert on calls](docs/how-to/assert-calls.md) — count assertions, expression filters, call history
- [Load static config](docs/how-to/load-static-config.md) — JSON/YAML route files at startup
- [Configure logging](docs/how-to/configure-logging.md) — log levels, formats, OpenTelemetry export

---

## SDKs

[`@bamboozle/sdk`](sdks/README.md) — TypeScript/JavaScript client for the control API.

---

## Contributing

See [docs/contributing/](docs/contributing/) for architecture, request lifecycle, and how to add a feature.
