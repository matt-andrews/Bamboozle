<div align="center">
  <img src="https://raw.githubusercontent.com/matt-andrews/Bamboozle/main/.assets/logo_full_19apr26.png" width=256 alt="Bamboozle Logo">
  <h1>Bamboozle</h1>

  [![Docker Image Size](https://img.shields.io/docker/image-size/mattisthegreatest/bamboozle?style=for-the-badge)](https://hub.docker.com/r/mattisthegreatest/bamboozle)
  [![Docker Image Version](https://img.shields.io/docker/v/mattisthegreatest/bamboozle?style=for-the-badge&sort=semver)](https://hub.docker.com/r/mattisthegreatest/bamboozle)
  [![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=for-the-badge)](#license)
</div>

Bamboozle is a fast, lightweight HTTP mock server for integration tests and CI. It runs out of process, works with any language, and exercises real HTTP behavior instead of replacing your application's client code.

It provides:

- Separate mock (`:8080`) and control (`:9090`) HTTP surfaces
- Routes registered at runtime or loaded from JSON and YAML files
- Templated, file-based, binary, and loopback responses
- Call recording, assertions, typed route parameters, state, latency, and fault simulation
- Optional TLS and OpenTelemetry support
- Very fast (~10ms) startup time and very small (~5mb) image size

## Quick start

Start the container:

```bash
docker run -p 8080:8080 -p 9090:9090 mattisthegreatest/bamboozle
```

Register a route through the control API:

```http
POST http://localhost:9090/control/routes
Content-Type: application/json

{
  "match": { "verb": "GET", "pattern": "/version" },
  "response": {
    "status": "200",
    "content": "1.0.0",
    "headers": { "Content-Type": "text/plain" }
  }
}
```

Call the mock surface:

```bash
curl http://localhost:8080/version
```

The response is `1.0.0`. Bamboozle records the call so your test can inspect or assert on it through the control API.

## Learn more

- Open `http://localhost:9090/` while Bamboozle is running for the interactive API reference.
- See [configuration](./docs/configuration.md) for static routes, logging, limits, and TLS.
- Run the [executable examples](./examples) for realistic API shapes and advanced features.
- See the compact [expression reference](./docs/expression-syntax.md) for filtering recorded calls.

## Project status

Bamboozle is pre-1.0 and its APIs may change. It is intended only for controlled testing environments, not production or security-sensitive use.

See [CONTRIBUTING.md](./CONTRIBUTING.md) to build, test, or change the project.


## License

Bamboozle is available under either of the following licenses, at your option:

- [Apache License 2.0](https://github.com/matt-andrews/Bamboozle/blob/main/LICENSE-APACHE)
- [MIT License](https://github.com/matt-andrews/Bamboozle/blob/main/LICENSE-MIT)
