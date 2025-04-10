#!/bin/bash
set -e

echo "Building HIVCluster-RS and HIVAnnotate-RS for WebAssembly..."

# Install wasm-pack if not already installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack is not installed. Installing..."
    cargo install wasm-pack
fi

# Build HIVCluster for web
echo "Building HIVCluster for web target..."
wasm-pack build --target web --out-dir pkg/web
# Update package name for web target
sed -i.bak 's/"name": "hivcluster_rs"/"name": "hivcluster_rs_web"/' pkg/web/package.json
rm pkg/web/package.json.bak

# Build HIVCluster for Node.js
echo "Building HIVCluster for nodejs target..."
wasm-pack build --target nodejs --out-dir pkg/node
# Update package name for node target
sed -i.bak 's/"name": "hivcluster_rs"/"name": "hivcluster_rs_node"/' pkg/node/package.json
rm pkg/node/package.json.bak

# Build HIVAnnotate for web
echo "Building HIVAnnotate for web target..."
RUSTFLAGS='--cfg feature="annotation"' wasm-pack build --target web --out-dir pkg/hivannotate-web -- --features "annotation"
# Update package name for web target
sed -i.bak 's/"name": "hivcluster_rs"/"name": "hivannotate_rs_web"/' pkg/hivannotate-web/package.json
# Update description
sed -i.bak 's/"description": ".*"/"description": "WebAssembly bindings for HIVAnnotate - Annotating HIV transmission networks"/' pkg/hivannotate-web/package.json
rm pkg/hivannotate-web/package.json.bak

# Build HIVAnnotate for Node.js
echo "Building HIVAnnotate for nodejs target..."
RUSTFLAGS='--cfg feature="annotation"' wasm-pack build --target nodejs --out-dir pkg/hivannotate-node -- --features "annotation"
# Update package name for node target
sed -i.bak 's/"name": "hivcluster_rs"/"name": "hivannotate_rs_node"/' pkg/hivannotate-node/package.json
# Update description
sed -i.bak 's/"description": ".*"/"description": "Node.js bindings for HIVAnnotate - Annotating HIV transmission networks"/' pkg/hivannotate-node/package.json
rm pkg/hivannotate-node/package.json.bak

echo "WebAssembly build completed!"
echo "Output files in pkg/ directory"

# Publish options
if [ "$1" == "--publish" ]; then
    echo "Publishing packages to npm..."
    
    cd pkg/web
    npm publish --access public
    cd ../..
    
    cd pkg/node
    npm publish --access public
    cd ../..
    
    cd pkg/hivannotate-web
    npm publish --access public
    cd ../..
    
    cd pkg/hivannotate-node
    npm publish --access public
    cd ../..
    
    echo "Packages published to npm successfully!"
elif [ "$1" == "--pack" ]; then
    echo "Creating npm packages without publishing..."
    
    cd pkg/web
    npm pack
    mv *.tgz ../..
    cd ../..
    
    cd pkg/node
    npm pack
    mv *.tgz ../..
    cd ../..
    
    cd pkg/hivannotate-web
    npm pack
    mv *.tgz ../..
    cd ../..
    
    cd pkg/hivannotate-node
    npm pack
    mv *.tgz ../..
    cd ../..
    
    echo "npm packages created in root directory!"
else
    echo ""
    echo "To publish to npm, run: ./build_wasm.sh --publish"
    echo "To create npm packages without publishing, run: ./build_wasm.sh --pack"
fi