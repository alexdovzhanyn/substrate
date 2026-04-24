#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-}"
PROFILE_DIR="release"
BINARY_NAME="substrate"

if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version>"
  echo "Example: $0 0.1.0"
  exit 1
fi

TARGET_TRIPLE="x86_64-apple-darwin"

echo "Building Rust release binary..."
cargo build --release

echo "Building web app..."
npm --prefix web ci
npm --prefix web run build

if [ -d "target/$TARGET_TRIPLE/$PROFILE_DIR" ]; then
  TARGET_DIR="target/$TARGET_TRIPLE/$PROFILE_DIR"
  ARTIFACT_TARGET="$TARGET_TRIPLE"
else
  TARGET_DIR="target/$PROFILE_DIR"
  ARTIFACT_TARGET="$(rustc -vV | awk '/host:/ { print $2 }')"
fi

DIST_ROOT="dist"
PACKAGE_NAME="substrate-v${VERSION}-${ARTIFACT_TARGET}"
PACKAGE_DIR="$DIST_ROOT/$PACKAGE_NAME"

echo "Creating package directory..."
rm -rf "$PACKAGE_DIR"
mkdir -p "$PACKAGE_DIR"

echo "Copying runtime files..."
cp "$TARGET_DIR/$BINARY_NAME" "$PACKAGE_DIR/$BINARY_NAME"

mkdir -p "$PACKAGE_DIR/models"
rsync -a --delete models/ "$PACKAGE_DIR/models/"

mkdir -p "$PACKAGE_DIR/web"
rsync -a --delete web/dist/ "$PACKAGE_DIR/web/"

if [ -f README.md ]; then
  cp README.md "$PACKAGE_DIR/README.md"
fi

if [ -f LICENSE ]; then
  cp LICENSE "$PACKAGE_DIR/LICENSE.md"
fi

echo "Creating archive..."
tar -czf "$DIST_ROOT/$PACKAGE_NAME.tar.gz" -C "$DIST_ROOT" "$PACKAGE_NAME"

echo "Package complete:"
echo "  $DIST_ROOT/$PACKAGE_NAME.tar.gz"
