# HIVCluster and HIVAnnotate Rust Implementation

A Rust implementation of the HIVCluster algorithm for constructing HIV transmission networks from genetic distance data, and the HIVAnnotate functionality for annotating networks with patient attributes.

## Features

- Fast performance using Rust's memory safety and concurrency features
- Accurate cluster detection and network statistics
- Support for multiple input formats (CSV, JSON)
- Flexible distance threshold configuration
- Network annotation with patient attributes
- WebAssembly support for browser-based applications

## Installation

### Prerequisites

- Rust (1.54 or later)
- Cargo (included with Rust)

### Building from source

```bash
# Clone the repository
git clone https://github.com/your-username/hivcluster_rs.git
cd hivcluster_rs

# Build the project
cargo build --release

# Run the binary
./target/release/hivcluster --help
```

## Usage

### HIVCluster

```bash
# Process a CSV file
./target/release/hivcluster -i input.csv -t 0.015 -o output.json

# Process a JSON file
./target/release/hivcluster -i input.json -t 0.015 -o output.json

# Get help
./target/release/hivcluster --help
```

#### Command-line options

- `-i`, `--input`: Input file path (CSV or JSON)
- `-t`, `--threshold`: Distance threshold for edge creation
- `-o`, `--output`: Output file path for results
- `-f`, `--format`: Output format (default: JSON)

### HIVAnnotate

HIVAnnotate allows you to annotate a network with patient attribute data. The annotation process adds patient attributes to the network nodes and includes the attribute schema in the network JSON.

You can use the WebAssembly bindings to annotate networks in both browser and Node.js environments.

## WebAssembly Support

This implementation includes WebAssembly bindings for both HIVCluster and HIVAnnotate.

### Building for WASM

```bash
# Build WASM packages for both web and Node.js
./build_wasm.sh

# Create npm packages without publishing
./build_wasm.sh --pack

# Publish packages to npm
./build_wasm.sh --publish
```

### Using WASM Packages

#### HIVCluster in a Browser project
```bash
# Install the web package
npm install hivcluster_rs_web
```

```javascript
// Import and use the package
import * as hivcluster from 'hivcluster_rs_web';

// Initialize the module
await hivcluster.default();

// Process your data
const result = hivcluster.build_network(csvData, threshold, format);
```

#### HIVCluster in a Node.js project
```bash
# Install the Node.js package
npm install hivcluster_rs_node
```

```javascript
// Import and use the package
const hivcluster = require('hivcluster_rs_node');

// Process your data
const result = hivcluster.build_network(csvData, threshold, format);
```

#### HIVAnnotate in a Browser project
```bash
# Install the web package
npm install hivannotate_rs_web
```

```javascript
// Import and use the package
import * as hivannotate from 'hivannotate_rs_web';

// Initialize the module
await hivannotate.default();

// Annotate your network
const result = hivannotate.annotate_network_json(
  networkJson, 
  attributesJson, 
  schemaJson
);
```

#### HIVAnnotate in a Node.js project
```bash
# Install the Node.js package
npm install hivannotate_rs_node
```

```javascript
// Import and use the package
const hivannotate = require('hivannotate_rs_node');

// Annotate your network
const result = hivannotate.annotate_network_json(
  networkJson, 
  attributesJson, 
  schemaJson
);
```

### Example inputs for HIVAnnotate

#### Network JSON (output from HIVCluster)
```json
{
  "trace_results": {
    "Nodes": [
      {"id": "KU190031", "cluster": 1},
      {"id": "KU190032", "cluster": 1}
    ],
    "Edges": [
      {"source": 0, "target": 1, "distance": 0.01}
    ]
  }
}
```

#### Attributes JSON
```json
[
  {
    "ehars_uid": "KU190031",
    "country": "Canada",
    "collectionDate": "2007-01-03"
  },
  {
    "ehars_uid": "KU190032",
    "country": "USA",
    "collectionDate": "2007-03-23"
  }
]
```

#### Schema JSON
```json
{
  "ehars_uid": {
    "type": "String",
    "label": "Patient ID"
  },
  "country": {
    "type": "String",
    "label": "Country"
  },
  "collectionDate": {
    "type": "String",
    "label": "Collection Date"
  }
}
```

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test network_test

# Run annotation tests
cargo test annotation_test
```

## License

[MIT License](LICENSE)