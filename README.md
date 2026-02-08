# idt

A fast, ergonomic CLI tool for working with various ID formats.

## Installation

```bash
cargo install idt
```

For other installation option, see [here](https://sh-cho.github.io/idt/installation.html)

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

# Compare two IDs
idt compare <ID1> <ID2>

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
idt gen snowflake --epoch 1420070400000    # Discord epoch
idt gen snowflake --machine-id 1
idt gen snowflake --datacenter-id 1
```

## Output Formats

```bash
# JSON output
idt gen uuid --json
idt inspect <ID> --json --pretty

# Encoding formats
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

  Time       2026-01-28T13:57:47.416Z
  Version    7
  Variant    RFC4122
  Random     62 bits

  Hex        019c04e561187b2295cba10e84dad469
  Base64     AZwE5WEYeyKVy6EOhNrUaQ==
  Int        2139325608653621017571381452845274217
```

Inspect ULID:
```bash
$ idt inspect 01ARZ3NDEKTSV4RRFFQ69G5FAV
ULID
  01ARZ3NDEKTSV4RRFFQ69G5FAV

  Time       2016-07-30T23:54:10.259Z
  Random     80 bits

  Hex        01563e3ab5d3d6764c61efb99302bd5b
  Base64     AVY+OrXT1nZMYe+5kwK9Ww==
  Int        1777027686520646174104517696511196507
```

Pipe support:
```bash
idt gen uuid | idt inspect
idt gen ulid -n 100 | idt validate
echo "550e8400-e29b-41d4-a716-446655440000" | idt convert -f base64
```

## Options

| Flag | Description |
|------|-------------|
| `-n, --count` | Number of IDs to generate |
| `-f, --format` | Output encoding format |
| `-t, --type` | ID type hint |
| `-j, --json` | JSON output |
| `-p, --pretty` | Pretty print JSON |
| `--no-color` | Disable colors |

## License

MIT
