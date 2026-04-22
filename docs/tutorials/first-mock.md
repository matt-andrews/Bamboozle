# Your first mock

By the end of this you'll have a container running, a route registered, a mock response verified, and an assertion passing.

**Prerequisites:** Docker, `curl`.

---

## Start the container

```bash
docker run -p 8080:8080 -p 9090:9090 mattisthegreatest/bamboozle
```

Bamboozle runs two servers. Your system under test calls `:8080` (the mock surface). Your test code talks to `:9090` (the control API) to configure routes and assert behaviour.

Wait for the log line:

```
INFO bamboozle: mock listening on 0.0.0.0:8080
INFO bamboozle: control listening on 0.0.0.0:9090
```

---

## Register a route

```http
POST http://localhost:9090/control/routes
Content-Type: application/json

{
  "match": {
    "verb": "GET",
    "pattern": "/version"
  },
  "response": {
    "status": "200",
    "content": "1.0.0",
    "headers": { "Content-Type": "text/plain" }
  }
}
```

The route is active immediately. No restart needed.

---

## Call the mock

```bash
curl http://localhost:8080/version
```

```
1.0.0
```

Bamboozle records every request, whether it matches a route or not.

---

## Assert it was called

```http
POST http://localhost:9090/control/routes/GET/version/assert?called_exactly=1
Content-Type: application/json

{}
```

`200 OK` — the assertion passes. If the route hadn't been called, you'd get `418 I'm a Teapot`.

---

## Tear down

```http
POST http://localhost:9090/control/reset
```

All routes and call history are cleared. The container keeps running.

---

## Next steps

- [Write responses](../how-to/write-responses.md) — templates, file responses, loopback
- [Assert on calls](../how-to/assert-calls.md) — count qualifiers, expression filters
- [Manage routes](../how-to/manage-routes.md) — idempotent setup, bulk teardown
