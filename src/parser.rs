use crate::types::{InputFormat, NetworkError, ParsedPatient};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use regex::Regex;

/// Parse a patient ID based on the specified format
pub fn parse_patient_id(
    id: &str,
    format: InputFormat,
    default_date: Option<DateTime<Utc>>,
) -> Result<ParsedPatient, NetworkError> {
    match format {
        InputFormat::Plain => parse_plain_id(id, default_date),
        InputFormat::AEH => parse_aeh_id(id),
        InputFormat::LANL => parse_lanl_id(id),
        InputFormat::Regex => parse_regex_id(id, default_date),
    }
}

/// Parse a plain ID without any metadata
fn parse_plain_id(
    id: &str,
    default_date: Option<DateTime<Utc>>,
) -> Result<ParsedPatient, NetworkError> {
    if id.trim().is_empty() {
        return Err(NetworkError::Format("Empty ID is not allowed".to_string()));
    }

    let patient = ParsedPatient::new(id.trim().to_string(), default_date);
    Ok(patient)
}

/// Parse an AEH format ID (ID | date | other fields)
fn parse_aeh_id(id: &str) -> Result<ParsedPatient, NetworkError> {
    let parts: Vec<&str> = id.split('|').collect();

    if parts.is_empty() || parts[0].trim().is_empty() {
        return Err(NetworkError::Format(format!(
            "Invalid AEH format for ID: {}",
            id
        )));
    }

    let patient_id = parts[0].trim().to_string();

    // Extract date if available (field index 1)
    let date = if parts.len() > 1 && !parts[1].trim().is_empty() {
        match parse_date(parts[1].trim()) {
            Ok(date) => Some(date),
            Err(_) => None,
        }
    } else {
        None
    };

    // Create patient
    let mut patient = ParsedPatient::new(patient_id, date);

    // Extract additional attributes (field index 2+)
    for (i, field) in parts.iter().enumerate().skip(2) {
        if !field.trim().is_empty() {
            patient.add_attribute(&format!("field_{}", i), field.trim().to_string());
        }
    }

    Ok(patient)
}

/// Parse a LANL format ID (subtype_country_id_year)
fn parse_lanl_id(id: &str) -> Result<ParsedPatient, NetworkError> {
    let parts: Vec<&str> = id.split('_').collect();

    if parts.len() < 3 {
        return Err(NetworkError::Format(format!(
            "Invalid LANL format for ID: {}. Expected at least 3 parts separated by '_'",
            id
        )));
    }

    // Extract patient ID (field index 2)
    let patient_id = if parts.len() > 2 && !parts[2].trim().is_empty() {
        parts[2].trim().to_string()
    } else {
        id.to_string() // Fall back to the entire string if parts aren't available
    };

    // Extract year (field index 3) and convert to date
    let date = if parts.len() > 3 && !parts[3].trim().is_empty() {
        match parts[3].trim().parse::<i32>() {
            Ok(year) => {
                if (1900..=2100).contains(&year) {
                    // Set date to January 1st of the year
                    match Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0) {
                        chrono::LocalResult::Single(date) => Some(date),
                        _ => None,
                    }
                } else {
                    None // Year outside reasonable range
                }
            }
            Err(_) => None,
        }
    } else {
        None
    };

    // Create patient
    let mut patient = ParsedPatient::new(patient_id, date);

    // Add subtype attribute if available (field index 0)
    if !parts[0].trim().is_empty() {
        patient.add_attribute("subtype", parts[0].trim().to_string());
    }

    // Add country attribute if available (field index 1)
    if parts.len() > 1 && !parts[1].trim().is_empty() {
        patient.add_attribute("country", parts[1].trim().to_string());
    }

    Ok(patient)
}

/// Parse ID with a custom regex pattern
fn parse_regex_id(
    id: &str,
    default_date: Option<DateTime<Utc>>,
) -> Result<ParsedPatient, NetworkError> {
    // This is a placeholder implementation - in a real system, you would configure
    // regex patterns and named capture groups

    // Example: Try to extract an ISO date (YYYY-MM-DD) and ID from a string
    let iso_date_pattern = Regex::new(r"(\d{4}-\d{2}-\d{2})").unwrap();

    let mut patient = ParsedPatient::new(id.to_string(), default_date);

    // Extract date if present
    if let Some(date_match) = iso_date_pattern.find(id) {
        if let Ok(date) = parse_date(date_match.as_str()) {
            patient.date = Some(date);

            // Use the part before the date as ID if possible
            let id_part = id.split(date_match.as_str()).next().unwrap_or(id).trim();
            if !id_part.is_empty() {
                patient = ParsedPatient::new(id_part.to_string(), Some(date));
            }
        }
    }

    Ok(patient)
}

/// Parse a date string into a DateTime<Utc>
pub fn parse_date(date_str: &str) -> Result<DateTime<Utc>, NetworkError> {
    // Try common date formats
    let formats = [
        "%Y-%m-%d",          // 2020-12-31
        "%d-%m-%Y",          // 31-12-2020
        "%d/%m/%Y",          // 31/12/2020
        "%Y/%m/%d",          // 2020/12/31
        "%Y-%m-%d %H:%M:%S", // 2020-12-31 12:34:56
        "%d-%b-%Y",          // 31-Dec-2020
        "%d %b %Y",          // 31 Dec 2020
        "%b %d, %Y",         // Dec 31, 2020
        "%B %d, %Y",         // December 31, 2020
    ];

    // First try formats with time
    for format in formats.iter().filter(|f| f.contains("%H:%M:%S")) {
        if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, format) {
            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc));
        }
    }

    // Then try formats without time (append 00:00:00)
    for format in formats.iter().filter(|f| !f.contains("%H:%M:%S")) {
        if let Ok(dt) = NaiveDateTime::parse_from_str(
            &format!("{} 00:00:00", date_str),
            &format!("{} %H:%M:%S", format),
        ) {
            return Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc));
        }
    }

    // Special case for year-only
    if let Ok(year) = date_str.parse::<i32>() {
        if (1900..=2100).contains(&year) {
            if let chrono::LocalResult::Single(date) = Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0) {
                return Ok(date);
            }
        }
    }

    Err(NetworkError::Format(format!(
        "Unable to parse date: {}",
        date_str
    )))
}
