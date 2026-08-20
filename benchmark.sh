#!/usr/bin/env bash
set -Eeuo pipefail

# Compare Sparky with a real Redis server using the same redis-benchmark workload.
# Usage: ./benchmark.sh [requests] [clients]
# Example: ./benchmark.sh 100000 50

REQUESTS="${1:-${REQUESTS:-10000}}"
CLIENTS="${2:-${CLIENTS:-50}}"
SPARKY_PORT="${SPARKY_PORT:-6970}"
REDIS_PORT="${REDIS_PORT:-6380}"
SPARKY_AOF_FSYNC="${SPARKY_AOF_FSYNC:-everysec}"
REDIS_AOF_FSYNC="${REDIS_AOF_FSYNC:-everysec}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/sparky-benchmark.XXXXXX")"
SPARKY_AOF="${TMP_DIR}/sparky.aof"
SPARKY_LOG="${TMP_DIR}/sparky.log"
REDIS_LOG="${TMP_DIR}/redis.log"
SPARKY_PID=""
REDIS_PID=""

cleanup() {
    set +e
    if [[ -n "${SPARKY_PID}" ]] && kill -0 "${SPARKY_PID}" 2>/dev/null; then
        kill "${SPARKY_PID}"
        wait "${SPARKY_PID}" 2>/dev/null
    fi
    if [[ -n "${REDIS_PID}" ]] && kill -0 "${REDIS_PID}" 2>/dev/null; then
        kill "${REDIS_PID}"
        wait "${REDIS_PID}" 2>/dev/null
    fi
    rm -rf "${TMP_DIR}"
}
trap cleanup EXIT INT TERM

require_command() {
    command -v "$1" >/dev/null 2>&1 || {
        echo "error: '$1' is required but was not found in PATH" >&2
        exit 1
    }
}

wait_for_server() {
    local port="$1"
    local name="$2"
    for _ in {1..100}; do
        if redis-cli -h 127.0.0.1 -p "$port" PING 2>/dev/null | grep -qx PONG; then
            return 0
        fi
        sleep 0.1
    done
    echo "error: ${name} did not start on port ${port}" >&2
    exit 1
}

require_command cargo
require_command redis-cli
require_command redis-server
require_command redis-benchmark

if [[ "${SPARKY_PORT}" == "${REDIS_PORT}" ]]; then
    echo "error: SPARKY_PORT and REDIS_PORT must be different" >&2
    exit 1
fi

echo "Building Sparky in release mode..."
cargo build --release --quiet --manifest-path "${ROOT_DIR}/Cargo.toml"

echo "Starting Sparky on 127.0.0.1:${SPARKY_PORT}"
SPARKY_PORT="${SPARKY_PORT}" SPARKY_AOF="${SPARKY_AOF}" \
SPARKY_AOF_FSYNC="${SPARKY_AOF_FSYNC}" \
    "${ROOT_DIR}/target/release/sparky" >"${SPARKY_LOG}" 2>&1 &
SPARKY_PID=$!

echo "Starting Redis on 127.0.0.1:${REDIS_PORT}"
# Match Sparky's durable-write behavior as closely as possible.
redis-server \
    --bind 127.0.0.1 \
    --port "${REDIS_PORT}" \
    --dir "${TMP_DIR}" \
    --dbfilename redis.rdb \
    --save "" \
    --appendonly yes \
    --appendfilename redis.aof \
    --appendfsync "${REDIS_AOF_FSYNC}" \
    --daemonize no >"${REDIS_LOG}" 2>&1 &
REDIS_PID=$!

wait_for_server "${SPARKY_PORT}" "Sparky"
wait_for_server "${REDIS_PORT}" "Redis"

echo
echo "Benchmark configuration: requests=${REQUESTS}, clients=${CLIENTS}"
echo "Sparky AOF fsync=${SPARKY_AOF_FSYNC}; Redis AOF fsync=${REDIS_AOF_FSYNC}."
echo

run_benchmark() {
    local name="$1"
    local port="$2"
    echo "===== ${name} (${port}) ====="
    redis-benchmark \
        -h 127.0.0.1 \
        -p "${port}" \
        -n "${REQUESTS}" \
        -c "${CLIENTS}" \
        --csv \
        SET benchmark:key 0123456789012345
    redis-benchmark \
        -h 127.0.0.1 \
        -p "${port}" \
        -n "${REQUESTS}" \
        -c "${CLIENTS}" \
        --csv \
        GET benchmark:key
    echo
}

run_benchmark "Sparky" "${SPARKY_PORT}"
run_benchmark "Redis" "${REDIS_PORT}"

echo "Benchmark complete; temporary server files and logs were cleaned up."
