#!/usr/bin/env bash

# Build Swift dylib
set -euo pipefail

echo "🔨  Building Apple On-Device AI library components …"

# Check if we're on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "❌ Error: This library can only be built on macOS"
    exit 1
fi

# Require macOS 26+ (FoundationModels)
MACOS_MAJOR=$(sw_vers -productVersion | cut -d. -f1)
if (( MACOS_MAJOR < 26 )); then
  echo "❌  Need macOS 26.0+ (FoundationModels). Current: $(sw_vers -productVersion)" >&2
  exit 1
fi

# Create build directory
mkdir -p build

# Build Swift dylib
echo "📦  Swift → build/libappleai.dylib"
swiftc \
  -O -whole-module-optimization \
  -emit-library -emit-module -module-name AppleOnDeviceAI \
  -framework Foundation -framework FoundationModels \
  -target arm64-apple-macos26.0 \
  -Xlinker -install_name -Xlinker @rpath/libappleai.dylib \
  -Xlinker -rpath -Xlinker @loader_path \
  ../ailib/apple-ai.swift \
  -o build/libappleai.dylib

echo "✅  Swift dylib built"
