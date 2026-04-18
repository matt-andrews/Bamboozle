#!/usr/bin/env bash
set -euo pipefail

docker compose -f docker-compose.dev.yml down --rmi all && docker compose -f docker-compose.dev.yml up -d