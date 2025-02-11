
// New module to encapsulate the reading, parsing, and updating of package-lock.json, as well as the URL update functionality
pub mod package_lock_lib {
    use regex::Regex;
    use serde_json::Value;
    use std::fs;
    use std::path::Path;

    // Function to update URLs in a JSON value recursively
    pub fn update_urls(value: &mut Value, new_url: &str) {
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

    // Function that reads pkg.config.json and package-lock.json, updates the package-lock.json with the new URL,
    // and writes the updated JSON back to package-lock.json
    pub fn update_urls_in_package_lock(arg: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Check that the required files exist
        if !Path::new("pkg.config.json").exists() {
            return Err("pkg.config.json not found".into());
        }
        if !Path::new("package-lock.json").exists() {
            return Err("package-lock.json not found".into());
        }

        // Read and parse package-lock.json
        let file_content = fs::read_to_string("package-lock.json")?;
        let mut json_content: Value = serde_json::from_str(&file_content)?;

        // Read and parse pkg.config.json
        let config_content = fs::read_to_string("pkg.config.json")?;
        let config: Value = serde_json::from_str(&config_content)?;

        // Determine new URL based on argument
        let new_url = if arg == "--local" {
            config["local"].as_str().ok_or("Local URL not found in pkg.config.json")?
        } else if arg == "--remote" {
            config["remote"].as_str().ok_or("Remote URL not found in pkg.config.json")?
        } else {
            return Err("Invalid argument. Use --local or --remote.".into());
        };

        // Update URLs using the update_urls function within this module
        update_urls(&mut json_content, new_url);

        // Write the updated JSON back to package-lock.json
        let updated_content = serde_json::to_string_pretty(&json_content)?;
        fs::write("package-lock.json", updated_content)?;
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_json::json;
        use std::fs;

        #[test]
        fn test_update_urls_simple() {
            let mut json = json!({
                "resolved": "https://registry.npmjs.org/package/-/package-1.0.0.tgz"
            });
            update_urls(&mut json, "http://localhost:4873");
            assert_eq!(
                json["resolved"],
                "http://localhost:4873/package/-/package-1.0.0.tgz"
            );
        }

        #[test]
        fn test_update_urls_nested() {
            let mut json = json!({
                "dependencies": {
                    "package": {
                        "resolved": "https://registry.npmjs.org/package/-/package-1.0.0.tgz"
                    }
                }
            });
            update_urls(&mut json, "http://localhost:4873");
            assert_eq!(
                json["dependencies"]["package"]["resolved"],
                "http://localhost:4873/package/-/package-1.0.0.tgz"
            );
        }

        #[test]
        fn test_update_urls_no_resolved_field() {
            let mut json = json!({
                "name": "test-package",
                "version": "1.0.0"
            });
            update_urls(&mut json, "http://localhost:4873");
            assert_eq!(json["name"], "test-package");
            assert_eq!(json["version"], "1.0.0");
        }

        #[test]
        fn test_update_urls_in_package_lock() {
            // Create temporary pkg.config.json
            let pkg_config = r#"{
                "local": "http://localhost:4873",
                "remote": "https://registry.npmjs.org"
            }"#;
            fs::write("pkg.config.json", pkg_config).unwrap();

            // Create temporary package-lock.json
            let package_lock = r#"{
                "dependencies": {
                    "package-a": {
                        "resolved": "https://registry.npmjs.org/package-a/-/package-a-1.0.0.tgz"
                    }
                }
            }"#;
            fs::write("package-lock.json", package_lock).unwrap();

            // Call the function with --local argument
            update_urls_in_package_lock("--local").unwrap();

            // Read updated package-lock.json
            let updated_content = fs::read_to_string("package-lock.json").unwrap();
            assert!(updated_content.contains("http://localhost:4873"));
            assert!(!updated_content.contains("https://registry.npmjs.org"));

            // Clean up temporary files
            fs::remove_file("pkg.config.json").unwrap();
            fs::remove_file("package-lock.json").unwrap();
        }
    }
} 