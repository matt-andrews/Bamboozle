docker compose -f docker-compose.dev.yml up -d
bash perf-test/k6/wait-for-health.sh 60

MSYS_NO_PATHCONV=1 docker run --rm \
  -v "$(pwd)/perf-test/k6:/scripts" \
  -e BASE_URL=http://host.docker.internal:18080 \
  grafana/k6:0.57.0 run --no-color /scripts/load-test.js

docker compose -f docker-compose.dev.yml down