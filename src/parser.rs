use crate::types::{InputFormat, NetworkError};
use chrono::{DateTime, NaiveDate, Utc};
use regex::Regex;

/// A parsed patient description with ID and metadata
#[derive(Debug, Clone)]
pub struct ParsedPatient {
    pub id: String,
    pub date: Option<DateTime<Utc>>,
    pub raw_id: String,
    pub metadata: Option<String>,
}

/// Parse a patient ID string based on the specified format
pub fn parse_patient_id(id_str: &str, format: InputFormat, regex_pattern: Option<&Regex>) -> Result<ParsedPatient, NetworkError> {
    match format {
        InputFormat::AEH => parse_aeh(id_str),
        InputFormat::LANL => parse_lanl(id_str),
        InputFormat::Plain => parse_plain(id_str),
        InputFormat::Regex => parse_regex(id_str, regex_pattern),
    }
}

/// Parse an AEH format ID (ID | sample_date | otherfields)
fn parse_aeh(id_str: &str) -> Result<ParsedPatient, NetworkError> {
    let bits: Vec<&str> = id_str.trim().split('|').collect();
    
    if bits.len() < 2 {
        return Err(NetworkError::Format(format!(
            "Improperly formatted AEH header (need at least \"ID|Sample date in mmddyyyy format\"): {}", 
            id_str
        )));
    }
    
    // Parse date in format mmddyyyy
    let date = parse_date(bits[1], "%m%d%Y")?;
    
    let patient = ParsedPatient {
        id: bits[0].to_string(),
        date: Some(date),
        raw_id: id_str.to_string(),
        metadata: if bits.len() > 2 {
            Some(bits[2..].join("|"))
        } else {
            None
        },
    };
    
    Ok(patient)
}

/// Parse a LANL format ID (subtype_country_id_year)
fn parse_lanl(id_str: &str) -> Result<ParsedPatient, NetworkError> {
    let bits: Vec<&str> = id_str.trim().split('_').collect();
    
    if bits.len() < 4 {
        return Err(NetworkError::Format(format!(
            "Improperly formatted LANL header (need at least \"subtype_country_accession_yyyy\"): {}", 
            id_str
        )));
    }
    
    // Parse year
    let date = parse_date(bits[3], "%Y")?;
    
    let patient = ParsedPatient {
        id: bits[2].to_string(),
        date: Some(date),
        raw_id: id_str.to_string(),
        metadata: if bits.len() > 4 {
            Some(bits[4..].join("_"))
        } else {
            None
        },
    };
    
    Ok(patient)
}

/// Parse a plain ID (no metadata)
fn parse_plain(id_str: &str) -> Result<ParsedPatient, NetworkError> {
    let patient = ParsedPatient {
        id: id_str.trim().to_string(),
        date: None,
        raw_id: id_str.to_string(),
        metadata: None,
    };
    
    Ok(patient)
}

/// Parse an ID using a regular expression
fn parse_regex(id_str: &str, pattern: Option<&Regex>) -> Result<ParsedPatient, NetworkError> {
    let regex = pattern.ok_or_else(|| {
        NetworkError::Format("Regex pattern is required for Regex format".to_string())
    })?;
    
    let captures = regex.captures(id_str.trim()).ok_or_else(|| {
        NetworkError::Format(format!("ID doesn't match provided regex pattern: {}", id_str))
    })?;
    
    // First capture group is the ID
    if captures.len() < 2 {
        return Err(NetworkError::Format(
            "Regex must have at least one capture group for the ID".to_string()
        ));
    }
    
    let id = captures.get(1).unwrap().as_str().to_string();
    
    // Optional date in second capture group
    let mut date = None;
    if captures.len() > 2 && captures.get(2).is_some() {
        let date_str = captures.get(2).unwrap().as_str();
        
        // Try common date formats
        for format in &["%m%d%Y", "%m/%d/%y", "%Y%m%d", "%m_%d_%y", "%m-%d-%y", "%Y"] {
            if let Ok(parsed_date) = parse_date(date_str, format) {
                date = Some(parsed_date);
                break;
            }
        }
    }
    
    let patient = ParsedPatient {
        id,
        date,
        raw_id: id_str.to_string(),
        metadata: if captures.len() > 3 { 
            captures.get(3).map(|m| m.as_str().to_string())
        } else {
            None
        },
    };
    
    Ok(patient)
}

/// Parse a date string using the given format
fn parse_date(date_str: &str, format: &str) -> Result<DateTime<Utc>, NetworkError> {
    let naive_date = NaiveDate::parse_from_str(date_str, format)
        .map_err(|_| NetworkError::Format(format!("Failed to parse date: {}", date_str)))?;
    
    // Create a datetime at midnight
    let naive_datetime = naive_date.and_hms_opt(0, 0, 0).unwrap();
    
    // Convert to UTC
    Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc))
}