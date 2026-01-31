# ULID

ULID (Universally Unique Lexicographically Sortable Identifier) is a 128-bit identifier designed for distributed systems that need sortable, URL-friendly IDs.

## Overview

| Property | Value |
|----------|-------|
| Bits | 128 |
| Sortable | Yes |
| Timestamp | Yes (millisecond precision) |
| Format | Crockford Base32 |
| Length | 26 characters |

## Format

```
 01ARZ3NDEKTSV4RRFFQ69G5FAV
 |----------|--------------|
  Timestamp      Random
  (48 bits)     (80 bits)
```

Example: `01ARZ3NDEKTSV4RRFFQ69G5FAV`

## Characteristics

### Time-Sortable

ULIDs sort lexicographically in chronological order:

```bash
$ idt gen ulid
01ARZ3NDEKTSV4RRFFQ69G5FAV
$ idt gen ulid
01ARZ3NDEKTSV4RRFFQ69G5FAW
```

The second ULID will always sort after the first.

### Monotonic

Within the same millisecond, ULIDs increment the random component to maintain ordering:

```bash
# Generated in same millisecond
01ARZ3NDEKTSV4RRFFQ69G5FAV
01ARZ3NDEKTSV4RRFFQ69G5FAW  # Random incremented
01ARZ3NDEKTSV4RRFFQ69G5FAX  # Random incremented again
```

### Case-Insensitive

ULIDs use Crockford Base32, which is case-insensitive:

```bash
# These are equivalent
idt inspect 01ARZ3NDEKTSV4RRFFQ69G5FAV
idt inspect 01arz3ndektsv4rrffq69g5fav
```

### URL-Safe

The Crockford Base32 alphabet excludes ambiguous characters (I, L, O, U) and is URL-safe.

## Generation

```bash
# Generate a ULID
idt gen ulid

# Generate multiple ULIDs
idt gen ulid -n 10
```

## Inspection

```bash
$ idt inspect 01ARZ3NDEKTSV4RRFFQ69G5FAV
ULID
  01ARZ3NDEKTSV4RRFFQ69G5FAV

  Time       2016-07-30T23:54:10.259Z
  Random     80 bits

  Hex        01563e3ab5d3d6764c61efb99302bd5b
  Base64     AVY+OrXT1nZMYe+5kwK9Ww==
  Int        1777027686520646174104517696511196507
```

## Conversion

### ULID to UUID

ULIDs are 128-bit and can be converted to UUID format:

```bash
# Get hex representation (same as UUID without dashes)
idt convert 01ARZ3NDEKTSV4RRFFQ69G5FAV -f hex
# Output: 01563e3ab5d3d6764c61efb99302bd5b
```

### To Other Formats

```bash
# Base64
idt convert 01ARZ3NDEKTSV4RRFFQ69G5FAV -f base64

# Integer
idt convert 01ARZ3NDEKTSV4RRFFQ69G5FAV -f int

# Binary
idt convert 01ARZ3NDEKTSV4RRFFQ69G5FAV -f bits
```

## Comparison with UUID

| Feature | ULID | UUIDv4 | UUIDv7 |
|---------|------|--------|--------|
| Bits | 128 | 128 | 128 |
| Sortable | Yes | No | Yes |
| Timestamp | Yes | No | Yes |
| String length | 26 | 36 | 36 |
| Case-sensitive | No | No | No |
| URL-safe | Yes | With encoding | With encoding |

## When to Use ULID

**Good for:**
- Database primary keys (sortable, compact)
- Distributed systems needing time-ordered IDs
- URLs and APIs (shorter than UUID)
- Any use case needing sortable unique IDs

**Consider alternatives if:**
- You need UUID compatibility (use UUIDv7)
- You need sub-millisecond precision
- You need 64-bit IDs (use Snowflake)

## Specification

- GitHub: https://github.com/ulid/spec

## Crockford Base32 Alphabet

```
0123456789ABCDEFGHJKMNPQRSTVWXYZ
```

Excludes: I, L, O, U (to avoid confusion with 1, 1, 0, V)
