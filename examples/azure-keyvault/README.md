# Bamboozle Example: Azure Key Vault Mocking

This example demonstrates how to test .NET applications that use Azure Key Vault by mocking the dependency with Bamboozle's native HTTPS support.

By pointing the standard `SecretClient` at Bamboozle with TLS enabled, you can seamlessly mock Key Vault REST API responses without modifying your application logic (other than providing a `MockTokenCredential` for the test environment to bypass Entra ID OAuth).

## Prerequisites

- [.NET 10 SDK](https://dotnet.microsoft.com/download/dotnet/10.0)
- [Docker](https://docs.docker.com/get-docker/)
- Bamboozle certificate tool (`bamboozle-cert`) — see [Enable TLS guide](../../docs/how-to/enable-tls.md)

## 1. Generate Certificates

Azure Key Vault SDK requires HTTPS. You must generate a local certificate and mount it into the Bamboozle container.

1. Generate the certificates using `bamboozle-cert` (outputs to `./certs/` by default).
2. (Optional but recommended) Install `certs/ca.crt` into your OS trust store so your .NET application trusts the mock certificate.

## 2. Start Bamboozle with TLS

Run Bamboozle, mounting both the certificates and the `routes.yaml` file from this example.

```bash
# Assuming you are in the examples/azure-keyvault directory
docker run \
  -v ./certs:/certs \
  -v ./routes.yaml:/etc/bamboozle/routes.yaml \
  -e TLS_CERT_FILE=/certs/cert.pem \
  -e TLS_KEY_FILE=/certs/key.pem \
  -e 'ROUTE_CONFIG_FOLDERS=["/etc/bamboozle"]' \
  -p 8080:8080 -p 9090:9090 \
  mattisthegreatest/bamboozle
```

## 3. Run the Tests

Once Bamboozle is running, execute the xUnit tests. The tests use a custom `MockTokenCredential` to bypass OAuth and point the `SecretClient` directly to `https://localhost:8080`.

```bash
dotnet test
```

## How It Works

- `routes.yaml` defines the mock endpoints (`GET /secrets/{secretName}` and `PUT /secrets/{secretName}`) returning JSON payloads formatted exactly like the real Azure Key Vault REST API.
- `KeyVaultTests.cs` configures a standard `SecretClient` that points to the Bamboozle endpoint instead of `vault.azure.net`.
