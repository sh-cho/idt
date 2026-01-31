# Converting Formats

Examples for converting IDs between different encoding formats.

## Basic Conversions

### To Hexadecimal

```bash
# UUID to hex (removes dashes)
idt convert 550e8400-e29b-41d4-a716-446655440000 -f hex
# Output: 550e8400e29b41d4a716446655440000

# ULID to hex
idt convert 01ARZ3NDEKTSV4RRFFQ69G5FAV -f hex
# Output: 01563e3ab5d3d6764c61efb99302bd5b
```

### To Base64

```bash
# Standard Base64
idt convert 550e8400-e29b-41d4-a716-446655440000 -f base64
# Output: VQ6EAOKbQdSnFkRmVUQAAA==

# URL-safe Base64 (no padding)
idt convert 550e8400-e29b-41d4-a716-446655440000 -f base64url
# Output: VQ6EAOKbQdSnFkRmVUQAAA
```

### To Base58

```bash
idt convert 550e8400-e29b-41d4-a716-446655440000 -f base58
# Output: 6K8FVbLqP4V8nDqTJNXH6k
```

### To Integer

```bash
idt convert 550e8400-e29b-41d4-a716-446655440000 -f int
# Output: 113059749145936325402354257176981405696
```

## All Formats

```bash
ID="550e8400-e29b-41d4-a716-446655440000"

# Each encoding format
idt convert "$ID" -f canonical  # Original format
idt convert "$ID" -f hex        # Hexadecimal
idt convert "$ID" -f base32     # Base32
idt convert "$ID" -f base58     # Base58
idt convert "$ID" -f base64     # Base64
idt convert "$ID" -f base64url  # URL-safe Base64
idt convert "$ID" -f bits       # Binary string
idt convert "$ID" -f int        # Integer
idt convert "$ID" -f bytes      # Space-separated hex bytes
```

## Case Transformation

### Uppercase

```bash
# Uppercase hex
idt convert 550e8400-e29b-41d4-a716-446655440000 -f hex -U
# Output: 550E8400E29B41D4A716446655440000
```

### Lowercase

```bash
# Lowercase (normalize)
idt convert 550E8400-E29B-41D4-A716-446655440000 -f hex -L
# Output: 550e8400e29b41d4a716446655440000
```

## Batch Conversion

### Multiple IDs

```bash
# Convert multiple IDs
idt convert id1 id2 id3 -f base64
```

### From File

```bash
# Convert all IDs in a file
cat uuids.txt | idt convert -f hex

# Save converted IDs
cat uuids.txt | idt convert -f base64 > encoded.txt
```

### Pipeline

```bash
# Generate and convert
idt gen uuid | idt convert -f base64

# Convert output from another command
grep -o '[0-9a-f-]\{36\}' logfile.log | idt convert -f hex
```

## Practical Examples

### Database Compatibility

```bash
# PostgreSQL bytea format
idt convert 550e8400-e29b-41d4-a716-446655440000 -f hex
# Use as: INSERT INTO table (id) VALUES (decode('550e8400...', 'hex'))

# Binary storage
idt convert 550e8400-e29b-41d4-a716-446655440000 -f bytes
# Output: 55 0e 84 00 e2 9b 41 d4 a7 16 44 66 55 44 00 00
```

### URL Encoding

```bash
# URL-safe encoding for API calls
ID=$(idt gen uuidv7 --no-newline)
ENCODED=$(echo "$ID" | idt convert -f base64url)
curl "https://api.example.com/item/$ENCODED"
```

### Cross-System Integration

```bash
# System A uses UUIDs with dashes
UUID="550e8400-e29b-41d4-a716-446655440000"

# System B uses hex without dashes
HEX=$(idt convert "$UUID" -f hex)
# 550e8400e29b41d4a716446655440000

# System C uses Base64
B64=$(idt convert "$UUID" -f base64)
# VQ6EAOKbQdSnFkRmVUQAAA==
```

### Data Migration

```bash
# Convert exported IDs for import
cat export.csv | while IFS=, read id name; do
    NEW_ID=$(idt convert "$id" -f hex)
    echo "$NEW_ID,$name"
done > import.csv
```

### Compact Storage

```bash
# Store IDs more compactly
# UUID: 36 chars -> Base64: 24 chars -> Base58: ~22 chars

# Original
550e8400-e29b-41d4-a716-446655440000  # 36 chars

# Base64
idt convert 550e8400-e29b-41d4-a716-446655440000 -f base64
VQ6EAOKbQdSnFkRmVUQAAA==              # 24 chars

# Base58
idt convert 550e8400-e29b-41d4-a716-446655440000 -f base58
6K8FVbLqP4V8nDqTJNXH6k                 # 22 chars
```

### Binary Analysis

```bash
# View binary representation
idt convert 550e8400-e29b-41d4-a716-446655440000 -f bits
# Output: 01010101000011101000010000000000...

# View byte-by-byte
idt convert 550e8400-e29b-41d4-a716-446655440000 -f bytes
# Output: 55 0e 84 00 e2 9b 41 d4 a7 16 44 66 55 44 00 00
```

## JSON Output

```bash
# Get converted value as JSON
idt convert 550e8400-e29b-41d4-a716-446655440000 -f hex --json
# Output: "550e8400e29b41d4a716446655440000"

# Multiple IDs
idt convert id1 id2 -f base64 --json
# Output: ["VQ6E...","6ba7..."]
```

## Type Hints

When auto-detection fails:

```bash
# Interpret as UUID
idt convert -t uuid 550e8400e29b41d4a716446655440000 -f canonical
# Output: 550e8400-e29b-41d4-a716-446655440000
```
