// New module to encapsulate the reading, parsing, and updating of package-lock.json, as well as the URL update functionality
pub mod package_lock_lib {
    use regex::Regex;
    use serde_json::Value;
    use std::fs;
    use std::path::Path;

    // Function to update URLs in a JSON value recursively.
    // Compiles the URL regex once and delegates to an inner helper for recursion.
    pub fn update_urls(value: &mut Value, new_url: &str) {
        let re = Regex::new(r"https?://[^/]+").unwrap();
        update_urls_inner(value, new_url, &re);
    }

    fn update_urls_inner(value: &mut Value, new_url: &str, re: &Regex) {
        match value {
            Value::Object(map) => {
                if let Some(resolved) = map.get_mut("resolved") {
                    if let Some(old_url) = resolved.as_str() {
                        // Replace the matched part of the URL with the new URL
                        let updated_url = re.replace(old_url, new_url).into_owned();
                        *resolved = Value::String(updated_url);
                    }
                }
                // Recursively update nested objects
                for v in map.values_mut() {
                    update_urls_inner(v, new_url, re);
                }
            }
            Value::Array(arr) => {
                for item in arr.iter_mut() {
                    update_urls_inner(item, new_url, re);
                }
            }
            _ => {}
        }
    }

    // Core primitive: read the given lockfile, rewrite its URLs to `new_url`, and write it back.
    // This function knows nothing about pkg.config.json or --local/--remote.
    pub fn rewrite_lockfile(
        lockfile: &Path,
        new_url: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !lockfile.exists() {
            return Err(format!("lockfile not found: {}", lockfile.display()).into());
        }

        // Read and parse the lockfile
        let file_content = fs::read_to_string(lockfile)?;
        let mut json_content: Value = serde_json::from_str(&file_content)?;

        // Update URLs using the update_urls function within this module
        update_urls(&mut json_content, new_url);

        // Write the updated JSON back to the lockfile
        let updated_content = serde_json::to_string_pretty(&json_content)?;
        fs::write(lockfile, updated_content)?;
        Ok(())
    }

    // Resolves the URL to use from `config` based on `arg` (--local or --remote), then
    // delegates to `rewrite_lockfile` against the given lockfile path.
    pub fn update_urls_from_config(
        config: &Path,
        lockfile: &Path,
        arg: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check that the required files exist (preserve historical error messages)
        if !config.exists() {
            return Err("pkg.config.json not found".into());
        }
        if !lockfile.exists() {
            return Err("package-lock.json not found".into());
        }

        // Read+parse lockfile FIRST (matches historical ordering for malformed-JSON precedence).
        let file_content = fs::read_to_string(lockfile)?;
        let mut json_content: Value = serde_json::from_str(&file_content)?;

        // Then read+parse pkg.config.json.
        let config_content = fs::read_to_string(config)?;
        let config_json: Value = serde_json::from_str(&config_content)?;

        // Determine new URL based on argument
        let new_url = if arg == "--local" {
            config_json["local"]
                .as_str()
                .ok_or("Local URL not found in pkg.config.json")?
        } else if arg == "--remote" {
            config_json["remote"]
                .as_str()
                .ok_or("Remote URL not found in pkg.config.json")?
        } else {
            return Err("Invalid argument. Use --local or --remote.".into());
        };

        update_urls(&mut json_content, new_url);
        let updated_content = serde_json::to_string_pretty(&json_content)?;
        fs::write(lockfile, updated_content)?;
        Ok(())
    }

    // Back-compat wrapper that resolves cwd-relative `pkg.config.json` and `package-lock.json`,
    // picks the URL based on `--local` or `--remote`, then delegates to `rewrite_lockfile`.
    // This is the single site where cwd-coupling is encoded.
    pub fn update_urls_in_package_lock(arg: &str) -> Result<(), Box<dyn std::error::Error>> {
        update_urls_from_config(
            Path::new("pkg.config.json"),
            Path::new("package-lock.json"),
            arg,
        )
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
        fn test_update_urls_array_recursion() {
            // Top-level array of objects with `resolved`
            let mut top_array = json!([
                { "resolved": "https://registry.npmjs.org/a/-/a-1.0.0.tgz" },
                { "resolved": "https://registry.npmjs.org/b/-/b-2.0.0.tgz" }
            ]);
            update_urls(&mut top_array, "http://localhost:4873");
            assert_eq!(
                top_array[0]["resolved"],
                "http://localhost:4873/a/-/a-1.0.0.tgz"
            );
            assert_eq!(
                top_array[1]["resolved"],
                "http://localhost:4873/b/-/b-2.0.0.tgz"
            );

            // Nested: object holding an array of objects with `resolved`
            let mut nested = json!({
                "packages": [
                    { "resolved": "https://registry.npmjs.org/c/-/c-3.0.0.tgz" },
                    {
                        "nested": {
                            "resolved": "https://registry.npmjs.org/d/-/d-4.0.0.tgz"
                        }
                    }
                ]
            });
            update_urls(&mut nested, "http://localhost:4873");
            assert_eq!(
                nested["packages"][0]["resolved"],
                "http://localhost:4873/c/-/c-3.0.0.tgz"
            );
            assert_eq!(
                nested["packages"][1]["nested"]["resolved"],
                "http://localhost:4873/d/-/d-4.0.0.tgz"
            );
        }

        #[test]
        fn test_update_urls_mixed_array() {
            let mut mixed = json!([
                "string",
                42,
                null,
                { "resolved": "https://registry.npmjs.org/x/-/x-1.0.0.tgz" }
            ]);
            update_urls(&mut mixed, "http://localhost:4873");
            assert_eq!(mixed[0], "string");
            assert_eq!(mixed[1], 42);
            assert!(mixed[2].is_null());
            assert_eq!(
                mixed[3]["resolved"],
                "http://localhost:4873/x/-/x-1.0.0.tgz"
            );
        }

        #[test]
        fn test_rewrite_lockfile_explicit_path() {
            // Use a uniquely-named subdir under the system temp dir so this test
            // does not pollute the repo root (unlike test_update_urls_in_package_lock,
            // which task #3 will rewrite using tempfile).
            let dir = std::env::temp_dir().join(format!(
                "pkglock-rewrite-lockfile-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(&dir).unwrap();
            let lockfile = dir.join("package-lock.json");

            let package_lock = r#"{
                "dependencies": {
                    "package-a": {
                        "resolved": "https://registry.npmjs.org/package-a/-/package-a-1.0.0.tgz"
                    }
                }
            }"#;
            fs::write(&lockfile, package_lock).unwrap();

            rewrite_lockfile(&lockfile, "http://localhost:4873").unwrap();

            let updated_content = fs::read_to_string(&lockfile).unwrap();
            assert!(updated_content.contains("http://localhost:4873"));
            assert!(!updated_content.contains("https://registry.npmjs.org"));

            // Missing-file error path
            let missing = dir.join("does-not-exist.json");
            let err = rewrite_lockfile(&missing, "http://localhost:4873").unwrap_err();
            let msg = err.to_string();
            assert!(
                msg.contains("lockfile not found"),
                "unexpected error message: {msg}"
            );
            assert!(
                msg.contains(&missing.display().to_string()),
                "error message did not include path: {msg}"
            );

            // Cleanup
            fs::remove_dir_all(&dir).unwrap();
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
