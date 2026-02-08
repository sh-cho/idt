# UUID Family

UUID (Universally Unique Identifier) is a 128-bit identifier standardized by RFC 4122 and updated by RFC 9562. UUIDs are the most widely used identifier format.

## Format

All UUIDs share the same canonical format:

```
xxxxxxxx-xxxx-Mxxx-Nxxx-xxxxxxxxxxxx
```

Where:
- `M` indicates the version (1-8)
- `N` indicates the variant (usually 8, 9, a, or b for RFC 4122)

Example: `550e8400-e29b-41d4-a716-446655440000`

## Versions

### UUIDv1 - Timestamp + MAC Address

- **Specification**: RFC 4122
- **Timestamp**: Yes (100-nanosecond intervals since October 15, 1582)
- **Sortable**: No (timestamp is split across the ID)
- **Privacy**: Contains MAC address

```bash
idt gen uuidv1
# Example: 6ba7b810-9dad-11d1-80b4-00c04fd430c8
```

**Structure:**
- Time-low (32 bits)
- Time-mid (16 bits)
- Version + Time-high (16 bits)
- Variant + Clock sequence (16 bits)
- Node/MAC address (48 bits)

### UUIDv3 - MD5 Namespace Hash

- **Specification**: RFC 4122
- **Timestamp**: No
- **Sortable**: No
- **Deterministic**: Yes (same namespace + name = same UUID)

```bash
# Currently generation is not supported
# Useful for inspecting existing UUIDv3s
idt inspect 5df41881-3aed-3515-88a7-2f4a814cf09e
```

### UUIDv4 - Random

- **Specification**: RFC 4122
- **Timestamp**: No
- **Sortable**: No
- **Random bits**: 122

The most commonly used UUID version. Purely random except for version and variant bits.

```bash
idt gen uuid
# or
idt gen uuidv4
# Example: 550e8400-e29b-41d4-a716-446655440000
```

**Collision probability**: With 122 random bits, you'd need to generate about 2.71 quintillion UUIDs to have a 50% chance of collision.

### UUIDv5 - SHA-1 Namespace Hash

- **Specification**: RFC 4122
- **Timestamp**: No
- **Sortable**: No
- **Deterministic**: Yes (same namespace + name = same UUID)

Similar to UUIDv3 but uses SHA-1 instead of MD5.

```bash
# Currently generation is not supported
# Useful for inspecting existing UUIDv5s
idt inspect 21f7f8de-8051-5b89-8680-0195ef798b6a
```

### UUIDv6 - Reordered Timestamp

- **Specification**: RFC 9562
- **Timestamp**: Yes (same resolution as v1)
- **Sortable**: Yes
- **Privacy**: Contains MAC address (like v1)

Reorders v1's timestamp bits for lexicographic sorting.

```bash
idt gen uuidv6
# Example: 1ec9414c-232a-6b00-b3c8-9e6bdeced846
```

**When to use**: When you need v1 compatibility but want sortable IDs.

### UUIDv7 - Unix Timestamp + Random

- **Specification**: RFC 9562
- **Timestamp**: Yes (Unix milliseconds)
- **Sortable**: Yes
- **Random bits**: 62
- **Recommended**: Yes, for new applications

The recommended choice for new applications needing sortable UUIDs.

```bash
idt gen uuidv7
# Example: 019c04e5-6118-7b22-95cb-a10e84dad469
```

**Structure:**
- Unix timestamp in milliseconds (48 bits)
- Version (4 bits)
- Random (12 bits)
- Variant (2 bits)
- Random (62 bits)

**Advantages:**
- Naturally sorted by creation time
- Compatible with existing UUID infrastructure
- No MAC address (privacy-friendly)
- Sufficient randomness to avoid collisions

### UUID-nil - All Zeros

The nil UUID is all zeros, often used as a placeholder or "no value" indicator.

```bash
idt gen uuid-nil
# Output: 00000000-0000-0000-0000-000000000000
```

### UUID-max - All Ones

The max UUID is all ones, sometimes used as a sentinel value.

```bash
idt gen uuid-max
# Output: ffffffff-ffff-ffff-ffff-ffffffffffff
```

## Inspecting UUIDs

```bash
$ idt inspect 019c04e5-6118-7b22-95cb-a10e84dad469
UUIDV7
  019c04e5-6118-7b22-95cb-a10e84dad469

  Time       2026-01-28T13:57:47.416Z
  Version    7
  Variant    RFC4122
  Random     62 bits

  Hex        019c04e561187b2295cba10e84dad469
  Base64     AZwE5WEYeyKVy6EOhNrUaQ==
  Int        2139325608653621017571381452845274217
```

## Converting UUIDs

```bash
# Remove dashes
idt convert 550e8400-e29b-41d4-a716-446655440000 -f hex

# To Base64
idt convert 550e8400-e29b-41d4-a716-446655440000 -f base64

# To integer
idt convert 550e8400-e29b-41d4-a716-446655440000 -f int
```

## Choosing a UUID Version

| Use Case | Recommended Version |
|----------|---------------------|
| General purpose, no special requirements | UUIDv4 |
| Need sortable IDs | UUIDv7 |
| Deterministic IDs from names | UUIDv5 |
| Legacy system compatibility | UUIDv1 |
| Sortable + v1 compatibility | UUIDv6 |

## Specifications

- **RFC 4122**: Original UUID specification (v1-v5)
  - https://datatracker.ietf.org/doc/html/rfc4122
- **RFC 9562**: New UUID formats (v6-v8)
  - https://datatracker.ietf.org/doc/html/rfc9562
