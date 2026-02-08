# convert - Convert Formats

Convert identifiers between different encoding formats.

## Usage

```bash
idt convert [OPTIONS] [ID]...
```

## Arguments

| Argument | Description |
|----------|-------------|
| `ID` | ID(s) to convert (reads from stdin if omitted) |

## Options

| Option | Description |
|--------|-------------|
| `-t, --type <TYPE>` | Source ID type (auto-detect if omitted) |
| `-f, --format <FORMAT>` | Target encoding format |
| `--to <TYPE>` | Convert to different ID type (if compatible) |
| `-U, --uppercase` | Uppercase output |
| `-L, --lowercase` | Lowercase output |

## Encoding Formats

| Format | Description | Example |
|--------|-------------|---------|
| `canonical` | Original format | `550e8400-e29b-41d4-a716-446655440000` |
| `hex` | Hexadecimal | `550e8400e29b41d4a716446655440000` |
| `base32` | Base32 (RFC 4648) | `KUHIBAASSNE5JJYWIRDFKRAAAA` |
| `base58` | Base58 | `6K8FVbLqP4V8nDqTJNXH6k` |
| `base64` | Base64 | `VQ6EAOKbQdSnFkRmVUQAAA==` |
| `base64url` | URL-safe Base64 | `VQ6EAOKbQdSnFkRmVUQAAA` |
| `bits` | Binary string | `01010101000011101000...` |
| `int` | Integer | `113059749145936325402354257176981405696` |
| `bytes` | Space-separated hex bytes | `55 0e 84 00 e2 9b 41 d4...` |

## Examples

### Basic Conversion

```bash
# Convert to hex
idt convert 550e8400-e29b-41d4-a716-446655440000 -f hex
# Output: 550e8400e29b41d4a716446655440000

# Convert to Base64
idt convert 550e8400-e29b-41d4-a716-446655440000 -f base64
# Output: VQ6EAOKbQdSnFkRmVUQAAA==

# Convert to Base58
idt convert 550e8400-e29b-41d4-a716-446655440000 -f base58
# Output: 6K8FVbLqP4V8nDqTJNXH6k

# Convert to integer
idt convert 550e8400-e29b-41d4-a716-446655440000 -f int
# Output: 113059749145936325402354257176981405696
```

### Case Transformation

```bash
# Uppercase hex
idt convert 550e8400-e29b-41d4-a716-446655440000 -f hex -U
# Output: 550E8400E29B41D4A716446655440000

# Lowercase
idt convert 550E8400-E29B-41D4-A716-446655440000 -f hex -L
# Output: 550e8400e29b41d4a716446655440000
```

### Converting ULID

```bash
# ULID to hex
idt convert 01ARZ3NDEKTSV4RRFFQ69G5FAV -f hex
# Output: 01563e3ab5d3d6764c61efb99302bd5b

# ULID to Base64
idt convert 01ARZ3NDEKTSV4RRFFQ69G5FAV -f base64
# Output: AVY+OrXT1nZMYe+5kwK9Ww==
```

### Binary Representation

```bash
# Convert to binary string
idt convert 550e8400-e29b-41d4-a716-446655440000 -f bits
# Output: 01010101000011101000010000000000111000101001101101000001110101001010011100010110010001000110011001010101010001000000000000000000

# Convert to space-separated bytes
idt convert 550e8400-e29b-41d4-a716-446655440000 -f bytes
# Output: 55 0e 84 00 e2 9b 41 d4 a7 16 44 66 55 44 00 00
```

### URL-Safe Base64

```bash
# Standard Base64 (with padding)
idt convert 550e8400-e29b-41d4-a716-446655440000 -f base64
# Output: VQ6EAOKbQdSnFkRmVUQAAA==

# URL-safe Base64 (no padding)
idt convert 550e8400-e29b-41d4-a716-446655440000 -f base64url
# Output: VQ6EAOKbQdSnFkRmVUQAAA
```

### Reading from stdin

```bash
# Pipe from gen
idt gen uuid | idt convert -f base64
# Possible output: dkIwdr8eQ+WS5BaKwkF55g==

# Convert multiple IDs
echo -e "550e8400-e29b-41d4-a716-446655440000\n6ba7b810-9dad-11d1-80b4-00c04fd430c8" | idt convert -f hex
# Output:
# 550e8400e29b41d4a716446655440000
# 6ba7b8109dad11d180b400c04fd430c8
```

### JSON Output

```bash
idt convert 550e8400-e29b-41d4-a716-446655440000 -f hex --json
# Output: "550e8400e29b41d4a716446655440000"
```

### Type Hints

For ambiguous inputs, specify the source type:

```bash
# Interpret as UUID (no dashes)
idt convert -t uuid 550e8400e29b41d4a716446655440000 -f canonical
# Output: 550e8400-e29b-41d4-a716-446655440000
```
