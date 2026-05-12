# pkglock

A small CLI utility that rewrites URLs in `package-lock.json` — swap between a local npm registry (Verdaccio, Nexus, etc.) and the public registry, and keep local URLs from accidentally landing in commits.

## Installation

### Installing with Cargo

```bash
cargo install pkglock
```

This installs the `pkglock` binary into the Cargo bin directory. No Cargo? Get [Rustup](https://rustup.rs/).

## Usage

```
pkglock <--local | --remote | --to-public | --to-local [URL] | install-hook>
```

All commands operate on `./package-lock.json` (the current directory). Run from your project root.

### Smart mode (no config file needed)

#### `pkglock --to-public`

Rewrite every `resolved` URL whose host looks local back to `https://registry.npmjs.org`, preserving the rest of the URL (path, query, fragment). Useful as a "make this lockfile safe to commit" pre-flight.

A host is treated as local if it matches any of:

- `localhost` (case-insensitive)
- Any hostname ending in `.test`, `.local`, or `.lan` (with or without a trailing dot)
- An IPv4 literal in `127.0.0.0/8`, `10.0.0.0/8`, `172.16.0.0/12`, or `192.168.0.0/16`
- The IPv6 loopback literal `::1` (with or without brackets)

#### `pkglock --to-local [URL]`

Rewrite every `resolved` URL whose host is exactly `registry.npmjs.org` to `<URL>`, preserving the rest of the URL. Useful when switching a checkout onto a local mirror.

`<URL>` must be `http://...` or `https://...`, must not embed credentials (use `.npmrc` `_authToken` lines for that), and must not have a query or fragment. A trailing slash is fine and is normalized away.

If `<URL>` is omitted, `pkglock` reads `./.npmrc` and uses the value of the bare `registry=` line. Scoped overrides (`@org:registry=...`) are intentionally ignored. Environment variable interpolation (`${VAR}`) is not performed.

```bash
pkglock --to-local http://localhost:4873
pkglock --to-local https://verdaccio.lan/repo
pkglock --to-local                              # autodetect from ./.npmrc
```

### Pre-commit hook

#### `pkglock install-hook`

Writes a small shell hook to `.git/hooks/pre-commit` that runs `pkglock --to-public` against `package-lock.json` whenever it's staged, then re-stages the rewritten file. The result: you can't accidentally commit local registry URLs.

```bash
pkglock install-hook
```

Hook properties:

- **Local-only.** `.git/hooks/` is not under version control, so installing on a public repo has **zero impact on contributors**. Each developer installs it (or not) per clone.
- **Refuses to overwrite.** If a `pre-commit` hook already exists, the command prints a diagnostic and exits successfully without modifying it. See "Manual integration" below.
- **Idempotent.** Re-running on a repo that's already installed prints the same diagnostic and exits 0.
- **Fails the commit if `pkglock` is missing from `PATH`.** Better to refuse than silently let a bad commit through. Bypass with `git commit --no-verify` if you really need to.

#### A note on staged vs working-tree changes

The hook runs `pkglock --to-public` against the working-tree `package-lock.json`, not the staged version. If you've staged some changes to `package-lock.json` and have **further unstaged changes** on top, those unstaged changes will also be rewritten by the hook and re-staged with `git add`. Review with `git diff --cached` before completing the commit. (This is a v0.3 limitation; a future version may operate on the staged version directly.)

#### Manual integration with an existing pre-commit hook

If `pkglock install-hook` refuses because `.git/hooks/pre-commit` already exists, append (or integrate) the following into your existing hook:

```sh
# pkglock: rewrite local URLs in staged package-lock.json
if git diff --cached --name-only --diff-filter=ACMR | grep -q '^package-lock\.json$'; then
    if ! command -v pkglock >/dev/null 2>&1; then
        echo "pkglock: command not found on PATH — install pkglock or commit with --no-verify" >&2
        exit 1
    fi
    cd "$(git rev-parse --show-toplevel)"
    pkglock --to-public
    git add package-lock.json
    echo "pkglock: rewrote local URLs in package-lock.json before commit"
fi
```

If you use a framework (husky, pre-commit.com), follow its instructions for adding a custom check.

### Config-file mode (`--local` / `--remote`)

For unusual setups (non-standard registry hosts, multi-environment overrides), `pkglock --local` and `pkglock --remote` read URLs from a `pkg.config.json` file next to your lockfile:

```json
{
  "local": "http://localhost:4873",
  "remote": "https://registry.npmjs.org"
}
```

```bash
pkglock --local      # rewrite scheme+host in every resolved URL to config.local
pkglock --remote     # ...to config.remote
```

These flags do an unconditional scheme+authority replacement on every `resolved` URL — they don't check whether the existing host is local or public. Use `--to-public` / `--to-local` for the conditional, no-config-needed flow.

## Development

The project pins Rust 1.86 via `mise.toml`. With [mise](https://mise.jdx.dev) installed, `mise install` provisions the right toolchain automatically; otherwise install Rust 1.86 or newer manually.

Common workflows are wrapped in a [Justfile](https://github.com/casey/just):

```bash
just check          # fmt-check + clippy + test — matches CI exactly
just release 0.4.0  # bump version, update CHANGELOG, commit, tag
just publish        # upload to crates.io
```

Run `just` with no arguments to list every recipe. See [`docs/development.md`](docs/development.md) for the full guide: daily loop, MSRV verification, release flows (both pre-bumped and from-scratch), and recovery from a botched publish.

## Why use pkglock?

`npm` install is slow because every dependency triggers a network round trip to the public registry. A local mirror (e.g. [Verdaccio](https://verdaccio.org)) caches packages and dramatically speeds up cold installs, but switching the lockfile between mirror URLs and public URLs by hand is tedious. `pkglock` makes that switch a one-liner — and the pre-commit hook keeps you from ever shipping a mirror URL by accident.

## Troubleshooting

### Ensuring the Cargo bin directory is in your PATH

To execute `pkglock` from any location, ensure that the Cargo bin directory is on your PATH.

**Unix-like systems (Linux/macOS):**

Add the following line to your shell profile (`.bash_profile`, `.bashrc`, `.zshrc`, etc.):

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Reload the profile:

```bash
source ~/.bash_profile
```

**Windows:**

Open the Start menu, search for "Environment Variables," and choose "Edit the system environment variables." In the System Properties window, click "Environment Variables." In the System Variables section, edit the `Path` variable to include the Cargo bin directory:

```
C:\Users\<YourUsername>\.cargo\bin
```

Click OK to save, and close the remaining windows.
