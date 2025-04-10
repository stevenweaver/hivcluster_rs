use hivcluster_rs::AnnotationError;
use std::env;
use std::fs;
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

    // Read input files
    let network_json = match fs::read_to_string(&config.network_file) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading network file: {}", e);
            process::exit(1);
        }
    };

    let attributes_json = match fs::read_to_string(&config.attributes_file) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading attributes file: {}", e);
            process::exit(1);
        }
    };

    let schema_json = match fs::read_to_string(&config.schema_file) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading schema file: {}", e);
            process::exit(1);
        }
    };

    // Annotate the network
    let result = match hivcluster_rs::annotate_network(&network_json, &attributes_json, &schema_json) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error annotating network: {}", e);
            process::exit(1);
        }
    };

    // Write output
    match &config.output_file {
        Some(file) => {
            match fs::write(file, &result) {
                Ok(_) => {
                    println!("Annotated network saved to '{}'", file);
                }
                Err(e) => {
                    eprintln!("Error writing to file '{}': {}", file, e);
                    process::exit(1);
                }
            }
        }
        None => {
            // Print to stdout
            println!("{}", result);
        }
    }
}

/// Configuration for the program
struct Config {
    network_file: String,
    attributes_file: String,
    schema_file: String,
    output_file: Option<String>,
}

/// Parse command line arguments
fn parse_args(args: &[String]) -> Result<Config, String> {
    if args.len() < 4 {
        return Err("Not enough arguments".to_string());
    }

    let mut config = Config {
        network_file: String::new(),
        attributes_file: String::new(),
        schema_file: String::new(),
        output_file: None,
    };

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-n" | "--network" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing network file".to_string());
                }
                config.network_file = args[i].clone();
            }
            "-a" | "--attributes" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing attributes file".to_string());
                }
                config.attributes_file = args[i].clone();
            }
            "-s" | "--schema" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing schema file".to_string());
                }
                config.schema_file = args[i].clone();
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing output file".to_string());
                }
                config.output_file = Some(args[i].clone());
            }
            _ => {
                return Err(format!("Unknown option: {}", args[i]));
            }
        }
        i += 1;
    }

    // Check required fields
    if config.network_file.is_empty() {
        return Err("Network file is required".to_string());
    }
    if config.attributes_file.is_empty() {
        return Err("Attributes file is required".to_string());
    }
    if config.schema_file.is_empty() {
        return Err("Schema file is required".to_string());
    }

    Ok(config)
}

/// Print usage information
fn print_usage(program_name: &str) {
    eprintln!("Usage: {} [options]", program_name);
    eprintln!("Options:");
    eprintln!("  -n, --network <file>      Input network JSON file (required)");
    eprintln!("  -a, --attributes <file>   Patient attributes JSON file (required)");
    eprintln!("  -s, --schema <file>       Attribute schema JSON file (required)");
    eprintln!("  -o, --output <file>       Output JSON file (default: stdout)");
    eprintln!("");
    eprintln!("Example:");
    eprintln!("  {} -n network.json -a attributes.json -s schema.json -o annotated_network.json", program_name);
}