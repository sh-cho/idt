# gen - Generate IDs

Generate new identifiers of various types.

## Usage

```bash
idt gen <TYPE> [OPTIONS]
```

## Arguments

| Argument | Description |
|----------|-------------|
| `TYPE` | ID type to generate (uuid, uuidv7, ulid, nanoid, snowflake, etc.) |

## Options

| Option | Description |
|--------|-------------|
| `-n, --count <N>` | Number of IDs to generate (default: 1) |
| `-f, --format <FORMAT>` | Output encoding format |
| `-o, --output <FILE>` | Write output to file |
| `--no-newline` | Don't print trailing newline (single ID only) |

### UUID Options

| Option | Description |
|--------|-------------|
| `--uuid-version <V>` | UUID version (1, 4, 6, 7) |
| `--namespace <NS>` | Namespace for UUID v3/v5 (dns, url, oid, x500, or UUID) |
| `--name <NAME>` | Name for UUID v3/v5 |

### NanoID Options

| Option | Description |
|--------|-------------|
| `--alphabet <CHARS>` | Custom alphabet |
| `--length <N>` | Custom length (default: 21) |

### Snowflake Options

| Option | Description |
|--------|-------------|
| `--epoch <MS>` | Custom epoch in milliseconds (or "twitter"/"discord") |
| `--machine-id <N>` | Machine/worker ID (0-31) |
| `--datacenter-id <N>` | Datacenter ID (0-31) |

### TypeID Options

| Option | Description |
|--------|-------------|
| `--prefix <PREFIX>` | Type prefix for TypeID |

## Supported Types

| Type | Alias | Description |
|------|-------|-------------|
| `uuid` | - | UUIDv4 (random) by default |
| `uuidv1` | - | UUIDv1 (timestamp + MAC) |
| `uuidv4` | - | UUIDv4 (random) |
| `uuidv6` | - | UUIDv6 (reordered timestamp) |
| `uuidv7` | - | UUIDv7 (Unix timestamp + random) |
| `uuid-nil` | - | Nil UUID (all zeros) |
| `uuid-max` | - | Max UUID (all ones) |
| `ulid` | - | ULID |
| `nanoid` | - | NanoID |
| `snowflake` | - | Snowflake ID |

## Examples

### Basic Generation

```bash
# Generate a random UUID (v4)
idt gen uuid

# Generate UUIDv7 (time-sortable)
idt gen uuidv7

# Generate ULID
idt gen ulid

# Generate NanoID
idt gen nanoid

# Generate Snowflake ID
idt gen snowflake
```

### Multiple IDs

```bash
# Generate 10 UUIDs
idt gen uuid -n 10

# Generate 100 ULIDs
idt gen ulid -n 100
```

### UUID Versions

```bash
# UUIDv1 (timestamp-based)
idt gen uuidv1

# UUIDv6 (reordered timestamp)
idt gen uuidv6

# UUIDv7 (Unix timestamp)
idt gen uuidv7

# Or use --uuid-version flag
idt gen uuid --uuid-version 7
```

### NanoID Customization

```bash
# Custom length
idt gen nanoid --length 32

# Custom alphabet (hex characters only)
idt gen nanoid --alphabet "0123456789abcdef"

# Both
idt gen nanoid --length 16 --alphabet "0123456789ABCDEF"
```

### Snowflake Customization

```bash
# Discord epoch
idt gen snowflake --epoch discord

# Twitter epoch
idt gen snowflake --epoch twitter

# Custom epoch (milliseconds since Unix epoch)
idt gen snowflake --epoch 1420070400000

# With machine and datacenter IDs
idt gen snowflake --machine-id 1 --datacenter-id 2
```

### Output Formats

```bash
# Generate and output as hex
idt gen uuid -f hex

# Generate and output as Base64
idt gen uuidv7 -f base64

# Save to file
idt gen uuid -n 1000 -o uuids.txt
```

### JSON Output

```bash
# Single ID as JSON
idt gen uuid --json
# Output: {"id":"550e8400-e29b-41d4-a716-446655440000"}

# Multiple IDs as JSON array
idt gen uuid -n 3 --json
# Output: ["550e8400-...", "6ba7b810-...", "7c9e6679-..."]
```

### Without Trailing Newline

```bash
# Useful for scripting
ID=$(idt gen uuid --no-newline)
echo "Generated: $ID"
```
