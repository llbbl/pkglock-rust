# AGENTS.md

This file provides context for AI agents working on this codebase.

## Project Overview

**pkglock-rust** is a Rust CLI utility that switches between local and remote npm registries in `package-lock.json` files. It replaces resolved URLs to point to either a local registry (like Verdaccio) or the public npm registry.

## Project Structure

```
src/
├── main.rs          # CLI entry point, argument parsing
└── lib.rs           # Core library with URL update logic
tests/
└── integration_test.rs
docs/
└── maintenance-suggestions.md
```

## Key Components

- **main.rs**: CLI interface, validates `--local` or `--remote` args, exits with code 2 on invalid input
- **lib.rs** (`package_lock_lib` module):
  - `update_urls()` - Recursively traverses JSON, replaces registry URLs via regex
  - `update_urls_in_package_lock()` - Main entry point that reads config, updates lock file, writes output

## Configuration

Requires `pkg.config.json` in the same directory as `package-lock.json`:

```json
{
  "local": "http://localhost:4873",
  "remote": "https://registry.npmjs.org"
}
```

## Development Commands

```bash
# Build
cargo build

# Run
cargo run -- --local
cargo run -- --remote

# Test (unit + integration)
cargo test

# Format check
cargo fmt -- --check

# Clippy lints
cargo clippy
```

## Dependencies

- `serde` / `serde_json` - JSON serialization
- `regex` - URL pattern matching

## CI

GitHub Actions runs `cargo fmt -- --check` on push/PR to main.

## Code Conventions

- Rust 2021 edition
- Exit code 2 for invalid CLI arguments
- Unit tests in `lib.rs`, integration tests in `tests/`

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
