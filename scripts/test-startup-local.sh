#!/usr/bin/env bash
set -euo pipefail

echo "building image..."
docker build -f src/Bamboozle/Dockerfile -t bamboozle:local src/
echo "running tests..."
bash scripts/test-startup.sh bamboozle:local 5