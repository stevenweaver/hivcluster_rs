# HIVCluster Rust Implementation

A Rust implementation of the HIVCluster algorithm for constructing HIV transmission networks from genetic distance data.

## Features

- Fast performance using Rust's memory safety and concurrency features
- Accurate cluster detection and network statistics
- Support for multiple input formats (CSV, JSON)
- Flexible distance threshold configuration
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

```bash
# Process a CSV file
./target/release/hivcluster -i input.csv -t 0.015 -o output.json

# Process a JSON file
./target/release/hivcluster -i input.json -t 0.015 -o output.json

# Get help
./target/release/hivcluster --help
```

### Command-line options

- `-i`, `--input`: Input file path (CSV or JSON)
- `-t`, `--threshold`: Distance threshold for edge creation
- `-o`, `--output`: Output file path for results
- `-f`, `--format`: Output format (default: JSON)

## WebAssembly Support

This implementation includes WebAssembly bindings for use in web applications.

### Building for WASM

```bash
./build_wasm.sh
```

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test network_test
```

## License

[MIT License](LICENSE)