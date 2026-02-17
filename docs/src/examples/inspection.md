# Inspecting IDs

Examples for analyzing and decoding identifiers to extract embedded information.

## Basic Inspection

### Auto-Detection

idt automatically detects the ID type:

```bash
# UUID
idt inspect 550e8400-e29b-41d4-a716-446655440000

# ULID
idt inspect 01ARZ3NDEKTSV4RRFFQ69G5FAV

# Snowflake
idt inspect 1234567890123456789
```

### With Type Hint

Force interpretation as a specific type:

```bash
# Parse as UUID (even without dashes)
idt inspect -t uuid 550e8400e29b41d4a716446655440000

# Parse as ULID
idt inspect -t ulid 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

## Extracting Information

### Timestamps

```bash
# Get creation time from UUIDv7
idt inspect 019c04e5-6118-7b22-95cb-a10e84dad469

# Output includes:
#   Time (UTC)          2026-01-28T13:57:47.416Z
#   Local Time (+09:00) 2026-01-28T22:57:47.416+09:00
```

Using JSON to extract specific fields:

```bash
# Get timestamp as ISO string (UTC)
idt inspect 019c04e5-6118-7b22-95cb-a10e84dad469 --json | jq -r '.timestamp_iso'

# Get timestamp in local timezone
idt inspect 019c04e5-6118-7b22-95cb-a10e84dad469 --json | jq -r '.timestamp_local_iso'

# Get timestamp as milliseconds
idt inspect 019c04e5-6118-7b22-95cb-a10e84dad469 --json | jq '.timestamp'
```

### UUID Version and Variant

```bash
# Inspect UUID to see version
idt inspect 550e8400-e29b-41d4-a716-446655440000

# Output includes:
#   Version    4
#   Variant    RFC4122
```

### Multiple Encodings

Every inspection shows the ID in multiple formats:

```bash
idt inspect 01ARZ3NDEKTSV4RRFFQ69G5FAV

# Output includes:
#   Hex        01563e3ab5d3d6764c61efb99302bd5b
#   Base64     AVY+OrXT1nZMYe+5kwK9Ww==
#   Int        1777027686520646174104517696511196507
```

## Batch Inspection

### Multiple Arguments

```bash
# Inspect multiple IDs
idt inspect id1 id2 id3
```

### From File

```bash
# Inspect all IDs in a file
cat ids.txt | idt inspect

# Only valid IDs
cat ids.txt | idt inspect 2>/dev/null
```

### Pipeline

```bash
# Generate and immediately inspect
idt gen uuidv7 | idt inspect

# Inspect IDs from another command
grep -o '[0-9a-f-]\{36\}' logfile.log | idt inspect
```

## JSON Output

### Single ID

```bash
idt inspect 550e8400-e29b-41d4-a716-446655440000 --json --pretty
```

Output:
```json
{
  "id_type": "uuidv4",
  "canonical": "550e8400-e29b-41d4-a716-446655440000",
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

### Extract Specific Fields

```bash
# Get ID type
idt inspect "$ID" --json | jq -r '.id_type'

# Get all encodings
idt inspect "$ID" --json | jq '.encodings'

# Get hex representation
idt inspect "$ID" --json | jq -r '.encodings.hex'
```

## Practical Examples

### Debug Logging

```bash
# Decode ID from error logs
grep "failed.*id:" error.log | awk '{print $NF}' | idt inspect

# Find when an event occurred (UTC)
idt inspect 019c04e5-6118-7b22-95cb-a10e84dad469 --json | jq -r '.timestamp_iso'
# Output: 2026-01-28T13:57:47.416Z

# Find when an event occurred (local time)
idt inspect 019c04e5-6118-7b22-95cb-a10e84dad469 --json | jq -r '.timestamp_local_iso'
# Output: 2026-01-28T22:57:47.416+09:00
```

### Data Analysis

```bash
# Analyze IDs from database export
psql -c "SELECT id FROM events" -t | idt inspect --json | jq -r '.timestamp_iso' | sort

# Group by ID type
cat mixed_ids.txt | while read id; do
    TYPE=$(idt inspect "$id" --json 2>/dev/null | jq -r '.id_type')
    echo "$TYPE: $id"
done
```

### Verify ID Properties

```bash
# Check if ID is time-sortable
TYPE=$(idt inspect "$ID" --json | jq -r '.id_type')
case "$TYPE" in
    uuidv7|ulid|snowflake) echo "Time-sortable" ;;
    *) echo "Not time-sortable" ;;
esac

# Check if ID has timestamp
if idt inspect "$ID" --json | jq -e '.timestamp' > /dev/null 2>&1; then
    echo "Has timestamp"
else
    echo "No timestamp"
fi
```

### Convert Between Systems

```bash
# Get UUID-compatible hex for ULID
idt inspect 01ARZ3NDEKTSV4RRFFQ69G5FAV --json | jq -r '.encodings.hex'
# Can be used as: INSERT INTO uuid_col VALUES('01563e3ab5d3d6764c61efb99302bd5b')
```

## Error Handling

### Invalid IDs

```bash
# Invalid ID shows error
idt inspect "not-a-valid-id"
# Error parsing 'not-a-valid-id': Not a recognized ID format

# Quiet mode for scripts
if idt inspect -q "$ID" 2>/dev/null; then
    echo "Valid"
else
    echo "Invalid"
fi
```

### Ambiguous IDs

Some strings could be multiple ID types:

```bash
# Provide type hint to disambiguate
idt inspect -t uuid "$AMBIGUOUS_ID"
```
