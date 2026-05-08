#!/usr/bin/env bash
# Run the full test suite (backend + frontend) with coverage on the
# backend. Mirrors the CI pipeline at .github/workflows/test.yml so a
# green local run means a green CI run, modulo platform differences.
#
# Usage:
#   tools/test.sh              # backend coverage + frontend type-check + tests
#   tools/test.sh --backend    # backend only
#   tools/test.sh --frontend   # frontend only
#   tools/test.sh --enforce    # also enforce the 90% coverage gate (CI does this)
#
# Requires:
#   - cargo-llvm-cov + the llvm-tools-preview rustup component
#       cargo install cargo-llvm-cov --locked
#       rustup component add llvm-tools-preview
#   - TEST_DATABASE_URL set to a local Postgres with CREATEDB privileges
#       (kept distinct from DATABASE_URL on purpose; see backend/tests/common/mod.rs)
#   - Node + npm for the frontend half

set -euo pipefail

# Files excluded from coverage measurement. Justification — these are not
# unit-tested here:
#   main.rs / lib.rs           — boot code (covered implicitly by integration tests)
#   db.rs / config.rs / state.rs — pool setup, env loading, struct init
#   error.rs                   — From impls + IntoResponse
#   auth/cookie.rs             — trivial cookie builder
#   auth/extractor.rs          — Axum extractor; needs HTTP-level testing
#   auth/recaptcha.rs          — calls Google's API
#   auth/session.rs            — Axum-coupled session issuance
#   email/resend.rs            — calls Resend's API
#   routes/*                   — HTTP handlers; we test the lib functions they delegate to
COVERAGE_EXCLUDE='(src/main\.rs|src/lib\.rs|src/db\.rs|src/config\.rs|src/state\.rs|src/error\.rs|src/auth/cookie\.rs|src/auth/extractor\.rs|src/auth/recaptcha\.rs|src/auth/session\.rs|src/email/resend\.rs|src/routes/)'

COVERAGE_MIN=90

run_backend=true
run_frontend=true
enforce=false

for arg in "$@"; do
  case "$arg" in
    --backend) run_frontend=false ;;
    --frontend) run_backend=false ;;
    --enforce) enforce=true ;;
    -h|--help) sed -n '2,12p' "$0"; exit 0 ;;
    *) echo "Unknown flag: $arg" >&2; exit 2 ;;
  esac
done

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [ "$run_backend" = true ]; then
  echo "──── Backend coverage ────"
  cd "$repo_root/backend"
  llvm_cov_args=(
    --summary-only
    --ignore-filename-regex "$COVERAGE_EXCLUDE"
  )
  if [ "$enforce" = true ]; then
    llvm_cov_args+=(--fail-under-lines "$COVERAGE_MIN")
  fi
  cargo llvm-cov "${llvm_cov_args[@]}"
fi

if [ "$run_frontend" = true ]; then
  echo
  echo "──── Frontend type-check ────"
  cd "$repo_root/frontend"
  npx vue-tsc --noEmit

  echo
  echo "──── Frontend tests ────"
  npx vitest run
fi

echo
echo "All checks passed."
