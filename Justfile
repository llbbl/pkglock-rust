# pkglock — local development tasks
# Run `just` (or `just --list`) to see everything.
# Detailed docs: docs/development.md

default:
    @just --list

# Build the debug binary
build:
    cargo build --all-targets --all-features

# Build the optimized release binary
build-release:
    cargo build --release

# Run all tests
test:
    cargo test --all-targets --all-features

# Clippy with -D warnings (matches CI)
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Apply rustfmt
fmt:
    cargo fmt

# Check formatting without modifying (matches CI)
fmt-check:
    cargo fmt -- --check

# Full preflight: fmt-check + lint + test. Run before pushing.
check: fmt-check lint test

# Build against the declared MSRV (requires `rustup toolchain install 1.86`)
msrv-build:
    cargo +1.86 build --all-targets --all-features

# Run the binary with arguments forwarded. Example: just run --to-public
run *ARGS:
    cargo run -- {{ARGS}}

# Install the binary locally from current source
install-local:
    cargo install --path . --force

# Remove build artifacts
clean:
    cargo clean

# Tag the version currently in Cargo.toml. Use when the version commit is
# already on main and you just need to mark the release point.
tag:
    #!/usr/bin/env bash
    set -euo pipefail
    version=$(grep -E '^version = ' Cargo.toml | head -1 | sed -E 's/version = "(.+)"/\1/')
    if [ -z "$version" ]; then
        echo "Error: could not parse version from Cargo.toml" >&2
        exit 1
    fi
    if git rev-parse --verify "v$version" >/dev/null 2>&1; then
        echo "Error: tag v$version already exists" >&2
        exit 1
    fi
    git tag -a "v$version" -m "Release $version"
    echo "Created tag v$version. Push it with: git push origin v$version"

# Cut a release from scratch: bump version, update CHANGELOG, commit, tag locally.
# Usage: just release 0.4.0
# After this completes:
#   just publish-dry   # verify the crate packages cleanly
#   just publish       # upload to crates.io
#   git push origin main --tags
release VERSION:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! git diff-index --quiet HEAD --; then
        echo "Error: working tree has uncommitted changes" >&2
        exit 1
    fi
    if git rev-parse --verify "v{{VERSION}}" >/dev/null 2>&1; then
        echo "Error: tag v{{VERSION}} already exists" >&2
        exit 1
    fi
    sed -i.bak -E 's/^version = ".*"$/version = "{{VERSION}}"/' Cargo.toml
    rm Cargo.toml.bak
    cargo build --quiet
    today=$(date +%Y-%m-%d)
    awk -v ver="{{VERSION}}" -v date="$today" '
        /^## \[Unreleased\]$/ {
            print
            print ""
            print "## [" ver "] - " date
            next
        }
        { print }
    ' CHANGELOG.md > CHANGELOG.md.tmp
    mv CHANGELOG.md.tmp CHANGELOG.md
    git add Cargo.toml Cargo.lock CHANGELOG.md
    git commit -m "Release {{VERSION}}"
    git tag -a "v{{VERSION}}" -m "Release {{VERSION}}"
    echo
    echo "Release commit and tag v{{VERSION}} created locally."
    echo "Next: just publish-dry; just publish; git push origin main --tags"

# Dry-run a crates.io publish (verifies packaging without uploading)
publish-dry:
    cargo publish --dry-run

# Publish to crates.io (requires `cargo login` first)
publish:
    cargo publish
