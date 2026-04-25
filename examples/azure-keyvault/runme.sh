#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CERTS_DIR="$SCRIPT_DIR/certs"

cleanup() {
  echo "--- Stopping containers ---"
  docker compose -f "$SCRIPT_DIR/docker-compose.yml" down --remove-orphans || true
  case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*)
      certutil -delstore -user Root "Bamboozle Local CA" > /dev/null 2>&1 || true
      ;;
    Darwin)
      sudo security delete-certificate -c "Bamboozle Local CA" \
        /Library/Keychains/System.keychain 2>/dev/null || true
      ;;
  esac
}
trap cleanup EXIT

# Build Docker image first (needed for cert generation and the server)
echo "--- Building Docker image ---"
docker compose -f "$SCRIPT_DIR/docker-compose.yml" build

# Generate certs using the built image
echo "--- Generating certificates ---"
mkdir -p "$CERTS_DIR"
docker run --rm -v "$CERTS_DIR:/certs" bamboozle:dev generate-certs --out /certs

# Trust the CA
echo "--- Trusting CA certificate ---"
case "$(uname -s)" in
  Linux)
    sudo cp "$CERTS_DIR/ca.crt" /usr/local/share/ca-certificates/bamboozle-ca.crt
    sudo update-ca-certificates
    ;;
  Darwin)
    sudo security add-trusted-cert -d -r trustRoot \
      -k /Library/Keychains/System.keychain "$CERTS_DIR/ca.crt"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    certutil -addstore -user Root "$(cygpath -w "$CERTS_DIR/ca.crt")"
    ;;
  *)
    echo "WARNING: unsupported platform for automatic CA trust, skipping"
    ;;
esac

echo "--- Starting containers ---"
docker compose -f "$SCRIPT_DIR/docker-compose.yml" up -d --force-recreate

# Wait for the service to be ready
echo "--- Waiting for bamboozle to be ready ---"
for i in $(seq 1 60); do
  if curl -sfS --ssl-no-revoke --cacert "$CERTS_DIR/ca.crt" https://localhost:44044/ > /dev/null; then
    echo "Service is ready."
    break
  fi
  if [ "$i" -eq 60 ]; then
    echo "ERROR: service did not become ready in time"
    docker compose -f "$SCRIPT_DIR/docker-compose.yml" logs
    exit 1
  fi
  sleep 1
done

# Run tests
echo "--- Running dotnet tests ---"
dotnet test "$SCRIPT_DIR"
