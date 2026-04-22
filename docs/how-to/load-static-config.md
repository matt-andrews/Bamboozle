# Load static config

Routes can be loaded from JSON or YAML files at startup. This is useful for routes that are constant across all tests — health checks, version endpoints, shared stubs.

## Configuration

| Variable | Default | Description |
|---|---|---|
| `ROUTE_CONFIG_FOLDERS` | `[]` | JSON array of directory paths to scan |
| `ROUTE_CONFIG_THROW_ON_ERROR` | `false` | Fail startup if any file can't be parsed |

```bash
docker run \
  -e 'ROUTE_CONFIG_FOLDERS=["/etc/bamboozle/routes"]' \
  -e 'ROUTE_CONFIG_THROW_ON_ERROR=true' \
  -v ./routes:/etc/bamboozle/routes \
  -p 8080:8080 -p 9090:9090 \
  mattisthegreatest/bamboozle
```

Files with `.json`, `.yaml`, or `.yml` extensions are loaded automatically. The mock listener doesn't start until all files are processed, so statically configured routes are always active before the container becomes reachable.

## YAML format

```yaml
routes:
  - match:
      verb: GET
      pattern: /health
    response:
      status: "200"
      content: ok
      headers:
        Content-Type: text/plain

  - match:
      verb: GET
      pattern: /version
    response:
      status: "200"
      content: "1.0.0"
      headers:
        Content-Type: text/plain
```

## JSON format

```json
{
  "routes": [
    {
      "match": { "verb": "GET", "pattern": "/health" },
      "response": {
        "status": "200",
        "content": "ok",
        "headers": { "Content-Type": "text/plain" }
      }
    }
  ]
}
```

Static routes support all the same features as runtime-registered routes: Liquid templates, typed route parameters, simulation, and `setState`.

---

**See also:** [Route definition reference](../reference/route-definition.md) · [Environment variables reference](../reference/environment-variables.md)
