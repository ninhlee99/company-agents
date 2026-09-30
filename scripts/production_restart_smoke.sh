#!/usr/bin/env bash
set -euo pipefail

PROJECT="${COMPOSE_PROJECT_NAME:-company-agents-restart-smoke-$$}"
export COMPOSE_PROJECT_NAME="$PROJECT"

cleanup() {
  docker compose down --volumes --remove-orphans >/dev/null 2>&1 || true
}
trap cleanup EXIT

export COMPANY_ID="00000000-0000-0000-0000-000000000001"
export COMPANY_NAME="Restart Smoke Company"
export COMPANY_CURRENCY="USD"
export LLM_PROVIDER="mock"
export CONTROL_PLANE_TOKEN="restart-smoke-token-please-change"
export AGENT_CYCLE_SECONDS="3600"
export AGENT_MAX_CALLS_PER_WINDOW="1"
export AGENT_MAX_MODEL_CALLS_PER_WINDOW="1"

wait_http() {
  local url="$1"
  local expected="$2"
  local deadline=$((SECONDS + 180))
  while (( SECONDS < deadline )); do
    local code
    code="$(curl -sS -o /tmp/company-agents-smoke-body -w '%{http_code}' "$url" || true)"
    if [[ "$code" == "$expected" ]]; then
      return 0
    fi
    sleep 2
  done
  echo "Timed out waiting for $url (expected HTTP $expected)" >&2
  cat /tmp/company-agents-smoke-body >&2 || true
  return 1
}

wait_ready() {
  wait_http "http://127.0.0.1:8080/healthz" "200"
  wait_http "http://127.0.0.1:8080/readyz" "200"
}

assert_authenticated_request() {
  local headers="/tmp/company-agents-smoke-headers"
  local status
  status="$(curl -sS -D "$headers" -o /tmp/company-agents-smoke-body -w '%{http_code}' -H "Authorization: Bearer $CONTROL_PLANE_TOKEN" -H "X-Request-Id: restart-smoke-request" "http://127.0.0.1:8080/api/integrations/readiness")"
  [[ "$status" == "200" ]]
  grep -Eiq '^x-request-id: *restart-smoke-request' "$headers"
  grep -q '"status"' /tmp/company-agents-smoke-body
}

assert_readonly_load() {
  local total=40
  local parallel=8
  local url="http://127.0.0.1:8080/api/integrations/readiness"
  rm -rf /tmp/company-agents-load
  mkdir -p /tmp/company-agents-load

  seq "$total" | xargs -P "$parallel" -I{} bash -c '
    code="$(curl -sS -o "/tmp/company-agents-load/{}.body" -w "%{http_code}"       -H "Authorization: Bearer $CONTROL_PLANE_TOKEN"       -H "X-Request-Id: load-smoke-{}"       "$0" || true)"
    printf "%s\n" "$code" > "/tmp/company-agents-load/{}.status"
  ' "$url"

  local failed
  failed="$(grep -L '^200  local metrics
  metrics="$(curl -sS "http://127.0.0.1:8080/metrics")"
  grep -q 'company_control_plane_requests_total' <<<"$metrics"
  grep -Eq 'company_control_plane_requests_total [1-9][0-9]*' <<<"$metrics"
  grep -q 'company_control_plane_last_latency_ms' <<<"$metrics"
}

docker compose up -d postgres redis
docker compose build company-os outbox-worker
docker compose up -d company-os outbox-worker
wait_ready
assert_authenticated_request
assert_readonly_load
assert_metrics_observed

docker compose restart company-os
wait_ready
assert_authenticated_request

docker compose restart postgres
wait_ready
assert_authenticated_request

docker compose restart outbox-worker
docker compose ps --status running outbox-worker >/tmp/company-agents-smoke-worker
grep -q 'outbox-worker' /tmp/company-agents-smoke-worker

echo "RESTART + READONLY LOAD CHAOS SMOKE PASSED"
 /tmp/company-agents-load/*.status | wc -l | tr -d ' ')"
  if [[ "$failed" != "0" ]]; then
    echo "Readonly load smoke had $failed non-200 responses" >&2
    grep -H -v '^200  local metrics
  metrics="$(curl -sS "http://127.0.0.1:8080/metrics")"
  grep -q 'company_control_plane_requests_total' <<<"$metrics"
  grep -Eq 'company_control_plane_requests_total [1-9][0-9]*' <<<"$metrics"
  grep -q 'company_control_plane_last_latency_ms' <<<"$metrics"
}

docker compose up -d postgres redis
docker compose build company-os outbox-worker
docker compose up -d company-os outbox-worker
wait_ready
assert_authenticated_request
assert_metrics_observed

docker compose restart company-os
wait_ready
assert_authenticated_request

docker compose restart postgres
wait_ready
assert_authenticated_request

docker compose restart outbox-worker
docker compose ps --status running outbox-worker >/tmp/company-agents-smoke-worker
grep -q 'outbox-worker' /tmp/company-agents-smoke-worker

echo "RESTART CHAOS SMOKE PASSED"
 /tmp/company-agents-load/*.status >&2 || true
    return 1
  fi

  local observed
  observed="$(curl -sS "http://127.0.0.1:8080/metrics" | awk '/company_control_plane_requests_total / {print $2; exit}')"
  [[ "$observed" =~ ^[0-9]+$ ]]
  (( observed >= total + 1 ))
}

assert_metrics_observed() {
  local metrics
  metrics="$(curl -sS "http://127.0.0.1:8080/metrics")"
  grep -q 'company_control_plane_requests_total' <<<"$metrics"
  grep -Eq 'company_control_plane_requests_total [1-9][0-9]*' <<<"$metrics"
  grep -q 'company_control_plane_last_latency_ms' <<<"$metrics"
}

docker compose up -d postgres redis
docker compose build company-os outbox-worker
docker compose up -d company-os outbox-worker
wait_ready
assert_authenticated_request
assert_metrics_observed

docker compose restart company-os
wait_ready
assert_authenticated_request

docker compose restart postgres
wait_ready
assert_authenticated_request

docker compose restart outbox-worker
docker compose ps --status running outbox-worker >/tmp/company-agents-smoke-worker
grep -q 'outbox-worker' /tmp/company-agents-smoke-worker

echo "RESTART CHAOS SMOKE PASSED"
