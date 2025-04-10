use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnnotationError {
    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),
    
    #[error("Missing field in input data: {0}")]
    MissingField(String),
    
    #[error("Invalid data format: {0}")]
    InvalidFormat(String),
    
    #[error("Key construction error: {0}")]
    KeyConstructionError(String),
}

// Default key fields and delimiter
const DEFAULT_KEY_FIELDS: [&str; 1] = ["ehars_uid"];
const DEFAULT_KEY_DELIMITER: &str = "~";

/// Main function to annotate a network JSON with attribute data
pub fn annotate_network(
    network_json: &str,
    attributes_json: &str,
    schema_json: &str,
) -> Result<String, AnnotationError> {
    // Parse input JSON files
    let mut network: Value = serde_json::from_str(network_json)?;
    let attributes: Vec<HashMap<String, Value>> = parse_attributes(attributes_json)?;
    let schema: HashMap<String, Value> = serde_json::from_str(schema_json)?;
    
    // Check if we have a "trace_results" key at the root
    let root_trace_results = network.get("trace_results").is_some();
    
    // Get network data (either at root or under trace_results)
    let network_data = if root_trace_results {
        network.get_mut("trace_results").unwrap()
    } else {
        &mut network
    };
    
    // Extract key fields and delimiter from schema, or use defaults
    let (key_fields, key_delimiter) = extract_key_info(&schema);
    
    // Ensure patient_attribute_schema exists
    ensure_key(network_data, "patient_attribute_schema");
    
    // Create attribute schema from input schema
    create_attribute_schema(network_data, &schema)?;
    
    // Handle hivcluster_rs format - nodes as object with parallel arrays
    let mut node_key_map: HashMap<String, usize> = HashMap::new();
    let mut uninjected_fields: HashMap<String, HashSet<String>> = HashMap::new();
    
    // Initialize uninjected fields for tracking
    for (field, _) in schema.iter() {
        if field != "keying" {
            uninjected_fields.insert(field.clone(), HashSet::new());
        }
    }
    
    // Check if Nodes exists and is an object
    if !network_data.get("Nodes").is_some() {
        return Err(AnnotationError::MissingField("Nodes field".to_string()));
    }
    
    // Handle hivcluster format - Nodes is an object with parallel arrays including id
    if let Some(nodes_obj) = network_data["Nodes"].as_object() {
        // The id array must exist
        if !nodes_obj.contains_key("id") {
            return Err(AnnotationError::MissingField("Nodes.id array".to_string()));
        }
        
        let ids = nodes_obj["id"].as_array()
            .ok_or_else(|| AnnotationError::MissingField("Nodes.id is not an array".to_string()))?;
        
        // Track all node IDs
        for (idx, id_value) in ids.iter().enumerate() {
            if let Some(id) = id_value.as_str() {
                let node_key = construct_node_key(id, &key_fields, &key_delimiter)?;
                node_key_map.insert(node_key, idx);
                
                // Track all node IDs as initially uninjected for each field
                for (_, field_set) in uninjected_fields.iter_mut() {
                    field_set.insert(id.to_string());
                }
            }
        }
        
        // Initialize patient_attributes if needed
        if !nodes_obj.contains_key("patient_attributes") {
            // First store the number of nodes we need
            let num_nodes = ids.len();
            
            // Create an array with empty objects for each node
            let patient_attrs = vec![json!({}); num_nodes];
            
            // Now add it to the network data
            network_data["Nodes"].as_object_mut().unwrap()
                .insert("patient_attributes".to_string(), json!(patient_attrs));
        }
    } else {
        return Err(AnnotationError::MissingField("Nodes must be an object with id array".to_string()));
    }
    
    // Create a map of attribute records keyed by the constructed key
    let mut attribute_map: HashMap<String, HashMap<String, Value>> = HashMap::new();
    for attrs in attributes.iter() {
        if let Ok(key) = construct_key_from_record(attrs, &key_fields, &key_delimiter) {
            attribute_map.insert(key, attrs.clone());
        }
    }
    
    // We don't need to pre-calculate the number of nodes anymore
    
    // No need to prepare patient_attributes fields for array of objects format
    // We'll create/update attributes directly when applying them
    
    // Apply attributes to nodes
    for (node_key, node_idx) in node_key_map.iter() {
        if let Some(attributes) = attribute_map.get(node_key) {
            // Get the node ID
            let node_id = {
                let nodes_obj = network_data["Nodes"].as_object().unwrap();
                let ids = nodes_obj["id"].as_array().unwrap();
                ids[*node_idx].as_str().unwrap().to_string()
            };
            
            // Apply each attribute to the node
            for (field_name, field_value) in attributes.iter() {
                if schema.contains_key(field_name) && field_name != "keying" {
                    let nodes_obj = network_data["Nodes"].as_object_mut().unwrap();
                    
                    // Get the patient_attributes array
                    let patient_attrs_array = nodes_obj["patient_attributes"].as_array_mut().unwrap();
                    
                    // Add the attribute to the node's patient_attributes object
                    // Ensure that null values are converted to empty strings
                    let processed_value = if field_value.is_null() {
                        json!("")
                    } else {
                        field_value.clone()
                    };
                    
                    patient_attrs_array[*node_idx][field_name] = processed_value;
                    
                    // Remove node from uninjected set for this field
                    if let Some(field_set) = uninjected_fields.get_mut(field_name) {
                        field_set.remove(&node_id);
                    }
                }
            }
        }
    }
    
    // Process uninjected fields - ensure any remaining null values are replaced with empty strings
    if let Some(nodes_obj) = network_data.get_mut("Nodes").and_then(|n| n.as_object_mut()) {
        if let Some(patient_attrs_array) = nodes_obj.get_mut("patient_attributes").and_then(|p| p.as_array_mut()) {
            for attr_obj in patient_attrs_array.iter_mut() {
                if let Some(obj) = attr_obj.as_object_mut() {
                    // Ensure all schema fields exist in each patient_attributes object
                    for (field_name, _) in schema.iter() {
                        if field_name != "keying" {
                            // If field doesn't exist or is null, set it to empty string
                            if !obj.contains_key(field_name) || obj[field_name].is_null() {
                                obj.insert(field_name.clone(), json!(""));
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Convert to JSON string
    let result = serde_json::to_string_pretty(&network)?;
    Ok(result)
}

/// Parse attributes from JSON string, handling both array and object formats
fn parse_attributes(json_str: &str) -> Result<Vec<HashMap<String, Value>>, AnnotationError> {
    // Try parsing as an array first
    let result: Result<Vec<HashMap<String, Value>>, _> = serde_json::from_str(json_str);
    if let Ok(array) = result {
        return Ok(array);
    }
    
    // If that fails, try parsing as a single object
    let obj: Result<HashMap<String, Value>, _> = serde_json::from_str(json_str);
    if let Ok(map) = obj {
        return Ok(vec![map]);
    }
    
    // If both fail, return an error
    Err(AnnotationError::InvalidFormat("Attributes JSON must be an array or object".to_string()))
}

/// Extract key fields and delimiter from schema, or use defaults
fn extract_key_info(schema: &HashMap<String, Value>) -> (Vec<String>, String) {
    let mut key_fields = DEFAULT_KEY_FIELDS.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let mut key_delimiter = DEFAULT_KEY_DELIMITER.to_string();
    
    if let Some(keying) = schema.get("keying") {
        if let Some(fields) = keying.get("fields") {
            if let Some(fields_array) = fields.as_array() {
                key_fields = fields_array.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }
        }
        
        if let Some(delimiter) = keying.get("delimiter") {
            if let Some(delim_str) = delimiter.as_str() {
                key_delimiter = delim_str.to_string();
            }
        }
    }
    
    (key_fields, key_delimiter)
}

/// Ensure a key exists in a JSON object
fn ensure_key<'a>(obj: &'a mut Value, key: &str) -> &'a mut Value {
    if !obj.as_object().unwrap().contains_key(key) {
        obj[key] = json!({});
    }
    
    obj.get_mut(key).unwrap()
}

/// Create the attribute schema in the network data
fn create_attribute_schema(network_data: &mut Value, schema: &HashMap<String, Value>) -> Result<(), AnnotationError> {
    for (field_name, field_info) in schema.iter() {
        // Skip the "keying" field as it's not part of the actual schema
        if field_name == "keying" {
            continue;
        }
        
        let schema_entry = network_data["patient_attribute_schema"].as_object_mut().unwrap();
        
        // Get field type and label
        let field_type = field_info.get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("String");
            
        let field_label = field_info.get("label")
            .and_then(|l| l.as_str())
            .unwrap_or(field_name);
        
        // Add schema entry
        schema_entry.insert(
            field_name.clone(),
            json!({
                "name": field_name,
                "type": field_type,
                "label": field_label
            })
        );
        
        // Handle enum type
        if field_type == "enum" {
            if let Some(enum_values) = field_info.get("enum") {
                if let Some(enum_array) = enum_values.as_array() {
                    schema_entry.get_mut(field_name).unwrap()
                        .as_object_mut().unwrap()
                        .insert("enum".to_string(), json!(enum_array));
                }
            }
        }
    }
    
    Ok(())
}

/// Construct a key from a node ID and key fields
fn construct_node_key(node_id: &str, key_fields: &[String], delimiter: &str) -> Result<String, AnnotationError> {
    // If we need to extract parts from the node ID
    if key_fields.len() > 1 {
        let parts: Vec<&str> = node_id.split(delimiter).collect();
        if parts.len() < key_fields.len() {
            return Err(AnnotationError::KeyConstructionError(
                format!("Node ID '{}' doesn't contain enough parts for key fields", node_id)
            ));
        }
        Ok(parts[0..key_fields.len()].join(delimiter))
    } else {
        // Simple case - just return the node ID
        Ok(node_id.to_string())
    }
}

/// Construct a key from an attribute record
fn construct_key_from_record(
    record: &HashMap<String, Value>,
    key_fields: &[String],
    delimiter: &str,
) -> Result<String, AnnotationError> {
    let mut key_parts = Vec::new();
    
    for field in key_fields {
        match record.get(field) {
            Some(value) => {
                let value_str = match value {
                    Value::String(s) => s.clone(),
                    _ => value.to_string(),
                };
                key_parts.push(value_str);
            },
            None => return Err(AnnotationError::KeyConstructionError(
                format!("Missing field '{}' in record", field)
            )),
        }
    }
    
    Ok(key_parts.join(delimiter))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_construct_key_from_record() {
        let mut record = HashMap::new();
        record.insert("ehars_uid".to_string(), json!("KU190031"));
        record.insert("other_field".to_string(), json!("value"));
        
        let key_fields = vec!["ehars_uid".to_string()];
        let delimiter = "~";
        
        let result = construct_key_from_record(&record, &key_fields, delimiter).unwrap();
        assert_eq!(result, "KU190031");
    }
    
    #[test]
    fn test_construct_node_key() {
        let node_id = "KU190031";
        let key_fields = vec!["ehars_uid".to_string()];
        let delimiter = "~";
        
        let result = construct_node_key(node_id, &key_fields, delimiter).unwrap();
        assert_eq!(result, "KU190031");
    }
    
    #[test]
    fn test_ensure_key() {
        let mut obj = json!({});
        ensure_key(&mut obj, "test_key");
        
        assert!(obj.as_object().unwrap().contains_key("test_key"));
        assert_eq!(obj["test_key"], json!({}));
    }
    
    #[test]
    fn test_extract_key_info() {
        let mut schema = HashMap::new();
        schema.insert("keying".to_string(), json!({
            "fields": ["field1", "field2"],
            "delimiter": "|"
        }));
        
        let (key_fields, delimiter) = extract_key_info(&schema);
        
        assert_eq!(key_fields, vec!["field1", "field2"]);
        assert_eq!(delimiter, "|");
    }
    
    #[test]
    fn test_extract_key_info_defaults() {
        let schema = HashMap::new();
        let (key_fields, delimiter) = extract_key_info(&schema);
        
        assert_eq!(key_fields, vec!["ehars_uid"]);
        assert_eq!(delimiter, "~");
    }
}