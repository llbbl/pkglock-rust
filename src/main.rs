use std::env;
use std::path::Path;
use std::process;

// Import the helpers from the pkglock_lib crate
use pkglock_lib::{
    npmrc_registry, rewrite_lockfile_to_local, rewrite_lockfile_to_public,
    update_urls_in_package_lock,
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
        _ => usage_and_exit(),
    }

    Ok(())
}

fn usage_and_exit() -> ! {
    eprintln!("Usage: pkglock --local|--remote|--to-public|--to-local [URL]");
    process::exit(2);
}
