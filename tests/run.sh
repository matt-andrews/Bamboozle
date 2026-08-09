#!/usr/bin/env sh
set -eu

test_root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
compose_file="$test_root/docker-compose.yml"

cleanup() {
  docker compose -f "$compose_file" down --remove-orphans
}
trap cleanup EXIT INT TERM

docker compose -f "$compose_file" up \
  --build \
  --force-recreate \
  --abort-on-container-exit \
  --exit-code-from tempest
