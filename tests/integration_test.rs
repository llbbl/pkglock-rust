use serde_json::Value;
use std::fs;
use pkglock_lib::package_lock_lib::{update_urls, update_urls_in_package_lock};

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
    update_urls(&mut json_content, "http://localhost:4873");

    // Verify the URL was updated
    assert_eq!(
        json_content["dependencies"]["package-a"]["resolved"],
        "http://localhost:4873/package-a/-/package-a-1.0.0.tgz"
    );

    // Clean up test files
    fs::remove_file("test_pkg.config.json").unwrap();
    fs::remove_file("test_package-lock.json").unwrap();
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

    // Check that the URL has been updated
    assert!(updated_content.contains("http://localhost:4873"));
    assert!(!updated_content.contains("https://registry.npmjs.org"));

    // Clean up temporary files
    fs::remove_file("pkg.config.json").unwrap();
    fs::remove_file("package-lock.json").unwrap();
} 