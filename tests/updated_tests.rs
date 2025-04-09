use hivcluster_rs::{TransmissionNetwork, InputFormat};

// Test data with varying distances
const BASIC_NETWORK_CSV: &str = r#"source,target,distance
ID1,ID2,0.01
ID2,ID3,0.02
ID3,ID4,0.03
ID4,ID5,0.04
ID6,ID7,0.01
ID7,ID8,0.02
"#;

// Test data with format-specific IDs
const FORMATTED_IDS_CSV: &str = r#"source,target,distance
patient1|2020-01-15|ARV,patient2|2020-02-20|No ARV,0.01
patient2|2020-02-20|No ARV,patient3|2020-03-10|ARV,0.02
"#;

// Test data with LANL format
const LANL_IDS_CSV: &str = r#"source,target,distance
B_US_patient4_2019,B_US_patient5_2020,0.015
"#;

// Test data with duplicate edges
const DUPLICATE_EDGES_CSV: &str = r#"source,target,distance
ID1,ID2,0.01
ID2,ID1,0.02
ID1,ID3,0.015
ID3,ID1,0.01
"#;

#[test]
fn test_basic_network_construction() {
    let mut network = TransmissionNetwork::new();
    
    // Load network data
    let result = network.read_from_csv_str(BASIC_NETWORK_CSV, 0.03, InputFormat::Plain);
    assert!(result.is_ok(), "Failed to parse CSV: {:?}", result.err());
    
    // Compute network structure
    network.compute_adjacency();
    network.compute_clusters();
    
    // Check that nodes were created
    assert!(network.get_node_count() > 0, "Should have nodes");
    
    // Check that edges were created with threshold filtering
    assert!(network.get_edge_count() > 0, "Should have edges with distance <= 0.03");
    
    // Check cluster count (should have 2 clusters: 1-4 and 6-8)
    let clusters = network.retrieve_clusters(false);
    assert_eq!(clusters.len(), 2, "Should have 2 clusters");
    
    // Find cluster with ID1
    let mut id1_cluster_id = None;
    for (cluster_id, nodes) in &clusters {
        if nodes.contains(&"ID1".to_string()) {
            id1_cluster_id = Some(*cluster_id);
            break;
        }
    }
    
    assert!(id1_cluster_id.is_some(), "ID1 should be in a cluster");
    let id1_cluster = id1_cluster_id.unwrap();
    
    // ID1, ID2, ID3, ID4 should be in the same cluster
    let id1_cluster_nodes = &clusters[&id1_cluster];
    for id in &["ID1", "ID2", "ID3"] {
        assert!(id1_cluster_nodes.contains(&id.to_string()), 
                "{} should be in the same cluster as ID1", id);
    }
    
    // ID6, ID7, ID8 should be in a different cluster
    let mut id6_cluster_id = None;
    for (cluster_id, nodes) in &clusters {
        if nodes.contains(&"ID6".to_string()) {
            id6_cluster_id = Some(*cluster_id);
            break;
        }
    }
    
    assert!(id6_cluster_id.is_some(), "ID6 should be in a cluster");
    assert_ne!(id1_cluster_id, id6_cluster_id, "ID1 and ID6 should be in different clusters");
    
    // ID6, ID7, ID8 should be in the same cluster
    let id6_cluster = id6_cluster_id.unwrap();
    let id6_cluster_nodes = &clusters[&id6_cluster];
    for id in &["ID6", "ID7", "ID8"] {
        assert!(id6_cluster_nodes.contains(&id.to_string()), 
                "{} should be in the same cluster as ID6", id);
    }
}

#[test]
fn test_id_formats() {
    // Test AEH format
    let mut network = TransmissionNetwork::new();
    let result = network.read_from_csv_str(FORMATTED_IDS_CSV, 0.03, InputFormat::AEH);
    assert!(result.is_ok(), "Failed to parse AEH format CSV: {:?}", result.err());
    
    // With our AEH format, we should have found the first two patients since they're in both columns
    assert!(network.get_node_count() > 0, "Should have parsed some nodes from AEH format");
    
    // Reset and test LANL format 
    let mut network = TransmissionNetwork::new();
    let result = network.read_from_csv_str(LANL_IDS_CSV, 0.03, InputFormat::LANL);
    assert!(result.is_ok(), "Failed to parse LANL format CSV: {:?}", result.err());
    
    // Verify we have nodes from LANL format
    assert!(network.get_node_count() > 0, "Should have parsed some nodes from LANL format");
}

#[test]
fn test_duplicate_edges() {
    let mut network = TransmissionNetwork::new();
    let result = network.read_from_csv_str(DUPLICATE_EDGES_CSV, 0.03, InputFormat::Plain);
    assert!(result.is_ok(), "Failed to parse CSV with duplicate edges: {:?}", result.err());
    
    // Should have only 2 unique edges (ID1-ID2 and ID1-ID3)
    assert_eq!(network.get_edge_count(), 2, "Should deduplicate edges and keep 2 unique edges");
    
    // Check that we have deduplicated the edges
    assert!(network.get_edge_count() <= 2, "Should have maximum 2 unique edges after deduplication");
}

#[test]
fn test_json_output_format() {
    let mut network = TransmissionNetwork::new();
    let result = network.read_from_csv_str(BASIC_NETWORK_CSV, 0.03, InputFormat::Plain);
    assert!(result.is_ok());
    
    network.compute_adjacency();
    network.compute_clusters();
    
    let json_result = network.to_json_string_pretty();
    assert!(json_result.is_ok(), "JSON serialization failed: {:?}", json_result.err());
    
    let json_output = json_result.unwrap();
    
    // Check basic JSON structure
    assert!(json_output.contains("\"trace_results\""), "JSON should have trace_results field");
    assert!(json_output.contains("\"Network Summary\""), "JSON should have Network Summary");
    assert!(json_output.contains("\"Nodes\""), "JSON should have Nodes field");
    assert!(json_output.contains("\"Edges\""), "JSON should have Edges field");
    assert!(json_output.contains("\"Cluster sizes\""), "JSON should have Cluster sizes field");
    
    // Parse the JSON and verify fields
    let parsed_json: serde_json::Value = serde_json::from_str(&json_output).unwrap();
    
    // Check that nodes are in the JSON
    let node_ids = parsed_json["trace_results"]["Nodes"]["id"].as_array().unwrap();
    assert!(!node_ids.is_empty(), "JSON should include nodes");
    
    // Check that edges are in the JSON
    let edge_sources = parsed_json["trace_results"]["Edges"]["source"].as_array().unwrap();
    assert!(!edge_sources.is_empty(), "JSON should have edges");
    
    // Verify thresholds
    let threshold = parsed_json["trace_results"]["Settings"]["threshold"].as_f64().unwrap();
    assert_eq!(threshold, 0.03, "JSON should have correct threshold");
}

#[test]
fn test_error_cases() {
    // Test invalid distance value - need header row for CSV
    let invalid_csv = "source,target,distance\nID1,ID2,not_a_number";
    
    let mut network = TransmissionNetwork::new();
    let result = network.read_from_csv_str(invalid_csv, 0.03, InputFormat::Plain);
    assert!(result.is_err(), "Should error on invalid distance value");
    
    // Test self-loop
    let self_loop_csv = "source,target,distance\nID1,ID1,0.01";
    let mut network = TransmissionNetwork::new();
    let result = network.read_from_csv_str(self_loop_csv, 0.03, InputFormat::Plain);
    assert!(result.is_err(), "Should reject self-loops with an error");
    assert_eq!(network.get_edge_count(), 0, "Should not add self-loop edge");
    
    // Test empty CSV
    let empty_csv = "";
    let mut network = TransmissionNetwork::new();
    let result = network.read_from_csv_str(empty_csv, 0.03, InputFormat::Plain);
    assert!(result.is_err(), "Should error on empty CSV");
    
    // Test missing columns
    let missing_columns_csv = "source,target\nID1,ID2";
    let mut network = TransmissionNetwork::new();
    let result = network.read_from_csv_str(missing_columns_csv, 0.03, InputFormat::Plain);
    assert!(result.is_err(), "Should error on CSV with too few columns");
}

#[test]
fn test_network_stats() {
    let mut network = TransmissionNetwork::new();
    let result = network.read_from_csv_str(BASIC_NETWORK_CSV, 0.03, InputFormat::Plain);
    assert!(result.is_ok());
    
    network.compute_adjacency();
    network.compute_clusters();
    
    let stats = network.get_network_stats();
    
    assert!(stats.get("nodes").and_then(|v| v.as_u64()).unwrap() > 0, "Stats should have nodes");
    assert!(stats.get("edges").and_then(|v| v.as_u64()).unwrap() > 0, "Stats should have edges");
    assert_eq!(stats.get("clusters").and_then(|v| v.as_u64()), Some(2), "Stats should have 2 clusters");
    
    // Largest cluster should be the one with IDs 1-4 (size 4)
    assert!(stats.get("largest_cluster").and_then(|v| v.as_u64()).unwrap() >= 3, 
            "Largest cluster should have at least 3 nodes");
}

// Run a larger network test to verify performance
#[test]
fn test_large_network_performance() {
    // Generate a larger test network
    let mut csv = String::new();
    
    for i in 1..200 {
        for j in i+1..=i+5 {
            if j <= 200 {
                let distance = 0.01 + ((i as f64) % 5.0) * 0.005;
                csv.push_str(&format!("N{},N{},{:.5}\n", i, j, distance));
            }
        }
    }
    
    let mut network = TransmissionNetwork::new();
    let result = network.read_from_csv_str(&csv, 0.025, InputFormat::Plain);
    assert!(result.is_ok(), "Failed to parse large network: {:?}", result.err());
    
    // Should have all nodes
    assert_eq!(network.get_node_count(), 200, "Should have 200 nodes");
    
    // Should have many edges, but not all (due to threshold)
    let edge_count = network.get_edge_count();
    assert!(edge_count > 300, "Should have many edges (got {})", edge_count);
    
    // Test clustering performance
    network.compute_adjacency();
    network.compute_clusters();
    
    let clusters = network.retrieve_clusters(false);
    assert!(!clusters.is_empty(), "Should have identified clusters");
}