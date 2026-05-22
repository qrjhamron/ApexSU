#!/usr/bin/env bash
set -euo pipefail

EVIDENCE_FILE="${RELEASE_EVIDENCE_FILE:-docs/release/RELEASE_EVIDENCE.md}"

require_line() {
  local pattern="$1"
  local label="$2"
  if grep -Eq "$pattern" "$EVIDENCE_FILE"; then
    echo "[PASS] $label"
  else
    echo "[FAIL] $label" >&2
    failures=$((failures + 1))
  fi
}

failures=0

echo "== Release readiness gate =="

if [[ ! -f "$EVIDENCE_FILE" ]]; then
  echo "[FAIL] missing release evidence file: $EVIDENCE_FILE" >&2
  echo "Create/update evidence before release." >&2
  exit 1
fi

echo "Using evidence file: $EVIDENCE_FILE"

if grep -Eq 'PENDING|TODO|TBD|NOT RUN|NOT VERIFIED' "$EVIDENCE_FILE"; then
  echo "[FAIL] evidence file contains unresolved placeholders" >&2
  failures=$((failures + 1))
fi

require_line '^Release tag:[[:space:]]+v[^[:space:]]+' 'release tag recorded'
require_line '^Host smoke:[[:space:]]+PASS' 'host smoke evidence marked PASS'
require_line '^Android ksud target build/test:[[:space:]]+PASS' 'Android ksud evidence marked PASS'
require_line '^Android/GKI kernel build:[[:space:]]+PASS' 'Android/GKI kernel evidence marked PASS'
require_line '^KUnit:[[:space:]]+PASS' 'KUnit evidence marked PASS'
require_line '^Runtime stress:[[:space:]]+PASS' 'runtime stress evidence marked PASS'
require_line '^Reviewed by:[[:space:]]+.+' 'reviewer sign-off present'

if [[ -n "${GITHUB_REF_NAME:-}" ]]; then
  if grep -Eq "^Release tag:[[:space:]]+${GITHUB_REF_NAME}$" "$EVIDENCE_FILE"; then
    echo "[PASS] evidence tag matches GITHUB_REF_NAME=${GITHUB_REF_NAME}"
  else
    echo "[FAIL] evidence tag does not match GITHUB_REF_NAME=${GITHUB_REF_NAME}" >&2
    failures=$((failures + 1))
  fi
fi

if compgen -G 'android*-lkm/*_kernelsu.ko' >/dev/null; then
  echo "[PASS] release artifacts include android*-lkm/*_kernelsu.ko"
else
  echo "[FAIL] release artifacts missing android*-lkm/*_kernelsu.ko" >&2
  failures=$((failures + 1))
fi

if compgen -G 'ksud/ksud-*' >/dev/null; then
  echo "[PASS] release artifacts include ksud binaries"
else
  echo "[FAIL] release artifacts missing ksud binaries" >&2
  failures=$((failures + 1))
fi

if compgen -G 'manager/*.apk' >/dev/null; then
  echo "[PASS] release artifacts include manager APK"
else
  echo "[FAIL] release artifacts missing manager APK" >&2
  failures=$((failures + 1))
fi

if [[ "$failures" -ne 0 ]]; then
  echo "Release readiness gate failed ($failures blockers)." >&2
  exit 1
fi

echo "Release readiness gate passed."
