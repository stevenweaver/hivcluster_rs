mod network;
mod parser;
mod types;
mod utils;

// Re-export main types and functions
pub use network::TransmissionNetwork;
pub use types::{InputFormat, NetworkError, Edge, Patient, ParsedPatient};

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;
    use super::*;
    
    // Initialize logging for WASM
    #[wasm_bindgen(start)]
    pub fn init() {
        utils::setup_logging();
    }
    
    /// WASM bindings for the network builder
    #[wasm_bindgen]
    pub fn build_network(
        csv_data: &str,
        threshold: f64,
        format: &str,
    ) -> Result<String, JsValue> {
        let input_format = match format.to_lowercase().as_str() {
            "aeh" => InputFormat::AEH,
            "lanl" => InputFormat::LANL,
            "regex" => InputFormat::Regex,
            _ => InputFormat::Plain,
        };
        
        // Build the network
        let result = build_network_internal(csv_data, threshold, input_format)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
            
        Ok(result)
    }
    
    /// Get network statistics in JSON format
    #[wasm_bindgen]
    pub fn get_network_stats(
        csv_data: &str,
        threshold: f64,
        format: &str,
    ) -> Result<String, JsValue> {
        let input_format = match format.to_lowercase().as_str() {
            "aeh" => InputFormat::AEH,
            "lanl" => InputFormat::LANL,
            "regex" => InputFormat::Regex,
            _ => InputFormat::Plain,
        };
        
        // Create a new network
        let mut network = TransmissionNetwork::new();
        
        // Parse CSV and build the network
        network.read_from_csv_str(csv_data, threshold, input_format)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        // Compute the network structure
        network.compute_adjacency();
        network.compute_clusters();
        
        // Get stats as JSON
        let stats = network.get_network_stats();
        let json = serde_json::to_string(&stats)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
            
        Ok(json)
    }
}

/// Build network and return JSON representation
pub fn build_network_internal(
    csv_data: &str,
    threshold: f64,
    format: InputFormat,
) -> Result<String, NetworkError> {
    // Create a new network
    let mut network = TransmissionNetwork::new();
    
    // Parse CSV and build the network
    network.read_from_csv_str(csv_data, threshold, format)?;
    
    // Compute the network structure
    network.compute_adjacency();
    network.compute_clusters();
    
    // Convert to JSON string
    network.to_json_string()
}