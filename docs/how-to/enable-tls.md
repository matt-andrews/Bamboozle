# Enable HTTPS (TLS)

Bamboozle supports optional TLS on the mock server port (`:8080`).
When enabled, your system under test can call `https://localhost:8080` — useful for mocking services like Azure Key Vault that require HTTPS.

The control port (`:9090`) always stays plain HTTP.

> [!NOTE]
> TLS is compiled into the default Docker image but only activates when you set `TLS_CERT_FILE` and `TLS_KEY_FILE`. There is zero overhead when these variables are absent.

---

## 1. Generate certificates

Use `bamboozle-cert` to create a local CA and leaf certificate. Download it from [GitHub Releases](https://github.com/matt-andrews/Bamboozle/releases) or build from source.

### Windows (PowerShell)

```powershell
# Download
Invoke-WebRequest -Uri "https://github.com/matt-andrews/Bamboozle/releases/latest/download/bamboozle-cert-windows.exe" -OutFile bamboozle-cert.exe

# Generate certs (outputs to ./certs/)
.\bamboozle-cert.exe
```

### macOS / Linux

```bash
# Download (replace OS with 'linux' or 'macos')
curl -L "https://github.com/matt-andrews/Bamboozle/releases/latest/download/bamboozle-cert-${OS}" -o bamboozle-cert
chmod +x bamboozle-cert

# Generate certs (outputs to ./certs/)
./bamboozle-cert
```

### Custom SANs

By default, certificates are valid for `localhost`, `127.0.0.1`, and `::1`. Add custom SANs with `--san`:

```bash
bamboozle-cert --san localhost --san 127.0.0.1 --san my-mock.local
```

### Output files

| File | Purpose |
|------|---------|
| `certs/ca.crt` | CA certificate — install in your OS/browser trust store |
| `certs/cert.pem` | Leaf certificate — mount into the Bamboozle container |
| `certs/key.pem` | Private key — mount into the Bamboozle container |

---

## 2. Trust the CA (optional but recommended)

Installing the CA certificate into your OS trust store means browsers, SDKs, and HTTP clients will accept the Bamboozle certificate without warnings or code changes.

### Windows

```powershell
certutil -addstore -user Root "C:\full\path\to\certs\ca.crt"
```

### macOS

```bash
sudo security add-trusted-cert -d -r trustRoot \
  -k /Library/Keychains/System.keychain certs/ca.crt
```

### Linux

```bash
sudo cp certs/ca.crt /usr/local/share/ca-certificates/bamboozle-ca.crt
sudo update-ca-certificates
```

> [!WARNING]
> Never share `key.pem` or `ca.crt` outside your development environment. Anyone with the CA key can issue certificates that your machine will trust.

---

## 3. Start Bamboozle with TLS

### Docker run

```bash
docker run \
  -v ./certs:/certs \
  -e TLS_CERT_FILE=/certs/cert.pem \
  -e TLS_KEY_FILE=/certs/key.pem \
  -p 8080:8080 -p 9090:9090 \
  mattisthegreatest/bamboozle
```

### Docker Compose

```yaml
services:
  bamboozle:
    image: mattisthegreatest/bamboozle
    ports:
      - "8080:8080"
      - "9090:9090"
    volumes:
      - ./certs:/certs
      - ./routes:/routes
    environment:
      - TLS_CERT_FILE=/certs/cert.pem
      - TLS_KEY_FILE=/certs/key.pem
      - ROUTE_CONFIG_FOLDERS=["/routes"]
```

### Verify

```bash
# Mock port — HTTPS (use --cacert if you haven't trusted the CA)
curl --cacert certs/ca.crt https://localhost:8080/

# Control port — still plain HTTP
curl http://localhost:9090/routes
```

---

## Example: mocking Azure Key Vault

Azure Key Vault SDKs require HTTPS. With Bamboozle TLS enabled:

1. Generate certs and trust the CA (steps 1–2 above)
2. Load your Key Vault route config via `ROUTE_CONFIG_FOLDERS`
3. Point the SDK at `https://localhost:8080` — the SDK will accept the certificate because your OS trusts the Bamboozle CA

```csharp
// C# example — no custom HttpClient needed when CA is trusted
var client = new SecretClient(
    new Uri("https://localhost:8080"),
    new DefaultAzureCredential()
);
```

---

## Related

- [Environment variables reference](../reference/environment-variables.md) — `TLS_CERT_FILE`, `TLS_KEY_FILE`
- [Load static config](load-static-config.md) — `ROUTE_CONFIG_FOLDERS`
