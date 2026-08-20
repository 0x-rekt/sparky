#!/usr/bin/env bash
set -Eeuo pipefail

# Full Sparky vs Redis benchmark: throughput (--csv) + latency percentile
# distribution for strings, lists, hashes, and sets.
#
# Usage: ./benchmark_full.sh [requests] [clients]
# Example: ./benchmark_full.sh 100000 50

REQUESTS="${1:-${REQUESTS:-100000}}"
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
RESULTS_DIR="${ROOT_DIR}/benchmark_results"
SPARKY_PID=""
REDIS_PID=""

# Commands to test. Add/remove based on what Sparky supports.
# Format matches redis-benchmark's -t flag test names.
TESTS="set,get,lpush,rpush,lpop,rpop,sadd,hset,lrange_100"

mkdir -p "${RESULTS_DIR}"

cleanup() {
    set +e
    if [[ -n "${SPARKY_PID}" ]] && kill -0 "${SPARKY_PID}" 2>/dev/null; then
        kill "${SPARKY_PID}"; wait "${SPARKY_PID}" 2>/dev/null
    fi
    if [[ -n "${REDIS_PID}" ]] && kill -0 "${REDIS_PID}" 2>/dev/null; then
        kill "${REDIS_PID}"; wait "${REDIS_PID}" 2>/dev/null
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
    local port="$1" name="$2"
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
echo "Tests: ${TESTS}"
echo "Sparky AOF fsync=${SPARKY_AOF_FSYNC}; Redis AOF fsync=${REDIS_AOF_FSYNC}."
echo "Results will be saved to: ${RESULTS_DIR}"
echo

run_throughput() {
    local name="$1" port="$2" outfile="$3"
    echo "===== ${name}: THROUGHPUT (${port}) ====="
    redis-benchmark \
        -h 127.0.0.1 -p "${port}" \
        -n "${REQUESTS}" -c "${CLIENTS}" \
        -t "${TESTS}" \
        --csv | tee "${outfile}"
    echo
}

run_latency() {
    local name="$1" port="$2" outfile="$3"
    echo "===== ${name}: LATENCY PERCENTILES (${port}) ====="
    # Smaller -n keeps this section fast; percentile shape is stable at 20k+.
    redis-benchmark \
        -h 127.0.0.1 -p "${port}" \
        -n 20000 -c "${CLIENTS}" \
        -t "${TESTS}" | tee "${outfile}"
    echo
}

run_throughput "Sparky" "${SPARKY_PORT}" "${RESULTS_DIR}/sparky_throughput.csv"
run_throughput "Redis"  "${REDIS_PORT}"  "${RESULTS_DIR}/redis_throughput.csv"

run_latency "Sparky" "${SPARKY_PORT}" "${RESULTS_DIR}/sparky_latency.txt"
run_latency "Redis"  "${REDIS_PORT}"  "${RESULTS_DIR}/redis_latency.txt"

echo "Benchmark complete."
echo "Throughput CSVs and latency percentile logs saved in: ${RESULTS_DIR}"
echo "Server logs (for debugging only) were in: ${TMP_DIR} (now cleaned up)"