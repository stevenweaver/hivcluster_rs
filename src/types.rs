use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
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
}

/// Available input formats for parsing node IDs
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone)]
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
    pub named_attributes: std::collections::HashMap<String, String>,
}

impl Patient {
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
            named_attributes: std::collections::HashMap::new(),
        }
    }

    pub fn add_date(&mut self, date: Option<DateTime<Utc>>) {
        if !self.dates.contains(&date) {
            self.dates.push(date);
        }
    }

    pub fn add_attribute(&mut self, attr: &str) {
        self.attributes.insert(attr.to_string());
    }
    
    pub fn has_attribute(&self, attr: &str) -> bool {
        self.attributes.contains(attr)
    }
    
    pub fn remove_attribute(&mut self, attr: &str) {
        self.attributes.remove(attr);
    }
    
    pub fn add_named_attribute(&mut self, key: &str, value: Option<String>) {
        if let Some(val) = value {
            self.named_attributes.insert(key.to_string(), val);
        } else if self.named_attributes.contains_key(key) {
            self.named_attributes.remove(key);
        }
    }
    
    pub fn increment_degree(&mut self) {
        self.degree += 1;
    }
}

impl Hash for Patient {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Patient {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Patient {}

impl PartialOrd for Patient {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Patient {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

/// A connection between two patients in the network
#[derive(Debug, Clone)]
pub struct Edge {
    pub source: Rc<Patient>,
    pub target: Rc<Patient>,
    pub source_date: Option<DateTime<Utc>>,
    pub target_date: Option<DateTime<Utc>>,
    pub visible: bool,
    pub attributes: HashSet<String>,
    pub sequences: Option<Vec<String>>,
    pub distance: f64,
    pub is_unsupported: bool,
}

impl Edge {
    pub fn new(
        source: Rc<Patient>, 
        target: Rc<Patient>,
        source_date: Option<DateTime<Utc>>,
        target_date: Option<DateTime<Utc>>,
        visible: bool,
        distance: f64,
    ) -> Result<Self, NetworkError> {
        // Ensure no self-loops
        if Rc::ptr_eq(&source, &target) {
            return Err(NetworkError::SelfLoop);
        }
        
        let (source, target, source_date, target_date) = if source.id < target.id {
            (source, target, source_date, target_date)
        } else {
            (target, source, target_date, source_date)
        };
        
        Ok(Edge {
            source,
            target,
            source_date,
            target_date,
            visible,
            attributes: HashSet::new(),
            sequences: None,
            distance,
            is_unsupported: false,
        })
    }
    
    pub fn add_attribute(&mut self, attr: &str) {
        self.attributes.insert(attr.to_string());
    }
    
    pub fn has_attribute(&self, attr: &str) -> bool {
        self.attributes.contains(attr)
    }
    
    pub fn remove_attribute(&mut self, attr: &str) {
        self.attributes.remove(attr);
    }
    
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
}

impl Hash for Edge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Use pointer equality for the patient Rcs
        std::ptr::addr_of!(*self.source).hash(state);
        std::ptr::addr_of!(*self.target).hash(state);
        
        // Only include dates in hash if comparing edge versions with the same nodes
        if let Some(date) = &self.source_date {
            date.hash(state);
        }
        if let Some(date) = &self.target_date {
            date.hash(state);
        }
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        // Check if nodes are the same (using pointer equality)
        let nodes_equal = Rc::ptr_eq(&self.source, &other.source) && 
                        Rc::ptr_eq(&self.target, &other.target);
        
        // If nodes are the same, check dates
        if nodes_equal {
            self.source_date == other.source_date && self.target_date == other.target_date
        } else {
            false
        }
    }
}

impl Eq for Edge {}