# Installation

## From Source (Cargo)

The recommended way to install idt is using Cargo, Rust's package manager.

### Prerequisites

You need Rust installed on your system. If you don't have it, install it via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install from crates.io

```bash
cargo install idt
```

### Install from Git Repository

To install the latest development version:

```bash
cargo install --git https://github.com/sh-cho/idt
```

### Build from Source

Clone the repository and build:

```bash
git clone https://github.com/sh-cho/idt.git
cd idt
cargo install --path .
```

## Verify Installation

After installation, verify idt is working:

```bash
idt --version
```

You should see the version number. Try generating your first ID:

```bash
idt gen uuid
```

## Shell Completion

idt supports shell completion for bash, zsh, fish, and PowerShell. Generate completion scripts using your shell's completion mechanism with clap's built-in support.

## Updating

To update idt to the latest version:

```bash
cargo install idt --force
```

## Uninstalling

To remove idt:

```bash
cargo uninstall idt
```

## Next Steps

Now that you have idt installed, head to the [Quick Start](./quickstart.md) guide to learn the basics.
