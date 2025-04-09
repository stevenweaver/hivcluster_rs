use hivcluster_rs::{InputFormat, TransmissionNetwork};

const TEST_CSV: &str = "ID1,ID2,0.01
ID1,ID3,0.02
ID2,ID4,0.015
ID5,ID6,0.03
ID7,ID8,0.01";

#[test]
fn test_simple_network() {
    // Build network
    let mut network = TransmissionNetwork::new();
    network.read_from_csv_str(TEST_CSV, 0.03, InputFormat::Plain).unwrap();
    network.compute_adjacency();
    
    // Verify basic properties
    assert_eq!(network.get_node_count(), 8, "Should have 8 nodes");
    assert_eq!(network.get_edge_count(), 5, "Should have 5 edges");
    
    // Check that nodes are connected (which indicates edges exist)
    assert!(network.is_node_connected("ID1"), "ID1 should be connected");
    assert!(network.is_node_connected("ID2"), "ID2 should be connected");
    assert!(network.is_node_connected("ID3"), "ID3 should be connected");
    assert!(network.is_node_connected("ID4"), "ID4 should be connected");
    assert!(network.is_node_connected("ID5"), "ID5 should be connected");
    assert!(network.is_node_connected("ID6"), "ID6 should be connected");
    assert!(network.is_node_connected("ID7"), "ID7 should be connected");
    assert!(network.is_node_connected("ID8"), "ID8 should be connected");
    
    // Compute clusters and export to JSON
    network.compute_clusters();
    let json = network.to_json();
    
    // Verify JSON structure
    assert_eq!(json.trace_results.network_summary.Nodes, 8, "JSON should contain 8 nodes");
    assert_eq!(json.trace_results.network_summary.Edges, 5, "JSON should contain 5 edges");
    
    // Verify JSON contains expected number of nodes, edges, and clusters
    // We can't directly access the node IDs and edges in the current API
    
    // Verify node and edge counts
    assert_eq!(json.trace_results.network_summary.Nodes, 8, "JSON should contain 8 nodes");
    assert_eq!(json.trace_results.network_summary.Edges, 5, "JSON should contain 5 edges");
    
    // Verify we have the expected number of clusters (should be 3)
    // Cluster 1: ID1-ID2-ID3-ID4
    // Cluster 2: ID5-ID6
    // Cluster 3: ID7-ID8
    assert_eq!(json.trace_results.network_summary.Clusters, 3, "JSON should contain 3 clusters");
}