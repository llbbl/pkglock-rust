use regex::Regex;
use serde_json::Value;

// Public function to update URLs in a JSON value
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
} 