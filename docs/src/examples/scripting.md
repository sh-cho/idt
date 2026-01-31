# Shell Scripting

Examples for integrating idt into shell scripts and Unix pipelines.

## Variable Assignment

### Basic Assignment

```bash
# Generate and store ID
ID=$(idt gen uuid)
echo "Generated ID: $ID"

# Use --no-newline for cleaner assignment
ID=$(idt gen uuidv7 --no-newline)
```

### Multiple IDs

```bash
# Store in array (bash)
mapfile -t IDS < <(idt gen uuid -n 10)
echo "First ID: ${IDS[0]}"
echo "Total: ${#IDS[@]}"
```

## Conditional Logic

### Validation

```bash
# Check if ID is valid
if idt validate -q "$ID" 2>/dev/null; then
    echo "Valid ID: $ID"
else
    echo "Invalid ID: $ID"
    exit 1
fi
```

### Type Checking

```bash
# Check ID type
TYPE=$(idt inspect "$ID" --json 2>/dev/null | jq -r '.id_type')
case "$TYPE" in
    uuidv7|ulid)
        echo "Time-sortable ID"
        ;;
    uuidv4)
        echo "Random UUID"
        ;;
    *)
        echo "Other type: $TYPE"
        ;;
esac
```

## Pipelines

### Generate and Process

```bash
# Generate, convert, and use
idt gen uuidv7 | idt convert -f base64 | xargs echo "Encoded:"

# Generate multiple and filter
idt gen uuid -n 100 | grep '^[0-4]'  # IDs starting with 0-4
```

### Process Files

```bash
# Validate IDs from file
cat ids.txt | idt validate

# Convert all IDs in file
cat uuids.txt | idt convert -f hex > hex_ids.txt

# Inspect and extract timestamps
cat ids.txt | idt inspect --json | jq -r '.timestamp_iso'
```

### Chain Commands

```bash
# Generate -> Inspect -> Extract
idt gen uuidv7 | idt inspect --json | jq -r '.timestamp_iso'

# Complex pipeline
idt gen ulid -n 1000 | \
    idt inspect --json | \
    jq -r '.encodings.hex' | \
    sort | \
    head -10
```

## Loops

### Process Multiple IDs

```bash
# Process each ID individually
while read -r id; do
    echo "Processing: $id"
    idt inspect "$id" --json | jq -r '.id_type'
done < ids.txt
```

### Generate with Custom Logic

```bash
# Generate IDs with prefixes
for type in user order item; do
    ID=$(idt gen nanoid --length 16 --no-newline)
    echo "${type}_${ID}"
done
```

### Batch Processing

```bash
# Process in batches
idt gen uuid -n 1000 | while read -r id; do
    # Process each ID
    echo "INSERT INTO table VALUES ('$id');"
done | psql mydb
```

## Error Handling

### Check Exit Codes

```bash
# Validate with error handling
if ! idt validate -q "$ID" 2>/dev/null; then
    echo "Error: Invalid ID '$ID'" >&2
    exit 1
fi
```

### Capture Errors

```bash
# Capture both output and errors
RESULT=$(idt inspect "$ID" 2>&1)
if [ $? -eq 0 ]; then
    echo "Success: $RESULT"
else
    echo "Error: $RESULT" >&2
fi
```

### Continue on Error

```bash
# Process all, report errors
cat ids.txt | while read -r id; do
    if idt validate -q "$id" 2>/dev/null; then
        echo "$id" >> valid.txt
    else
        echo "$id" >> invalid.txt
    fi
done
```

## Practical Scripts

### Database Seeding Script

```bash
#!/bin/bash
# seed_database.sh - Generate test data with unique IDs

COUNT=${1:-100}
TABLE=${2:-users}

echo "Seeding $TABLE with $COUNT records..."

idt gen uuidv7 -n "$COUNT" | while read -r id; do
    NAME="User_$(idt gen nanoid --length 8 --no-newline)"
    echo "INSERT INTO $TABLE (id, name) VALUES ('$id', '$NAME');"
done | psql mydb

echo "Done!"
```

### ID Migration Script

```bash
#!/bin/bash
# migrate_ids.sh - Convert IDs between formats

INPUT_FILE=$1
OUTPUT_FORMAT=${2:-base64}

if [ -z "$INPUT_FILE" ]; then
    echo "Usage: $0 <input_file> [format]"
    exit 1
fi

cat "$INPUT_FILE" | while read -r id; do
    CONVERTED=$(idt convert "$id" -f "$OUTPUT_FORMAT" 2>/dev/null)
    if [ -n "$CONVERTED" ]; then
        echo "$CONVERTED"
    else
        echo "# Failed: $id" >&2
    fi
done
```

### ID Validation Script

```bash
#!/bin/bash
# validate_export.sh - Validate IDs in CSV export

FILE=$1
COLUMN=${2:-1}

if [ -z "$FILE" ]; then
    echo "Usage: $0 <csv_file> [column_number]"
    exit 1
fi

TOTAL=0
VALID=0
INVALID=0

tail -n +2 "$FILE" | cut -d',' -f"$COLUMN" | while read -r id; do
    ((TOTAL++))
    if idt validate -q "$id" 2>/dev/null; then
        ((VALID++))
    else
        ((INVALID++))
        echo "Invalid at line $TOTAL: $id" >&2
    fi
done

echo "Total: $TOTAL, Valid: $VALID, Invalid: $INVALID"
```

### Timestamp Extraction Script

```bash
#!/bin/bash
# extract_timestamps.sh - Extract creation times from IDs

while read -r id; do
    TIMESTAMP=$(idt inspect "$id" --json 2>/dev/null | jq -r '.timestamp_iso // "N/A"')
    echo "$id -> $TIMESTAMP"
done
```

## Integration with Other Tools

### With jq

```bash
# Extract specific fields
idt inspect "$ID" --json | jq '{type: .id_type, time: .timestamp_iso}'

# Filter by type
idt gen uuid -n 10 --json | jq '.[]'
```

### With xargs

```bash
# Process IDs in parallel
cat ids.txt | xargs -P 4 -I {} sh -c 'idt inspect "{}" --json'
```

### With awk

```bash
# Combine with awk processing
idt gen uuid -n 100 | awk '{print NR": "$0}'
```

### With parallel

```bash
# High-performance parallel processing
cat large_ids.txt | parallel -j 8 'idt inspect {} --json'
```
