use std::fs;
use regex::Regex;
use serde_json::Value;
use std::env;

fn update_urls(value: &mut Value, new_url: &str) {
    if let Value::Object(map) = value {
        if let Some(resolved) = map.get_mut("resolved") {
            if resolved.is_string() {
                let old_url = resolved.as_str().unwrap();

                // Create a regex to match URLs
                let re = Regex::new(r"https?://[^/]+").unwrap();

                // Replace the matched part of the URL with the new URL
                let updated_url = re.replace(old_url, new_url);
                *resolved = Value::String(updated_url.into_owned());
            }
        }

        // Recursively update nested objects
        for key in map.keys().cloned().collect::<Vec<_>>() {
            update_urls(&mut map[&key], new_url);
        }
    }
}


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

    // Update URLs in package-lock.json
    update_urls(&mut json_content, new_url);

    // Write the updated JSON back to package-lock.json
    file_content = serde_json::to_string_pretty(&json_content)?;
    fs::write("package-lock.json", file_content)?;

    println!("URLs in package-lock.json have been updated.");
    Ok(())
}
