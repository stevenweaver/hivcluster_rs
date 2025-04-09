use wasm_bindgen::prelude::*;
mod network;
mod parser;
mod types;
mod utils;

pub use network::*;
pub use parser::*;
pub use types::*;
pub use utils::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Initialize panic hook for better error reporting in WASM
#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
}

/// Main entry point for WASM bindings to build a network from CSV data
#[wasm_bindgen]
pub fn build_network(csv_data: &str, distance_threshold: f64) -> JsValue {
    // Create network
    let mut network = TransmissionNetwork::new();
    
    // Process the network
    if let Err(e) = network.read_from_csv_str(csv_data, distance_threshold, InputFormat::Plain) {
        log(&format!("Error building network: {}", e));
        return JsValue::NULL;
    }
    
    // Compute clusters
    network.compute_clusters();
    
    // Convert to JSON and return
    let json = serde_json::to_string(&network.to_json()).unwrap_or_else(|_| "{}".to_string());
    JsValue::from_str(&json)
}

/// Build network from CSV data
pub fn build_network_internal(csv_data: &str, distance_threshold: f64) -> Result<TransmissionNetwork, NetworkError> {
    // Create network
    let mut network = TransmissionNetwork::new();
    
    // Process the network
    network.read_from_csv_str(csv_data, distance_threshold, InputFormat::Plain)?;
    
    // Compute clusters
    network.compute_clusters();
    
    Ok(network)
}