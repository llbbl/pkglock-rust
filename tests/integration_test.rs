use serde_json::Value;
use std::fs;
use std::path::Path;

#[test]
fn test_file_operations() {
    // Create a temporary pkg.config.json
    let config = r#"{
        "local": "http://localhost:4873",
        "remote": "https://registry.npmjs.org"
    }"#;
    fs::write("test_pkg.config.json", config).unwrap();

    // Create a temporary package-lock.json
    let package_lock = r#"{
        "name": "test-package",
        "dependencies": {
            "package-a": {
                "resolved": "https://registry.npmjs.org/package-a/-/package-a-1.0.0.tgz"
            }
        }
    }"#;
    fs::write("test_package-lock.json", package_lock).unwrap();

    // Parse the package-lock.json
    let mut json_content: Value = serde_json::from_str(package_lock).unwrap();

    // Update URLs
    pkglock_rust::lib::update_urls(&mut json_content, "http://localhost:4873");

    // Verify the URL was updated
    assert_eq!(
        json_content["dependencies"]["package-a"]["resolved"],
        "http://localhost:4873/package-a/-/package-a-1.0.0.tgz"
    );

    // Clean up test files
    fs::remove_file("test_pkg.config.json").unwrap();
    fs::remove_file("test_package-lock.json").unwrap();
} 