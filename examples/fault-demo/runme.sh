#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "$SCRIPT_DIR"

cleanup() {
  echo "--- Stopping Bamboozle ---"
  docker compose down --remove-orphans || true
}
trap cleanup EXIT

echo "--- Installing dependencies ---"
npm ci

echo "--- Starting Bamboozle ---"
npm run bamboozle:up

echo "--- Running tests ---"
npm test
