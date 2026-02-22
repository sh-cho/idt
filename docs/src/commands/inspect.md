# inspect - Analyze IDs

Analyze and decode identifiers to extract embedded information like timestamps, versions, and other metadata.

## Usage

```bash
idt inspect [OPTIONS] [ID]...
```

## Arguments

| Argument | Description |
|----------|-------------|
| `ID` | ID(s) to inspect (reads from stdin if omitted) |

## Options

| Option | Description |
|--------|-------------|
| `-t, --type <TYPE>` | Hint the ID type (skip auto-detection) |
| `--epoch <EPOCH>` | Epoch for Snowflake IDs (`discord`, `twitter`, or milliseconds since Unix epoch) |
| `-q, --quiet` | Only show errors (for validation use) |

## Output Fields

When inspecting an ID, idt displays:

| Field | Description |
|-------|-------------|
| Type | Detected ID type (e.g., UUIDV7, ULID) |
| Canonical | The ID in its canonical format |
| Time (UTC) | Embedded timestamp in UTC (if available) |
| Local Time | Timestamp in local timezone with UTC offset (if available) |
| Version | UUID version number (for UUIDs) |
| Variant | UUID variant (for UUIDs) |
| Random | Number of random bits |
| Hex | Hexadecimal encoding |
| Base64 | Base64 encoding |
| Int | Integer representation |

## Examples

### Basic Inspection

```bash
# Inspect a UUID
idt inspect 550e8400-e29b-41d4-a716-446655440000

# Inspect a ULID
idt inspect 01ARZ3NDEKTSV4RRFFQ69G5FAV

# Inspect multiple IDs
idt inspect 550e8400-e29b-41d4-a716-446655440000 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

### Example Output

```bash
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

### ULID Output

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

### Type Hints

When auto-detection is ambiguous, provide a type hint:

```bash
# Force interpretation as UUID
idt inspect -t uuid 550e8400e29b41d4a716446655440000
```

### Snowflake Epochs

Snowflake IDs encode timestamps relative to a custom epoch. Use `--epoch` to decode with the correct epoch:

```bash
# Discord Snowflake
idt inspect -t snowflake --epoch discord 1474004412518240339

# Twitter Snowflake
idt inspect -t snowflake --epoch twitter 1234567890123456789

# Custom epoch (milliseconds since Unix epoch)
idt inspect -t snowflake --epoch 1420070400000 1474004412518240339
```

Without `--epoch`, Snowflake IDs are decoded using the Unix epoch (0), which may produce incorrect timestamps for IDs generated with a custom epoch.

### Reading from stdin

```bash
# Pipe from gen command
idt gen uuidv7 | idt inspect

# Read multiple IDs from file
cat ids.txt | idt inspect

# Here-string
idt inspect <<< "550e8400-e29b-41d4-a716-446655440000"
```

### JSON Output

```bash
# JSON output
idt inspect 550e8400-e29b-41d4-a716-446655440000 --json

# Pretty-printed JSON
idt inspect 550e8400-e29b-41d4-a716-446655440000 --json --pretty
```

Example JSON output:

```json
{
  "id_type": "uuidv4",
  "canonical": "550e8400-e29b-41d4-a716-446655440000",
  "valid": true,
  "version": "4",
  "variant": "RFC4122",
  "random_bits": 122,
  "encodings": {
    "hex": "550e8400e29b41d4a716446655440000",
    "base64": "VQ6EAOKbQdSnFkRmVUQAAA==",
    "int": "113059749145936325402354257176981405696"
  }
}
```

For timestamped IDs, additional fields are included:
```json
{
  "timestamp": { "millis": 1706450267416 },
  "timestamp_iso": "2026-01-28T13:57:47.416Z",
  "timestamp_local_iso": "2026-01-28T22:57:47.416+09:00"
}
```

### Quiet Mode

Quiet mode exits with code 0 for valid IDs, 1 for invalid:

```bash
if idt inspect -q "$ID" 2>/dev/null; then
    echo "Valid and parseable"
fi
```

### Inspecting Generated IDs

```bash
# Generate and inspect in one pipeline
idt gen uuidv7 | idt inspect

# Generate multiple and inspect
idt gen ulid -n 5 | idt inspect
```
