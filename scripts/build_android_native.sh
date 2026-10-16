#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-release}"
case "$MODE" in
  debug) PROFILE=debug; PROFILE_FLAG=() ;;
  release) PROFILE=release; PROFILE_FLAG=(--release) ;;
  *) echo "Usage: $0 [debug|release]" >&2; exit 2 ;;
esac

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
NDK="${ANDROID_NDK_HOME:-${ANDROID_NDK_ROOT:-/home/teacher/Android/Sdk/ndk/27.2.12479018}}"
BIN="$NDK/toolchains/llvm/prebuilt/linux-x86_64/bin"
if [[ ! -x "$BIN/clang" && -z "${ANDROID_NDK_HOME:-}${ANDROID_NDK_ROOT:-}" && -x /home/teacher/devtools/android/ndk/27.2.12479018/toolchains/llvm/prebuilt/linux-x86_64/bin/clang ]]; then
  NDK=/home/teacher/devtools/android/ndk/27.2.12479018
  BIN="$NDK/toolchains/llvm/prebuilt/linux-x86_64/bin"
fi
if [[ ! -x "$BIN/clang" ]]; then
  echo "Android NDK LLVM toolchain missing at $BIN (finish NDK installation or set ANDROID_NDK_HOME)" >&2
  exit 1
fi

export CC_aarch64_linux_android="$BIN/aarch64-linux-android24-clang"
export CC_x86_64_linux_android="$BIN/x86_64-linux-android24-clang"
export AR_aarch64_linux_android="$BIN/llvm-ar"
export AR_x86_64_linux_android="$BIN/llvm-ar"
export RANLIB="$BIN/llvm-ranlib"
export PATH="$BIN:$PATH"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$CC_aarch64_linux_android"
export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$CC_x86_64_linux_android"
export CFLAGS_aarch64_linux_android="--sysroot=$NDK/toolchains/llvm/prebuilt/linux-x86_64/sysroot"
export CFLAGS_x86_64_linux_android="$CFLAGS_aarch64_linux_android"
export ANDROID_NDK_HOME="$NDK"

for target in aarch64-linux-android x86_64-linux-android; do
  if ! rustup target list --installed | /bin/grep -qx "$target"; then
    echo "Missing Rust target $target; install with rustup target add $target" >&2
    exit 1
  fi
done

for spec in 'aarch64-linux-android arm64-v8a' 'x86_64-linux-android x86_64'; do
  read -r target abi <<< "$spec"
  cargo build --manifest-path "$ROOT/native/nauterm_ffi/Cargo.toml" \
    --locked --target "$target" --no-default-features "${PROFILE_FLAG[@]}"
  destination="$ROOT/android/app/src/main/jniLibs/$abi"
  mkdir -p "$destination"
  cp "$ROOT/native/nauterm_ffi/target/$target/$PROFILE/libnauterm_ffi.so" "$destination/"
done
