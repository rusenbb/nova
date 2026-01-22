#!/bin/bash
#
# Build script for Nova macOS frontend
#
# Usage: ./scripts/build-macos.sh [debug|release]
#
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FRONTEND_DIR="$PROJECT_ROOT/frontends/macos"
BUILD_TYPE="${1:-debug}"

echo "==> Building Nova for macOS ($BUILD_TYPE)"

# Step 1: Build Rust library
echo "==> Building Rust library..."
if [ "$BUILD_TYPE" == "release" ]; then
    cargo build --lib --release --no-default-features
    LIB_DIR="$PROJECT_ROOT/target/release"
else
    cargo build --lib --no-default-features
    LIB_DIR="$PROJECT_ROOT/target/debug"
fi

# Verify dylib exists
if [ ! -f "$LIB_DIR/libnova.dylib" ]; then
    echo "Error: libnova.dylib not found in $LIB_DIR"
    exit 1
fi
echo "==> Rust library built: $LIB_DIR/libnova.dylib"

# Step 2: Check for xcodegen
if ! command -v xcodegen &> /dev/null; then
    echo "==> xcodegen not found, installing..."
    brew install xcodegen
fi

# Step 3: Generate Xcode project
echo "==> Generating Xcode project..."
cd "$FRONTEND_DIR"
xcodegen generate

# Step 4: Build with xcodebuild
echo "==> Building Swift frontend..."
if [ "$BUILD_TYPE" == "release" ]; then
    xcodebuild -project Nova.xcodeproj \
        -scheme Nova \
        -configuration Release \
        -derivedDataPath build \
        LIBRARY_SEARCH_PATHS="$LIB_DIR" \
        build
    APP_PATH="$FRONTEND_DIR/build/Build/Products/Release/Nova.app"
else
    xcodebuild -project Nova.xcodeproj \
        -scheme Nova \
        -configuration Debug \
        -derivedDataPath build \
        LIBRARY_SEARCH_PATHS="$LIB_DIR" \
        build
    APP_PATH="$FRONTEND_DIR/build/Build/Products/Debug/Nova.app"
fi

# Step 5: Copy dylib into app bundle
echo "==> Copying libnova.dylib into app bundle..."
mkdir -p "$APP_PATH/Contents/Frameworks"
cp "$LIB_DIR/libnova.dylib" "$APP_PATH/Contents/Frameworks/"

# Fix dylib path
install_name_tool -change \
    "libnova.dylib" \
    "@executable_path/../Frameworks/libnova.dylib" \
    "$APP_PATH/Contents/MacOS/Nova" 2>/dev/null || true

echo "==> Build complete!"
echo "    App: $APP_PATH"
echo ""
echo "To run: open \"$APP_PATH\""
echo "Or: \"$APP_PATH/Contents/MacOS/Nova\""
