# State chaining

Bamboozle can carry state forward across successive calls to the same route. This lets a single route return different responses depending on how many times it has been called — without registering multiple routes.

## How it works

A route can have a `setState` field — a Liquid template evaluated after each matched request. The result is stored per-route. On the next request to the same route, that value is available as `{{ state }}` in response templates, and the full context snapshot of the previous request is available as `{{ previousContext }}`.

```json
{
  "match": { "verb": "GET", "pattern": "/resource" },
  "setState": "{% if previousContext == nil %}1{% else %}{% assign n = previousContext.state | plus: 1 %}{{ n }}{% endif %}",
  "response": {
    "status": "200",
    "content": "call number {{ state }}"
  }
}
```

First call: `previousContext` is nil, so `setState` produces `"1"`. The response renders with `state = ""` (not yet set for this call).  
Second call: `previousContext.state` is `"1"`, so `setState` produces `"2"`. The response renders with `state = "1"`.

The state written by request N is the state read by request N+1.

## Varying the status code

Template variables work in `status` as well as `content`:

```json
{
  "match": { "verb": "GET", "pattern": "/idempotent-create" },
  "response": {
    "status": "{% if previousContext != nil %}409{% else %}201{% endif %}",
    "content": "ok"
  }
}
```

First call returns `201`. Every subsequent call returns `409`.

## Carrying state in response headers

```json
{
  "match": { "verb": "GET", "pattern": "/counter" },
  "setState": "{% if previousContext == nil %}1{% else %}{% assign n = previousContext.state | plus: 1 %}{{ n }}{% endif %}",
  "response": {
    "status": "200",
    "headers": { "x-call-count": "{{ previousContext.state }}" },
    "content": "ok"
  }
}
```

The `x-call-count` header on request N reflects the state stored by request N-1.

## Depth limit

`previousContext` holds a snapshot of the previous call's context, including its own `previousContext`. To prevent unbounded chain depth, Bamboozle strips the nested `previousContext` from the stored snapshot — so `previousContext.previousContext` is always nil.

## When to use this vs separate routes

State chaining is best for sequences where the number of expected calls is small and fixed — first call creates, second call conflicts, third call is unexpected. If you need complex conditional logic or more than three steps, registering separate routes with `DELETE` between them is usually clearer.

State is stored per-route key (verb + pattern), not per-session. Parallel tests against a shared Bamboozle instance will share state on overlapping routes. Use `POST /control/reset` between tests, or run a dedicated container per test suite.
