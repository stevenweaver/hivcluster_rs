use hivcluster_rs::InputFormat;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Print usage if not enough arguments
    if args.len() < 2 {
        eprintln!("Usage: {} [options] <input.csv>", args[0]);
        eprintln!("Options:");
        eprintln!("  -t, --threshold <value>  Distance threshold (default: 0.015)");
        eprintln!("  -o, --output <file>      Output JSON file (default: stdout)");
        eprintln!("  -f, --format <format>    Input format: aeh, lanl, plain (default: plain)");
        process::exit(1);
    }
    
    // Parse arguments
    let mut input_file = None;
    let mut output_file = None;
    let mut threshold = 0.015;
    let mut input_format = InputFormat::Plain;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-t" | "--threshold" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: Missing threshold value");
                    process::exit(1);
                }
                threshold = match args[i].parse::<f64>() {
                    Ok(t) => t,
                    Err(_) => {
                        eprintln!("Error: Invalid threshold value");
                        process::exit(1);
                    }
                };
            },
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: Missing output file");
                    process::exit(1);
                }
                output_file = Some(args[i].clone());
            },
            "-f" | "--format" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: Missing format");
                    process::exit(1);
                }
                input_format = match args[i].to_lowercase().as_str() {
                    "aeh" => InputFormat::AEH,
                    "lanl" => InputFormat::LANL,
                    "plain" => InputFormat::Plain,
                    _ => {
                        eprintln!("Error: Unknown format '{}', using plain", args[i]);
                        InputFormat::Plain
                    }
                };
            },
            _ => {
                if input_file.is_none() && !args[i].starts_with('-') {
                    input_file = Some(args[i].clone());
                } else {
                    eprintln!("Warning: Ignoring unknown argument: {}", args[i]);
                }
            }
        }
        i += 1;
    }
    
    // Read input data
    let input_data = if let Some(file) = input_file {
        match fs::read_to_string(&file) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading file '{}': {}", file, e);
                process::exit(1);
            }
        }
    } else {
        // Read from stdin
        let mut buffer = String::new();
        match io::stdin().read_to_string(&mut buffer) {
            Ok(_) => buffer,
            Err(e) => {
                eprintln!("Error reading from stdin: {}", e);
                process::exit(1);
            }
        }
    };
    
    // Process the network
    let mut network = hivcluster_rs::TransmissionNetwork::new();
    
    if let Err(e) = network.read_from_csv_str(&input_data, threshold, input_format) {
        eprintln!("Error processing network: {}", e);
        process::exit(1);
    }
    
    network.compute_clusters();
    
    // Generate JSON output
    let network_json = network.to_json();
    let json_str = serde_json::to_string_pretty(&network_json).unwrap();
    
    // Write output
    if let Some(file) = output_file {
        match fs::write(&file, json_str) {
            Ok(_) => {
                println!("Network saved to '{}'", file);
                // Print summary stats
                let stats = network.get_network_stats();
                println!("Network summary:");
                println!("  Nodes: {}", stats.get("nodes").unwrap());
                println!("  Edges: {}", stats.get("edges").unwrap());
                println!("  Clusters: {}", stats.get("clusters").unwrap());
                println!("  Largest cluster size: {}", stats.get("largest_cluster").unwrap());
            },
            Err(e) => {
                eprintln!("Error writing to file '{}': {}", file, e);
                process::exit(1);
            }
        }
    } else {
        // Print to stdout
        println!("{}", json_str);
    }
}