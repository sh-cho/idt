# Installation

## Using Cargo

```bash
# Install from crates.io
cargo install idt

# Install from git repository
cargo install --git https://github.com/sh-cho/idt
```

### Build from Source

Clone the repository and build:

```bash
git clone https://github.com/sh-cho/idt.git
cd idt
cargo install --path .
```

## Using Homebrew

```bash
brew install sh-cho/tap/idt

# or
brew tap sh-cho/tap
brew install idt
```

## Using Nix (flakes)

```bash
# Try without installing
nix run github:sh-cho/idt -- --help

# Install persistently
nix profile install github:sh-cho/idt
```

## Using Docker

```bash
# From Docker hub
docker run --rm seonghyeon/idt:{version} --help

# From GitHub Container Registry
docker run --rm ghcr.io/sh-cho/idt:{version} --help
```

Docker image is published to the registries below:

- [Docker hub](https://hub.docker.com/r/seonghyeon/idt)
- [GitHub Container Registry](https://github.com/sh-cho/idt/pkgs/container/idt)

## Next Steps

Now that you have idt installed, head to the [Quick Start](./quickstart.md) guide to learn the basics.
