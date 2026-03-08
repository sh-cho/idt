# Contributing to idt

Thank you for your interest in contributing! Please open an issue or a PR if you have any suggestions.

## Prerequisites

- Rust (see `rust-version` in [Cargo.toml](Cargo.toml) for MSRV)
- [cargo-release](https://github.com/crate-ci/cargo-release) (for maintainers)

## Development

```bash
# Build
cargo build

# Run
cargo run -- gen uuid

# Test
cargo test

# Format
cargo fmt

# Lint
cargo clippy -- -D warnings
```

Please run `cargo fmt` and `cargo clippy` before submitting a PR.

## Generating shell completions and man pages

```bash
# Shell completions
cargo run -- completions bash
cargo run -- completions zsh
cargo run -- completions fish

# Man pages (stdout)
cargo run -- manpage

# Man pages (write to directory)
cargo run -- manpage /tmp/idt-man
man /tmp/idt-man/idt.1
```

## Release process

We use [cargo-release](https://github.com/crate-ci/cargo-release) to automate version bumps, and [GoReleaser](https://goreleaser.com/) (triggered by git tag push) to build binaries and publish the Homebrew formula.

### Setup

```bash
cargo install cargo-release
```

### Creating a release

```bash
# Dry run first (default behavior)
cargo release patch   # 0.1.8 → 0.1.9
cargo release minor   # 0.1.8 → 0.2.0
cargo release major   # 0.1.8 → 1.0.0

# Actually execute the release
cargo release patch --execute
```

`cargo release` will:

1. Bump the version in `Cargo.toml`
2. Update `Cargo.lock`
3. Create a git commit for the version bump
4. Create a git tag
5. Push the commit and tag to the remote

Publishing to crates.io is handled by CI, so `cargo-release` should be configured with `publish = false` (see configuration below).

Once the tag is pushed, GitHub Actions triggers GoReleaser and crates.io publish, which:

- Builds binaries for all target platforms
- Creates a GitHub release with the binaries
- Updates the Homebrew formula in [sh-cho/homebrew-tap](https://github.com/sh-cho/homebrew-tap)
- etc..

See more in [release.yml](.github/workflows/release.yml).
