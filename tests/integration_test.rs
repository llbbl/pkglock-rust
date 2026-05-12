use pkglock_lib::{update_urls, update_urls_from_config};
use serde_json::Value;
use std::fs;

#[test]
fn test_resolved_url_rewrite_in_dependencies_tree() {
    let package_lock = r#"{
        "dependencies": {
            "package-a": {
                "resolved": "https://registry.npmjs.org/package-a/-/package-a-1.0.0.tgz"
            }
        }
    }"#;
    let mut json_content: Value = serde_json::from_str(package_lock).unwrap();
    update_urls(&mut json_content, "http://localhost:4873");
    assert_eq!(
        json_content["dependencies"]["package-a"]["resolved"],
        "http://localhost:4873/package-a/-/package-a-1.0.0.tgz"
    );
}

#[test]
fn test_update_urls_in_package_lock() {
    // Use TempDir + the path-accepting API so this test does not pollute the
    // repo root and is safe under cargo's parallel test execution.
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("pkg.config.json");
    let lockfile_path = dir.path().join("package-lock.json");

    let pkg_config = r#"{
        "local": "http://localhost:4873",
        "remote": "https://registry.npmjs.org"
    }"#;
    fs::write(&config_path, pkg_config).unwrap();

    let package_lock = r#"{
        "dependencies": {
            "package-a": {
                "resolved": "https://registry.npmjs.org/package-a/-/package-a-1.0.0.tgz"
            }
        }
    }"#;
    fs::write(&lockfile_path, package_lock).unwrap();

    update_urls_from_config(&config_path, &lockfile_path, "--local").unwrap();

    // Read updated package-lock.json
    let updated_content = fs::read_to_string(&lockfile_path).unwrap();

    // Check that the URL has been updated
    assert!(updated_content.contains("http://localhost:4873"));
    assert!(!updated_content.contains("https://registry.npmjs.org"));
}
