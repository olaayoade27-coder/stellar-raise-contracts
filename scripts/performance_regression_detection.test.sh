#!/usr/bin/env bash
# @title   performance_regression_detection.test.sh
# @notice  Behavioral tests for performance_regression_detection.sh.
# @dev     The suite executes the real script against isolated fixture
#          repositories to verify both passing and failing regression scenarios.
#          Exit code policy:
#            0 = all tests passed
#            1 = one or more tests failed
# @custom:security-note  Fixture repositories contain only synthetic workflow
#          YAML, so no secrets, network calls, or external CI systems are used.

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly SCRIPT_UNDER_TEST="${REPO_ROOT}/scripts/performance_regression_detection.sh"

TESTS_PASSED=0
TESTS_FAILED=0
TMPDIRS=()

cleanup() {
  if [[ "${#TMPDIRS[@]}" -gt 0 ]]; then
    rm -rf "${TMPDIRS[@]}"
  fi
}
trap cleanup EXIT

pass() {
  echo "PASS: $1"
  TESTS_PASSED=$((TESTS_PASSED + 1))
}

fail() {
  echo "FAIL: $1"
  TESTS_FAILED=$((TESTS_FAILED + 1))
}

make_tmpdir() {
  local dir
  dir="$(mktemp -d)"
  TMPDIRS+=("${dir}")
  echo "${dir}"
}

make_valid_fixture() {
  local root="$1"
  mkdir -p "${root}/.github/workflows"

  cat > "${root}/.github/workflows/rust_ci.yml" <<'EOF'
name: Rust CI
on:
  pull_request:
    branches: [main]
jobs:
  frontend:
    name: Frontend UI Tests
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
      - name: Install dependencies
        run: npm ci
      - name: Run frontend tests with coverage
        run: npm run test:coverage -- --ci --reporters=default
  check:
    name: Check, Lint & Test
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - name: Record job start time
        run: echo "JOB_START=$(date +%s)" >> "$GITHUB_ENV"
      - name: Cache Rust dependencies
        uses: Swatinem/rust-cache@v2
      - name: Run performance regression detection
        run: bash scripts/performance_regression_detection.sh
      - name: Run performance regression detection tests
        run: bash scripts/performance_regression_detection.test.sh
      - name: Build crowdfund WASM for tests
        timeout-minutes: 10
        run: cargo build --release --target wasm32-unknown-unknown -p crowdfund
      - name: Run tests including property-based tests
        timeout-minutes: 15
        run: cargo test --workspace
      - name: Log total job elapsed time
        if: always()
        run: |
          END=$(date +%s)
          ELAPSED=$(( END - JOB_START ))
          echo "Total job time: ${ELAPSED}s"
          SOFT_LIMIT_S=$(( 20 * 60 ))
          if [ "$ELAPSED" -gt "$SOFT_LIMIT_S" ]; then
            echo "::warning::Job took ${ELAPSED}s"
          fi
EOF
}

run_script() {
  local project_root="$1"
  PROJECT_ROOT="${project_root}" bash "${SCRIPT_UNDER_TEST}" 2>&1
}

assert_exit_and_pattern() {
  local description="$1"
  local expected_exit="$2"
  local pattern="$3"
  shift 3
  local output
  local actual=0

  output="$("$@" 2>&1)" || actual=$?

  if [[ "${actual}" -ne "${expected_exit}" ]]; then
    fail "${description} (expected exit ${expected_exit}, got ${actual})"
    return
  fi

  if [[ -n "${pattern}" ]] && ! grep -Eq "${pattern}" <<< "${output}"; then
    fail "${description} (pattern not found: ${pattern})"
    return
  fi

  pass "${description}"
}

test_valid_fixture_passes() {
  local root
  root="$(make_tmpdir)"
  make_valid_fixture "${root}"
  assert_exit_and_pattern "valid fixture passes" 0 'PERFORMANCE REGRESSION POLICY: PASSED' run_script "${root}"
}

test_report_file_is_generated() {
  local root
  root="$(make_tmpdir)"
  make_valid_fixture "${root}"
  run_script "${root}" >/dev/null

  local report_count
  report_count=$(find "${root}/.performance-reports" -type f -name 'performance_regression_report_*.txt' | wc -l)
  if [[ "${report_count}" -ge 1 ]]; then
    pass "report artefact is generated"
  else
    fail "report artefact is generated"
  fi
}

test_missing_workflow_exits_2() {
  local root
  root="$(make_tmpdir)"
  mkdir -p "${root}/.github/workflows"
  assert_exit_and_pattern "missing rust_ci exits 2" 2 'missing .+rust_ci\.yml' run_script "${root}"
}

test_missing_frontend_coverage_fails() {
  local root
  root="$(make_tmpdir)"
  make_valid_fixture "${root}"
  perl -0pi -e 's#      - name: Run frontend tests with coverage\n        run: npm run test:coverage -- --ci --reporters=default\n##' "${root}/.github/workflows/rust_ci.yml"
  assert_exit_and_pattern "missing frontend coverage fails" 1 'frontend coverage command is missing' run_script "${root}"
}

test_missing_cache_fails() {
  local root
  root="$(make_tmpdir)"
  make_valid_fixture "${root}"
  perl -0pi -e 's#      - name: Cache Rust dependencies\n        uses: Swatinem/rust-cache\@v2\n##' "${root}/.github/workflows/rust_ci.yml"
  assert_exit_and_pattern "missing rust-cache fails" 1 'rust-cache is missing' run_script "${root}"
}

test_high_check_timeout_fails() {
  local root
  root="$(make_tmpdir)"
  make_valid_fixture "${root}"
  perl -0pi -e 's/    timeout-minutes: 30/    timeout-minutes: 45/' "${root}/.github/workflows/rust_ci.yml"
  assert_exit_and_pattern "high check timeout fails" 1 'check job timeout missing or above 30 minutes' run_script "${root}"
}

test_missing_wasm_timeout_fails() {
  local root
  root="$(make_tmpdir)"
  make_valid_fixture "${root}"
  perl -0pi -e 's#      - name: Build crowdfund WASM for tests\n        timeout-minutes: 10\n#      - name: Build crowdfund WASM for tests\n#' "${root}/.github/workflows/rust_ci.yml"
  assert_exit_and_pattern "missing wasm timeout fails" 1 'WASM build timeout missing or above 10 minutes' run_script "${root}"
}

test_unscoped_wasm_build_fails() {
  local root
  root="$(make_tmpdir)"
  make_valid_fixture "${root}"
  perl -0pi -e 's/cargo build --release --target wasm32-unknown-unknown -p crowdfund/cargo build --release --target wasm32-unknown-unknown/' "${root}/.github/workflows/rust_ci.yml"
  assert_exit_and_pattern "unscoped wasm build fails" 1 'WASM build is missing crate scoping to -p crowdfund' run_script "${root}"
}

test_duplicate_wasm_build_fails() {
  local root
  root="$(make_tmpdir)"
  make_valid_fixture "${root}"
  cat >> "${root}/.github/workflows/rust_ci.yml" <<'EOF'
      - name: Duplicate WASM build
        run: cargo build --release --target wasm32-unknown-unknown
EOF
  assert_exit_and_pattern "duplicate wasm build fails" 1 'duplicate WASM builds detected' run_script "${root}"
}

test_missing_elapsed_logging_fails() {
  local root
  root="$(make_tmpdir)"
  make_valid_fixture "${root}"
  perl -0pi -e 's/          ELAPSED=\$\(\( END - JOB_START \)\)\n//' "${root}/.github/workflows/rust_ci.yml"
  assert_exit_and_pattern "missing elapsed logging fails" 1 'elapsed time calculation is missing' run_script "${root}"
}

test_missing_self_integration_fails() {
  local root
  root="$(make_tmpdir)"
  make_valid_fixture "${root}"
  perl -0pi -e 's#      - name: Run performance regression detection tests\n        run: bash scripts/performance_regression_detection.test.sh\n##' "${root}/.github/workflows/rust_ci.yml"
  assert_exit_and_pattern "missing self integration fails" 1 'rust_ci\.yml does not run performance_regression_detection\.test\.sh' run_script "${root}"
}

test_real_repo_passes() {
  assert_exit_and_pattern "repository workflows satisfy policy" 0 'PERFORMANCE REGRESSION POLICY: PASSED' run_script "${REPO_ROOT}"
}

main() {
  test_valid_fixture_passes
  test_report_file_is_generated
  test_missing_workflow_exits_2
  test_missing_frontend_coverage_fails
  test_missing_cache_fails
  test_high_check_timeout_fails
  test_missing_wasm_timeout_fails
  test_unscoped_wasm_build_fails
  test_duplicate_wasm_build_fails
  test_missing_elapsed_logging_fails
  test_missing_self_integration_fails
  test_real_repo_passes

  echo "Tests passed: ${TESTS_PASSED}"
  echo "Tests failed: ${TESTS_FAILED}"

  if [[ "${TESTS_FAILED}" -eq 0 ]]; then
    exit 0
  fi

  exit 1
}

main "$@"
