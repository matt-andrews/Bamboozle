#!/usr/bin/env bash
set -euo pipefail

echo "building image..."
docker build -f bamboozle-rust/Dockerfile -t bamboozle:local bamboozle-rust/
echo "running tests..."
bash scripts/test-startup.sh bamboozle:local 5