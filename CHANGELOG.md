# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0]

### Added

- **`--to-public`** flag: rewrite every `resolved` URL whose host looks
  local (`localhost`, `*.test`/`*.local`/`*.lan`, RFC1918 IPv4, `::1`)
  back to `https://registry.npmjs.org`, preserving the rest of the URL.
  No `pkg.config.json` required.
- **`--to-local [URL]`** flag: rewrite every `resolved` URL whose host is
  exactly `registry.npmjs.org` to `<URL>`. When `<URL>` is omitted, reads
  the bare `registry=` line from `./.npmrc` (last-wins semantics; scoped
  overrides and `${VAR}` interpolation intentionally ignored). Rejects
  URLs that embed credentials, use non-`http(s)` schemes, or carry
  query/fragment components.
- **`install-hook`** subcommand: writes a POSIX-sh `pre-commit` hook to
  `.git/hooks/pre-commit` that runs `pkglock --to-public` on staged
  `package-lock.json` files and re-stages the result. Refuses to
  overwrite an existing hook (no `--force`); exits 0 in both the install
  and the already-exists case to remain idempotent. README documents
  manual integration for users with framework-managed hooks.
- New public lib API: `rewrite_lockfile`, `rewrite_lockfile_to_public`,
  `rewrite_lockfile_to_local`, `update_urls_from_config`, `npmrc_registry`,
  `install_pre_commit_hook`, and `InstallHookResult`. All accept explicit
  `&Path` arguments so callers (tests, the hook script, downstream
  tooling) aren't forced to `chdir` into the project root.
- `mise.toml` pins the Rust toolchain so `mise install` provisions the
  right rustc for contributors.
- CI now runs `cargo test` and `cargo clippy --all-targets --all-features
  -- -D warnings` alongside the existing `rustfmt` job. A separate
  `msrv-build` lane pinned to Rust 1.86 verifies the declared MSRV holds.
- `Justfile` wraps the common cargo invocations and adds a from-scratch
  `release VERSION` recipe (bump + CHANGELOG + commit + tag) plus a
  `tag` recipe for the post-PR-merge case where the version commit is
  already on `main`.
- `docs/development.md` documents the daily loop, MSRV verification,
  both release flows, and recovery from a botched publish. README's
  Development section links to it.

### Changed

- Module layout flattened: items previously addressed as
  `pkglock_lib::package_lock_lib::*` now live at the crate root
  (`pkglock_lib::*`).
- `update_urls` recurses into JSON arrays in addition to objects, and
  compiles its URL regex once per invocation instead of at every node.
- File-backed tests now use `tempfile::TempDir` instead of writing into
  the repo root. Eliminates cross-test races under cargo's parallel
  runner.
- Usage line in the CLI now lists all entry points:
  `pkglock <--local | --remote | --to-public | --to-local [URL] | install-hook>`.

### Declared

- **MSRV: Rust 1.86.** Set by the resolved dependency graph
  (`icu_*` 2.2.x → 1.86; `idna_adapter` requires `edition2024` from 1.85).
  Declared via `rust-version` in `Cargo.toml` and verified by the
  `msrv-build` CI lane.

### Dependencies

- Added `url = "2"` (runtime) for URL parsing in the smart-mode flags.
- Added `tempfile = "3.27"` (dev) for race-free file-backed tests.

## [0.2.0]

- Previous release. See git history.
