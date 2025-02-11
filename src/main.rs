use std::fs;
use serde_json::Value;
use std::env;

// Import the update_urls function from the library crate
use pkglock_lib::update_urls;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read command-line arguments
    let args: Vec<String> = env::args().collect();

    // check if command-line argument is provided else return error
    if args.len() != 2 || (args[1] != "--local" && args[1] != "--remote") {
        println!("Usage: program --local | --remote");
        return Ok(());
    }

    // check if file exists else return error
    if !std::path::Path::new("pkg.config.json").exists() {
        return Err("pkg.config.json not found".into());
    }

    // check if file exists else return error
    if !std::path::Path::new("package-lock.json").exists() {
        return Err("package-lock.json not found".into());
    }

    // Read and parse package-lock.json
    let mut file_content = fs::read_to_string("package-lock.json")?;
    let mut json_content: Value = serde_json::from_str(&file_content)?;

    // Read and parse pkg.config.json
    let config_content = fs::read_to_string("pkg.config.json")?;
    let config: Value = serde_json::from_str(&config_content)?;

    // Determine new URL based on command-line argument
    let new_url = if args[1] == "--local" {
        config["local"].as_str().unwrap()
    } else {
        config["remote"].as_str().unwrap()
    };

    // Update URLs in package-lock.json using the library function
    update_urls(&mut json_content, new_url);

    // Write the updated JSON back to package-lock.json
    file_content = serde_json::to_string_pretty(&json_content)?;
    fs::write("package-lock.json", file_content)?;

    println!("URLs in package-lock.json have been updated.");
    Ok(())
}
