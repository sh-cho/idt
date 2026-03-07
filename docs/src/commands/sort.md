# sort - Sort IDs by Timestamp

Sort a list of IDs by their embedded timestamps. Useful when debugging distributed systems, analyzing logs, or working with database records where you need to understand temporal ordering.

## Usage

```bash
idt sort [OPTIONS] [ID...]
```

## Arguments

| Argument | Description |
|----------|-------------|
| `ID...` | IDs to sort (reads from stdin if omitted) |

## Options

| Option | Description |
|--------|-------------|
| `-t, --id-type <TYPE>` | ID type hint (skip auto-detection) |
| `-r, --reverse` | Sort in descending order (newest first) |
| `--show-time` | Display timestamps alongside IDs |
| `--epoch <EPOCH>` | Snowflake epoch (`discord`, `twitter`, or milliseconds) |
| `--on-unsortable <POLICY>` | Policy for IDs without timestamps: `skip` (default), `error`, `end` |

## Examples

### Basic Sorting

```bash
# Sort ULIDs by creation time
idt sort 01KK3ZE8GEVTC9PGC0NTY1RY03 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

Output:
```
01ARZ3NDEKTSV4RRFFQ69G5FAV
01KK3ZE8GEVTC9PGC0NTY1RY03
```

### Show Timestamps

```bash
idt sort --show-time 01KK3ZE8GEVTC9PGC0NTY1RY03 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

Output:
```
2016-07-30T23:54:10.259Z  01ARZ3NDEKTSV4RRFFQ69G5FAV
2026-03-07T11:03:08.046Z  01KK3ZE8GEVTC9PGC0NTY1RY03
```

### Reverse Order (Newest First)

```bash
idt sort --reverse --show-time 01ARZ3NDEKTSV4RRFFQ69G5FAV 01KK3ZE8GEVTC9PGC0NTY1RY03
```

Output:
```
2026-03-07T11:03:08.046Z  01KK3ZE8GEVTC9PGC0NTY1RY03
2016-07-30T23:54:10.259Z  01ARZ3NDEKTSV4RRFFQ69G5FAV
```

### Pipe from Generation

```bash
# Generate 5 ULIDs and sort them with timestamps
idt gen ulid -n 5 | idt sort --show-time
```

### Mixed ID Types

IDs of different types can be sorted together as long as they have timestamps:

```bash
# Sort a mix of UUIDv7 and ULID
idt sort 019c04e5-6118-7b22-95cb-a10e84dad469 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

### JSON Output

```bash
idt sort --json --pretty 01ARZ3NDEKTSV4RRFFQ69G5FAV 01KK3ZE8GEVTC9PGC0NTY1RY03
```

Output:
```json
{
  "sorted": [
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "id_type": "ulid",
      "timestamp_ms": 1469922850259,
      "timestamp_iso": "2016-07-30T23:54:10.259Z"
    },
    {
      "id": "01KK3ZE8GEVTC9PGC0NTY1RY03",
      "id_type": "ulid",
      "timestamp_ms": 1772881388046,
      "timestamp_iso": "2026-03-07T11:03:08.046Z"
    }
  ],
  "unsortable": [],
  "count": 2
}
```

## Unsortable ID Policies

Some ID types (NanoID, CUID2, UUIDv4) don't have embedded timestamps. The `--on-unsortable` option controls how these are handled:

| Policy | Behavior |
|--------|----------|
| `skip` (default) | Skip the ID with a warning to stderr |
| `error` | Fail immediately with an error |
| `end` | Append unsortable IDs after the sorted ones |

```bash
# Skip unsortable IDs (default)
idt sort 01ARZ3NDEKTSV4RRFFQ69G5FAV some-nanoid-value

# Fail on unsortable IDs
idt sort --on-unsortable error 01ARZ3NDEKTSV4RRFFQ69G5FAV some-nanoid-value

# Append unsortable IDs at the end
idt sort --on-unsortable end 01ARZ3NDEKTSV4RRFFQ69G5FAV some-nanoid-value
```

## Use Cases

**Order log entries by creation time:**
```bash
cat log_ids.txt | idt sort --show-time
```

**Find the newest/oldest ID:**
```bash
# Oldest
cat ids.txt | idt sort | head -1

# Newest
cat ids.txt | idt sort --reverse | head -1
```

**Timeline reconstruction:**
```bash
cat event_ids.txt | idt sort --show-time --json | jq '.sorted[] | "\(.timestamp_iso) \(.id)"'
```
