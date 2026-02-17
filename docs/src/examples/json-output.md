# JSON Output

Examples for using idt's JSON output for machine-readable data processing.

## Enabling JSON Output

### Basic JSON

```bash
# Add --json flag to any command
idt gen uuid --json
idt inspect "$ID" --json
idt validate "$ID" --json
idt compare "$ID1" "$ID2" --json
idt info --json
```

### Pretty-Printed JSON

```bash
# Add --pretty for formatted output
idt inspect "$ID" --json --pretty
```

## Command Outputs

### gen Command

```bash
# Single ID
idt gen uuid --json
# {"id":"550e8400-e29b-41d4-a716-446655440000"}

# Multiple IDs
idt gen uuid -n 3 --json
# ["550e8400-...","6ba7b810-...","7c9e6679-..."]
```

### inspect Command

```bash
idt inspect 019c04e5-6118-7b22-95cb-a10e84dad469 --json --pretty
```

Output:
```json
{
  "id_type": "uuidv7",
  "canonical": "019c04e5-6118-7b22-95cb-a10e84dad469",
  "timestamp": 1706450267416,
  "timestamp_iso": "2026-01-28T13:57:47.416Z",
  "timestamp_local_iso": "2026-01-28T22:57:47.416+09:00",
  "version": "7",
  "variant": "RFC4122",
  "random_bits": 62,
  "encodings": {
    "hex": "019c04e561187b2295cba10e84dad469",
    "base64": "AZwE5WEYeyKVy6EOhNrUaQ==",
    "int": "2139325608653621017571381452845274217"
  }
}
```

### validate Command

```bash
idt validate 550e8400-e29b-41d4-a716-446655440000 --json
```

Output:
```json
{
  "input": "550e8400-e29b-41d4-a716-446655440000",
  "valid": true,
  "id_type": "uuidv4"
}
```

Invalid ID:
```json
{
  "input": "not-valid",
  "valid": false,
  "error": "Not a recognized ID format"
}
```

### compare Command

```bash
idt compare "$ID1" "$ID2" --json --pretty
```

Output:
```json
{
  "id1": "019c04e5-6118-7b22-95cb-a10e84dad469",
  "id2": "019c04e5-6119-7000-8000-000000000000",
  "type1": "uuidv7",
  "type2": "uuidv7",
  "binary_order": "less",
  "lexicographic_order": "less",
  "chronological_order": "less",
  "time_diff_ms": 1000,
  "timestamp1": 1706450267416,
  "timestamp2": 1706450268416
}
```

### info Command

```bash
idt info uuidv7 --json --pretty
```

Output:
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

## Processing with jq

### Extract Fields

```bash
# Get ID type
idt inspect "$ID" --json | jq -r '.id_type'

# Get timestamp
idt inspect "$ID" --json | jq -r '.timestamp_iso'

# Get specific encoding
idt inspect "$ID" --json | jq -r '.encodings.hex'
```

### Filter and Transform

```bash
# Get only sortable types
idt info --json | jq '[.[] | select(.is_sortable)]'

# Extract names of types with timestamps
idt info --json | jq -r '.[] | select(.has_timestamp) | .name'

# Create custom object
idt inspect "$ID" --json | jq '{id: .canonical, created: .timestamp_iso}'
```

### Batch Processing

```bash
# Process multiple IDs
cat ids.txt | while read id; do
    idt inspect "$id" --json
done | jq -s '.'  # Combine into array

# Extract timestamps from multiple IDs
cat ids.txt | while read id; do
    idt inspect "$id" --json
done | jq -r '.timestamp_iso'
```

## Integration Examples

### Store in Database

```bash
# Generate structured data for insertion
idt gen uuidv7 --json | jq -r '
    "INSERT INTO ids (id, created_at) VALUES (\(.id), NOW());"
'
```

### API Integration

```bash
# Generate ID and create JSON payload
ID=$(idt gen uuidv7 --no-newline)
PAYLOAD=$(jq -n --arg id "$ID" '{"resource_id": $id, "type": "new"}')
curl -X POST -d "$PAYLOAD" https://api.example.com/resources
```

### Logging

```bash
# Structured logging
idt inspect "$ID" --json | jq -c '{
    event: "id_inspected",
    id_type: .id_type,
    timestamp: .timestamp_iso
}'
```

### Configuration Files

```bash
# Generate config with new IDs
cat config.template.json | jq --arg id "$(idt gen uuidv7 --no-newline)" '
    .session_id = $id
'
```

## Converting Output

### To CSV

```bash
# Convert JSON output to CSV
idt info --json | jq -r '
    ["name","has_timestamp","is_sortable","bit_length"],
    (.[] | [.name, .has_timestamp, .is_sortable, .bit_length])
    | @csv
'
```

### To TSV

```bash
# Convert to tab-separated
idt info --json | jq -r '.[] | [.name, .description] | @tsv'
```

### To Environment Variables

```bash
# Export as env vars
eval $(idt inspect "$ID" --json | jq -r '
    "export ID_TYPE=\(.id_type)",
    "export ID_TIMESTAMP=\(.timestamp_iso // "none")"
')
```

## Error Handling

### Check for Errors

```bash
# Check if valid JSON was returned
RESULT=$(idt inspect "$ID" --json 2>&1)
if echo "$RESULT" | jq -e . >/dev/null 2>&1; then
    echo "Valid JSON: $RESULT"
else
    echo "Error: $RESULT" >&2
fi
```

### Handle Missing Fields

```bash
# Use // for default values
idt inspect "$ID" --json | jq -r '.timestamp_iso // "No timestamp"'
```

## Performance Tips

### Avoid Pretty Print in Pipelines

```bash
# Fast (compact JSON)
idt gen uuid -n 1000 --json | jq '.[]'

# Slower (pretty printed)
idt gen uuid -n 1000 --json --pretty | jq '.[]'
```

### Use jq Streaming for Large Data

```bash
# Stream processing for large datasets
idt gen uuid -n 10000 --json | jq -c '.[]'
```
