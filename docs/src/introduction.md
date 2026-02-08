# Introduction

**idt** (ID Tool) is a fast, ergonomic CLI tool for working with various identifier formats. Whether you need to generate, inspect, convert, or validate IDs, idt provides a unified interface for all your identifier needs.

## Key Features

- **Multi-format Support**: Work with UUID (all versions), ULID, NanoID, Snowflake, and more
- **Generate IDs**: Create new identifiers with customizable options
- **Inspect IDs**: Decode and analyze any supported ID format
- **Convert Formats**: Transform IDs between different encodings (hex, base64, base58, etc.)
- **Validate IDs**: Check if strings are valid identifiers
- **Compare IDs**: Analyze relationships between IDs (chronological, binary, lexicographic)
- **Pipe-friendly**: Designed for shell scripting and Unix pipelines
- **JSON Output**: Machine-readable output for integration with other tools

## Quick Example

```bash
# Generate a UUIDv7 (time-sortable)
$ idt gen uuidv7
019c04e5-6118-7b22-95cb-a10e84dad469

# Inspect the generated ID
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

# Convert to different formats
$ idt convert 019c04e5-6118-7b22-95cb-a10e84dad469 -f base64
AZwE5WEYeyKVy6EOhNrUaQ==
```

## Why idt?

Working with different ID formats often requires multiple tools or libraries. idt consolidates these into a single, fast CLI tool that:

1. **Auto-detects ID types** - No need to specify the format when inspecting or converting
2. **Provides rich metadata** - Extract timestamps, version info, and other embedded data
3. **Supports modern formats** - UUIDv7, ULID, and other time-sortable IDs
4. **Integrates with your workflow** - JSON output, stdin support, and Unix-friendly design

## Supported ID Types

| Type | Sortable | Timestamp | Bits | Description |
|------|----------|-----------|------|-------------|
| UUIDv1 | No | Yes | 128 | Timestamp + MAC address |
| UUIDv4 | No | No | 128 | Random |
| UUIDv6 | Yes | Yes | 128 | Reordered timestamp |
| UUIDv7 | Yes | Yes | 128 | Unix timestamp + random |
| ULID | Yes | Yes | 128 | Crockford Base32, lexicographically sortable |
| NanoID | No | No | ~126 | Compact URL-friendly ID |
| Snowflake | Yes | Yes | 64 | Twitter/Discord-style distributed ID |

See the [ID Types Overview](./id-types/README.md) for complete details on all supported formats.

## Getting Started

Ready to start using idt? Head to the [Installation](./installation.md) guide to get set up, then check out the [Quick Start](./quickstart.md) for common usage patterns.
