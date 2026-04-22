# Simulate faults

Add a `simulation` object to any route to inject latency or failures. Routes without a `simulation` field behave normally.

## Delay

### Fixed

Always delays by exactly `ms` milliseconds.

```json
{
  "match": { "verb": "GET", "pattern": "/orders" },
  "response": { "status": "200", "content": "[]" },
  "simulation": {
    "delay": { "type": "fixed", "ms": 250 }
  }
}
```

### Random

Uniform random delay in the range `[minMs, maxMs]`.

```json
"delay": { "type": "random", "minMs": 100, "maxMs": 800 }
```

### Gaussian

Normally-distributed delay centred on `meanMs`, clamped to 0. Useful for realistic latency modelling.

```json
"delay": { "type": "gaussian", "meanMs": 300, "stdDevMs": 80 }
```

Delay is async — the waiting task yields to the executor, so other routes serve concurrently and no threads block.

## Faults

### Connection reset

Sends response headers then abruptly closes the connection. The client sees a broken-pipe or connection-reset error. Tests retry logic and circuit breakers.

```json
{
  "match": { "verb": "POST", "pattern": "/payments" },
  "response": { "status": "200" },
  "simulation": {
    "fault": { "type": "connectionReset" }
  }
}
```

### Empty response

Returns `200 OK` with an empty body.

```json
"fault": { "type": "emptyResponse" }
```

## Transient faults

Add `probability` (0.0–1.0) to make a fault intermittent. The default is `1.0` (always).

```json
"fault": { "type": "connectionReset", "probability": 0.1 }
```

10% of requests fail; the other 90% return normally. Useful for chaos-style tests where only a fraction of calls should fail.

## Combining delay and fault

Both fields may be set together. Delay always fires first, then the fault check runs.

```json
{
  "match": { "verb": "GET", "pattern": "/inventory" },
  "response": { "status": "200", "content": "[]" },
  "simulation": {
    "delay": { "type": "gaussian", "meanMs": 300, "stdDevMs": 80 },
    "fault": { "type": "emptyResponse", "probability": 0.25 }
  }
}
```

---

**See also:** [Route definition reference](../reference/route-definition.md)
