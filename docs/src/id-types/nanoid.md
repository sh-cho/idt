# NanoID

NanoID is a tiny, secure, URL-friendly unique string ID generator. It's designed to be compact while maintaining sufficient uniqueness for most use cases.

## Overview

| Property | Value |
|----------|-------|
| Default bits | ~126 |
| Sortable | No |
| Timestamp | No |
| Default length | 21 characters |
| Customizable | Yes (alphabet and length) |

## Format

```
V1StGXR8_Z5jdHi6B-myT
|---------------------|
  21 random characters
```

Example: `V1StGXR8_Z5jdHi6B-myT`

## Characteristics

### URL-Safe

The default alphabet is URL-safe:
```
A-Za-z0-9_-
```

No encoding needed when used in URLs.

### Compact

At 21 characters, NanoID is shorter than UUID (36 chars) and ULID (26 chars).

### Customizable

Both alphabet and length can be customized:

```bash
# Custom length
idt gen nanoid --length 32

# Custom alphabet
idt gen nanoid --alphabet "0123456789abcdef"

# Both
idt gen nanoid --length 16 --alphabet "ABCDEFGHIJKLMNOP"
```

### Secure

Uses cryptographically secure random number generation.

## Generation

```bash
# Default NanoID (21 characters)
idt gen nanoid

# Longer NanoID
idt gen nanoid --length 32

# Multiple NanoIDs
idt gen nanoid -n 10
```

### Custom Alphabets

```bash
# Hex characters only
idt gen nanoid --alphabet "0123456789abcdef"

# Numbers only
idt gen nanoid --alphabet "0123456789"

# Lowercase letters only
idt gen nanoid --alphabet "abcdefghijklmnopqrstuvwxyz"

# Alphanumeric (no special chars)
idt gen nanoid --alphabet "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
```

## Inspection

```bash
$ idt inspect V1StGXR8_Z5jdHi6B-myT
NANOID
  V1StGXR8_Z5jdHi6B-myT

  Hex        ...
  Base64     ...
```

Note: NanoID doesn't embed timestamps or other structured data.

## Collision Probability

With default settings (21 characters, 64-character alphabet):

- ~126 bits of entropy
- For 1% collision probability: ~149 billion IDs
- For 50% collision probability: ~2.4 trillion IDs

### Adjusting for Your Needs

Use this formula to calculate collision probability:

```
bits = log2(alphabet_size) * length
```

Examples:
- 21 chars, 64 alphabet: ~126 bits
- 32 chars, 64 alphabet: ~192 bits
- 16 chars, 16 alphabet (hex): ~64 bits

## Comparison with Other IDs

| Feature | NanoID | UUID | ULID |
|---------|--------|------|------|
| Length | 21 | 36 | 26 |
| Sortable | No | No* | Yes |
| Timestamp | No | No* | Yes |
| URL-safe | Yes | No | Yes |
| Customizable | Yes | No | No |

*UUIDv7 has timestamp and sortability

## When to Use NanoID

**Good for:**
- Short, URL-friendly IDs
- Client-side ID generation
- Non-sequential IDs (no ordering needed)
- Custom ID formats

**Consider alternatives if:**
- You need time-sortable IDs (use ULID or UUIDv7)
- You need to extract timestamps (use ULID or UUIDv7)
- You need distributed coordination (use Snowflake)
- You need UUID compatibility (use UUID)

## Common Configurations

### API Keys

```bash
# Long, alphanumeric
idt gen nanoid --length 32 --alphabet "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
```

### Short Codes

```bash
# 8-character codes (for sharing)
idt gen nanoid --length 8
```

### Numeric IDs

```bash
# Numeric only (for phone-friendly codes)
idt gen nanoid --length 6 --alphabet "0123456789"
```

## Specification

- GitHub: https://github.com/ai/nanoid
- Collision calculator: https://zelark.github.io/nano-id-cc/
