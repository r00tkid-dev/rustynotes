#!/bin/bash

# Build script for rustynotes - creates release binaries for all platforms
# Usage: ./build.sh

set -e

echo "Building rustynotes for all platforms..."

# Create release directory
mkdir -p release

# Build for Linux x86_64
echo "Building for Linux x86_64..."
cross build --target x86_64-unknown-linux-gnu --release
cp target/x86_64-unknown-linux-gnu/release/rustynotes release/
tar -czf release/rustynotes-linux-x86_64.tar.gz -C release rustynotes
rm release/rustynotes

# Build for Windows x86_64
echo "Building for Windows x86_64..."
cross build --target x86_64-pc-windows-gnu --release
cp target/x86_64-pc-windows-gnu/release/rustynotes.exe release/
cd release
zip rustynotes-windows-x86_64.zip rustynotes.exe
rm rustynotes.exe
cd ..

# Build for macOS x86_64 (Intel)
echo "Building for macOS x86_64..."
cross build --target x86_64-apple-darwin --release
cp target/x86_64-apple-darwin/release/rustynotes release/rustynotes-x86_64
tar -czf release/rustynotes-macos-x86_64.tar.gz -C release rustynotes-x86_64
rm release/rustynotes-x86_64

# Build for macOS ARM64 (Apple Silicon)
echo "Building for macOS ARM64..."
cross build --target aarch64-apple-darwin --release
cp target/aarch64-apple-darwin/release/rustynotes release/rustynotes-arm64
tar -czf release/rustynotes-macos-arm64.tar.gz -C release rustynotes-arm64
rm release/rustynotes-arm64

# Generate checksums
echo "Generating checksums..."
cd release
sha256sum * > checksums.txt
cd ..

echo "Build complete! Release files in ./release/"
echo "Files created:"
ls -la release/

echo ""
echo "To create a GitHub release:"
echo "1. Upload the files from ./release/ to GitHub Releases"
echo "2. Copy the checksums.txt content to the release notes"
echo "3. Update the version in Cargo.toml if needed"