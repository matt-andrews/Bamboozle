<div align="center">
  <img src="./assets/logo_full_19apr26.png" width=256 alt="Bamboozle Logo" >
  <h1>Bamboozle</h1>

  [![Startup Time](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/matt-andrews/Bamboozle/badges/badge-startup.json?v=1&cacheSeconds=3600&style=for-the-badge)](https://github.com/matt-andrews/Bamboozle/actions/workflows/startup-time.yml)
  [![Docker Image Size](https://img.shields.io/docker/image-size/mattisthegreatest/bamboozle?style=for-the-badge)](https://hub.docker.com/r/mattisthegreatest/bamboozle)
  ![Docker Image Version](https://img.shields.io/docker/v/mattisthegreatest/bamboozle?style=for-the-badge&sort=semver)
  ![GitHub License](https://img.shields.io/github/license/matt-andrews/Bamboozle?style=for-the-badge)

</div>

Bamboozle is a container-native HTTP mock server for integration testing. It runs as a self-contained Docker image so that your tests talk to it over real HTTP, and you can assert with confidence that your application is making the expected HTTP requests.

```bash
docker run -p 8080:8080 -p 9090:9090 mattisthegreatest/bamboozle
```

Your system under test calls `:8080`. Your test code configures and asserts via `:9090`.

> [!CAUTION]
> DO NOT use this in any uncontrolled or production environment! This is 100% not safe in any way shape or form against the internet. For testing purposes only! You have been warned!

---

## Documentation

| | |
|---|---|
| **[Tutorial: your first mock](docs/tutorials/first-mock.md)** | Start here. Register a route, call it, assert it was called. |
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
- [Enable TLS](docs/how-to/enable-tls.md) — HTTPS on the mock server, certificate generation
- [Fault Tolerance Example](examples/fault-demo/README.md) — Example: simulated faults including latency injection, connection resets, and transient failures

---

## SDKs

| Download | Code | Notes |
| - | - | - |
| `npm install @matt-andrews/bamboozle-sdk` | [`Node`](sdks/npm/README.md) | TypeScript/JavaScript client for the control API |
| `dotnet add package Bamboozle.Core` | [`dotnet`](sdks/dotnet/Bamboozle/) | dotnet client for the control API |

---

## Contributing

See [docs/contributing/](docs/contributing/) for architecture, request lifecycle, and how to add a feature.
