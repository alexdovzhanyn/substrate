#!/usr/bin/env bash
set -euo pipefail

PROFILE="${1:-debug}"

CARGO_TARGET_ARGS=""

if [ "${SUBSTRATE_LOCAL_X86_BUILD:-0}" = "1" ]; then
  CARGO_TARGET_ARGS="--target x86_64-apple-darwin"

  export ORT_DYLIB_PATH="$PWD/vendor/onnxruntime/build/MacOS/Release/libonnxruntime.1.26.0.dylib"
  export MACOSX_DEPLOYMENT_TARGET="12.3"
fi

case "$PROFILE" in
  debug)
    cargo build $CARGO_TARGET_ARGS
    PROFILE_DIR="debug"
    ;;
  release)
    cargo build --release $CARGO_TARGET_ARGS
    PROFILE_DIR="release"
    ;;
  *)
    echo "Usage: $0 [debug|release]"
    exit 1
    ;;
esac

if [ "${SUBSTRATE_LOCAL_X86_BUILD:-0}" = "1" ]; then
  TARGET_DIR="target/x86_64-apple-darwin/$PROFILE_DIR"
else
  TARGET_DIR="target/$PROFILE_DIR"
fi

mkdir -p "$TARGET_DIR/models"
rsync -a --delete models/ "$TARGET_DIR/models/"

mkdir -p "$TARGET_DIR/web"
rsync -a --delete web/dist/ "$TARGET_DIR/web/"

echo "Build complete: $TARGET_DIR"
