# idt

[![Crates.io Version](https://img.shields.io/crates/v/idt)](https://crates.io/crates/idt)
[![Crates.io MSRV](https://img.shields.io/crates/msrv/idt)](Cargo.toml)
[![built with nix](https://img.shields.io/badge/Built_with_Nix-7EBAE4?style=flat&logo=nixos&logoColor=white&labelColor=5277C3)](https://builtwithnix.org)
[![slsa level 3](https://slsa.dev/images/gh-badge-level3.svg)](https://slsa.dev)
[![OpenSSF Scorecard](https://api.scorecard.dev/projects/github.com/sh-cho/idt/badge)](https://scorecard.dev/viewer/?uri=github.com/sh-cho/idt)
[![OpenSSF Best Practices](https://www.bestpractices.dev/projects/12174/badge)](https://www.bestpractices.dev/projects/12174)
[![codecov](https://codecov.io/gh/sh-cho/idt/graph/badge.svg?token=NY79FLDDIE)](https://app.codecov.io/gh/sh-cho/idt)

[![Crates.io Size](https://img.shields.io/crates/size/idt?logo=rust&label=crate%20size)](https://crates.io/crates/idt)
[![Docker Image Size](https://img.shields.io/docker/image-size/seonghyeon/idt?logo=docker&logoColor=white)](https://hub.docker.com/r/seonghyeon/idt)

idt(id tool) — A fast, ergonomic CLI tool for working with various ID formats.

![demo](assets/demo.gif)

## Installation

```bash
# Cargo
cargo install idt

# Homebrew
brew install sh-cho/tap/idt

# Nix (flakes)
nix run github:sh-cho/idt -- --help
```

### Docker

```bash
# From Docker hub
docker run --rm seonghyeon/idt:{version} --help

# From GitHub Container Registry
docker run --rm ghcr.io/sh-cho/idt:{version} --help
```

Docker image is published to the registries below:

- [Docker hub](https://hub.docker.com/r/seonghyeon/idt)
- [GitHub Container Registry](https://github.com/sh-cho/idt/pkgs/container/idt)

See [installation docs](https://sh-cho.github.io/idt/installation.html) for more.

## Usage

```bash
# Generate IDs
idt gen uuid                      # UUIDv4 (default)
idt gen uuidv7                    # UUIDv7 (time-sortable)
idt gen ulid                      # ULID
idt gen nanoid                    # NanoID
idt gen snowflake                 # Snowflake ID

# Generate multiple IDs
idt gen uuid -n 10

# Inspect any ID
idt inspect 550e8400-e29b-41d4-a716-446655440000
idt inspect 01ARZ3NDEKTSV4RRFFQ69G5FAV

# Convert formats
idt convert <ID> -f hex
idt convert <ID> -f base64
idt convert <ID> -f base58

# Validate IDs
idt validate <ID>
idt validate -t uuid <ID>
idt validate -t isbn13 978-0-306-40615-7
idt validate -t isin US0378331005

# Compare two IDs
idt compare <ID1> <ID2>

# Sort IDs by timestamp
idt sort <ID>...
idt gen ulid -n 5 | idt sort --show-time

# Show supported types
idt info
idt info uuidv7
```

## Supported ID Types

| Type | Sortable | Time | Bits | Description |
|------|----------|------|------|-------------|
| uuidv1 | No | Yes | 128 | Timestamp + MAC address |
| uuidv3 | No | No | 128 | MD5 namespace hash |
| uuidv4 | No | No | 128 | Random |
| uuidv5 | No | No | 128 | SHA-1 namespace hash |
| uuidv6 | Yes | Yes | 128 | Reordered timestamp |
| uuidv7 | Yes | Yes | 128 | Unix timestamp + random |
| uuid-nil | - | - | 128 | All zeros |
| uuid-max | - | - | 128 | All ones |
| ulid | Yes | Yes | 128 | Crockford Base32, lexicographically sortable |
| nanoid | No | No | ~126 | Compact URL-friendly ID |
| ksuid | Yes | Yes | 160 | K-Sortable Unique Identifier |
| snowflake | Yes | Yes | 64 | Twitter/Discord-style distributed ID |
| objectid | Partial | Yes | 96 | MongoDB ObjectId |
| typeid | Yes | Yes | 128 | Type-prefixed sortable ID |
| xid | Yes | Yes | 96 | Globally unique sortable ID |
| cuid | Partial | Yes | 128 | Collision-resistant ID |
| cuid2 | No | No | 128 | Secure collision-resistant ID |
| tsid | Yes | Yes | 64 | Time-sorted unique identifier |

### Assigned IDs (validate & inspect only)

| Type | Description | Check Digit |
|------|-------------|-------------|
| ean13 | EAN-13 (International Article Number) | Mod 10 |
| isbn13 | ISBN-13 (International Standard Book Number) | Mod 10 |
| isbn10 | ISBN-10 (legacy book identifier) | Mod 11 |
| isin | ISIN (International Securities Identification Number) | Luhn |
| ean8 | EAN-8 (8-digit barcode for small items) | Mod 10 |
| upca | UPC-A (Universal Product Code) | Mod 10 |
| issn | ISSN (International Standard Serial Number) | Mod 11 |
| ismn | ISMN (International Standard Music Number) | Mod 10 |
| isni | ISNI (International Standard Name Identifier) | ISO 7064 MOD 11-2 |
| gtin14 | GTIN-14 (Global Trade Item Number) | Mod 10 |
| asin | ASIN (Amazon Standard Identification Number) | None (format only) |

These IDs are assigned by external registries and standards bodies — idt supports validation, inspection, and auto-detection, but not generation.

## Generation Options

```bash
# UUID versions
idt gen uuid                      # v4 (default)
idt gen uuid --uuid-version 7     # v7
idt gen uuidv1                    # v1
idt gen uuidv6                    # v6
idt gen uuidv7                    # v7

# NanoID customization
idt gen nanoid --length 32
idt gen nanoid --alphabet "0123456789abcdef"

# Snowflake customization
idt gen snowflake --preset twitter        # Twitter layout + epoch
idt gen snowflake --preset discord        # Discord layout + epoch
idt gen snowflake --preset instagram --field shard_id=42
idt gen snowflake --preset sonyflake      # 10ms resolution
idt gen snowflake --preset mastodon
idt gen snowflake --epoch 1420070400000   # Custom epoch (backward compat)
idt gen snowflake --machine-id 1 --datacenter-id 1
```

## Output Formats

```bash
# Structured output (JSON, YAML, TOML)
idt gen uuid --output json
idt gen uuid --output yaml
idt gen uuid --output toml
idt inspect <ID> --output json --pretty

# Shorthand for JSON
idt gen uuid -j
idt inspect <ID> --json --pretty

# Encoding formats (for convert/gen)
idt convert <ID> -f hex           # Hexadecimal
idt convert <ID> -f base32        # Base32
idt convert <ID> -f base58        # Base58
idt convert <ID> -f base64        # Base64
idt convert <ID> -f base64url     # URL-safe Base64
idt convert <ID> -f bits          # Binary string
idt convert <ID> -f int           # Integer
idt convert <ID> -f bytes         # Space-separated hex bytes
```

## Examples

Generate and inspect UUIDv7:
```bash
$ idt gen uuidv7
019c04e5-6118-7b22-95cb-a10e84dad469

$ idt inspect 019c04e5-6118-7b22-95cb-a10e84dad469
UUIDV7
  019c04e5-6118-7b22-95cb-a10e84dad469

  Time (UTC)          2026-01-28T13:57:47.416Z
  Local Time (+09:00) 2026-01-28T22:57:47.416+09:00
  Version             7
  Variant             RFC4122
  Random              62 bits

  Hex                 019c04e561187b2295cba10e84dad469
  Base64              AZwE5WEYeyKVy6EOhNrUaQ==
  Int                 2139325608653621017571381452845274217
```

Inspect ULID:
```bash
$ idt inspect 01ARZ3NDEKTSV4RRFFQ69G5FAV
ULID
  01ARZ3NDEKTSV4RRFFQ69G5FAV

  Time (UTC)          2016-07-30T23:54:10.259Z
  Local Time (+09:00) 2016-07-31T08:54:10.259+09:00
  Random              80 bits

  Hex                 01563e3ab5d3d6764c61efb99302bd5b
  Base64              AVY+OrXT1nZMYe+5kwK9Ww==
  Int                 1777027686520646174104517696511196507
```

Sort IDs by timestamp:
```bash
$ idt sort --show-time 01ARZ3NDEKTSV4RRFFQ69G5FAV 01KK3ZE8GEVTC9PGC0NTY1RY03
2016-07-30T23:54:10.259Z  01ARZ3NDEKTSV4RRFFQ69G5FAV
2026-03-07T11:03:08.046Z  01KK3ZE8GEVTC9PGC0NTY1RY03
```

Pipe support:
```bash
idt gen uuid | idt inspect
idt gen ulid -n 100 | idt validate
idt gen ulid -n 5 | idt sort --reverse
echo "550e8400-e29b-41d4-a716-446655440000" | idt convert -f base64
```

## Options

| Flag | Description |
|------|-------------|
| `-n, --count` | Number of IDs to generate |
| `-f, --format` | Output encoding format |
| `-r, --reverse` | Sort in descending order |
| `--show-time` | Show timestamps alongside IDs |
| `--preset` | Snowflake preset (`twitter`, `discord`, `instagram`, `sonyflake`, `mastodon`) |
| `--field` | Set a Snowflake field value (e.g., `--field shard_id=42`) |
| `-T, --template` | Wrap each ID in a format string (`{}` = placeholder) |
| `-t, --type` | ID type hint |
| `-j, --json` | JSON output (shorthand for `--output json`) |
| `-o, --output` | Output format (`json`, `yaml`, `toml`) |
| `-p, --pretty` | Pretty print JSON |
| `--no-color` | Disable colors |

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md)

## Security

See [SECURITY.md](./SECURITY.md)

## License

Licensed under either of [Apache License Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
