# performance_regression_detection

`scripts/performance_regression_detection.sh` enforces CI workflow controls that catch likely performance regressions before they land in `main` or `develop`.

## What it checks

1. `.github/workflows/rust_ci.yml` exists.
2. The `frontend` job remains present and still runs coverage.
3. Rust dependency caching with `Swatinem/rust-cache@v2` remains enabled.
4. The main `check` job stays bounded to 30 minutes or less.
5. The WASM build step stays bounded to 10 minutes or less.
6. The test step stays bounded to 15 minutes or less.
7. The WASM build remains scoped to `-p crowdfund`.
8. Duplicate WASM builds are not introduced.
9. Job elapsed time logging and the 20-minute soft-limit warning remain present.
10. `rust_ci.yml` actually runs the performance regression detector and its test suite.

## Usage

```bash
bash scripts/performance_regression_detection.sh
bash scripts/performance_regression_detection.test.sh
```

The script writes a plain-text report to `.performance-reports/performance_regression_report_<timestamp>.txt`.

## CI integration

Add these steps to `.github/workflows/rust_ci.yml`:

```yaml
- name: Run performance regression detection
  run: bash scripts/performance_regression_detection.sh

- name: Run performance regression detection tests
  run: bash scripts/performance_regression_detection.test.sh
```

## Security assumptions

- Workflow YAML checked into the repository is the source of truth for CI performance controls.
- The detector remains read-only and does not execute workflow shell content.
- Performance budgets are intentionally conservative to catch regressions early rather than after merge.
- The self-integration check ensures the detector cannot silently drift out of CI coverage.

## Test coverage notes

The companion shell test suite covers:

- the happy-path fixture repository
- report generation
- missing workflow file
- missing frontend coverage command
- missing Rust cache
- oversized check-job timeout
- missing WASM timeout
- unscoped WASM build
- duplicate WASM builds
- missing elapsed-time calculation
- missing self-integration
- the real repository configuration

This exercises each policy rule and the success, failure, and missing-file exit paths.
