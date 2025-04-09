# HIVCluster-RS Implementation Details

This document describes the core implementation details of the Rust rewrite of the HIVClustering library, focusing on similarities and differences with the original Python implementation.

## Core Architecture

The Rust implementation maintains the essential architecture of the original Python version, with some simplifications and optimizations:

### Data Structures

1. **Patient (Node)**
   - Similar fields as the Python version (id, dates, attributes, cluster_id)
   - Uses Rc<Patient> for shared ownership instead of Python's reference semantics
   - Implemented Eq, Hash, Ord traits for compatibility with collections

2. **Edge**
   - Similar concept to Python but optimized for WASM memory efficiency
   - Always normalizes source/target so source.id < target.id for consistent identification
   - Contains distance value directly in the Edge struct

3. **TransmissionNetwork**
   - Manages collections of nodes and edges
   - Uses Vec<Rc<Patient>> and Vec<Edge> for storage instead of dictionaries
   - Maintains adjacency lists for efficient graph traversal
   - Builds indices for fast ID-based lookups

### Core Algorithms

1. **Network Construction**
   - Similar to Python but with stronger type safety
   - Reads CSV input and applies distance threshold filtering
   - Creates nodes and edges in one pass through the data

2. **Clustering Algorithm**
   - Uses breadth-first search to identify connected components
   - Simplified approach focusing on core functionality
   - O(V+E) complexity similar to the original

3. **JSON Serialization**
   - Formats data to match the original Python output structure
   - Optimized for WASM with no intermediary steps

## Key Differences

### Added Features

1. **WebAssembly Support**
   - First-class WASM support for in-browser execution
   - Optimized memory model for browser environment
   - JS-friendly API design

2. **Strict Type Safety**
   - Comprehensive error handling with custom error types
   - No runtime type errors possible in the Rust code

### Omitted Features

1. **Edge Filtering**
   - The complex sequence-based edge filtering with triangles/cycles was omitted
   - This functionality required Python-specific dependencies

2. **Degree Distribution Analysis**
   - Statistical analysis for network degree distribution was removed
   - This was considered secondary to the core network construction

3. **Automatic Threshold Tuning**
   - The auto-tuning algorithm was not implemented
   - This was a complex, heuristic-based feature not essential for core functionality

## Performance Optimizations

1. **Memory Efficiency**
   - Uses Rust's ownership model to minimize allocations
   - Avoids unnecessary clones of large data structures
   - Indexed lookups for O(1) complexity

2. **Algorithmic Improvements**
   - Pre-allocation of data structures where possible
   - Efficient graph representations for traversal
   - Normalized edge representation to avoid duplicated logic

3. **WASM Considerations**
   - Avoids features that perform poorly in WASM (like threading)
   - Minimizes copying of large data structures
   - Provides clean interface boundaries for JS interop

## Testing Strategy

1. **Unit Tests**
   - Tests for each core component
   - Input validation and error handling
   - Edge cases like self-loops, invalid inputs

2. **Integration Tests**
   - End-to-end tests with sample networks
   - Verification of output format
   - Performance tests with larger networks

## Compatibility with Original Format

The Rust implementation:

1. Reads the same CSV format as the original
2. Produces compatible JSON output
3. Maintains the same clustering behavior
4. Provides equivalent network statistics

This ensures that it can be used as a drop-in replacement for applications that only require the core network construction functionality.