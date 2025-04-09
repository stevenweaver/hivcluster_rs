use crate::parser::{parse_patient_id, ParsedPatient};
use crate::types::{Edge, InputFormat, NetworkError, Patient};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;

/// The main network structure
#[derive(Debug, Clone)]
pub struct TransmissionNetwork {
    /// All patients/nodes in the network
    pub nodes: Vec<Rc<Patient>>,
    
    /// Index of nodes by ID for faster lookup
    node_index: HashMap<String, usize>,
    
    /// All edges in the network
    pub edges: Vec<Edge>,
    
    /// Adjacency list representation (node indices -> edge indices)
    adjacency: HashMap<usize, Vec<usize>>,
    
    /// Edge distances
    pub distances: HashMap<(String, String), f64>,
    
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

/// Output JSON format
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkJSON {
    pub nodes: Vec<NodeJSON>,
    pub edges: Vec<EdgeJSON>,
    pub clusters: Vec<Cluster>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Node representation for JSON output
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeJSON {
    pub id: String,
    pub cluster: Option<usize>,
    pub degree: usize,
    pub attributes: Vec<String>,
}

/// Edge representation for JSON output
#[derive(Debug, Serialize, Deserialize)]
pub struct EdgeJSON {
    pub source: String,
    pub target: String,
    pub distance: f64,
    pub visible: bool,
}

impl TransmissionNetwork {
    /// Create a new empty network
    pub fn new() -> Self {
        TransmissionNetwork {
            nodes: Vec::new(),
            node_index: HashMap::new(),
            edges: Vec::new(),
            adjacency: HashMap::new(),
            distances: HashMap::new(),
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
        let mut rdr = csv::ReaderBuilder::new()
            .flexible(true)
            .from_reader(csv_str.as_bytes());
        
        for result in rdr.records() {
            let record = result?;
            
            if record.len() < 3 {
                return Err(NetworkError::Format(
                    "CSV must have at least 3 columns: node1,node2,distance".to_string()
                ));
            }
            
            let id1 = record.get(0).unwrap();
            let id2 = record.get(1).unwrap();
            let distance = record.get(2).unwrap().parse::<f64>()
                .map_err(|_| NetworkError::Format(format!("Invalid distance: {}", record.get(2).unwrap())))?;
            
            // Skip edges with distance greater than threshold
            if distance > distance_threshold {
                continue;
            }
            
            // Parse node IDs
            let patient1 = parse_patient_id(id1, format, None)?;
            let patient2 = parse_patient_id(id2, format, None)?;
            
            // Add nodes and edge to the network
            self.add_edge(patient1, patient2, distance)?;
        }
        
        // Compute adjacency list after loading all edges
        self.compute_adjacency();
        
        Ok(())
    }
    
    /// Add a node to the network or retrieve existing node
    fn add_node(&mut self, patient_data: ParsedPatient) -> usize {
        if let Some(&idx) = self.node_index.get(&patient_data.id) {
            // Node exists, update it if possible
            if let Some(node) = Rc::get_mut(&mut self.nodes[idx]) {
                node.add_date(patient_data.date);
            }
            return idx;
        }
        
        // Create new node
        let mut patient = Patient::new(&patient_data.id);
        patient.add_date(patient_data.date);
        
        // Add to network
        let idx = self.nodes.len();
        self.nodes.push(Rc::new(patient));
        self.node_index.insert(patient_data.id.clone(), idx);
        
        idx
    }
    
    /// Add an edge between two patients
    fn add_edge(
        &mut self,
        patient1: ParsedPatient,
        patient2: ParsedPatient,
        distance: f64,
    ) -> Result<(), NetworkError> {
        // Add nodes
        let idx1 = self.add_node(patient1.clone());
        let idx2 = self.add_node(patient2.clone());
        
        // Make sure we are not adding a self-loop
        if idx1 == idx2 {
            return Err(NetworkError::SelfLoop);
        }
        
        // Create bidirectional key for distance storage
        let key = if patient1.id < patient2.id {
            (patient1.id.clone(), patient2.id.clone())
        } else {
            (patient2.id.clone(), patient1.id.clone())
        };
        
        // Store distance
        self.distances.insert(key, distance);
        
        // Create edge
        let source = Rc::clone(&self.nodes[idx1]);
        let target = Rc::clone(&self.nodes[idx2]);
        
        // Always make patient IDs ordered in the edge
        let edge = Edge::new(
            source,
            target,
            patient1.date,
            patient2.date,
            true,
            distance,
        )?;
        
        // Add edge to the list
        self.edges.push(edge);
        
        // Update node degrees
        if let Some(node) = Rc::get_mut(&mut self.nodes[idx1]) {
            node.increment_degree();
        }
        
        if let Some(node) = Rc::get_mut(&mut self.nodes[idx2]) {
            node.increment_degree();
        }
        
        Ok(())
    }
    
    /// Compute adjacency list for efficient graph traversal
    pub fn compute_adjacency(&mut self) {
        self.adjacency.clear();
        
        for (edge_idx, edge) in self.edges.iter().enumerate() {
            if !edge.visible {
                continue;
            }
            
            let source_idx = match self.node_index.get(&edge.source.id) {
                Some(&idx) => idx,
                None => continue,
            };
            
            let target_idx = match self.node_index.get(&edge.target.id) {
                Some(&idx) => idx,
                None => continue,
            };
            
            // Add edge to source's adjacency list
            self.adjacency.entry(source_idx)
                .or_insert_with(Vec::new)
                .push(edge_idx);
                
            // Add edge to target's adjacency list
            self.adjacency.entry(target_idx)
                .or_insert_with(Vec::new)
                .push(edge_idx);
        }
    }
    
    /// Identify connected components (clusters) in the network
    pub fn compute_clusters(&mut self) {
        // Reset all cluster assignments
        for i in 0..self.nodes.len() {
            if let Some(node) = Rc::get_mut(&mut self.nodes[i]) {
                node.cluster_id = None;
            }
        }
        
        let mut cluster_id = 0;
        let mut visited = HashSet::new();
        
        // Process each node
        for node_idx in 0..self.nodes.len() {
            if visited.contains(&node_idx) {
                continue;
            }
            
            // BFS to find all nodes in this cluster
            self.breadth_first_traverse(node_idx, cluster_id, &mut visited);
            cluster_id += 1;
        }
    }
    
    /// Breadth-first search to identify a cluster
    fn breadth_first_traverse(&mut self, start_idx: usize, cluster_id: usize, visited: &mut HashSet<usize>) {
        let mut queue = VecDeque::new();
        queue.push_back(start_idx);
        visited.insert(start_idx);
        
        // Assign cluster ID to starting node
        if let Some(node) = Rc::get_mut(&mut self.nodes[start_idx]) {
            node.cluster_id = Some(cluster_id);
        }
        
        while let Some(node_idx) = queue.pop_front() {
            // Get all adjacent nodes
            if let Some(edge_indices) = self.adjacency.get(&node_idx) {
                for &edge_idx in edge_indices {
                    let edge = &self.edges[edge_idx];
                    if !edge.visible {
                        continue;
                    }
                    
                    // Determine the other node in this edge
                    let source_idx = match self.node_index.get(&edge.source.id) {
                        Some(&idx) => idx,
                        None => continue,
                    };
                    
                    let target_idx = match self.node_index.get(&edge.target.id) {
                        Some(&idx) => idx,
                        None => continue,
                    };
                    
                    let other_idx = if node_idx == source_idx { target_idx } else { source_idx };
                    
                    // If not visited, add to queue
                    if !visited.contains(&other_idx) {
                        visited.insert(other_idx);
                        
                        // Assign cluster ID
                        if let Some(node) = Rc::get_mut(&mut self.nodes[other_idx]) {
                            node.cluster_id = Some(cluster_id);
                        }
                        
                        queue.push_back(other_idx);
                    }
                }
            }
        }
    }
    
    /// Retrieve all clusters as a map of cluster ID -> list of node IDs
    pub fn retrieve_clusters(&self, include_singletons: bool) -> HashMap<usize, Vec<String>> {
        let mut clusters: HashMap<usize, Vec<String>> = HashMap::new();
        
        for node in &self.nodes {
            if let Some(cluster_id) = node.cluster_id {
                if include_singletons || node.degree > 0 {
                    clusters.entry(cluster_id)
                        .or_insert_with(Vec::new)
                        .push(node.id.clone());
                }
            }
        }
        
        clusters
    }
    
    /// Extract nodes that have no connections (singletons)
    pub fn extract_singleton_nodes(&self) -> Vec<String> {
        self.nodes.iter()
            .filter(|node| node.degree == 0)
            .map(|node| node.id.clone())
            .collect()
    }
    
    /// Convert the network to JSON format for output
    pub fn to_json(&self) -> NetworkJSON {
        // Create node JSON objects
        let nodes = self.nodes.iter()
            .map(|node| NodeJSON {
                id: node.id.clone(),
                cluster: node.cluster_id,
                degree: node.degree,
                attributes: node.attributes.iter().cloned().collect(),
            })
            .collect();
        
        // Create edge JSON objects
        let edges = self.edges.iter()
            .filter(|edge| edge.visible)
            .map(|edge| EdgeJSON {
                source: edge.source.id.clone(),
                target: edge.target.id.clone(),
                distance: edge.distance,
                visible: edge.visible,
            })
            .collect();
        
        // Create cluster objects
        let clusters_map = self.retrieve_clusters(true);
        let clusters = clusters_map.into_iter()
            .map(|(id, nodes)| Cluster {
                id,
                nodes: nodes.clone(),
                size: nodes.len(),
            })
            .collect();
        
        NetworkJSON {
            nodes,
            edges,
            clusters,
            metadata: self.metadata.clone(),
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
        let clusters = self.retrieve_clusters(false);
        stats.insert("clusters".to_string(), serde_json::json!(clusters.len()));
        
        // Get largest cluster size
        let largest_cluster_size = clusters.values()
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
    
    /// Get the number of clusters in the network
    pub fn get_cluster_count(&self) -> usize {
        let clusters = self.retrieve_clusters(false);
        clusters.len()
    }
    
    /// Convert network to JSON string
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(&self.to_json()).unwrap_or_else(|_| "{}".to_string())
    }
}