// Library entry point: reads, parses, and updates package-lock.json, plus URL update functionality.
use regex::Regex;
use serde_json::Value;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::Path;
use url::Url;

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
pub fn rewrite_lockfile(lockfile: &Path, new_url: &str) -> Result<(), Box<dyn std::error::Error>> {
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

// ---------------------------------------------------------------------------
// Smart-mode rewriters (conditional, host-aware).
//
// These do not share code with `update_urls`/`update_urls_inner` above — that
// path is unconditional scheme+authority replacement driven by pkg.config.json.
// The smart-mode path needs to *inspect* each resolved URL and decide whether
// to rewrite it based on the host.
//
// The core is `walk_resolved_urls`, a predicate-based JSON walker. The
// `--to-public` entry point composes it with a local-host predicate; future
// flags (`--to-local <URL>`) will reuse the same walker with a different
// predicate.
// ---------------------------------------------------------------------------

/// Returns true if `host` should be treated as a "local" hostname for the
/// purposes of `--to-public` rewriting. See the task spec for the exact rules.
fn is_local_host(host: &str) -> bool {
    // IP literal? (handles IPv4, IPv6, and bracketed IPv6 from URL host_str.)
    let stripped = host.strip_prefix('[').and_then(|s| s.strip_suffix(']'));
    let ip_candidate = stripped.unwrap_or(host);
    if let Ok(ip) = ip_candidate.parse::<IpAddr>() {
        return is_local_ip(ip);
    }

    // Hostname matching is case-insensitive. Also strip a single trailing '.'
    // so FQDN forms like "foo.local." behave the same as "foo.local".
    let lower = host.to_ascii_lowercase();
    let lower = lower.strip_suffix('.').unwrap_or(&lower);

    if lower == "localhost" {
        return true;
    }

    // Dot-prefixed suffix match: must be a multi-label hostname.
    for suffix in [".test", ".local", ".lan"] {
        if lower.ends_with(suffix) && lower.len() > suffix.len() {
            return true;
        }
    }

    false
}

fn is_local_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_local_ipv4(v4),
        IpAddr::V6(v6) => is_local_ipv6(v6),
    }
}

fn is_local_ipv4(ip: Ipv4Addr) -> bool {
    let [a, b, _, _] = ip.octets();
    // 127.0.0.0/8
    if a == 127 {
        return true;
    }
    // 10.0.0.0/8
    if a == 10 {
        return true;
    }
    // 172.16.0.0/12  -> 172.16.0.0 through 172.31.255.255
    if a == 172 && (16..=31).contains(&b) {
        return true;
    }
    // 192.168.0.0/16
    if a == 192 && b == 168 {
        return true;
    }
    false
}

fn is_local_ipv6(ip: Ipv6Addr) -> bool {
    // Per spec: only the loopback literal `::1`.
    ip == Ipv6Addr::LOCALHOST
}

/// Decision returned by a URL-rewrite predicate.
enum RewriteDecision {
    /// Leave the URL untouched.
    Skip,
    /// Replace the `scheme://authority` portion of the URL with this string,
    /// preserving the path, query, and fragment.
    ReplaceSchemeAuthority(String),
}

/// Walks every `resolved` URL in the JSON tree, asks `decide` what to do with
/// it, and rewrites in place. Returns the number of URLs actually changed.
fn walk_resolved_urls<F>(value: &mut Value, decide: &F) -> usize
where
    F: Fn(&Url) -> RewriteDecision,
{
    let mut count = 0;
    match value {
        Value::Object(map) => {
            if let Some(resolved) = map.get_mut("resolved") {
                if let Some(old_url_str) = resolved.as_str() {
                    if let Ok(parsed) = Url::parse(old_url_str) {
                        if let RewriteDecision::ReplaceSchemeAuthority(new_prefix) = decide(&parsed)
                        {
                            let suffix = &old_url_str[scheme_authority_len(old_url_str, &parsed)..];
                            let new_url = format!("{}{}", new_prefix, suffix);
                            if new_url != old_url_str {
                                *resolved = Value::String(new_url);
                                count += 1;
                            }
                        }
                    }
                }
            }
            for v in map.values_mut() {
                count += walk_resolved_urls(v, decide);
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                count += walk_resolved_urls(item, decide);
            }
        }
        _ => {}
    }
    count
}

/// Compute the byte length of the `scheme://authority` prefix in `raw`, given
/// its parsed form. We can't easily ask `url::Url` for this directly, so we
/// locate the first `/` after the `scheme://` portion in the original string.
fn scheme_authority_len(raw: &str, parsed: &Url) -> usize {
    let scheme_len = parsed.scheme().len();
    // raw starts with "<scheme>://"
    let after_scheme = scheme_len + 3;
    // The authority ends at the first '/', '?', or '#' — whichever comes first.
    // If none are present, the whole string is scheme+authority.
    let tail = &raw[after_scheme..];
    let end = tail.find(['/', '?', '#']).unwrap_or(tail.len());
    after_scheme + end
}

/// Read the lockfile, rewrite any `resolved` URL whose host is local (see
/// `is_local_host`) to point at `https://registry.npmjs.org`, write it back,
/// and return the number of URLs changed.
pub fn rewrite_lockfile_to_public(lockfile: &Path) -> Result<usize, Box<dyn std::error::Error>> {
    if !lockfile.exists() {
        return Err(format!("lockfile not found: {}", lockfile.display()).into());
    }

    let file_content = fs::read_to_string(lockfile)?;
    let mut json_content: Value = serde_json::from_str(&file_content)?;

    let decide = |parsed: &Url| -> RewriteDecision {
        match parsed.host_str() {
            Some(host) if is_local_host(host) => {
                RewriteDecision::ReplaceSchemeAuthority("https://registry.npmjs.org".to_string())
            }
            _ => RewriteDecision::Skip,
        }
    };

    let count = walk_resolved_urls(&mut json_content, &decide);

    // Always write back. Cheap, and keeps behavior predictable.
    let updated_content = serde_json::to_string_pretty(&json_content)?;
    fs::write(lockfile, updated_content)?;
    Ok(count)
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
        // TempDir cleans up via Drop, so panics mid-test no longer leak directories.
        let dir = tempfile::tempdir().unwrap();
        let lockfile = dir.path().join("package-lock.json");

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
        let missing = dir.path().join("does-not-exist.json");
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
    }

    #[test]
    fn test_is_local_host_positives() {
        for host in [
            "localhost",
            "LOCALHOST",
            "myhost.test",
            "myhost.local",
            "myhost.lan",
            "a.b.test",
            "Foo.Local",
            "127.0.0.1",
            "127.255.255.254",
            "10.0.0.1",
            "10.255.255.255",
            "172.16.0.1",
            "172.16.0.0",
            "172.31.255.255",
            "192.168.1.1",
            "::1",
        ] {
            assert!(is_local_host(host), "expected {host} to be local");
        }
    }

    #[test]
    fn test_is_local_host_negatives() {
        for host in [
            "registry.npmjs.org",
            "example.com",
            "notlocalhost",
            "localhost.example.com",
            "mytest.com",
            "test",
            "local",
            "lan",
            ".test",
            ".local",
            ".lan",
            "172.15.0.1",
            "172.32.0.1",
            "11.0.0.1",
            "192.169.0.1",
            "8.8.8.8",
            "2001:db8::1",
        ] {
            assert!(!is_local_host(host), "expected {host} to NOT be local");
        }
    }

    #[test]
    fn test_is_local_host_bracketed_ipv6() {
        // url::Url::host_str() returns bracketed form for IPv6.
        assert!(is_local_host("[::1]"));
        assert!(!is_local_host("[2001:db8::1]"));
    }

    #[test]
    fn test_rewrite_lockfile_to_public_mixed() {
        let dir = tempfile::tempdir().unwrap();
        let lockfile = dir.path().join("package-lock.json");

        let package_lock = r#"{
            "dependencies": {
                "keep-me": {
                    "resolved": "https://registry.npmjs.org/keep-me/-/keep-me-1.0.0.tgz"
                },
                "from-localhost": {
                    "resolved": "http://localhost:4873/from-localhost/-/from-localhost-1.0.0.tgz"
                },
                "from-private-ip": {
                    "resolved": "http://192.168.1.10:4873/from-private-ip/-/from-private-ip-2.0.0.tgz"
                },
                "from-test-tld": {
                    "resolved": "http://myhost.test/from-test-tld/-/from-test-tld-3.0.0.tgz"
                },
                "from-external": {
                    "resolved": "https://example.com/from-external/-/from-external-4.0.0.tgz"
                }
            }
        }"#;
        fs::write(&lockfile, package_lock).unwrap();

        let count = rewrite_lockfile_to_public(&lockfile).unwrap();
        assert_eq!(count, 3, "expected 3 URLs to be rewritten");

        let updated: Value = serde_json::from_str(&fs::read_to_string(&lockfile).unwrap()).unwrap();

        assert_eq!(
            updated["dependencies"]["keep-me"]["resolved"],
            "https://registry.npmjs.org/keep-me/-/keep-me-1.0.0.tgz"
        );
        assert_eq!(
            updated["dependencies"]["from-localhost"]["resolved"],
            "https://registry.npmjs.org/from-localhost/-/from-localhost-1.0.0.tgz"
        );
        assert_eq!(
            updated["dependencies"]["from-private-ip"]["resolved"],
            "https://registry.npmjs.org/from-private-ip/-/from-private-ip-2.0.0.tgz"
        );
        assert_eq!(
            updated["dependencies"]["from-test-tld"]["resolved"],
            "https://registry.npmjs.org/from-test-tld/-/from-test-tld-3.0.0.tgz"
        );
        assert_eq!(
            updated["dependencies"]["from-external"]["resolved"],
            "https://example.com/from-external/-/from-external-4.0.0.tgz"
        );
    }

    #[test]
    fn test_rewrite_lockfile_to_public_no_matches() {
        let dir = tempfile::tempdir().unwrap();
        let lockfile = dir.path().join("package-lock.json");

        let package_lock = r#"{
            "dependencies": {
                "a": { "resolved": "https://registry.npmjs.org/a/-/a-1.0.0.tgz" }
            }
        }"#;
        fs::write(&lockfile, package_lock).unwrap();

        let count = rewrite_lockfile_to_public(&lockfile).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_rewrite_lockfile_to_public_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("nope.json");
        let err = rewrite_lockfile_to_public(&missing).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("lockfile not found"), "unexpected: {msg}");
    }

    #[test]
    fn test_is_local_host_trailing_dot_fqdn() {
        for host in ["foo.local.", "foo.test.", "foo.lan.", "localhost."] {
            assert!(is_local_host(host), "expected {host} to be local");
        }
    }

    #[test]
    fn test_rewrite_lockfile_to_public_preserves_query_only_no_path() {
        let dir = tempfile::tempdir().unwrap();
        let lockfile = dir.path().join("package-lock.json");
        let package_lock = r#"{
            "resolved": "http://localhost?token=abc"
        }"#;
        fs::write(&lockfile, package_lock).unwrap();
        let count = rewrite_lockfile_to_public(&lockfile).unwrap();
        assert_eq!(count, 1);
        let updated: Value = serde_json::from_str(&fs::read_to_string(&lockfile).unwrap()).unwrap();
        assert_eq!(updated["resolved"], "https://registry.npmjs.org?token=abc");
    }

    #[test]
    fn test_rewrite_lockfile_to_public_preserves_fragment_only_no_path() {
        let dir = tempfile::tempdir().unwrap();
        let lockfile = dir.path().join("package-lock.json");
        let package_lock = r#"{
            "resolved": "http://localhost#sha"
        }"#;
        fs::write(&lockfile, package_lock).unwrap();
        let count = rewrite_lockfile_to_public(&lockfile).unwrap();
        assert_eq!(count, 1);
        let updated: Value = serde_json::from_str(&fs::read_to_string(&lockfile).unwrap()).unwrap();
        assert_eq!(updated["resolved"], "https://registry.npmjs.org#sha");
    }

    #[test]
    fn test_rewrite_lockfile_to_public_preserves_query_and_fragment_no_path() {
        let dir = tempfile::tempdir().unwrap();
        let lockfile = dir.path().join("package-lock.json");
        let package_lock = r#"{
            "resolved": "http://localhost?q=1#sha"
        }"#;
        fs::write(&lockfile, package_lock).unwrap();
        let count = rewrite_lockfile_to_public(&lockfile).unwrap();
        assert_eq!(count, 1);
        let updated: Value = serde_json::from_str(&fs::read_to_string(&lockfile).unwrap()).unwrap();
        assert_eq!(updated["resolved"], "https://registry.npmjs.org?q=1#sha");
    }

    #[test]
    fn test_rewrite_lockfile_to_public_preserves_query_and_fragment() {
        let dir = tempfile::tempdir().unwrap();
        let lockfile = dir.path().join("package-lock.json");
        let package_lock = r#"{
            "resolved": "http://localhost:4873/pkg/-/pkg-1.0.0.tgz?token=abc#sha"
        }"#;
        fs::write(&lockfile, package_lock).unwrap();
        let count = rewrite_lockfile_to_public(&lockfile).unwrap();
        assert_eq!(count, 1);
        let updated: Value = serde_json::from_str(&fs::read_to_string(&lockfile).unwrap()).unwrap();
        assert_eq!(
            updated["resolved"],
            "https://registry.npmjs.org/pkg/-/pkg-1.0.0.tgz?token=abc#sha"
        );
    }
}
