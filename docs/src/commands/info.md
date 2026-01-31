# info - ID Type Information

Display information about supported ID types, including their characteristics, specifications, and usage notes.

## Usage

```bash
idt info [TYPE]
```

## Arguments

| Argument | Description |
|----------|-------------|
| `TYPE` | ID type to get information about (list all if omitted) |

## Examples

### List All Types

```bash
idt info
```

Output:
```
Supported ID Types
============================================================

UUID Family:
  uuidv1       [T-] Timestamp + MAC address
  uuidv3       [--] MD5 namespace hash
  uuidv4       [--] Random
  uuidv5       [--] SHA-1 namespace hash
  uuidv6       [TS] Reordered timestamp
  uuidv7       [TS] Unix timestamp + random
  uuid-nil     [--] All zeros
  uuid-max     [--] All ones

Modern Sortable IDs:
  ulid         [TS] Crockford Base32, lexicographically sortable
  snowflake    [TS] Twitter/Discord-style distributed ID

Compact IDs:
  nanoid       [--] Compact URL-friendly ID

Use 'idt info <TYPE>' for detailed information.
```

The flags in brackets indicate:
- `T` = Has timestamp
- `S` = Sortable
- `-` = No/Not applicable

### Detailed Type Information

```bash
idt info uuidv7
```

Output:
```
UUIDV7
============================================================

Unix timestamp + random

Has Timestamp:   Yes
Sortable:        Yes
Bit Length:      128 bits

Example:         019c04e5-6118-7b22-95cb-a10e84dad469

Specification:   https://datatracker.ietf.org/doc/html/rfc9562

Notes:
  - Recommended for new applications needing sortable UUIDs
  - Unix timestamp in milliseconds
  - Compatible with UUID infrastructure
```

### Other Type Examples

```bash
idt info ulid
```

Output:
```
ULID
============================================================

Crockford Base32, lexicographically sortable

Has Timestamp:   Yes
Sortable:        Yes
Bit Length:      128 bits

Example:         01ARZ3NDEKTSV4RRFFQ69G5FAV

Specification:   https://github.com/ulid/spec

Notes:
  - Case-insensitive (Crockford Base32)
  - Monotonic within same millisecond
  - Compatible with UUID (128-bit)
```

```bash
idt info snowflake
```

Output:
```
SNOWFLAKE
============================================================

Twitter/Discord-style distributed ID

Has Timestamp:   Yes
Sortable:        Yes
Bit Length:      64 bits

Example:         1234567890123456789

Specification:   https://en.wikipedia.org/wiki/Snowflake_ID

Notes:
  - Originally designed by Twitter
  - Requires coordination (machine/datacenter IDs)
  - Epoch can be customized
```

### JSON Output

```bash
# All types as JSON
idt info --json

# Specific type as JSON
idt info uuidv7 --json
```

Example JSON output:
```json
{
  "name": "uuidv7",
  "description": "Unix timestamp + random",
  "has_timestamp": true,
  "is_sortable": true,
  "bit_length": 128,
  "example": "019c04e5-6118-7b22-95cb-a10e84dad469",
  "spec_url": "https://datatracker.ietf.org/doc/html/rfc9562",
  "notes": [
    "Recommended for new applications needing sortable UUIDs",
    "Unix timestamp in milliseconds",
    "Compatible with UUID infrastructure"
  ]
}
```

### Querying Capabilities

Use JSON output to query capabilities programmatically:

```bash
# List all sortable types
idt info --json | jq '.[] | select(.is_sortable) | .name'

# List all types with timestamps
idt info --json | jq '.[] | select(.has_timestamp) | .name'

# Get bit length of a type
idt info uuidv7 --json | jq '.bit_length'
```
