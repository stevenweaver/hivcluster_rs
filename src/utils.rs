use chrono::{DateTime, Utc};
use std::cmp::Ordering;

/// Calculate time difference between two dates in days
pub fn date_difference_days(date1: &DateTime<Utc>, date2: &DateTime<Utc>) -> i64 {
    let diff = date2.signed_duration_since(*date1);
    diff.num_days()
}

/// Compare two optional dates
pub fn compare_dates(date1: &Option<DateTime<Utc>>, date2: &Option<DateTime<Utc>>) -> Ordering {
    match (date1, date2) {
        (Some(d1), Some(d2)) => d1.cmp(d2),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/// Set up logging for WASM
#[cfg(target_arch = "wasm32")]
pub fn setup_logging() {
    use wasm_bindgen::prelude::*;

    // Set panic hook to log panics to console
    console_error_panic_hook::set_once();
}

#[cfg(target_arch = "wasm32")]
pub fn log_to_console(message: &str) {
    use web_sys::console;
    console::log_1(&wasm_bindgen::JsValue::from_str(message));
}

/// Format a float value with the specified number of decimal places
pub fn format_float(value: f64, decimals: usize) -> String {
    format!("{:.*}", decimals, value)
}
