#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
CERTS_DIR="$SCRIPT_DIR/certs"

cleanup() {
  echo "--- Stopping containers ---"
  docker compose -f "$SCRIPT_DIR/docker-compose.yml" down --remove-orphans || true
}
trap cleanup EXIT

# Build bamboozle-cert
echo "--- Building bamboozle-cert ---"
cargo build --manifest-path "$REPO_ROOT/Cargo.toml" --bin bamboozle-cert

BAMBOOZLE_CERT="$REPO_ROOT/target/debug/bamboozle-cert"

# Generate certs
echo "--- Generating certificates ---"
mkdir -p "$CERTS_DIR"
"$BAMBOOZLE_CERT" --out "$CERTS_DIR"

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
  *)
    echo "WARNING: unsupported platform for automatic CA trust, skipping"
    ;;
esac

# Start containers
echo "--- Starting containers ---"
docker compose -f "$SCRIPT_DIR/docker-compose.yml" up -d --build

# Wait for the service to be ready
echo "--- Waiting for bamboozle to be ready ---"
for i in $(seq 1 30); do
  if curl -sf --cacert "$CERTS_DIR/ca.crt" https://localhost:44044/ > /dev/null 2>&1; then
    echo "Service is ready."
    break
  fi
  if [ "$i" -eq 30 ]; then
    echo "ERROR: service did not become ready in time"
    docker compose -f "$SCRIPT_DIR/docker-compose.yml" logs
    exit 1
  fi
  sleep 1
done

# Run tests
echo "--- Running dotnet tests ---"
dotnet test "$SCRIPT_DIR"
