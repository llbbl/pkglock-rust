use std::env;
use std::path::Path;
use std::process;

// Import the helpers from the pkglock_lib crate
use pkglock_lib::{
    install_pre_commit_hook, npmrc_registry, rewrite_lockfile_to_local, rewrite_lockfile_to_public,
    update_urls_in_package_lock, InstallHookResult,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        usage_and_exit();
    }

    match args[1].as_str() {
        "--local" | "--remote" => {
            if args.len() != 2 {
                usage_and_exit();
            }
            update_urls_in_package_lock(&args[1])?;
            println!("URLs in package-lock.json have been updated.");
        }
        "--to-public" => {
            if args.len() != 2 {
                usage_and_exit();
            }
            let count = rewrite_lockfile_to_public(Path::new("package-lock.json"))?;
            println!("pkglock: rewrote {count} resolved URLs (--to-public)");
        }
        "--to-local" => {
            if args.len() > 3 {
                usage_and_exit();
            }
            let effective_url: String = match args.get(2) {
                Some(url) => url.clone(),
                None => match npmrc_registry(Path::new(".npmrc")) {
                    Some(url) => url,
                    None => {
                        eprintln!(
                            "pkglock: --to-local requires a registry URL: pass it as an argument \
                             or add `registry=<url>` to ./.npmrc"
                        );
                        process::exit(2);
                    }
                },
            };
            let count = rewrite_lockfile_to_local(Path::new("package-lock.json"), &effective_url)?;
            println!("pkglock: rewrote {count} resolved URLs (--to-local {effective_url})");
        }
        "install-hook" => {
            if args.len() != 2 {
                usage_and_exit();
            }
            match install_pre_commit_hook(Path::new("."))? {
                InstallHookResult::Installed => {
                    println!("pkglock: installed pre-commit hook at .git/hooks/pre-commit");
                }
                InstallHookResult::AlreadyExists => {
                    eprintln!(
                        "pkglock: existing pre-commit hook detected at .git/hooks/pre-commit \
                         — not modifying. See README for manual integration."
                    );
                    // Exit 0: install-hook is idempotent. Running it twice in
                    // a row should produce a consistent exit code; the stderr
                    // message above is the audit trail.
                }
            }
        }
        _ => usage_and_exit(),
    }

    Ok(())
}

fn usage_and_exit() -> ! {
    eprintln!(
        "Usage: pkglock <--local | --remote | --to-public | --to-local [URL] | install-hook>"
    );
    process::exit(2);
}
