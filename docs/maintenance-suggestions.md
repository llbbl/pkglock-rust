# Maintenance suggestions for `pkglock`

This document captures optional improvements you can consider later. The project builds and tests cleanly today; these are mostly “future-proofing” and quality-of-life updates.

## Quick wins

- **Enforce formatting in CI**
  - `cargo fmt -- --check` currently fails due to formatting drift.
  - Add a CI job that runs `cargo fmt -- --check` so formatting stays consistent.

- **Improve CLI UX + exit codes**
  - Print `Usage: pkglock --local|--remote` (instead of `program ...`).
  - Return a non-zero exit code on invalid args (so scripts/CI can detect misuse).

## Code quality / robustness

- **Compile regex once**
  - `update_urls()` currently compiles the regex repeatedly; compile once and reuse (pass `&Regex` through recursion or use a static).

- **Handle arrays during recursion**
  - The URL walk recurses through JSON objects; consider also recursing through `Value::Array` in case future lockfiles include arrays of objects.

- **Simplify module naming**
  - `pkglock_lib::package_lock_lib::...` is redundant. Consider renaming to `package_lock` or exporting the functions at crate root.

## Tests

- **Avoid writing test files into the repo root**
  - Tests currently create `pkg.config.json` / `package-lock.json` in the working directory.
  - Consider using a temp dir (e.g. `tempfile`) and either:
    - run the code with `set_current_dir(temp_dir)`, or
    - change the API to accept paths.

## Automation to “keep it maintained”

- **CI (GitHub Actions)**
  - `cargo test`
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`

- **Dependency/security hygiene**
  - Enable Dependabot or Renovate for Rust.
  - Consider adding `cargo audit` (RustSec advisories).

- **Define MSRV**
  - Add `rust-version = "…"` in `Cargo.toml` and test it in CI (locks expectations for users).

- **Release hygiene**
  - Add `CHANGELOG.md`.
  - Optional: automate releases/publishing with `release-plz` or `cargo-dist`.

