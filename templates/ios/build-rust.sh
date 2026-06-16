#!/usr/bin/env bash
# Cross-compile the Rust core for iOS and bundle it as an XCFramework that the
# SwiftUI app links against. Re-run this whenever the Rust FFI surface changes.
#
#   bash build-rust.sh            # release build (default)
#   PROFILE=dev bash build-rust.sh   # faster, unoptimised build
set -euo pipefail

CRATE="[[ crate_name ]]"
LIB="lib${CRATE}.a"
PROFILE="${PROFILE:-release}"
OUT="ios/Frameworks/RustCore.xcframework"

# Device, Apple-silicon simulator, and Intel simulator.
DEVICE_TARGET="aarch64-apple-ios"
SIM_TARGETS=("aarch64-apple-ios-sim" "x86_64-apple-ios")
ALL_TARGETS=("$DEVICE_TARGET" "${SIM_TARGETS[@]}")

if ! xcodebuild -version >/dev/null 2>&1; then
  echo "error: full Xcode is required to build the XCFramework." >&2
  echo "       Install Xcode, then: sudo xcode-select -s /Applications/Xcode.app" >&2
  exit 1
fi

if [[ "$PROFILE" == "release" ]]; then
  CARGO_PROFILE_FLAG="--release"
  PROFILE_DIR="release"
else
  CARGO_PROFILE_FLAG=""
  PROFILE_DIR="debug"
fi

echo "▸ ensuring rustup targets are installed"
rustup target add "${ALL_TARGETS[@]}"

for target in "${ALL_TARGETS[@]}"; do
  echo "▸ cargo build ($target)"
  # shellcheck disable=SC2086
  cargo build $CARGO_PROFILE_FLAG --target "$target"
done

echo "▸ lipo simulator slices"
SIM_LIB="target/sim-${LIB}"
lipo -create \
  "target/aarch64-apple-ios-sim/${PROFILE_DIR}/${LIB}" \
  "target/x86_64-apple-ios/${PROFILE_DIR}/${LIB}" \
  -output "$SIM_LIB"

echo "▸ assembling $OUT"
rm -rf "$OUT"
mkdir -p ios/Frameworks
xcodebuild -create-xcframework \
  -library "target/${DEVICE_TARGET}/${PROFILE_DIR}/${LIB}" -headers include \
  -library "$SIM_LIB" -headers include \
  -output "$OUT"

echo "✓ built $OUT"
