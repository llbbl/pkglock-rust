use std::env;

// Import the update_urls_in_package_lock function from the package_lock_lib module
use pkglock_lib::package_lock_lib::update_urls_in_package_lock;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 || (args[1] != "--local" && args[1] != "--remote") {
        println!("Usage: program --local | --remote");
        return Ok(());
    }

    // Delegate processing to the lib module function
    update_urls_in_package_lock(&args[1])?;

    println!("URLs in package-lock.json have been updated.");
    Ok(())
}
