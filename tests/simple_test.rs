use hivcluster_rs::build_network_internal;

const TEST_CSV: &str = "ID1,ID2,0.01
ID1,ID3,0.02
ID2,ID4,0.015
ID5,ID6,0.03
ID7,ID8,0.01";

#[test]
fn test_simple_network() {
    // Build network
    let network = build_network_internal(TEST_CSV, 0.03).unwrap();
    
    // Verify basic properties
    assert_eq!(network.nodes.len(), 8, "Should have 8 nodes");
    assert_eq!(network.edges.len(), 4, "Should have 4 edges");
    
    // Export to JSON
    let json = network.to_json();
    
    // Verify JSON structure
    assert_eq!(json.nodes.len(), 8, "JSON should contain 8 nodes");
    assert_eq!(json.edges.len(), 4, "JSON should contain 4 edges");
    
    // Check node IDs in JSON
    let node_ids: Vec<String> = json.nodes.iter().map(|n| n.id.clone()).collect();
    for id in &["ID1", "ID2", "ID3", "ID4", "ID5", "ID6", "ID7", "ID8"] {
        assert!(node_ids.contains(&id.to_string()), "JSON should include node {}", id);
    }
    
    // Verify all edges are present in JSON
    let edge_pairs: Vec<(String, String)> = json.edges.iter()
        .map(|e| (e.source.clone(), e.target.clone()))
        .collect();
    
    assert!(edge_pairs.contains(&("ID1".to_string(), "ID3".to_string())) ||
            edge_pairs.contains(&("ID3".to_string(), "ID1".to_string())),
            "JSON should contain edge ID1-ID3");
            
    assert!(edge_pairs.contains(&("ID2".to_string(), "ID4".to_string())) ||
            edge_pairs.contains(&("ID4".to_string(), "ID2".to_string())),
            "JSON should contain edge ID2-ID4");
            
    assert!(edge_pairs.contains(&("ID5".to_string(), "ID6".to_string())) ||
            edge_pairs.contains(&("ID6".to_string(), "ID5".to_string())),
            "JSON should contain edge ID5-ID6");
            
    assert!(edge_pairs.contains(&("ID7".to_string(), "ID8".to_string())) ||
            edge_pairs.contains(&("ID8".to_string(), "ID7".to_string())),
            "JSON should contain edge ID7-ID8");
}