# Generating IDs

Examples for generating various types of identifiers.

## Basic Generation

### UUIDs

```bash
# Random UUID (v4) - most common
idt gen uuid

# Time-sortable UUID (v7) - recommended for new projects
idt gen uuidv7

# Timestamp-based UUID (v1)
idt gen uuidv1

# Reordered timestamp UUID (v6)
idt gen uuidv6
```

### Other Formats

```bash
# ULID - compact, sortable
idt gen ulid

# NanoID - short, URL-friendly
idt gen nanoid

# Snowflake - 64-bit distributed ID
idt gen snowflake
```

## Batch Generation

```bash
# Generate 10 UUIDs
idt gen uuid -n 10

# Generate 1000 ULIDs
idt gen ulid -n 1000

# Generate to file
idt gen uuidv7 -n 10000 -o ids.txt
```

## Customization

### NanoID Options

```bash
# Custom length (default is 21)
idt gen nanoid --length 16
idt gen nanoid --length 32

# Custom alphabet
idt gen nanoid --alphabet "0123456789"          # Numeric only
idt gen nanoid --alphabet "0123456789abcdef"    # Hex
idt gen nanoid --alphabet "ABCDEFGHIJKLMNOP"    # Uppercase only

# Combined
idt gen nanoid --length 8 --alphabet "0123456789"
```

### Snowflake Options

```bash
# With machine/datacenter IDs
idt gen snowflake --machine-id 1 --datacenter-id 2

# With custom epoch
idt gen snowflake --epoch 1420070400000  # Discord epoch

# Named epochs
idt gen snowflake --epoch twitter
idt gen snowflake --epoch discord
```

## Output Formats

### Different Encodings

```bash
# Generate as hex (no dashes)
idt gen uuid -f hex

# Generate as Base64
idt gen uuidv7 -f base64

# Generate as integer
idt gen ulid -f int
```

### JSON Output

```bash
# Single ID as JSON object
idt gen uuid --json
# {"id":"550e8400-e29b-41d4-a716-446655440000"}

# Multiple IDs as JSON array
idt gen uuid -n 5 --json
# ["550e8400-...","6ba7b810-...",...]

# Pretty-printed
idt gen uuid --json --pretty
```

### No Trailing Newline

```bash
# Useful for variable assignment
ID=$(idt gen uuid --no-newline)

# For inline use
echo "ID: $(idt gen uuid --no-newline)"
```

## Practical Examples

### Database Seeding

```bash
# Generate IDs for test data
for i in {1..100}; do
    ID=$(idt gen uuidv7 --no-newline)
    echo "INSERT INTO users (id, name) VALUES ('$ID', 'User $i');"
done
```

### API Key Generation

```bash
# Generate secure API keys
idt gen nanoid --length 32 -n 10

# With custom prefix
for i in {1..5}; do
    KEY=$(idt gen nanoid --length 24 --no-newline)
    echo "sk_live_$KEY"
done
```

### File Naming

```bash
# Generate unique filename
FILENAME="backup_$(idt gen nanoid --length 8 --no-newline).tar.gz"
tar -czf "$FILENAME" /data

# With timestamp from ULID
ULID=$(idt gen ulid --no-newline)
mv upload.pdf "document_${ULID}.pdf"
```

### Distributed System IDs

```bash
# Server 1 (machine-id 1)
idt gen snowflake --machine-id 1

# Server 2 (machine-id 2)
idt gen snowflake --machine-id 2

# Different datacenters
idt gen snowflake --machine-id 1 --datacenter-id 1  # DC 1
idt gen snowflake --machine-id 1 --datacenter-id 2  # DC 2
```

### Verification Codes

```bash
# 6-digit numeric code
idt gen nanoid --length 6 --alphabet "0123456789"

# Alphanumeric confirmation code
idt gen nanoid --length 8 --alphabet "ABCDEFGHJKLMNPQRSTUVWXYZ23456789"
```

## Performance

### Bulk Generation

```bash
# Generate 100,000 IDs efficiently
time idt gen uuid -n 100000 > /dev/null

# Write to file
idt gen uuidv7 -n 1000000 -o million_ids.txt
```

### Parallel Generation

```bash
# Use parallel for very large batches
seq 10 | parallel "idt gen uuid -n 100000" > all_ids.txt
```
