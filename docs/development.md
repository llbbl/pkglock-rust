# Development

This document covers local development, the test/CI loop, and how to cut a release.

## Prerequisites

- **Rust 1.86 or newer.** The MSRV is declared in `Cargo.toml`.
  - With [mise](https://mise.jdx.dev): `mise install` provisions the right toolchain automatically from `mise.toml`.
  - Without mise: `rustup install 1.86` (or any newer stable).
- **[just](https://github.com/casey/just)** for the recipes below (optional — every recipe is a thin wrapper over a `cargo` command).
- A crates.io API token in `~/.cargo/credentials.toml` (only needed to publish).

## Daily loop

```bash
just check        # fmt-check + clippy -D warnings + test (matches CI exactly)
just test         # tests only
just lint         # clippy only
just fmt          # apply rustfmt
just run -- --to-public   # run the binary with args forwarded
```

Run `just` (no args) to list every recipe.

### What `just check` actually runs

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

These are the same three commands CI runs on every PR. If `just check` passes locally, CI will pass (modulo the MSRV lane — see below).

## MSRV verification

CI runs a `msrv-build` lane pinned to Rust 1.86 that does `cargo build --all-targets --all-features`. To reproduce locally:

```bash
rustup toolchain install 1.86      # one-time
just msrv-build
```

The MSRV is build-only because dev-dependencies (e.g. `tempfile`) sometimes require newer rustc than the library contract demands. The contract for downstream consumers is "you can build pkglock with rustc 1.86" — `cargo build` is what verifies that.

## Cutting a release

Two flows depending on whether the version is already bumped.

### A. Version already committed (e.g. v0.3.0 — the version commit landed via PR)

After the PR merges to `main`:

```bash
git checkout main && git pull
just tag                    # creates `v<current Cargo.toml version>` tag locally
git push origin v0.3.0      # publish the tag

just publish-dry            # verify the crate packages cleanly
just publish                # upload to crates.io
```

### B. Bumping the version locally first (e.g. v0.4.0 onward)

`just release` handles the full bump+changelog+commit+tag flow:

```bash
just release 0.4.0
```

Behavior:
1. Refuses if the working tree is dirty.
2. Refuses if `v0.4.0` tag already exists.
3. Rewrites `version = "..."` in `Cargo.toml`.
4. Refreshes `Cargo.lock`.
5. Renames the `[Unreleased]` heading in `CHANGELOG.md` to `[0.4.0] - YYYY-MM-DD` and inserts a fresh empty `[Unreleased]` above it.
6. Commits with message `Release 0.4.0`.
7. Creates an annotated tag `v0.4.0`.

After that:

```bash
just publish-dry
just publish
git push origin main --tags
```

You're free to inspect the commit before pushing/publishing. Nothing leaves your machine until you run `just publish` and `git push`.

### Recovering from a botched release

If `just release` succeeded but `cargo publish` rejects the upload (e.g. a name collision, a packaging issue caught by the registry):

```bash
# Roll back the local commit + tag, fix the underlying issue, then re-run.
git tag -d v0.4.0
git reset --hard HEAD^
```

Don't do this if the tag has already been pushed — recall a published version via crates.io's [`cargo yank`](https://doc.rust-lang.org/cargo/commands/cargo-yank.html) instead.

## Full recipe reference

| Recipe | What it does |
|---|---|
| `just` / `just default` | List all recipes |
| `just build` | `cargo build --all-targets --all-features` |
| `just build-release` | `cargo build --release` |
| `just test` | `cargo test --all-targets --all-features` |
| `just lint` | `cargo clippy --all-targets --all-features -- -D warnings` |
| `just fmt` | `cargo fmt` |
| `just fmt-check` | `cargo fmt -- --check` |
| `just check` | Preflight: `fmt-check` + `lint` + `test` |
| `just msrv-build` | Build against Rust 1.86 (requires `rustup toolchain install 1.86`) |
| `just run [ARGS...]` | `cargo run -- [ARGS...]` |
| `just install-local` | `cargo install --path . --force` |
| `just clean` | `cargo clean` |
| `just tag` | Tag the version currently in `Cargo.toml` |
| `just release VERSION` | Bump + changelog + commit + tag (from-scratch flow) |
| `just publish-dry` | `cargo publish --dry-run` |
| `just publish` | `cargo publish` |

## Notes on the hook (when developing the hook itself)

`pkglock install-hook` writes a POSIX-sh script to `.git/hooks/pre-commit`. Local testing tips:

- Use `sh -n .git/hooks/pre-commit` to syntax-check without running.
- The hook calls `pkglock --to-public` via `command -v pkglock` — install your local build with `just install-local` if you want the hook to pick up your in-progress changes.
- Tests in `src/lib.rs` cover the install logic (fresh repo, pre-existing hook, symlinked `.git`, etc.) and a `sh -n` syntax check on the generated script. They run as part of `just test`.
