# Configure logging

## Log level

`RUST_LOG` accepts standard `env_logger` filter syntax.

```bash
# default
RUST_LOG=info

# show route matching and expression evaluation
RUST_LOG=debug

# errors only
RUST_LOG=error
```

## Log format

| Variable | Default | Options |
|---|---|---|
| `RUST_LOG_FORMAT` | `compact` | `compact`, `pretty`, `json` |
| `NO_COLOR` | unset | Set to any value to disable ANSI colour |

`compact` is readable in a terminal. `pretty` adds indentation for easier scanning during local development. `json` emits one JSON object per line.

## Structured JSON for log shippers

```bash
docker run -e RUST_LOG_FORMAT=json -p 8080:8080 -p 9090:9090 mattisthegreatest/bamboozle
```

Compatible with [Vector](https://vector.dev), [Promtail](https://grafana.com/docs/loki/latest/send-data/promtail/), [Fluent Bit](https://fluentbit.io), and any shipper that consumes newline-delimited JSON.

## OpenTelemetry OTLP export

> [!WARNING]
> Currently OTEL libraries are behind a feature flag. You must rebuild the docker image with `--features otel` to access this functionality

Set `OTEL_EXPORTER_OTLP_ENDPOINT` to send traces to any OTLP-compatible backend. Console output remains active alongside it.

| Variable | Description |
|---|---|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OTLP HTTP endpoint — setting this activates the exporter |
| `OTEL_EXPORTER_OTLP_HEADERS` | Comma-separated `key=value` pairs sent as HTTP headers (authentication) |

**Grafana Cloud**

```bash
docker run \
  -e OTEL_EXPORTER_OTLP_ENDPOINT=https://otlp-gateway-<zone>.grafana.net/otlp \
  -e 'OTEL_EXPORTER_OTLP_HEADERS=Authorization=Basic <base64-token>' \
  -p 8080:8080 -p 9090:9090 \
  mattisthegreatest/bamboozle
```

**New Relic**

```bash
docker run \
  -e OTEL_EXPORTER_OTLP_ENDPOINT=https://otlp.nr-data.net \
  -e 'OTEL_EXPORTER_OTLP_HEADERS=Api-Key=<your-license-key>' \
  -p 8080:8080 -p 9090:9090 \
  mattisthegreatest/bamboozle
```

## What gets logged

| Event | Level | Details |
|---|---|---|
| Unmatched request | `warn` | Verb, path, up to 3 fuzzy-matched route suggestions |
| Assertion failed | `warn` | Condition, matched count, total calls, expression |
| Assertion passed | `debug` | Matched count, expression |
| Expression error | `debug` | Expression string and specific evaluation failure |
| Route registered / deleted | `info` | Route key |
| Startup | `info` | Listening addresses |

---

**See also:** [Environment variables reference](../reference/environment-variables.md)
