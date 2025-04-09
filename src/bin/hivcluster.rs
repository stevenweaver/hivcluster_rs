use hivcluster_rs::{InputFormat, NetworkError, TransmissionNetwork};
use std::env;
use std::fs;
use std::io::{self, Read};
use std::process;

fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let config = match parse_args(&args) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error: {}", e);
            print_usage(&args[0]);
            process::exit(1);
        }
    };
    
    // Read input data
    let input_data = match read_input(&config.input_file) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading input: {}", e);
            process::exit(1);
        }
    };
    
    // Create network
    let mut network = TransmissionNetwork::new();
    
    // Parse input data and construct network
    match network.read_from_csv_str(&input_data, config.threshold, config.input_format) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error processing network: {}", e);
            process::exit(1);
        }
    }
    
    // Compute the adjacency list and identify clusters
    network.compute_adjacency();
    network.compute_clusters();
    
    // Generate JSON output
    let json_str = match network.to_json_string_pretty() {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Error generating JSON: {}", e);
            process::exit(1);
        }
    };
    
    // Write output
    match &config.output_file {
        Some(file) => {
            match fs::write(file, &json_str) {
                Ok(_) => {
                    println!("Network saved to '{}'", file);
                    
                    // Print summary stats
                    let stats = network.get_network_stats();
                    println!("Network summary:");
                    println!("  Nodes: {}", stats.get("nodes").unwrap_or(&serde_json::json!(0)));
                    println!("  Edges: {}", stats.get("edges").unwrap_or(&serde_json::json!(0)));
                    println!("  Clusters: {}", stats.get("clusters").unwrap_or(&serde_json::json!(0)));
                    println!("  Largest cluster size: {}", stats.get("largest_cluster").unwrap_or(&serde_json::json!(0)));
                },
                Err(e) => {
                    eprintln!("Error writing to file '{}': {}", file, e);
                    process::exit(1);
                }
            }
        },
        None => {
            // Print to stdout
            println!("{}", json_str);
        }
    }
}

/// Configuration for the program
struct Config {
    input_file: Option<String>,
    output_file: Option<String>,
    threshold: f64,
    input_format: InputFormat,
}

/// Parse command line arguments
fn parse_args(args: &[String]) -> Result<Config, String> {
    if args.len() < 2 {
        return Err("Not enough arguments".to_string());
    }
    
    let mut config = Config {
        input_file: None,
        output_file: None,
        threshold: 0.015, // Default threshold
        input_format: InputFormat::Plain,
    };
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-t" | "--threshold" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing threshold value".to_string());
                }
                
                config.threshold = match args[i].parse::<f64>() {
                    Ok(t) => {
                        if t <= 0.0 {
                            return Err("Threshold must be greater than 0".to_string());
                        }
                        t
                    },
                    Err(_) => return Err("Invalid threshold value".to_string()),
                };
            },
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing output file".to_string());
                }
                config.output_file = Some(args[i].clone());
            },
            "-f" | "--format" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing format".to_string());
                }
                
                config.input_format = match args[i].to_lowercase().as_str() {
                    "aeh" => InputFormat::AEH,
                    "lanl" => InputFormat::LANL,
                    "plain" => InputFormat::Plain,
                    "regex" => InputFormat::Regex,
                    _ => return Err(format!("Unknown format: {}", args[i])),
                };
            },
            // Check if this is a non-option argument (input file)
            _ if !args[i].starts_with('-') => {
                if config.input_file.is_none() {
                    config.input_file = Some(args[i].clone());
                } else {
                    return Err(format!("Unexpected argument: {}", args[i]));
                }
            },
            _ => {
                return Err(format!("Unknown option: {}", args[i]));
            }
        }
        i += 1;
    }
    
    Ok(config)
}

/// Read input from file or stdin
fn read_input(input_file: &Option<String>) -> Result<String, NetworkError> {
    match input_file {
        Some(file) => {
            fs::read_to_string(file).map_err(NetworkError::Io)
        },
        None => {
            // Read from stdin
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)
                .map_err(NetworkError::Io)?;
            Ok(buffer)
        }
    }
}

/// Print usage information
fn print_usage(program_name: &str) {
    eprintln!("Usage: {} [options] <input.csv>", program_name);
    eprintln!("Options:");
    eprintln!("  -t, --threshold <value>  Distance threshold (default: 0.015)");
    eprintln!("  -o, --output <file>      Output JSON file (default: stdout)");
    eprintln!("  -f, --format <format>    Input format: aeh, lanl, plain, regex (default: plain)");
    eprintln!("");
    eprintln!("Input formats:");
    eprintln!("  plain: Simple node IDs with no metadata");
    eprintln!("  aeh:   Format 'ID | date | other_fields'");
    eprintln!("  lanl:  Format 'subtype_country_id_year'");
    eprintln!("  regex: Extract dates from IDs using regex patterns");
}