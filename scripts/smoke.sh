#!/usr/bin/env bash
set -euo pipefail

base="${1:-http://localhost:3000}"
ui="${2:-http://localhost:3001}"

echo "Checking backend health..."
curl -fsS "$base/health"
echo
curl -fsS "$base/metrics" | head
echo
echo "Checking dashboard API..."
curl -fsS "$base/api/dashboard" | head -c 400
echo
echo "Checking frontend..."
curl -fsS -o /dev/null -w "%{http_code}\n" "$ui"
echo "Smoke OK"
