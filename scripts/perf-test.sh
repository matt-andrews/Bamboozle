#!/usr/bin/env bash
set -euo pipefail

echo "building image..."
bash dev.sh
echo "running tests..."
bash perf-test/test-startup.sh bamboozle:dev 5