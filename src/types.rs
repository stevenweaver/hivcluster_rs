use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::cmp::Ordering;
use thiserror::Error;

/// Error types for network operations
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("CSV parsing error: {0}")]
    Csv(#[from] csv::Error),
    
    #[error("Invalid data format: {0}")]
    Format(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Cannot create self-loop (node connecting to itself)")]
    SelfLoop,
    
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Available input formats for parsing node IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputFormat {
    /// ID | sample_date | otherfields
    AEH,
    /// subtype_country_id_year
    LANL,
    /// Plain ID with no metadata
    Plain,
    /// Custom regex format
    Regex,
}

/// A node in the network representing a patient
#[derive(Debug, Clone, PartialEq)]
pub struct Patient {
    pub id: String,
    pub dates: Vec<Option<DateTime<Utc>>>,
    pub edi: Option<DateTime<Utc>>, // estimated date of infection
    pub stage: String, // disease stage
    pub treatment_date: Option<DateTime<Utc>>,
    pub viral_load: Option<f64>,
    pub degree: usize,
    pub cluster_id: Option<usize>,
    pub treatment_naive: Option<bool>,
    pub attributes: HashSet<String>,
    pub named_attributes: HashMap<String, String>,
}

impl Patient {
    /// Create a new patient with the given ID
    pub fn new(id: &str) -> Self {
        Patient {
            id: id.to_string(),
            dates: Vec::new(),
            edi: None,
            stage: "Unknown".to_string(),
            treatment_date: None,
            viral_load: None,
            degree: 0,
            cluster_id: None,
            treatment_naive: None,
            attributes: HashSet::new(),
            named_attributes: HashMap::new(),
        }
    }

    /// Add a date to this patient's collection dates
    pub fn add_date(&mut self, date: Option<DateTime<Utc>>) {
        if !self.dates.contains(&date) {
            self.dates.push(date);
        }
    }

    /// Add an attribute to this patient
    pub fn add_attribute(&mut self, attr: &str) {
        self.attributes.insert(attr.to_string());
    }
    
    /// Check if patient has a specific attribute
    pub fn has_attribute(&self, attr: &str) -> bool {
        self.attributes.contains(attr)
    }
    
    /// Remove an attribute from this patient
    pub fn remove_attribute(&mut self, attr: &str) {
        self.attributes.remove(attr);
    }
    
    /// Add a named attribute with a value
    pub fn add_named_attribute(&mut self, key: &str, value: Option<String>) {
        if let Some(val) = value {
            if !val.is_empty() {
                self.named_attributes.insert(key.to_string(), val);
            }
        } else if self.named_attributes.contains_key(key) {
            self.named_attributes.remove(key);
        }
    }
    
    /// Increment the degree (number of connections) for this patient
    pub fn increment_degree(&mut self) {
        self.degree += 1;
    }
    
    /// Get the most recent date if available
    pub fn get_most_recent_date(&self) -> Option<DateTime<Utc>> {
        self.dates.iter()
            .filter_map(|&date| date)
            .max()
    }
}

impl Hash for Patient {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialOrd for Patient {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

/// A connection between two patients in the network
#[derive(Debug, Clone)]
pub struct Edge {
    pub source_id: String,
    pub target_id: String,
    pub source_date: Option<DateTime<Utc>>,
    pub target_date: Option<DateTime<Utc>>,
    pub visible: bool,
    pub attributes: HashSet<String>,
    pub sequences: Option<Vec<String>>,
    pub distance: f64,
    pub is_unsupported: bool,
}

impl Edge {
    /// Create a new edge between two patients
    pub fn new(
        source_id: String, 
        target_id: String,
        source_date: Option<DateTime<Utc>>,
        target_date: Option<DateTime<Utc>>,
        distance: f64,
    ) -> Result<Self, NetworkError> {
        // Ensure no self-loops
        if source_id == target_id {
            return Err(NetworkError::SelfLoop);
        }
        
        // Always normalize source_id and target_id to ensure source_id < target_id
        // This maintains consistent edge representation
        let (source_id, target_id, source_date, target_date) = if source_id < target_id {
            (source_id, target_id, source_date, target_date)
        } else {
            (target_id, source_id, target_date, source_date)
        };
        
        Ok(Edge {
            source_id,
            target_id,
            source_date,
            target_date,
            visible: true,
            attributes: HashSet::new(),
            sequences: None,
            distance,
            is_unsupported: false,
        })
    }
    
    /// Add an attribute to this edge
    pub fn add_attribute(&mut self, attr: &str) {
        self.attributes.insert(attr.to_string());
    }
    
    /// Check if edge has a specific attribute
    pub fn has_attribute(&self, attr: &str) -> bool {
        self.attributes.contains(attr)
    }
    
    /// Remove an attribute from this edge
    pub fn remove_attribute(&mut self, attr: &str) {
        self.attributes.remove(attr);
    }
    
    /// Update sequence information for this edge
    pub fn update_sequence_info(&mut self, seq_info: Vec<String>) {
        self.sequences = Some(seq_info);
    }
    
    /// Check if the edge dates meet a specific date condition
    pub fn check_date(&self, date: &DateTime<Utc>, newer: bool) -> bool {
        let source_date_ok = match self.source_date {
            Some(d) => if newer { d >= *date } else { d <= *date },
            None => true, // If date is missing, consider it passing the condition
        };
        
        let target_date_ok = match self.target_date {
            Some(d) => if newer { d >= *date } else { d <= *date },
            None => true, // If date is missing, consider it passing the condition
        };
        
        source_date_ok && target_date_ok
    }
    
    /// Get the edge key (source_id, target_id) for consistent lookup
    pub fn get_key(&self) -> (String, String) {
        (self.source_id.clone(), self.target_id.clone())
    }
}

/// A parsed patient ID with metadata
#[derive(Debug, Clone)]
pub struct ParsedPatient {
    pub id: String,
    pub date: Option<DateTime<Utc>>,
    pub attributes: HashMap<String, String>,
}

impl ParsedPatient {
    /// Create a new parsed patient
    pub fn new(id: String, date: Option<DateTime<Utc>>) -> Self {
        ParsedPatient {
            id,
            date,
            attributes: HashMap::new(),
        }
    }

    /// Add an attribute to this parsed patient
    pub fn add_attribute(&mut self, key: &str, value: String) {
        if !value.is_empty() {
            self.attributes.insert(key.to_string(), value);
        }
    }
}