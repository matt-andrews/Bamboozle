# Configuration

Bamboozle is configured with environment variables. Defaults are suitable for a local container with no preloaded routes.

| Variable | Default | Purpose |
|---|---|---|
| `ROUTE_CONFIG_FOLDERS` | `[]` | JSON array of folders containing `.json`, `.yaml`, or `.yml` route files |
| `ROUTE_CONFIG_THROW_ON_ERROR` | `false` | Fail startup when a route file cannot be loaded |
| `MAX_ROUTES` | `1000` | Maximum number of configured routes |
| `MAX_CONTENT_SIZE_BYTES` | `10485760` | Maximum inline response size in bytes |
| `TLS_CERT_FILE` | unset | PEM certificate chain for HTTPS on the mock surface |
| `TLS_KEY_FILE` | unset | PEM private key; must be set with `TLS_CERT_FILE` |
| `RUST_LOG` | `info` | Standard tracing filter, such as `debug` or `error` |
| `RUST_LOG_FORMAT` | `compact` | `compact`, `pretty`, or `json` |
| `NO_COLOR` | unset | Set to any value to disable ANSI color |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | unset | OTLP HTTP endpoint for builds with the `otel` feature |
| `OTEL_EXPORTER_OTLP_HEADERS` | unset | Comma-separated OTLP request headers |

## Static routes

Mount one or more route folders and pass their container paths as a JSON array:

```bash
docker run \
  -e 'ROUTE_CONFIG_FOLDERS=["/routes"]' \
  -e ROUTE_CONFIG_THROW_ON_ERROR=true \
  -v ./routes:/routes \
  -p 8080:8080 -p 9090:9090 \
  mattisthegreatest/bamboozle
```

Static files use the same route definitions as the control API. See [`examples/routes`](../examples/routes) for executable JSON and YAML configurations.

## Logging and telemetry

Set `RUST_LOG=debug` for route-matching and assertion diagnostics, or `RUST_LOG_FORMAT=json` for structured logs. OpenTelemetry export is available only in custom builds compiled with `--features otel`; setting `OTEL_EXPORTER_OTLP_ENDPOINT` activates it.

## TLS

The Docker image includes optional HTTPS support for the mock surface. Generate local certificates:

```bash
docker run --rm -v ./certs:/certs \
  mattisthegreatest/bamboozle generate-certs --out /certs
```

Then mount the certificate and key:

```bash
docker run \
  -v ./certs:/certs \
  -e TLS_CERT_FILE=/certs/cert.pem \
  -e TLS_KEY_FILE=/certs/key.pem \
  -p 8080:8080 -p 9090:9090 \
  mattisthegreatest/bamboozle
```

The mock surface is then available over HTTPS on `:8080`; the control API remains HTTP on `:9090`. Trust `certs/ca.crt` in the client environment when certificate verification is required, and keep `key.pem` private.
