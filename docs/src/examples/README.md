# Examples Overview

This section provides practical examples for common idt use cases.

## Quick Examples

### Generate and Use

```bash
# Generate a UUID and store it
ID=$(idt gen uuidv7 --no-newline)
echo "Created resource with ID: $ID"

# Generate and insert into database
idt gen uuid | xargs -I {} psql -c "INSERT INTO items (id) VALUES ('{}')"
```

### Inspect Unknown IDs

```bash
# What type of ID is this?
idt inspect 01ARZ3NDEKTSV4RRFFQ69G5FAV

# When was it created?
idt inspect 019c04e5-6118-7b22-95cb-a10e84dad469 --json | jq '.timestamp_iso'
```

### Batch Operations

```bash
# Generate 1000 UUIDs to file
idt gen uuid -n 1000 -o uuids.txt

# Validate all IDs in a file
cat ids.txt | idt validate

# Convert a list of IDs
cat uuids.txt | idt convert -f base64
```

## Example Categories

- [Generating IDs](./generation.md) - ID generation patterns
- [Inspecting IDs](./inspection.md) - Analyzing and decoding IDs
- [Converting Formats](./conversion.md) - Format transformation
- [Shell Scripting](./scripting.md) - Integration with shell scripts
- [JSON Output](./json-output.md) - Machine-readable output

## Common Patterns

### Database Primary Keys

```bash
# Generate sortable primary key
idt gen uuidv7

# Generate for existing UUID column
idt gen uuid
```

### API Tokens

```bash
# Long, secure token
idt gen nanoid --length 32

# URL-safe token
idt gen nanoid --alphabet "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
```

### Debugging

```bash
# Decode ID from logs
idt inspect "01ARZ3NDEKTSV4RRFFQ69G5FAV"

# Compare two IDs for ordering
idt compare "$OLD_ID" "$NEW_ID"
```

### Data Migration

```bash
# Convert UUID format for different system
idt convert "550e8400-e29b-41d4-a716-446655440000" -f base64

# Validate IDs before import
cat export.csv | cut -d, -f1 | idt validate
```
