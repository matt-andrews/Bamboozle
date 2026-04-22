# Write responses

## Inline content

The simplest response: a static string in `content`.

```json
{
  "match": { "verb": "GET", "pattern": "/status" },
  "response": {
    "status": "200",
    "content": "ok",
    "headers": { "Content-Type": "text/plain" }
  }
}
```

`content` is a [Liquid](https://shopify.github.io/liquid/) template. The following variables are available from the matched request:

| Variable | Value |
|---|---|
| `{{ routeValues.key }}` | Captured route parameter |
| `{{ queryParams.key }}` | Query string parameter |
| `{{ headers.key }}` | Request header (case-insensitive key) |
| `{{ body.key }}` | Top-level JSON body field |
| `{{ bodyRaw }}` | Raw request body as a string |
| `{{ state }}` | Value set by `setState` on the previous request |
| `{{ previousContext }}` | Full context snapshot of the previous matched request |

Template variables work in `content`, `contentFile`, response `headers`, `status`, and `setState`.

## Route parameters

Patterns can include typed parameters captured into `routeValues`:

| Syntax | Matches |
|---|---|
| `{id}` | Any path segment |
| `{id:int}` | Integers only |
| `{id:guid}` | Valid GUIDs only |
| `{slug?}` | Optional segment — route matches with or without it |

Full constraint list: `int`, `long`, `double`, `decimal`, `float`, `bool`, `guid`, `alpha`, `datetime`.

```json
{
  "match": { "verb": "GET", "pattern": "/orders/{id:int}" },
  "response": {
    "status": "200",
    "content": "{ \"orderId\": {{ routeValues.id }} }"
  }
}
```

`GET /orders/42` → `{ "orderId": 42 }`. `GET /orders/abc` → 404.

## File responses (text)

`contentFile` is a drop-in for `content`. The file is read at request time and rendered through the same Liquid engine — all template variables work.

```json
{
  "match": { "verb": "GET", "pattern": "/greet/{name}" },
  "response": {
    "status": "200",
    "contentFile": "/etc/bamboozle/templates/greeting.txt"
  }
}
```

`greeting.txt`:

```
Hello {{ routeValues.name }}!
```

`GET /greet/World` → `Hello World!`

## File responses (binary)

`binaryFile` serves raw bytes with no template processing. Use it for images, PDFs, archives.

```json
{
  "match": { "verb": "GET", "pattern": "/logo.png" },
  "response": {
    "status": "200",
    "headers": { "Content-Type": "image/png" },
    "binaryFile": "/etc/bamboozle/assets/logo.png"
  }
}
```

## Loopback

`loopback: true` echoes the request body back as the response body. Useful for verifying your client sends the right payload.

```json
{
  "match": { "verb": "POST", "pattern": "/echo" },
  "response": { "status": "200", "loopback": true }
}
```

## Mounting files into the container

File paths resolve inside the container. Mount a host directory as a volume:

```bash
docker run \
  -v ./assets:/etc/bamboozle/assets \
  -p 8080:8080 -p 9090:9090 \
  mattisthegreatest/bamboozle
```

If a file can't be read at request time, the mock returns `500` and logs the path and error.

## Mutual exclusion

Each route may use at most one of `content`, `contentFile`, `binaryFile`, or `loopback`. Specifying more than one returns `400 Bad Request` when the route is registered.

---

**See also:** [Route definition reference](../reference/route-definition.md) · [State chaining](../explanation/state-chaining.md)
