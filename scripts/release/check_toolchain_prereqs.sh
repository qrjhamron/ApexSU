#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/release/check_toolchain_prereqs.sh [--skip-gki-out]

Checks required Android/GKI build prerequisites:
  - aarch64-linux-android-clang
  - Android NDK path (ANDROID_NDK_HOME or ANDROID_NDK_ROOT)
  - Android/GKI kernel out directory (ANDROID_GKI_OUT), unless --skip-gki-out
USAGE
}

require_gki_out=1
if [[ "${1-}" == "--skip-gki-out" ]]; then
  require_gki_out=0
elif [[ "${1-}" == "-h" || "${1-}" == "--help" ]]; then
  usage
  exit 0
elif [[ -n "${1-}" ]]; then
  echo "unknown argument: $1" >&2
  usage >&2
  exit 2
fi

failures=0

ndk_path="${ANDROID_NDK_HOME:-${ANDROID_NDK_ROOT:-}}"
if [[ -z "$ndk_path" ]]; then
  echo "[FAIL] Android NDK path missing: set ANDROID_NDK_HOME or ANDROID_NDK_ROOT" >&2
  failures=$((failures + 1))
elif [[ ! -d "$ndk_path" ]]; then
  echo "[FAIL] Android NDK path does not exist: $ndk_path" >&2
  failures=$((failures + 1))
else
  echo "[PASS] Android NDK path: $ndk_path"
fi

clang_bin=""
if command -v aarch64-linux-android-clang >/dev/null 2>&1; then
  clang_bin="$(command -v aarch64-linux-android-clang)"
elif [[ -n "$ndk_path" ]]; then
  for prebuilt in "$ndk_path/toolchains/llvm/prebuilt"/*; do
    candidate="$prebuilt/bin/aarch64-linux-android-clang"
    if [[ -x "$candidate" ]]; then
      clang_bin="$candidate"
      break
    fi
  done
fi

if [[ -z "$clang_bin" ]]; then
  echo "[FAIL] aarch64-linux-android-clang not found in PATH or Android NDK toolchains" >&2
  failures=$((failures + 1))
else
  echo "[PASS] aarch64-linux-android-clang: $clang_bin"
fi

if [[ "$require_gki_out" -eq 1 ]]; then
  gki_out="${ANDROID_GKI_OUT:-}"
  if [[ -z "$gki_out" ]]; then
    echo "[FAIL] Android/GKI output directory missing: set ANDROID_GKI_OUT" >&2
    failures=$((failures + 1))
  elif [[ ! -d "$gki_out" ]]; then
    echo "[FAIL] Android/GKI output directory does not exist: $gki_out" >&2
    failures=$((failures + 1))
  elif [[ ! -f "$gki_out/include/config/auto.conf" && ! -f "$gki_out/.config" ]]; then
    echo "[FAIL] Android/GKI output directory looks unprepared (missing include/config/auto.conf or .config): $gki_out" >&2
    failures=$((failures + 1))
  else
    echo "[PASS] Android/GKI output directory: $gki_out"
  fi
fi

if [[ "$failures" -ne 0 ]]; then
  echo "Toolchain prerequisite check failed ($failures)." >&2
  exit 1
fi

echo "Toolchain prerequisite check passed."
