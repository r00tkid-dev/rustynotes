#!/bin/bash

# Simple build script for rustynotes - builds for current platform
# Usage: ./build-local.sh

set -e

echo "Building rustynotes for current platform..."

# Create release directory
mkdir -p release

# Build for current platform
echo "Building optimized release binary..."
cargo build --release

# Copy binary to release directory
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    # Windows
    cp target/release/rustynotes.exe release/
    cd release
    zip rustynotes-windows-x86_64.zip rustynotes.exe
    rm rustynotes.exe
    cd ..
    echo "Created: release/rustynotes-windows-x86_64.zip"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    cp target/release/rustynotes release/
    if [[ $(uname -m) == "arm64" ]]; then
        tar -czf release/rustynotes-macos-arm64.tar.gz -C release rustynotes
        echo "Created: release/rustynotes-macos-arm64.tar.gz"
    else
        tar -czf release/rustynotes-macos-x86_64.tar.gz -C release rustynotes
        echo "Created: release/rustynotes-macos-x86_64.tar.gz"
    fi
    rm release/rustynotes
else
    # Linux
    cp target/release/rustynotes release/
    tar -czf release/rustynotes-linux-x86_64.tar.gz -C release rustynotes
    rm release/rustynotes
    echo "Created: release/rustynotes-linux-x86_64.tar.gz"
fi

# Generate checksum
echo "Generating checksums..."
cd release
sha256sum * > checksums.txt
cd ..

echo "Build complete! Release files in ./release/"
echo "Files created:"
ls -la release/