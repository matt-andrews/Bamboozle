#!/usr/bin/env bash
set -euo pipefail

IMAGE="${1:-bamboozle:latest}"
ITERATIONS="${2:-5}"
HEALTH_URL="http://localhost:9090/control/health"
TIMEOUT_MS=30000

echo "Image:      $IMAGE"
echo "Iterations: $ITERATIONS"
echo ""

declare -a times=()
failed=0

for i in $(seq 1 "$ITERATIONS"); do
    echo "==> Run $i of $ITERATIONS"

    CID=$(docker run --rm -d \
        -p 8080:8080 \
        -p 9090:9090 \
        "$IMAGE")

    start=$(date +%s%3N)
    ready=false

    while true; do
        elapsed=$(( $(date +%s%3N) - start ))
        if [ "$elapsed" -ge "$TIMEOUT_MS" ]; then
            break
        fi
        if curl -sf --max-time 1 "$HEALTH_URL" >/dev/null 2>&1; then
            ready=true
            break
        fi
        sleep 0.05
    done

    elapsed=$(( $(date +%s%3N) - start ))
    docker stop "$CID" >/dev/null

    if [ "$ready" = false ]; then
        echo "    FAILED: container did not respond within 30s"
        failed=$(( failed + 1 ))
    else
        echo "    Ready in ${elapsed}ms"
        times+=("$elapsed")
    fi

    # Allow ports to be released before next run
    sleep 0.5
done

echo ""
echo "--- Summary ---"
echo "Successful: ${#times[@]} / $ITERATIONS"

if [ "${#times[@]}" -gt 0 ]; then
    min="${times[0]}"
    max="${times[0]}"
    total=0
    for t in "${times[@]}"; do
        total=$(( total + t ))
        [ "$t" -lt "$min" ] && min=$t
        [ "$t" -gt "$max" ] && max=$t
    done
    avg=$(( total / ${#times[@]} ))

    echo "Min: ${min}ms"
    echo "Max: ${max}ms"
    echo "Avg: ${avg}ms"

    echo "{\"min_ms\":$min,\"max_ms\":$max,\"avg_ms\":$avg,\"iterations\":$ITERATIONS,\"successful\":${#times[@]}}" > startup-results.json

    if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
        {
            echo "## Startup Time Results"
            echo ""
            echo "| Metric | Value |"
            echo "|--------|-------|"
            echo "| Successful runs | ${#times[@]} / $ITERATIONS |"
            echo "| Min | ${min}ms |"
            echo "| Max | ${max}ms |"
            echo "| Avg | ${avg}ms |"
            echo ""
            echo "### Per-run timings"
            echo ""
            echo "| Run | Time (ms) |"
            echo "|-----|-----------|"
            for idx in "${!times[@]}"; do
                echo "| $((idx + 1)) | ${times[$idx]} |"
            done
        } >> "$GITHUB_STEP_SUMMARY"
    fi
fi

if [ "$failed" -gt 0 ]; then
    echo ""
    echo "FAIL: $failed run(s) did not start within the timeout"
    exit 1
fi
