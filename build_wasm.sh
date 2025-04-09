#!/bin/bash
set -e

echo "Building HIVCluster-RS for WebAssembly..."

# Install wasm-pack if not already installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack is not installed. Installing..."
    cargo install wasm-pack
fi

# Build for web
echo "Building for web target..."
wasm-pack build --target web --out-dir pkg/web

# Build for Node.js
echo "Building for nodejs target..."
wasm-pack build --target nodejs --out-dir pkg/node

echo "WebAssembly build completed!"
echo "Output files in pkg/web and pkg/node directories"