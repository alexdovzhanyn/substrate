#!/usr/bin/env bash
set -euo pipefail

PROFILE="${1:-debug}"

case "$PROFILE" in
  debug)
    cargo build
    PROFILE_DIR="debug"
    ;;
  release)
    cargo build --release
    PROFILE_DIR="release"
    ;;
  *)
    echo "Usage: $0 [debug|release]"
    exit 1
    ;;
esac

if [ -d "target/x86_64-apple-darwin/$PROFILE_DIR" ]; then
  TARGET_DIR="target/x86_64-apple-darwin/$PROFILE_DIR"
else
  TARGET_DIR="target/$PROFILE_DIR"
fi

mkdir -p "$TARGET_DIR/models"
rsync -a --delete models/ "$TARGET_DIR/models/"

echo "Build complete: $TARGET_DIR"
