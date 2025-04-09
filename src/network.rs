use crate::parser::parse_patient_id;
use crate::types::{Edge, InputFormat, NetworkError, Patient, ParsedPatient};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use chrono::Utc;

/// The main network structure
#[derive(Debug)]
pub struct TransmissionNetwork {
    /// All patients/nodes in the network
    pub nodes: HashMap<String, Patient>,
    
    /// All edges in the network
    pub edges: Vec<Edge>,
    
    /// Adjacency list representation (node ID -> neighboring node IDs)
    pub adjacency: HashMap<String, Vec<String>>,
    
    /// Edge lookup by (source, target) pair
    pub edge_lookup: HashMap<(String, String), usize>,
    
    /// Network metadata for output
    pub metadata: HashMap<String, serde_json::Value>,
}

/// A simple cluster representation for output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cluster {
    pub id: usize,
    pub nodes: Vec<String>,
    pub size: usize,
}

/// Output JSON format compatible with legacy HIVCluster output
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkJSON {
    #[serde(rename = "trace_results")]
    pub trace_results: TraceResults,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraceResults {
    #[serde(rename = "Network Summary")]
    pub network_summary: NetworkSummary,
    #[serde(rename = "Multiple sequences")]
    pub multiple_sequences: MultipleSequences,
    #[serde(rename = "Cluster sizes")]
    pub cluster_sizes: Vec<usize>,
    #[serde(rename = "HIV Stages")]
    pub hiv_stages: HashMap<String, usize>,
    #[serde(rename = "Directed Edges")]
    pub directed_edges: DirectedEdges,
    #[serde(rename = "Degrees")]
    pub degrees: Degrees,
    #[serde(rename = "Settings")]
    pub settings: Settings,
    #[serde(rename = "Nodes")]
    pub nodes: NodesOutput,
    #[serde(rename = "Edges")]
    pub edges: EdgesOutput,
    #[serde(rename = "patient_attribute_schema")]
    pub patient_attribute_schema: HashMap<String, AttributeSchema>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkSummary {
    pub Edges: usize,
    pub Nodes: usize,
    #[serde(rename = "Sequences used to make links")]
    pub sequences_used: usize,
    pub Clusters: usize,
    pub Singletons: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MultipleSequences {
    #[serde(rename = "Subjects with")]
    pub subjects_with: usize,
    #[serde(rename = "Followup, days")]
    pub followup_days: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectedEdges {
    pub Count: usize,
    #[serde(rename = "Reasons for unresolved directions")]
    pub reasons: HashMap<String, usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Degrees {
    pub Distribution: Vec<usize>,
    pub Model: String,
    pub rho: f64,
    #[serde(rename = "rho CI")]
    pub rho_ci: Vec<f64>,
    pub BIC: f64,
    pub fitted: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub threshold: f64,
    #[serde(rename = "edge-filtering")]
    pub edge_filtering: Option<String>,
    pub contaminants: Option<serde_json::Value>,
    pub singletons: bool,
    pub compact_json: bool,
    pub created: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodesOutput {
    pub cluster: Vec<usize>,
    pub id: Vec<String>,
    pub patient_attributes: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EdgesOutput {
    pub directed: DirectedValues,
    pub sequences: Vec<Vec<String>>,
    pub target: Vec<usize>,
    pub length: Vec<f64>,
    pub attributes: AttributeValues,
    pub removed: DirectedValues,
    pub support: SupportValues,
    pub source: Vec<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectedValues {
    pub keys: HashMap<String, bool>,
    pub values: Vec<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttributeValues {
    pub keys: HashMap<String, Vec<String>>,
    pub values: Vec<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SupportValues {
    pub keys: HashMap<String, f64>,
    pub values: Vec<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttributeSchema {
    pub name: String,
    #[serde(rename = "type")]
    pub attr_type: String,
    pub label: String,
}

impl TransmissionNetwork {
    /// Create a new empty network
    pub fn new() -> Self {
        TransmissionNetwork {
            nodes: HashMap::new(),
            edges: Vec::new(),
            adjacency: HashMap::new(),
            edge_lookup: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Read network data from a CSV string
    pub fn read_from_csv_str(
        &mut self,
        csv_str: &str,
        distance_threshold: f64,
        format: InputFormat,
    ) -> Result<(), NetworkError> {
        // Check for empty input
        if csv_str.trim().is_empty() {
            return Err(NetworkError::Format("Empty CSV input".to_string()));
        }
        
        // Set threshold in metadata for later use
        self.metadata.insert("threshold".to_string(), serde_json::json!(distance_threshold));
        
        // Try to detect if the CSV has headers - this is a heuristic
        let has_headers = csv_str.lines().next()
            .map(|first_line| {
                let columns: Vec<&str> = first_line.split(',').collect();
                columns.len() >= 3 && columns[2].trim() == "distance"
            })
            .unwrap_or(false);
        
        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .has_headers(has_headers) // Auto-detect headers
            .from_reader(csv_str.as_bytes());
        
        // First pass: track all node IDs and collect valid edges
        let mut edges_to_add = Vec::new();
        let mut all_node_ids = HashSet::new();
        
        for result in reader.records() {
            let record = result?;
            
            if record.len() < 3 {
                return Err(NetworkError::Format(
                    "CSV row must have at least 3 columns: node1,node2,distance".to_string()
                ));
            }
            
            // Extract values from record
            let id1 = record.get(0).unwrap_or("").trim();
            let id2 = record.get(1).unwrap_or("").trim();
            
            if id1.is_empty() || id2.is_empty() {
                continue; // Skip rows with empty IDs
            }
            
            // Track all node IDs for singleton detection
            all_node_ids.insert(id1.to_string());
            all_node_ids.insert(id2.to_string());
            
            let distance = match record.get(2).unwrap_or("").trim().parse::<f64>() {
                Ok(d) => d,
                Err(_) => {
                    return Err(NetworkError::Format(
                        format!("Invalid distance value: {}", record.get(2).unwrap_or(""))
                    ));
                }
            };
            
            // Skip edges with distance greater than threshold
            if distance > distance_threshold {
                continue;
            }
            
            // Skip self loops (same ID for both nodes)
            if id1 == id2 {
                return Err(NetworkError::SelfLoop);
            }
            
            // Parse node IDs
            let patient1 = parse_patient_id(id1, format, None)?;
            let patient2 = parse_patient_id(id2, format, None)?;
            
            // Collect this edge for later addition
            edges_to_add.push((patient1, patient2, distance));
        }
        
        // Add all nodes first (including those without edges)
        for id in all_node_ids {
            let parsed_node = parse_patient_id(&id, format, None)?;
            self.add_node(&parsed_node)?;
        }
        
        // Now add all valid edges
        for (patient1, patient2, distance) in edges_to_add {
            self.add_edge(patient1, patient2, distance)?;
        }
        
        self.update_stats();
        
        Ok(())
    }
    
    /// Add a node to the network or update existing node
    fn add_node(&mut self, patient_data: &ParsedPatient) -> Result<(), NetworkError> {
        // Add or update node
        let node = self.nodes.entry(patient_data.id.clone())
            .or_insert_with(|| Patient::new(&patient_data.id));
        
        // Update node data
        node.add_date(patient_data.date);
        
        // Add any attributes
        for (key, value) in &patient_data.attributes {
            node.add_named_attribute(key, Some(value.clone()));
        }
        
        // Initialize adjacency list if needed
        self.adjacency.entry(patient_data.id.clone())
            .or_insert_with(Vec::new);
            
        Ok(())
    }
    
    /// Add an edge between two patients
    fn add_edge(
        &mut self,
        patient1: ParsedPatient,
        patient2: ParsedPatient,
        distance: f64,
    ) -> Result<(), NetworkError> {
        // Ensure nodes exist
        self.add_node(&patient1)?;
        self.add_node(&patient2)?;
        
        // Check for self-loops
        if patient1.id == patient2.id {
            return Err(NetworkError::SelfLoop);
        }
        
        // Create edge
        let edge = Edge::new(
            patient1.id.clone(),
            patient2.id.clone(),
            patient1.date,
            patient2.date,
            distance,
        )?;
        
        // Check if this edge already exists
        let edge_key = edge.get_key();
        if self.edge_lookup.contains_key(&edge_key) {
            // Edge already exists - keep the one with smaller distance
            let existing_edge_idx = self.edge_lookup[&edge_key];
            let existing_edge = &self.edges[existing_edge_idx];
            
            if distance < existing_edge.distance {
                // Replace with new edge that has smaller distance
                self.edges[existing_edge_idx] = edge;
            }
            
            return Ok(());
        }
        
        // Add edge to the adjacency lists using original patient IDs 
        // (not the normalized edge IDs)
        self.adjacency.entry(patient1.id.clone())
            .or_insert_with(Vec::new)
            .push(patient2.id.clone());
            
        self.adjacency.entry(patient2.id.clone())
            .or_insert_with(Vec::new)
            .push(patient1.id.clone());
            
        // Update node degrees using original patient IDs
        if let Some(node) = self.nodes.get_mut(&patient1.id) {
            node.increment_degree();
        }
        
        if let Some(node) = self.nodes.get_mut(&patient2.id) {
            node.increment_degree();
        }
        
        // Store edge
        let edge_idx = self.edges.len();
        self.edge_lookup.insert(edge_key, edge_idx);
        self.edges.push(edge);
        
        Ok(())
    }
    
    /// Update network statistics
    fn update_stats(&mut self) {
        self.metadata.insert("node_count".to_string(), serde_json::json!(self.nodes.len()));
        self.metadata.insert("edge_count".to_string(), serde_json::json!(self.edges.len()));
    }
    
    /// Compute adjacency list (rebuild from edges)
    pub fn compute_adjacency(&mut self) {
        self.adjacency.clear();
        
        // Initialize adjacency list for all nodes
        for node_id in self.nodes.keys() {
            self.adjacency.entry(node_id.clone())
                .or_insert_with(Vec::new);
        }
        
        // Add edges to adjacency lists
        for edge in &self.edges {
            if !edge.visible {
                continue;
            }
            
            // Use both IDs without normalization for proper connectivity
            let id1 = edge.source_id.clone();
            let id2 = edge.target_id.clone();
            
            self.adjacency.entry(id1.clone())
                .or_insert_with(Vec::new)
                .push(id2.clone());
                
            self.adjacency.entry(id2)
                .or_insert_with(Vec::new)
                .push(id1);
        }
    }
    
    /// Identify connected components (clusters) in the network
    pub fn compute_clusters(&mut self) {
        // Reset all cluster assignments
        for node in self.nodes.values_mut() {
            node.cluster_id = None;
        }
        
        let mut cluster_id = 0;
        let mut visited = HashSet::new();
        
        // First, assign clusters to connected nodes
        for node_id in self.nodes.keys().cloned().collect::<Vec<String>>() {
            if visited.contains(&node_id) {
                continue;
            }
            
            // Skip singleton nodes (they'll be processed separately)
            if let Some(node) = self.nodes.get(&node_id) {
                if node.degree == 0 {
                    continue;
                }
            }
            
            // BFS to find all nodes in this cluster
            self.breadth_first_traverse(&node_id, cluster_id, &mut visited);
            cluster_id += 1;
        }
        
        // Now assign singleton nodes to their own clusters
        for node_id in self.nodes.keys().cloned().collect::<Vec<String>>() {
            if visited.contains(&node_id) {
                continue;
            }
            
            // This must be a singleton (no connections)
            if let Some(node) = self.nodes.get_mut(&node_id) {
                if node.degree == 0 {
                    node.cluster_id = Some(cluster_id);
                    visited.insert(node_id.clone());
                    cluster_id += 1;
                }
            }
        }
    }
    
    /// Breadth-first search to identify a cluster
    fn breadth_first_traverse(&mut self, start_id: &str, cluster_id: usize, visited: &mut HashSet<String>) {
        // Assign cluster ID to starting node
        if let Some(node) = self.nodes.get_mut(start_id) {
            node.cluster_id = Some(cluster_id);
        } else {
            return; // Node not found
        }
        
        visited.insert(start_id.to_string());
        
        // Get the degree of this node to check if it's connected
        let node_degree = match self.nodes.get(start_id) {
            Some(node) => node.degree,
            None => return, // Node not found
        };
        
        // If the node has no connections, just return (it's a singleton cluster)
        if node_degree == 0 {
            return;
        }
        
        // BFS
        let mut queue = VecDeque::new();
        queue.push_back(start_id.to_string());
        
        while let Some(node_id) = queue.pop_front() {
            // Get all adjacent nodes
            if let Some(neighbors) = self.adjacency.get(&node_id) {
                for neighbor_id in neighbors {
                    if !visited.contains(neighbor_id) {
                        visited.insert(neighbor_id.to_string());
                        
                        // Assign cluster ID
                        if let Some(node) = self.nodes.get_mut(neighbor_id) {
                            node.cluster_id = Some(cluster_id);
                        }
                        
                        queue.push_back(neighbor_id.to_string());
                    }
                }
            }
        }
    }
    
    /// Retrieve all clusters as a map of cluster ID -> list of node IDs
    pub fn retrieve_clusters(&self, include_singletons: bool) -> HashMap<usize, Vec<String>> {
        let mut clusters: HashMap<usize, Vec<String>> = HashMap::new();
        
        for (id, node) in &self.nodes {
            if let Some(cluster_id) = node.cluster_id {
                if include_singletons || node.degree > 0 {
                    clusters.entry(cluster_id)
                        .or_insert_with(Vec::new)
                        .push(id.clone());
                }
            }
        }
        
        clusters
    }
    
    /// Extract nodes that have no connections (singletons)
    pub fn extract_singleton_nodes(&self) -> Vec<String> {
        self.nodes.iter()
            .filter(|(_, node)| node.degree == 0)
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    /// Convert the network to JSON format for output
    pub fn to_json(&self) -> NetworkJSON {
        // Get all clusters
        let all_clusters_map = self.retrieve_clusters(true);
        
        // Get counts of connected and singleton nodes
        let connected_nodes_count = self.nodes.values()
            .filter(|node| node.degree > 0)
            .count();
        
        let singleton_count = self.nodes.len() - connected_nodes_count;
            
        // Identify real clusters (with 2+ connected nodes)
        let mut connected_clusters: HashMap<usize, Vec<String>> = HashMap::new();
        
        // Track which clusters have connected nodes
        let mut real_cluster_ids = HashSet::new();
        
        for (&cluster_id, nodes) in &all_clusters_map {
            // Count nodes with degree > 0
            let connected_node_ids: Vec<String> = nodes.iter()
                .filter(|node_id| {
                    // Need to dereference the node_id
                    let key: &String = node_id;
                    if let Some(node) = self.nodes.get(key) {
                        node.degree > 0
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();
            
            // If we have 2+ connected nodes, this is a real cluster
            if connected_node_ids.len() > 1 {
                real_cluster_ids.insert(cluster_id);
                connected_clusters.insert(cluster_id, connected_node_ids);
            }
        }
        
        // Count real clusters with 2+ connected nodes
        // We use the connected_clusters.len() instead
        
        // Get counts
        let edge_count = self.edges.iter().filter(|edge| edge.visible).count();
        let node_count = self.nodes.len();
        let connected_node_count = connected_nodes_count; // Nodes with connections
        let cluster_count = connected_clusters.len(); // Only use connected clusters with 2+ nodes
        
        // Create cluster sizes
        let mut cluster_sizes: Vec<usize> = connected_clusters.values()
            .map(|nodes| nodes.len())
            .collect();
        cluster_sizes.sort_unstable();
        
        // Create vectors of nodes for output
        let mut node_ids: Vec<String> = Vec::with_capacity(node_count);
        let mut node_clusters: Vec<usize> = Vec::with_capacity(node_count);
        let mut node_attributes: Vec<serde_json::Value> = Vec::with_capacity(node_count);
        
        // For consistent ordering, get sorted node IDs
        let mut sorted_node_ids: Vec<&String> = self.nodes.keys().collect();
        sorted_node_ids.sort();
        
        // Create node index map and populate node vectors
        let mut node_id_to_index: HashMap<String, usize> = HashMap::with_capacity(node_count);
        
        for (idx, &node_id) in sorted_node_ids.iter().enumerate() {
            node_id_to_index.insert(node_id.clone(), idx);
            node_ids.push(node_id.clone());
            
            let node = &self.nodes[node_id];
            
            // Use 1-indexed cluster IDs as per original format
            let cluster_id = node.cluster_id.map(|id| id + 1).unwrap_or(0);
            node_clusters.push(cluster_id);
            
            // For compatibility, just provide minimal attributes
            node_attributes.push(serde_json::json!({}));
        }
        
        // Create edge vectors
        let mut edge_sequences: Vec<Vec<String>> = Vec::with_capacity(edge_count);
        let mut edge_sources: Vec<usize> = Vec::with_capacity(edge_count);
        let mut edge_targets: Vec<usize> = Vec::with_capacity(edge_count);
        let mut edge_lengths: Vec<f64> = Vec::with_capacity(edge_count);
        
        for edge in self.edges.iter().filter(|edge| edge.visible) {
            // Skip edges for nodes that don't exist in the index
            if !node_id_to_index.contains_key(&edge.source_id) || !node_id_to_index.contains_key(&edge.target_id) {
                continue;
            }
            
            let source_idx = node_id_to_index[&edge.source_id];
            let target_idx = node_id_to_index[&edge.target_id];
            
            edge_sequences.push(vec![edge.source_id.clone(), edge.target_id.clone()]);
            edge_sources.push(source_idx);
            edge_targets.push(target_idx);
            edge_lengths.push(edge.distance);
        }
        
        // Values for directed edges
        let directed_keys = HashMap::from([("0".to_string(), false)]);
        let directed_values = vec![0; edge_sources.len()];
        
        // Values for attributes
        let attribute_keys = HashMap::from([("0".to_string(), vec!["BULK".to_string()])]);
        let attribute_values = vec![0; edge_sources.len()];
        
        // Values for support
        let support_keys = HashMap::from([("0".to_string(), 0.0)]);
        let support_values = vec![0; edge_sources.len()];
        
        // Calculate degree distribution
        let max_degree = self.nodes.values()
            .map(|node| node.degree)
            .max()
            .unwrap_or(0);
        
        let mut degree_distribution = vec![0; max_degree + 1];
        for node in self.nodes.values() {
            degree_distribution[node.degree] += 1;
        }
        
        // Create HIV stages mapping
        let mut hiv_stages = HashMap::new();
        hiv_stages.insert("Unknown".to_string(), node_count);
        
        // Create attribute schema
        let mut attribute_schema = HashMap::new();
        attribute_schema.insert("id".to_string(), AttributeSchema {
            name: "id".to_string(),
            attr_type: "String".to_string(),
            label: "id".to_string(),
        });
        
        // Get threshold setting from metadata
        let threshold = self.metadata.get("threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.015);
        
        // Current timestamp
        let current_time = Utc::now().to_rfc3339();
        
        // Create output format
        NetworkJSON {
            trace_results: TraceResults {
                network_summary: NetworkSummary {
                    Edges: edge_count,
                    Nodes: node_count,
                    sequences_used: connected_node_count, // Only count nodes used in connections
                    Clusters: cluster_count,
                    Singletons: singleton_count,
                },
                multiple_sequences: MultipleSequences {
                    subjects_with: 0,
                    followup_days: None,
                },
                cluster_sizes,
                hiv_stages,
                directed_edges: DirectedEdges {
                    Count: 0,
                    reasons: HashMap::from([("Missing dates".to_string(), edge_count)]),
                },
                degrees: Degrees {
                    Distribution: degree_distribution,
                    Model: "None".to_string(),
                    rho: 0.0,
                    rho_ci: vec![0.0, 0.0],
                    BIC: 0.0,
                    fitted: None,
                },
                settings: Settings {
                    threshold,
                    edge_filtering: None,
                    contaminants: None,
                    singletons: true,
                    compact_json: true,
                    created: current_time,
                },
                nodes: NodesOutput {
                    cluster: node_clusters,
                    id: node_ids,
                    patient_attributes: node_attributes,
                },
                edges: EdgesOutput {
                    directed: DirectedValues {
                        keys: directed_keys.clone(),
                        values: directed_values.clone(),
                    },
                    sequences: edge_sequences,
                    target: edge_targets,
                    length: edge_lengths,
                    attributes: AttributeValues {
                        keys: attribute_keys,
                        values: attribute_values,
                    },
                    removed: DirectedValues {
                        keys: directed_keys,
                        values: directed_values,
                    },
                    support: SupportValues {
                        keys: support_keys,
                        values: support_values,
                    },
                    source: edge_sources,
                },
                patient_attribute_schema: attribute_schema,
            }
        }
    }
    
    /// Get network statistics
    pub fn get_network_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        // Count visible edges
        let visible_edges = self.edges.iter().filter(|e| e.visible).count();
        stats.insert("edges".to_string(), serde_json::json!(visible_edges));
        
        // Count nodes
        stats.insert("nodes".to_string(), serde_json::json!(self.nodes.len()));
        
        // Count clusters
        let connected_clusters = self.retrieve_clusters(false);
        let real_cluster_count = connected_clusters.values()
            .filter(|nodes| nodes.len() > 1)
            .count();
        stats.insert("clusters".to_string(), serde_json::json!(real_cluster_count));
        
        // Get largest cluster size
        let largest_cluster_size = connected_clusters.values()
            .map(|nodes| nodes.len())
            .max()
            .unwrap_or(0);
        stats.insert("largest_cluster".to_string(), serde_json::json!(largest_cluster_size));
        
        stats
    }
    
    /// Get the number of nodes in the network
    pub fn get_node_count(&self) -> usize {
        self.nodes.len()
    }
    
    /// Get the number of edges in the network
    pub fn get_edge_count(&self) -> usize {
        self.edges.iter().filter(|e| e.visible).count()
    }
    
    /// Convert network to JSON string
    pub fn to_json_string(&self) -> Result<String, NetworkError> {
        serde_json::to_string(&self.to_json())
            .map_err(NetworkError::Json)
    }
    
    /// Convert network to pretty-printed JSON string
    pub fn to_json_string_pretty(&self) -> Result<String, NetworkError> {
        serde_json::to_string_pretty(&self.to_json())
            .map_err(NetworkError::Json)
    }
    
    /// Check if a node has connections (degree > 0)
    pub fn is_node_connected(&self, node_id: &str) -> bool {
        self.nodes.get(node_id)
            .map(|node| node.degree > 0)
            .unwrap_or(false)
    }
    
}