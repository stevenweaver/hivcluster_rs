use crate::types::NetworkError;
use std::collections::HashMap;

/// Describe a numeric vector with statistical measures
pub fn describe_vector(mut vector: Vec<f64>) -> HashMap<String, f64> {
    if vector.is_empty() {
        return HashMap::new();
    }
    
    vector.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = vector.len();
    
    let mut result = HashMap::new();
    result.insert("count".to_string(), n as f64);
    result.insert("min".to_string(), vector[0]);
    result.insert("max".to_string(), vector[n - 1]);
    result.insert("mean".to_string(), vector.iter().sum::<f64>() / n as f64);
    
    // Median
    let median = if n % 2 == 1 {
        vector[n / 2]
    } else {
        (vector[n / 2 - 1] + vector[n / 2]) / 2.0
    };
    result.insert("median".to_string(), median);
    
    // Interquartile range
    result.insert("q1".to_string(), vector[n / 4]);
    result.insert("q3".to_string(), vector[3 * n / 4]);
    
    result
}

/// Get the date difference in days between two dates
pub fn date_diff_days(date1: &chrono::DateTime<chrono::Utc>, date2: &chrono::DateTime<chrono::Utc>) -> i64 {
    (*date2 - *date1).num_days()
}

/// Convert a CSV string into a vector of vectors
pub fn parse_csv(csv_str: &str) -> Result<Vec<Vec<String>>, NetworkError> {
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(csv_str.as_bytes());
    
    let mut result = Vec::new();
    for record in rdr.records() {
        let record = record.map_err(NetworkError::Csv)?;
        result.push(record.iter().map(|s| s.to_string()).collect());
    }
    
    Ok(result)
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    // Log to browser console
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
    
    #[wasm_bindgen(js_namespace = console)]
    pub fn error(s: &str);
}