# Environment variables

All variables Bamboozle reads at startup.

| Variable | Default | Description |
|---|---|---|
| `ROUTE_CONFIG_FOLDERS` | `[]` | JSON array of directory paths to scan for `.json`, `.yaml`, `.yml` route files |
| `ROUTE_CONFIG_THROW_ON_ERROR` | `false` | When `true`, fail startup if any route file can't be parsed |
| `TLS_CERT_FILE` | unset | Path to a PEM-encoded TLS certificate file — setting this (with `TLS_KEY_FILE`) enables HTTPS on the mock port |
| `TLS_KEY_FILE` | unset | Path to a PEM-encoded TLS private key file — must be set together with `TLS_CERT_FILE` |
| `RUST_LOG` | `info` | Log level filter — standard `env_logger` syntax (`trace`, `debug`, `info`, `warn`, `error`) |
| `RUST_LOG_FORMAT` | `compact` | Log output format: `compact`, `pretty`, or `json` |
| `NO_COLOR` | unset | Set to any value to strip ANSI colour codes from log output |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | unset | OTLP HTTP endpoint — setting this activates the exporter |
| `OTEL_EXPORTER_OTLP_HEADERS` | unset | Comma-separated `key=value` pairs added as HTTP headers on OTLP requests (used for authentication) |

## Related guides

- [Enable TLS](../how-to/enable-tls.md) — `TLS_CERT_FILE`, `TLS_KEY_FILE`
- [Load static config](../how-to/load-static-config.md) — `ROUTE_CONFIG_*` variables
- [Configure logging](../how-to/configure-logging.md) — `RUST_LOG*`, `NO_COLOR`, OTLP variables

