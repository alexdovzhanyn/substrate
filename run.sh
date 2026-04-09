#!/usr/bin/env bash
set -euo pipefail

PROFILE="${1:-debug}"

./build.sh "$PROFILE"

if [ -d "target/x86_64-apple-darwin/$PROFILE" ]; then
  TARGET_DIR="target/x86_64-apple-darwin/$PROFILE"
else
  TARGET_DIR="target/$PROFILE"
fi

BIN="$TARGET_DIR/tesseract"

if [ ! -f "$BIN" ]; then
  echo "Binary not found: $BIN"
  exit 1
fi

if [ -f "vendor/onnxruntime/build/MacOS/Release/libonnxruntime.dylib" ]; then
  export ORT_DYLIB_PATH="$PWD/vendor/onnxruntime/build/MacOS/Release/libonnxruntime.dylib"
  echo "Using vendored ONNX Runtime: $ORT_DYLIB_PATH"
fi

exec "$BIN"
