# Quick Start

This guide covers the essential idt commands to get you productive quickly.

## Generating IDs

Generate IDs with the `gen` command:

```bash
# Generate a random UUID (v4)
idt gen uuid

# Generate a time-sortable UUIDv7
idt gen uuidv7

# Generate a ULID
idt gen ulid

# Generate a NanoID
idt gen nanoid

# Generate a Snowflake ID
idt gen snowflake

# Generate multiple IDs
idt gen uuid -n 10
```

## Inspecting IDs

Analyze any ID with the `inspect` command:

```bash
# Inspect a UUID
idt inspect 550e8400-e29b-41d4-a716-446655440000

# Inspect a ULID
idt inspect 01ARZ3NDEKTSV4RRFFQ69G5FAV

# Auto-detection works for most formats
idt inspect 019c04e5-6118-7b22-95cb-a10e84dad469
```

Example output:

```
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

## Converting Formats

Convert IDs to different encodings:

```bash
# Convert to hexadecimal
idt convert 550e8400-e29b-41d4-a716-446655440000 -f hex

# Convert to Base64
idt convert 550e8400-e29b-41d4-a716-446655440000 -f base64

# Convert to Base58
idt convert 550e8400-e29b-41d4-a716-446655440000 -f base58

# Convert to integer
idt convert 550e8400-e29b-41d4-a716-446655440000 -f int
```

## Validating IDs

Check if a string is a valid ID:

```bash
# Validate any ID
idt validate 550e8400-e29b-41d4-a716-446655440000

# Validate as specific type
idt validate -t uuid 550e8400-e29b-41d4-a716-446655440000

# Strict validation (canonical form only)
idt validate --strict 550e8400-e29b-41d4-a716-446655440000
```

## Comparing IDs

Compare two IDs to understand their relationship:

```bash
idt compare 019c04e5-6118-7b22-95cb-a10e84dad469 019c04e5-6119-7000-8000-000000000000
```

This shows binary, lexicographic, and chronological comparisons.

## Using JSON Output

Get machine-readable JSON output for any command:

```bash
# JSON output
idt gen uuid --json

# Pretty-printed JSON
idt inspect 550e8400-e29b-41d4-a716-446655440000 --json --pretty
```

## Piping and Scripting

idt works great with Unix pipes:

```bash
# Generate and immediately inspect
idt gen uuid | idt inspect

# Validate multiple IDs
idt gen ulid -n 100 | idt validate

# Convert piped input
echo "550e8400-e29b-41d4-a716-446655440000" | idt convert -f base64
```

## Getting Help

View available commands and options:

```bash
# General help
idt --help

# Help for a specific command
idt gen --help
idt inspect --help
```

## Next Steps

- Learn about all [Commands](./commands/README.md)
- Explore [ID Types](./id-types/README.md) in detail
- See more [Examples](./examples/README.md)
