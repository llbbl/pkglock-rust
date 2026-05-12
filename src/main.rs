use std::env;
use std::path::Path;
use std::process;

// Import the helpers from the pkglock_lib crate
use pkglock_lib::{rewrite_lockfile_to_public, update_urls_in_package_lock};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        usage_and_exit();
    }

    match args[1].as_str() {
        "--local" | "--remote" => {
            update_urls_in_package_lock(&args[1])?;
            println!("URLs in package-lock.json have been updated.");
        }
        "--to-public" => {
            let count = rewrite_lockfile_to_public(Path::new("package-lock.json"))?;
            println!("pkglock: rewrote {count} resolved URLs (--to-public)");
        }
        _ => usage_and_exit(),
    }

    Ok(())
}

fn usage_and_exit() -> ! {
    eprintln!("Usage: pkglock --local|--remote|--to-public");
    process::exit(2);
}
