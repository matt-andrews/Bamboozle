# Bamboozle example APIs

This directory is an executable catalogue of API mocks. Each file under
[`routes`](./routes) is a standalone static Bamboozle configuration, and the
included Compose stack loads all of them into one server.

The examples deliberately use recognizable API shapes rather than toy `/foo`
routes. They are safe, local substitutes for exploring client serialization,
authentication headers, retries, error handling, webhooks, and stateful flows.

## Run the catalogue

From the repository root:

```bash
docker compose -f examples/docker-compose.yml up --build
```

The two Bamboozle surfaces are then available at:

| Surface | URL |
|---|---|
| Mock APIs | `http://localhost:18080` |
| Control API and Scalar UI | `http://localhost:19090` |

Try a few examples:

```bash
curl -H "x-request-id: demo-123" http://localhost:18080/hello/Ada

curl -H "Authorization: Bearer demo" \
  http://localhost:18080/repos/octocat/Hello-World

curl -X POST http://localhost:18080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"gpt-bamboozle","messages":[{"role":"user","content":"Say hello"}]}'

curl -X POST http://localhost:18080/graphql \
  -H "Content-Type: application/json" \
  -d '{"operationName":"GetUser","variables":{"login":"octocat"},"query":"query GetUser { user { login } }"}'
```

Stop the server with `Ctrl+C`, or run:

```bash
docker compose -f examples/docker-compose.yml down
```

## Catalogue

| File | API style | Bamboozle capabilities demonstrated |
|---|---|---|
| `00-basics.yml` | General HTTP | Inline responses, query/header/body templates, loopback, text and binary files, multiple verbs |
| `github.yml` | GitHub-like REST | Nested resources, typed IDs, auth-aware status codes, response headers |
| `stripe.yml` | Stripe-like payments | JSON request projection, idempotency headers, create/retrieve/refund flows |
| `openai.yml` | OpenAI-compatible | Nested request bodies, model routes, embeddings, SSE-shaped responses |
| `graphql.yml` | GraphQL | Operation dispatch from one endpoint and nested variables |
| `oauth.yml` | OAuth/OIDC | Discovery documents, form-body inspection, bearer authorization |
| `s3.yml` | S3-like object storage | XML, content files, binary files, and multi-verb object routes |
| `webhooks.yml` | Webhook receivers | Loopback, provider payloads, templated acknowledgements |
| `resilience.yml` | Unreliable dependencies | Fixed/random/Gaussian latency, faults, state chaining, `maxCalls` |
| `routing.yml` | Typed routing | All constraints, optional segments, static and constrained-route precedence |

The examples use repository-relative file paths. Run Bamboozle with the
repository root as its working directory when loading them outside Docker:

```bash
ROUTE_CONFIG_FOLDERS='["examples/routes"]' \
ROUTE_CONFIG_THROW_ON_ERROR=true \
cargo run --manifest-path bamboozle/Cargo.toml
```

Compose uses the same `18080`/`19090` host ports as the existing development
stack to avoid occupying the conventional `8080` port. Override them when
needed with `BAMBOOZLE_MOCK_PORT` and `BAMBOOZLE_CONTROL_PORT`.

The regression suite in [`../tests`](../tests) executes every example and also
tests the control API itself.
