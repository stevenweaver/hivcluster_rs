use hivcluster_rs::annotate_network;
use serde_json::{json, Value};

#[test]
fn test_basic_annotation() {
    // Create a simple network JSON
    let network_json = json!({
        "Nodes": [
            {
                "id": "KU190031",
                "cluster": 1
            },
            {
                "id": "KU190032",
                "cluster": 1
            },
            {
                "id": "KU190033",
                "cluster": 2
            }
        ],
        "Edges": [
            {
                "source": 0,
                "target": 1,
                "distance": 0.01
            }
        ]
    }).to_string();

    // Create attributes
    let attributes_json = json!([
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
    ]).to_string();

    // Create schema
    let schema_json = json!({
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
    }).to_string();

    // Run the annotator
    let result = annotate_network(&network_json, &attributes_json, &schema_json).unwrap();
    
    // Parse the result back to JSON for assertions
    let result_json: Value = serde_json::from_str(&result).unwrap();
    
    // Verify patient_attribute_schema is created
    assert!(result_json.get("patient_attribute_schema").is_some());
    
    // Verify attributes are applied to correct nodes
    let nodes = result_json["Nodes"].as_array().unwrap();
    
    // Check first node
    let node0 = &nodes[0];
    assert_eq!(node0["id"], "KU190031");
    assert!(node0.get("patient_attributes").is_some());
    assert_eq!(node0["patient_attributes"]["country"], "Canada");
    
    // Check second node
    let node1 = &nodes[1];
    assert_eq!(node1["id"], "KU190032");
    assert!(node1.get("patient_attributes").is_some());
    assert_eq!(node1["patient_attributes"]["country"], "USA");
    
    // Third node should have patient_attributes but no values
    let node2 = &nodes[2];
    assert_eq!(node2["id"], "KU190033");
    assert!(node2.get("patient_attributes").is_none());
}

#[test]
fn test_annotation_with_trace_results() {
    // Create a network JSON with trace_results wrapper
    let network_json = json!({
        "trace_results": {
            "Nodes": [
                {
                    "id": "KU190031",
                    "cluster": 1
                },
                {
                    "id": "KU190032",
                    "cluster": 1
                }
            ],
            "Edges": [
                {
                    "source": 0,
                    "target": 1,
                    "distance": 0.01
                }
            ]
        }
    }).to_string();

    // Create attributes
    let attributes_json = json!([
        {
            "ehars_uid": "KU190031",
            "country": "Canada"
        }
    ]).to_string();

    // Create schema
    let schema_json = json!({
        "ehars_uid": {
            "type": "String",
            "label": "Patient ID"
        },
        "country": {
            "type": "String",
            "label": "Country"
        }
    }).to_string();

    // Run the annotator
    let result = annotate_network(&network_json, &attributes_json, &schema_json).unwrap();
    
    // Parse the result back to JSON for assertions
    let result_json: Value = serde_json::from_str(&result).unwrap();
    
    // Verify trace_results structure is preserved
    assert!(result_json.get("trace_results").is_some());
    
    // Verify patient_attribute_schema is created inside trace_results
    assert!(result_json["trace_results"].get("patient_attribute_schema").is_some());
    
    // Verify attributes are applied to correct nodes
    let nodes = result_json["trace_results"]["Nodes"].as_array().unwrap();
    
    // Check first node
    let node0 = &nodes[0];
    assert_eq!(node0["id"], "KU190031");
    assert!(node0.get("patient_attributes").is_some());
    assert_eq!(node0["patient_attributes"]["country"], "Canada");
}

#[test]
fn test_annotation_with_multi_field_key() {
    // Create a simple network JSON
    let network_json = json!({
        "Nodes": [
            {
                "id": "Patient1~Sample1",
                "cluster": 1
            },
            {
                "id": "Patient2~Sample1",
                "cluster": 2
            }
        ],
        "Edges": []
    }).to_string();

    // Create attributes
    let attributes_json = json!([
        {
            "patient_id": "Patient1",
            "sample_id": "Sample1",
            "value": "Test1"
        },
        {
            "patient_id": "Patient2",
            "sample_id": "Sample1",
            "value": "Test2"
        }
    ]).to_string();

    // Create schema with custom keying
    let schema_json = json!({
        "keying": {
            "fields": ["patient_id", "sample_id"],
            "delimiter": "~"
        },
        "patient_id": {
            "type": "String",
            "label": "Patient ID"
        },
        "sample_id": {
            "type": "String",
            "label": "Sample ID"
        },
        "value": {
            "type": "String",
            "label": "Value"
        }
    }).to_string();

    // Run the annotator
    let result = annotate_network(&network_json, &attributes_json, &schema_json).unwrap();
    
    // Parse the result back to JSON for assertions
    let result_json: Value = serde_json::from_str(&result).unwrap();
    
    // Verify attributes are applied to correct nodes
    let nodes = result_json["Nodes"].as_array().unwrap();
    
    // Check first node
    let node0 = &nodes[0];
    assert_eq!(node0["id"], "Patient1~Sample1");
    assert!(node0.get("patient_attributes").is_some());
    assert_eq!(node0["patient_attributes"]["value"], "Test1");
    
    // Check second node
    let node1 = &nodes[1];
    assert_eq!(node1["id"], "Patient2~Sample1");
    assert!(node1.get("patient_attributes").is_some());
    assert_eq!(node1["patient_attributes"]["value"], "Test2");
}

#[test]
fn test_annotation_with_enum_type() {
    // Create a simple network JSON
    let network_json = json!({
        "Nodes": [
            {
                "id": "KU190031",
                "cluster": 1
            },
            {
                "id": "KU190032",
                "cluster": 1
            }
        ],
        "Edges": []
    }).to_string();

    // Create attributes
    let attributes_json = json!([
        {
            "ehars_uid": "KU190031",
            "category": "A"
        },
        {
            "ehars_uid": "KU190032",
            "category": "B"
        }
    ]).to_string();

    // Create schema with enum type
    let schema_json = json!({
        "ehars_uid": {
            "type": "String",
            "label": "Patient ID"
        },
        "category": {
            "type": "enum",
            "label": "Category",
            "enum": ["A", "B", "C"]
        }
    }).to_string();

    // Run the annotator
    let result = annotate_network(&network_json, &attributes_json, &schema_json).unwrap();
    
    // Parse the result back to JSON for assertions
    let result_json: Value = serde_json::from_str(&result).unwrap();
    
    // Verify enum is defined in the schema
    let schema = &result_json["patient_attribute_schema"]["category"];
    assert_eq!(schema["type"], "enum");
    assert!(schema.get("enum").is_some());
    
    // Verify attributes are applied correctly
    let nodes = result_json["Nodes"].as_array().unwrap();
    assert_eq!(nodes[0]["patient_attributes"]["category"], "A");
    assert_eq!(nodes[1]["patient_attributes"]["category"], "B");
}