use hivcluster_rs::{InputFormat, TransmissionNetwork};

const TEST_CSV: &str = r#"ID1,ID2,0.01
ID1,ID3,0.02
ID2,ID4,0.015
ID5,ID6,0.03
ID7,ID8,0.01
"#;

const TEST_CSV_THRESHOLD: &str = r#"ID1,ID2,0.01
ID1,ID3,0.02
ID2,ID4,0.015
ID5,ID6,0.05
ID7,ID8,0.04
"#;

#[test]
fn test_network_construction() {
    let mut network = TransmissionNetwork::new();
    network.read_from_csv_str(TEST_CSV, 0.03, InputFormat::Plain).unwrap();
    network.compute_adjacency();
    
    // Check node count - should be all nodes in the dataset
    assert_eq!(network.get_node_count(), 8, "Should have 8 nodes");
    
    // Check edge count - only edges meeting threshold
    // There are 5 edges in our test data
    assert_eq!(network.get_edge_count(), 5, "Should have 5 edges");
    
    // Check cluster count
    network.compute_clusters();
    let clusters = network.retrieve_clusters(false);
    assert_eq!(clusters.len(), 3, "Should have 3 clusters");
    
    // Check all nodes exist by count
    assert_eq!(network.get_node_count(), 8, "Should have all 8 nodes in the network");
    
    // Verify nodes are connected appropriately
    assert!(network.is_node_connected("ID1"), "ID1 should be connected");
    assert!(network.is_node_connected("ID2"), "ID2 should be connected");
    assert!(network.is_node_connected("ID3"), "ID3 should be connected");
    assert!(network.is_node_connected("ID4"), "ID4 should be connected");
    assert!(network.is_node_connected("ID5"), "ID5 should be connected");
    assert!(network.is_node_connected("ID6"), "ID6 should be connected");
    assert!(network.is_node_connected("ID7"), "ID7 should be connected");
    assert!(network.is_node_connected("ID8"), "ID8 should be connected");
}

#[test]
fn test_distance_threshold() {
    let mut network = TransmissionNetwork::new();
    network.read_from_csv_str(TEST_CSV_THRESHOLD, 0.03, InputFormat::Plain).unwrap();
    network.compute_adjacency();
    
    // Only edges with distance <= threshold should be included
    assert_eq!(network.get_edge_count(), 3, "Should have 3 edges");
    
    // Check edges exist by verifying nodes are connected
    assert!(network.is_node_connected("ID1"), "ID1 should be connected");
    assert!(network.is_node_connected("ID2"), "ID2 should be connected");
    
    // We can't directly check the distance since there's no getter, so we'll just 
    // verify the connection exists by checking that the nodes are connected
}

#[test]
fn test_cluster_detection() {
    let mut network = TransmissionNetwork::new();
    network.read_from_csv_str(TEST_CSV, 0.03, InputFormat::Plain).unwrap();
    network.compute_adjacency();
    network.compute_clusters();
    
    // There should be 3 clusters
    // Cluster 1: ID1, ID2, ID3, ID4
    // Cluster 2: ID5, ID6
    // Cluster 3: ID7, ID8
    let clusters = network.retrieve_clusters(false);
    assert_eq!(clusters.len(), 3, "Should have 3 clusters");
    
    // Since we can't directly check cluster IDs through the API,
    // we'll verify that the clusters contain the expected nodes
    
    // Extract all clusters
    let clusters = network.retrieve_clusters(false);
    assert_eq!(clusters.len(), 3, "Should have 3 clusters");
    
    // Find which cluster contains each node
    let mut id1_cluster = None;
    let mut id5_cluster = None;
    let mut id7_cluster = None;
    
    for (cluster_id, nodes) in &clusters {
        if nodes.contains(&"ID1".to_string()) {
            id1_cluster = Some(cluster_id);
        }
        if nodes.contains(&"ID5".to_string()) {
            id5_cluster = Some(cluster_id);
        }
        if nodes.contains(&"ID7".to_string()) {
            id7_cluster = Some(cluster_id);
        }
    }
    
    // Verify each cluster was found
    assert!(id1_cluster.is_some(), "Cluster containing ID1 not found");
    assert!(id5_cluster.is_some(), "Cluster containing ID5 not found");
    assert!(id7_cluster.is_some(), "Cluster containing ID7 not found");
    
    // Get the reference to each cluster's node list
    let id1_cluster = id1_cluster.unwrap();
    let id5_cluster = id5_cluster.unwrap();
    let id7_cluster = id7_cluster.unwrap();
    
    // Make sure these are different clusters
    assert_ne!(id1_cluster, id5_cluster, "ID1 and ID5 should be in different clusters");
    assert_ne!(id1_cluster, id7_cluster, "ID1 and ID7 should be in different clusters");
    assert_ne!(id5_cluster, id7_cluster, "ID5 and ID7 should be in different clusters");
    
    // Check that clusters contain the expected nodes
    if let Some(nodes) = clusters.get(id1_cluster) {
        assert!(nodes.contains(&"ID1".to_string()), "Cluster should contain ID1");
        assert!(nodes.contains(&"ID2".to_string()), "Cluster should contain ID2");
        assert!(nodes.contains(&"ID3".to_string()), "Cluster should contain ID3");
        assert!(nodes.contains(&"ID4".to_string()), "Cluster should contain ID4");
    }
    
    if let Some(nodes) = clusters.get(id5_cluster) {
        assert!(nodes.contains(&"ID5".to_string()), "Cluster should contain ID5");
        assert!(nodes.contains(&"ID6".to_string()), "Cluster should contain ID6");
    }
    
    if let Some(nodes) = clusters.get(id7_cluster) {
        assert!(nodes.contains(&"ID7".to_string()), "Cluster should contain ID7");
        assert!(nodes.contains(&"ID8".to_string()), "Cluster should contain ID8");
    }
}

#[test]
fn test_json_output() {
    let mut network = TransmissionNetwork::new();
    network.read_from_csv_str(TEST_CSV, 0.03, InputFormat::Plain).unwrap();
    network.compute_adjacency();
    network.compute_clusters();
    
    let json = network.to_json();
    
    // Validate JSON structure
    assert_eq!(json.trace_results.network_summary.Nodes, 8, "JSON should contain 8 total nodes");
    assert_eq!(json.trace_results.network_summary.Edges, 5, "JSON should contain 5 edges");
    assert_eq!(json.trace_results.network_summary.Clusters, 3, "JSON should have 3 clusters");
    
    // Check that nodes are in the JSON output
    // Note: We can't directly access the node IDs like this, so we'll just verify the counts
    assert_eq!(json.trace_results.network_summary.Nodes, 8, "JSON should contain 8 nodes");
    
    // And verify the presence of edges by count
    assert_eq!(json.trace_results.network_summary.Edges, 5, "JSON should contain 5 edges");
}

// Test with a larger network to verify performance
#[test]
fn test_larger_network() {
    // Create a larger test network with 100 nodes and ~150 edges
    let mut csv = String::new();
    
    for i in 1..100 {
        // Add some edges to create a realistic network structure
        // Connect to a few nodes ahead
        for j in 1..=3 {
            if i + j <= 100 {
                csv.push_str(&format!("ID{},ID{},{:.5}\n", i, i+j, 0.01 + (i as f64 * 0.0001)));
            }
        }
        
        // Add some random longer links to create multiple clusters
        if i % 10 == 0 {
            let target = (i + 20) % 100 + 1;
            csv.push_str(&format!("ID{},ID{},{:.5}\n", i, target, 0.02));
        }
    }
    
    let mut network = TransmissionNetwork::new();
    network.read_from_csv_str(&csv, 0.03, InputFormat::Plain).unwrap();
    network.compute_adjacency();
    
    // Just verify it computes without errors
    assert!(network.get_node_count() > 50, "Should have created a large network");
    assert!(network.get_edge_count() > 100, "Should have many edges");
    
    // Verify clustering works efficiently
    network.compute_clusters();
    let clusters = network.retrieve_clusters(false);
    assert!(clusters.len() > 0, "Should identify clusters");
}

// Test error cases
#[test]
fn test_error_cases() {
    // Test invalid CSV format
    let invalid_csv = "ID1,ID2\nID3,ID4,0.01";
    let mut network = TransmissionNetwork::new();
    let result = network.read_from_csv_str(invalid_csv, 0.03, InputFormat::Plain);
    assert!(result.is_err(), "Should error on invalid CSV");
    
    // Test self-loop rejection
    let self_loop_csv = "ID1,ID1,0.01";
    let mut network = TransmissionNetwork::new();
    let result = network.read_from_csv_str(self_loop_csv, 0.03, InputFormat::Plain);
    assert!(result.is_err(), "Should reject self-loop edges");
    
    // Test invalid distance value
    let invalid_dist_csv = "ID1,ID2,not_a_number";
    let mut network = TransmissionNetwork::new();
    let result = network.read_from_csv_str(invalid_dist_csv, 0.03, InputFormat::Plain);
    assert!(result.is_err(), "Should error on invalid distance value");
}