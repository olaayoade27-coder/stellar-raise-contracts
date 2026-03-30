#!/usr/bin/env bash
# @title   performance_regression_detection.sh
# @notice  Detects CI/CD workflow changes that would likely cause performance
#          regressions in build, test, or coverage execution.
# @dev     This script reads checked-in workflow YAML only. It does not execute
#          workflows or parse untrusted content with eval-like constructs.
#          Exit code policy:
#            0 = all regression checks passed
#            1 = one or more regression checks failed
#            2 = required workflow files are missing
# @custom:security-note  The script is file-read-only and validates only
#          repository-controlled paths, so it cannot mutate CI configuration or
#          execute injected shell from workflow content.

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="${PROJECT_ROOT:-$(cd "${SCRIPT_DIR}/.." && pwd)}"
readonly WORKFLOWS_DIR="${WORKFLOWS_DIR:-${PROJECT_ROOT}/.github/workflows}"
readonly RUST_CI_FILE="${RUST_CI_FILE:-${WORKFLOWS_DIR}/rust_ci.yml}"
readonly REPORT_DIR="${REPORT_DIR:-${PROJECT_ROOT}/.performance-reports}"
readonly REPORT_FILE="${REPORT_DIR}/performance_regression_report_$(date -u +%Y%m%d_%H%M%S).txt"
readonly MAX_CHECK_JOB_TIMEOUT=30
readonly MAX_WASM_STEP_TIMEOUT=10
readonly MAX_TEST_STEP_TIMEOUT=15
readonly MAX_SOFT_LIMIT_SECONDS=$((20 * 60))

FAILURES=0
REPORT_LINES=()

perf_pass() {
  local message="$1"
  echo "PASS: ${message}"
  REPORT_LINES+=("PASS|${message}")
}

perf_fail() {
  local message="$1"
  echo "FAIL: ${message}" >&2
  FAILURES=$((FAILURES + 1))
  REPORT_LINES+=("FAIL|${message}")
}

perf_warn() {
  local message="$1"
  echo "WARN: ${message}"
  REPORT_LINES+=("WARN|${message}")
}

# @notice  Writes a plain-text performance report suitable for CI artefacts.
write_report() {
  mkdir -p "${REPORT_DIR}"
  {
    echo "performance_regression_report"
    echo "timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "project_root=${PROJECT_ROOT}"
    echo "workflow_file=${RUST_CI_FILE}"
    echo "failures=${FAILURES}"
    printf '%s\n' "${REPORT_LINES[@]}"
  } > "${REPORT_FILE}"
  echo "Report: ${REPORT_FILE}"
}

# @notice  Extracts the first timeout-minutes value that appears after a marker.
# @param $1  marker line to search for
extract_timeout_after_marker() {
  local marker="$1"
  local marker_line
  marker_line="$(grep -nF "${marker}" "${RUST_CI_FILE}" | head -n1 | cut -d: -f1)"
  if [[ -z "${marker_line}" ]]; then
    return 0
  fi

  sed -n "$((marker_line + 1)),$((marker_line + 8))p" "${RUST_CI_FILE}" \
    | sed -n 's/^[[:space:]]*timeout-minutes:[[:space:]]*\([0-9][0-9]*\)[[:space:]]*$/\1/p' \
    | head -n1
}

# @notice  Requires the main Rust CI workflow file to exist.
check_required_files() {
  if [[ -f "${RUST_CI_FILE}" ]]; then
    perf_pass "rust_ci.yml exists"
    return 0
  fi

  echo "ERROR: missing ${RUST_CI_FILE}" >&2
  exit 2
}

# @notice  Ensures the frontend job and its coverage run remain enabled.
check_frontend_coverage_job() {
  if grep -Eq '^  frontend:' "${RUST_CI_FILE}"; then
    perf_pass "frontend job is present"
  else
    perf_fail "frontend job is missing"
  fi

  if grep -Fq 'run: npm ci' "${RUST_CI_FILE}"; then
    perf_pass "frontend job installs dependencies with npm ci"
  else
    perf_fail "frontend job is missing npm ci"
  fi

  if grep -Fq 'npm run test:coverage -- --ci --reporters=default' "${RUST_CI_FILE}"; then
    perf_pass "frontend coverage command is present"
  else
    perf_fail "frontend coverage command is missing"
  fi
}

# @notice  Ensures Rust dependency caching remains enabled.
check_cache_usage() {
  if grep -Fq 'uses: Swatinem/rust-cache@v2' "${RUST_CI_FILE}"; then
    perf_pass "rust-cache is configured"
  else
    perf_fail "rust-cache is missing"
  fi
}

# @notice  Verifies heavy CI steps stay bounded by explicit timeouts.
check_timeout_budgets() {
  local check_job_timeout wasm_timeout test_timeout

  check_job_timeout="$(extract_timeout_after_marker 'name: Check, Lint & Test' || true)"
  wasm_timeout="$(extract_timeout_after_marker 'name: Build crowdfund WASM for tests' || true)"
  test_timeout="$(extract_timeout_after_marker 'name: Run tests including property-based tests' || true)"

  if [[ -n "${check_job_timeout}" && "${check_job_timeout}" -le "${MAX_CHECK_JOB_TIMEOUT}" ]]; then
    perf_pass "check job timeout is ${check_job_timeout} minutes"
  else
    perf_fail "check job timeout missing or above ${MAX_CHECK_JOB_TIMEOUT} minutes"
  fi

  if [[ -n "${wasm_timeout}" && "${wasm_timeout}" -le "${MAX_WASM_STEP_TIMEOUT}" ]]; then
    perf_pass "WASM build timeout is ${wasm_timeout} minutes"
  else
    perf_fail "WASM build timeout missing or above ${MAX_WASM_STEP_TIMEOUT} minutes"
  fi

  if [[ -n "${test_timeout}" && "${test_timeout}" -le "${MAX_TEST_STEP_TIMEOUT}" ]]; then
    perf_pass "test timeout is ${test_timeout} minutes"
  else
    perf_fail "test timeout missing or above ${MAX_TEST_STEP_TIMEOUT} minutes"
  fi
}

# @notice  Requires build and test commands to remain scoped to current CI cost.
check_build_and_test_scope() {
  if grep -Fq 'cargo build --release --target wasm32-unknown-unknown -p crowdfund' "${RUST_CI_FILE}"; then
    perf_pass "WASM build remains scoped to -p crowdfund"
  else
    perf_fail "WASM build is missing crate scoping to -p crowdfund"
  fi

  if grep -Fq 'run: cargo test --workspace' "${RUST_CI_FILE}"; then
    perf_pass "workspace test command is present"
  else
    perf_fail "workspace test command is missing"
  fi

  local wasm_build_count
  wasm_build_count="$(grep -Fc 'cargo build --release --target wasm32-unknown-unknown' "${RUST_CI_FILE}")"
  if [[ "${wasm_build_count}" -le 1 ]]; then
    perf_pass "duplicate WASM builds are not present"
  else
    perf_fail "duplicate WASM builds detected (${wasm_build_count})"
  fi
}

# @notice  Requires elapsed time logging and a soft-limit warning for review.
check_elapsed_time_monitoring() {
  if grep -Fq 'JOB_START=$(date +%s)' "${RUST_CI_FILE}"; then
    perf_pass "job start time is recorded"
  else
    perf_fail "job start time recording is missing"
  fi

  if grep -Fq 'ELAPSED=$(( END - JOB_START ))' "${RUST_CI_FILE}"; then
    perf_pass "elapsed time is calculated"
  else
    perf_fail "elapsed time calculation is missing"
  fi

  if grep -Fq 'SOFT_LIMIT_S=$(( 20 * 60 ))' "${RUST_CI_FILE}"; then
    perf_pass "soft performance limit is defined"
  else
    perf_fail "soft performance limit is missing"
  fi

  if grep -Fq '::warning::Job took ${ELAPSED}s' "${RUST_CI_FILE}"; then
    perf_pass "soft-limit warning is emitted"
  else
    perf_fail "soft-limit warning is missing"
  fi

  if grep -Eq 'SOFT_LIMIT_S=\$\(\([[:space:]]*20[[:space:]]*\*[[:space:]]*60[[:space:]]*\)\)' "${RUST_CI_FILE}"; then
    perf_warn "soft limit is pinned to ${MAX_SOFT_LIMIT_SECONDS} seconds"
  fi
}

# @notice  Ensures this gate is actually executed in the Rust CI workflow.
check_self_integration() {
  if grep -Eq 'run:[[:space:]]+bash scripts/performance_regression_detection\.sh([[:space:]]|$)' "${RUST_CI_FILE}"; then
    perf_pass "rust_ci.yml runs performance_regression_detection.sh"
  else
    perf_fail "rust_ci.yml does not run performance_regression_detection.sh"
  fi

  if grep -Eq 'run:[[:space:]]+bash scripts/performance_regression_detection\.test\.sh([[:space:]]|$)' "${RUST_CI_FILE}"; then
    perf_pass "rust_ci.yml runs performance_regression_detection.test.sh"
  else
    perf_fail "rust_ci.yml does not run performance_regression_detection.test.sh"
  fi
}

main() {
  echo "Stellar Raise - Performance Regression Detection"
  echo "$(date -u +%Y-%m-%dT%H:%M:%SZ)"

  check_required_files
  check_frontend_coverage_job
  check_cache_usage
  check_timeout_budgets
  check_build_and_test_scope
  check_elapsed_time_monitoring
  check_self_integration
  write_report

  if [[ "${FAILURES}" -eq 0 ]]; then
    echo "PERFORMANCE REGRESSION POLICY: PASSED"
    exit 0
  fi

  echo "PERFORMANCE REGRESSION POLICY: FAILED (${FAILURES} issue(s))" >&2
  exit 1
}

main "$@"
