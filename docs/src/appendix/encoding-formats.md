# Encoding Formats

idt supports various encoding formats for converting and displaying IDs.

## Available Formats

| Format | Description | Example (128-bit UUID) |
|--------|-------------|------------------------|
| `canonical` | Original/standard format | `550e8400-e29b-41d4-a716-446655440000` |
| `hex` | Hexadecimal (lowercase) | `550e8400e29b41d4a716446655440000` |
| `base32` | RFC 4648 Base32 | `KUHIBAASSNE5JJYWIRDFKRAAAA` |
| `base58` | Bitcoin-style Base58 | `6K8FVbLqP4V8nDqTJNXH6k` |
| `base64` | Standard Base64 | `VQ6EAOKbQdSnFkRmVUQAAA==` |
| `base64url` | URL-safe Base64 | `VQ6EAOKbQdSnFkRmVUQAAA` |
| `bits` | Binary string | `01010101000011101000...` |
| `int` | Integer representation | `113059749145936325402354257176981405696` |
| `bytes` | Space-separated hex bytes | `55 0e 84 00 e2 9b 41 d4...` |

## Format Details

### Canonical

The standard format for each ID type:
- **UUID**: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` (36 characters)
- **ULID**: 26 Crockford Base32 characters
- **NanoID**: Variable length (default 21 characters)
- **Snowflake**: Decimal integer

```bash
idt convert "$ID" -f canonical
```

### Hexadecimal (hex)

Raw bytes as hexadecimal characters (lowercase).

- **Length**: 2 characters per byte
- **Characters**: `0-9`, `a-f`
- **Use case**: Database storage, binary comparison

```bash
idt convert 550e8400-e29b-41d4-a716-446655440000 -f hex
# Output: 550e8400e29b41d4a716446655440000
```

### Base32

RFC 4648 Base32 encoding (without padding).

- **Characters**: `A-Z`, `2-7`
- **Use case**: Case-insensitive encoding, DNS-safe

```bash
idt convert 550e8400-e29b-41d4-a716-446655440000 -f base32
# Output: KUHIBAASSNE5JJYWIRDFKRAAAA
```

### Base58

Bitcoin-style Base58 encoding (excludes 0, O, I, l).

- **Characters**: `1-9`, `A-H`, `J-N`, `P-Z`, `a-k`, `m-z`
- **Use case**: User-friendly, avoids ambiguous characters

```bash
idt convert 550e8400-e29b-41d4-a716-446655440000 -f base58
# Output: 6K8FVbLqP4V8nDqTJNXH6k
```

### Base64

Standard Base64 encoding with padding.

- **Characters**: `A-Z`, `a-z`, `0-9`, `+`, `/`, `=`
- **Use case**: Email-safe encoding, general data encoding

```bash
idt convert 550e8400-e29b-41d4-a716-446655440000 -f base64
# Output: VQ6EAOKbQdSnFkRmVUQAAA==
```

### Base64 URL-safe (base64url)

URL-safe Base64 without padding.

- **Characters**: `A-Z`, `a-z`, `0-9`, `-`, `_`
- **Use case**: URLs, filenames, HTTP headers

```bash
idt convert 550e8400-e29b-41d4-a716-446655440000 -f base64url
# Output: VQ6EAOKbQdSnFkRmVUQAAA
```

### Binary (bits)

Binary string representation.

- **Characters**: `0`, `1`
- **Use case**: Bit-level analysis, debugging

```bash
idt convert 550e8400-e29b-41d4-a716-446655440000 -f bits
# Output: 0101010100001110100001000000000011100010...
```

### Integer (int)

Decimal integer representation.

- **Characters**: `0-9`
- **Use case**: Numeric comparison, database integer columns

```bash
idt convert 550e8400-e29b-41d4-a716-446655440000 -f int
# Output: 113059749145936325402354257176981405696
```

Note: 128-bit IDs produce very large integers that may overflow in some languages.

### Bytes

Space-separated hexadecimal bytes.

- **Use case**: Byte-level inspection, debugging

```bash
idt convert 550e8400-e29b-41d4-a716-446655440000 -f bytes
# Output: 55 0e 84 00 e2 9b 41 d4 a7 16 44 66 55 44 00 00
```

## Comparison Table

| Format | Length (128-bit) | URL-Safe | Human-Readable |
|--------|------------------|----------|----------------|
| canonical | 36 | No | Good |
| hex | 32 | Yes | Fair |
| base32 | 26 | Yes | Fair |
| base58 | ~22 | Yes | Good |
| base64 | 24 | No | Fair |
| base64url | 22 | Yes | Fair |
| bits | 128 | Yes | Poor |
| int | ~39 | Yes | Poor |
| bytes | 47 | No | Good |

## Usage Examples

### Convert Command

```bash
# Convert to specific format
idt convert "$ID" -f hex
idt convert "$ID" -f base64
idt convert "$ID" -f base58

# With case transformation
idt convert "$ID" -f hex -U    # Uppercase
idt convert "$ID" -f hex -L    # Lowercase
```

### Generate with Format

```bash
# Generate in specific format
idt gen uuid -f hex
idt gen uuidv7 -f base64
```

### Inspect Output

The inspect command shows multiple formats automatically:

```bash
$ idt inspect 550e8400-e29b-41d4-a716-446655440000
UUIDV4
  550e8400-e29b-41d4-a716-446655440000

  Hex        550e8400e29b41d4a716446655440000
  Base64     VQ6EAOKbQdSnFkRmVUQAAA==
  Int        113059749145936325402354257176981405696
```

## Choosing a Format

| Use Case | Recommended Format |
|----------|-------------------|
| Database storage | `hex` or `canonical` |
| URLs and APIs | `base64url` or `base58` |
| Display to users | `canonical` or `base58` |
| Compact storage | `base58` or `base64url` |
| Cross-system compatibility | `hex` |
| Debugging | `bytes` or `bits` |
