docker compose -f docker-compose.dev.yml up -d
bash perf-test/k6/wait-for-health.sh 60

for script in perf-test/k6/*.js; do
  script_name=$(basename "$script")
  echo "=== running $script_name ==="
  MSYS_NO_PATHCONV=1 docker run --rm \
    -v "$(pwd)/perf-test/k6:/scripts" \
    -e BASE_URL=http://host.docker.internal:18080 \
    -e CONTROL_URL=http://host.docker.internal:19090 \
    grafana/k6:0.57.0 run --no-color "/scripts/$script_name"
done

docker compose -f docker-compose.dev.yml down