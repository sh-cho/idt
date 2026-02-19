# Other ID Types

This page covers additional ID types that idt can inspect and work with.

## KSUID

K-Sortable Unique Identifier - a 160-bit identifier from Segment.

| Property | Value |
|----------|-------|
| Bits | 160 |
| Sortable | Yes |
| Timestamp | Yes (second precision) |
| Format | Base62 |
| Length | 27 characters |

### Format

```
0ujsswThIGTUYm2K8FjOOfXtY1K
|-------------------------|
  27 Base62 characters
```

### Structure

- Timestamp: 32 bits (seconds since epoch)
- Payload: 128 bits (random)

### Characteristics

- Second-level timestamp precision
- 128 bits of randomness
- Base62 encoding (alphanumeric, case-sensitive)

### Specification

https://github.com/segmentio/ksuid

---

## MongoDB ObjectId

MongoDB's default document identifier.

| Property | Value |
|----------|-------|
| Bits | 96 |
| Sortable | Partial |
| Timestamp | Yes (second precision) |
| Format | Hexadecimal |
| Length | 24 characters |

### Format

```
507f1f77bcf86cd799439011
|----------------------|
  24 hex characters
```

### Structure

- Timestamp: 32 bits (Unix seconds)
- Machine identifier: 24 bits
- Process ID: 16 bits
- Counter: 24 bits

### Inspection

```bash
idt inspect 507f1f77bcf86cd799439011
```

### Specification

https://www.mongodb.com/docs/manual/reference/method/ObjectId/

---

## TypeID

Type-prefixed, sortable identifiers.

| Property | Value |
|----------|-------|
| Bits | 128 |
| Sortable | Yes |
| Timestamp | Yes |
| Format | Prefix + Base32 |

### Format

```
user_01h455vb4pex5vsknk084sn02q
|---|-------------------------|
prefix     UUIDv7 in Base32
```

### Characteristics

- Type prefix for clarity (e.g., `user_`, `order_`)
- UUIDv7 encoded in Base32
- Type-safe across systems

### Specification

https://github.com/jetify-com/typeid

---

## XID

Globally unique, sortable 96-bit identifier.

| Property | Value |
|----------|-------|
| Bits | 96 |
| Sortable | Yes |
| Timestamp | Yes (second precision) |
| Format | Base32 |
| Length | 20 characters |

### Format

```
9m4e2mr0ui3e8a215n4g
|------------------|
 20 Base32 characters
```

### Structure

- Timestamp: 32 bits (Unix seconds)
- Machine ID: 24 bits
- Process ID: 16 bits
- Counter: 24 bits

### Characteristics

- Compact (20 characters)
- URL-safe
- Inspired by MongoDB ObjectId

### Specification

https://github.com/rs/xid

---

## CUID

Collision-resistant Unique Identifier.

| Property | Value |
|----------|-------|
| Bits | ~128 |
| Sortable | Partial |
| Timestamp | Yes |
| Format | Custom Base36 |

### Format

```
cjld2cjxh0000qzrmn831i7rn
|-----------------------|
  25+ characters
```

### Structure

- `c` prefix
- Timestamp
- Counter
- Client fingerprint
- Random block

### Specification

https://github.com/paralleldrive/cuid

---

## CUID2

Secure, collision-resistant identifier (CUID successor).

| Property | Value |
|----------|-------|
| Bits | Variable |
| Sortable | No |
| Timestamp | No |
| Format | Base36 |
| Default length | 24 characters |

### Characteristics

- Cryptographically secure
- No timestamp (privacy-focused)
- Configurable length

### Specification

https://github.com/paralleldrive/cuid2

---

## TSID

Time-Sorted Unique Identifier.

| Property | Value |
|----------|-------|
| Bits | 64 |
| Sortable | Yes |
| Timestamp | Yes |
| Format | Base32 or numeric |

### Format

```
0HXNP0P6V80G8 (Base32)
38352658567418876 (numeric)
```

### Structure

- Timestamp: 42 bits
- Node: 10 bits
- Counter: 12 bits

### Characteristics

- 64-bit (fits in long integer)
- Similar to Snowflake but with different encoding options
- ~139 years of timestamps

### Specification

https://github.com/f4b6a3/tsid-creator

---

## Comparison Table

| Type | Bits | Sortable | Timestamp | Length |
|------|------|----------|-----------|--------|
| KSUID | 160 | Yes | Seconds | 27 |
| ObjectId | 96 | Partial | Seconds | 24 |
| TypeID | 128 | Yes | Millis | Variable |
| XID | 96 | Yes | Seconds | 20 |
| CUID | ~128 | Partial | Yes | 25+ |
| CUID2 | Variable | No | No | 24 |
| TSID | 64 | Yes | Millis | 13-17 |

## Support Status

These ID types have varying levels of support in idt:

| Type | Generate | Inspect | Convert | Validate |
|------|----------|---------|---------|----------|
| KSUID | Planned | Partial | Partial | Yes |
| ObjectId | Planned | Partial | Partial | Yes |
| TypeID | Planned | Partial | Partial | Yes |
| XID | Planned | Partial | Partial | Yes |
| CUID | No | Partial | Partial | Yes |
| CUID2 | No | Partial | Partial | Yes |
| TSID | Planned | Partial | Partial | Yes |

"Partial" means the feature works for basic cases but may not support all options.
