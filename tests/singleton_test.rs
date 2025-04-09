use hivcluster_rs::{TransmissionNetwork, InputFormat};

// Test data with nodes that have no connections (singletons)
const SINGLETON_CSV: &str = r#"ID1,ID2,0.01
ID3,ID4,0.02
ID5,ID6,0.01
ID7,ID8,0.2
ID9,ID10,0.3"#;

// We expect 3 clusters in this data with threshold 0.15:
// Cluster 1: ID1-ID2 (connected)
// Cluster 2: ID3-ID4 (connected)
// Cluster 3: ID5-ID6 (connected)
// Singletons: ID7, ID8, ID9, ID10 (not connected due to threshold)

#[test]
fn test_singleton_detection() {
    // Test with a threshold that excludes some connections
    let threshold = 0.15;
    let mut network = TransmissionNetwork::new();
    
    // Parse the CSV - only ID1-ID2, ID3-ID4, ID5-ID6 should have edges due to threshold
    let result = network.read_from_csv_str(SINGLETON_CSV, threshold, InputFormat::Plain);
    assert!(result.is_ok());
    
    // Compute the network structure
    network.compute_adjacency();
    network.compute_clusters();
    
    // Extract singletons
    let singletons = network.extract_singleton_nodes();
    
    // ID7, ID8, ID9, ID10 should be singleton nodes
    assert_eq!(singletons.len(), 4, "Should identify 4 singletons (ID7, ID8, ID9, ID10)");
    for id in &["ID7", "ID8", "ID9", "ID10"] {
        assert!(singletons.contains(&id.to_string()), 
                "{} should be identified as a singleton", id);
    }
    
    // Verify the JSON output has the correct network summary
    let json = network.to_json();
    let summary = &json.trace_results.network_summary;
    
    // Check singleton count
    assert_eq!(summary.Singletons, 4, "JSON should report 4 singleton nodes");
    
    // Check that total nodes = 10 (ID1-ID10)
    assert_eq!(summary.Nodes, 10, "JSON should report 10 total nodes");
    
    // Check that sequences used = 6 (nodes with connections: ID1-ID6)
    assert_eq!(summary.sequences_used, 6, "JSON should report 6 nodes used in connections");
    
    // Check edge count
    assert_eq!(summary.Edges, 3, "JSON should report 3 edges");
    
    // Print cluster info for debugging
    println!("Cluster count: {}", summary.Clusters);
    
    // Let's examine what clusters we have
    let all_clusters = network.retrieve_clusters(true);
    println!("All clusters: {:?}", all_clusters);
    
    // Also examine what's in the JSON output
    println!("JSON cluster count: {}", summary.Clusters);
    println!("JSON cluster sizes: {:?}", json.trace_results.cluster_sizes);
    
    // Check cluster count (ID1-ID2 cluster, ID3-ID4 cluster, and ID5-ID6 cluster)
    assert_eq!(summary.Clusters, 3, "JSON should report 3 clusters");
    
    // Verify degree distribution has the correct count of degree 0 nodes
    let degree_distribution = &json.trace_results.degrees.Distribution;
    assert!(degree_distribution.len() > 0, "Should have a degree distribution");
    assert_eq!(degree_distribution[0], 4, "Degree distribution should show 4 nodes with degree 0");
    
    // Nodes with connections should not be singletons
    for id in &["ID1", "ID2", "ID3", "ID4", "ID5", "ID6"] {
        assert!(!singletons.contains(&id.to_string()),
                "{} should not be identified as a singleton", id);
    }
}