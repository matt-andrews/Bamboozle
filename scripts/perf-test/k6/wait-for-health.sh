#!/usr/bin/env bash
set -euo pipefail

HEALTH_URL="http://localhost:19090/control/health"
TIMEOUT_S="${1:-60}"
TIMEOUT_MS=$(( TIMEOUT_S * 1000 ))

echo "Waiting for health at $HEALTH_URL (timeout ${TIMEOUT_S}s)..."
start=$(date +%s%3N)

while true; do
  elapsed=$(( $(date +%s%3N) - start ))
  if [ "$elapsed" -ge "$TIMEOUT_MS" ]; then
    echo "FAIL: service not healthy within ${TIMEOUT_S}s"
    exit 1
  fi
  if curl -sf --max-time 1 "$HEALTH_URL" >/dev/null 2>&1; then
    echo "Healthy after ${elapsed}ms"
    exit 0
  fi
  sleep 0.2
done
