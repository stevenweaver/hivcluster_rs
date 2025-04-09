use hivcluster_rs::{build_network_internal, InputFormat, TransmissionNetwork};

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
    let network = build_network_internal(TEST_CSV, 0.03).unwrap();
    
    // Check node count
    assert_eq!(network.nodes.len(), 6, "Should have 6 nodes");
    
    // Check edge count
    assert_eq!(network.edges.len(), 3, "Should have 3 edges");
    
    // Check cluster count
    let clusters = network.retrieve_clusters(false);
    assert_eq!(clusters.len(), 2, "Should have 2 clusters");
    
    // Verify cluster assignments
    let node_ids: Vec<String> = network.nodes.iter()
        .map(|n| n.id.clone())
        .collect();
    
    assert!(node_ids.contains(&"ID1".to_string()));
    assert!(node_ids.contains(&"ID2".to_string()));
    assert!(node_ids.contains(&"ID3".to_string()));
    assert!(node_ids.contains(&"ID4".to_string()));
    assert!(node_ids.contains(&"ID5".to_string()));
    assert!(node_ids.contains(&"ID6".to_string()));
}

#[test]
fn test_distance_threshold() {
    let network = build_network_internal(TEST_CSV_THRESHOLD, 0.03).unwrap();
    
    // Only edges with distance <= threshold should be included
    assert_eq!(network.edges.len(), 3, "Should have 3 edges");
    
    // Check distances stored
    let dist_key = if "ID1" < "ID2" {
        ("ID1".to_string(), "ID2".to_string())
    } else {
        ("ID2".to_string(), "ID1".to_string())
    };
    
    assert_eq!(network.distances.get(&dist_key), Some(&0.01));
}

#[test]
fn test_cluster_detection() {
    let mut network = TransmissionNetwork::new();
    network.read_from_csv_str(TEST_CSV, 0.03, InputFormat::Plain).unwrap();
    network.compute_clusters();
    
    // There should be 2 clusters
    // Cluster 1: ID1, ID2, ID3, ID4
    // Cluster 2: ID5, ID6
    let clusters = network.retrieve_clusters(false);
    assert_eq!(clusters.len(), 3, "Should have 3 clusters (ID7-ID8 not connected with threshold 0.03)");
    
    // Find which cluster has ID1
    let id1_cluster = network.nodes.iter()
        .find(|n| n.id == "ID1")
        .and_then(|n| n.cluster_id);
    
    assert!(id1_cluster.is_some(), "ID1 should have a cluster ID");
    
    // Check if ID2, ID3, ID4 are in the same cluster
    let id1_cluster = id1_cluster.unwrap();
    for id in &["ID2", "ID3", "ID4"] {
        let node_cluster = network.nodes.iter()
            .find(|n| n.id == *id)
            .and_then(|n| n.cluster_id);
        assert_eq!(node_cluster, Some(id1_cluster), "{} should be in the same cluster as ID1", id);
    }
    
    // ID5 and ID6 should be in a different cluster
    let id5_cluster = network.nodes.iter()
        .find(|n| n.id == "ID5")
        .and_then(|n| n.cluster_id);
    
    assert!(id5_cluster.is_some() && id5_cluster != Some(id1_cluster), 
           "ID5 should be in a different cluster than ID1");
    
    let id6_cluster = network.nodes.iter()
        .find(|n| n.id == "ID6")
        .and_then(|n| n.cluster_id);
    
    assert_eq!(id5_cluster, id6_cluster, "ID5 and ID6 should be in the same cluster");
}

#[test]
fn test_json_output() {
    let network = build_network_internal(TEST_CSV, 0.03).unwrap();
    let json = network.to_json();
    
    // Validate JSON structure
    assert_eq!(json.nodes.len(), 6, "JSON should contain 6 nodes");
    assert_eq!(json.edges.len(), 3, "JSON should contain 3 edges");
    
    // Check that cluster information is included
    assert!(json.clusters.len() > 0, "JSON should include clusters");
    
    // Edge source and target should be valid node IDs
    let node_ids: std::collections::HashSet<String> = json.nodes.iter()
        .map(|n| n.id.clone())
        .collect();
    
    for edge in &json.edges {
        assert!(node_ids.contains(&edge.source), "Edge source should be a valid node ID");
        assert!(node_ids.contains(&edge.target), "Edge target should be a valid node ID");
    }
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
    
    let mut network = build_network_internal(&csv, 0.03).unwrap();
    
    // Just verify it computes without errors
    assert!(network.nodes.len() > 50, "Should have created a large network");
    assert!(network.edges.len() > 100, "Should have many edges");
    
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
    let result = build_network_internal(invalid_csv, 0.03);
    assert!(result.is_err(), "Should error on invalid CSV");
    
    // Test self-loop rejection
    let self_loop_csv = "ID1,ID1,0.01";
    let result = build_network_internal(self_loop_csv, 0.03);
    assert!(result.is_err(), "Should reject self-loop edges");
    
    // Test invalid distance value
    let invalid_dist_csv = "ID1,ID2,not_a_number";
    let result = build_network_internal(invalid_dist_csv, 0.03);
    assert!(result.is_err(), "Should error on invalid distance value");
}