#!/usr/bin/env bash
set -euo pipefail

PROFILE="debug"
NO_BUILD=false

# Parse args
for arg in "$@"; do
  case "$arg" in
    --nobuild)
      NO_BUILD=true
      ;;
    debug|release)
      PROFILE="$arg"
      ;;
    *)
      echo "Unknown argument: $arg"
      exit 1
      ;;
  esac
done

if [ "$NO_BUILD" = false ]; then
  ./build.sh "$PROFILE"
else
  echo "Skipping build (--nobuild)"
fi

if [ -d "target/x86_64-apple-darwin/$PROFILE" ]; then
  TARGET_DIR="target/x86_64-apple-darwin/$PROFILE"
else
  TARGET_DIR="target/$PROFILE"
fi

BIN="$TARGET_DIR/substrate"

if [ ! -f "$BIN" ]; then
  echo "Binary not found: $BIN"
  exit 1
fi

if [ -f "vendor/onnxruntime/build/MacOS/Release/libonnxruntime.dylib" ]; then
  export ORT_DYLIB_PATH="$PWD/vendor/onnxruntime/build/MacOS/Release/libonnxruntime.dylib"
  echo -e "Using vendored ONNX Runtime: $ORT_DYLIB_PATH\n"
fi

exec "$BIN"
