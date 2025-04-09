use hivcluster_rs::build_network_internal;
use std::time::Instant;

#[test]
fn test_performance() {
    // Create a smaller synthetic network for test purposes
    let mut csv_data = String::new();
    
    // Generate a network with 1,000 nodes (instead of 10,000 for test speed)
    let node_count = 1_000;
    let connections_per_node = 3;
    
    for i in 1..node_count {
        // Connect each node to a few nodes ahead (create a mesh-like structure)
        for j in 1..=connections_per_node {
            if i + j < node_count {
                // Edge with distance that varies slightly based on node IDs
                let distance = 0.015 + (i as f64 * 0.0000001);
                csv_data.push_str(&format!("N{:05},N{:05},{:.6}\n", i, i+j, distance));
            }
        }
        
        // Add some random longer-distance connections to create more complex clustering
        if i % 100 == 0 {
            let target = (i + 200) % node_count;
            if target > 0 {
                csv_data.push_str(&format!("N{:05},N{:05},{:.6}\n", i, target, 0.025));
            }
        }
    }
    
    // Measure time to construct the network
    let start = Instant::now();
    let mut network = build_network_internal(&csv_data, 0.03).unwrap();
    let build_time = start.elapsed();
    
    println!("Built network with {} nodes in {:?}", network.nodes.len(), build_time);
    
    // Measure time to compute clusters
    let start = Instant::now();
    network.compute_clusters();
    let cluster_time = start.elapsed();
    
    let clusters = network.retrieve_clusters(false);
    println!("Computed {} clusters in {:?}", clusters.len(), cluster_time);
    
    // Measure JSON serialization time
    let start = Instant::now();
    let json = network.to_json();
    let json_time = start.elapsed();
    
    println!("Generated JSON with {} nodes and {} edges in {:?}", 
             json.nodes.len(), json.edges.len(), json_time);
    
    // Basic verification that network is constructed correctly
    assert!(network.nodes.len() > 0, "Should have created a network");
    assert!(network.edges.len() > 0, "Should have edges");
}