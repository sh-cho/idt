# ID Types Comparison

A comprehensive comparison of all ID types supported by idt.

## Quick Reference

| Type | Bits | Sortable | Timestamp | Random Bits | String Length |
|------|------|----------|-----------|-------------|---------------|
| UUIDv1 | 128 | No | Yes | ~14 | 36 |
| UUIDv3 | 128 | No | No | 0 | 36 |
| UUIDv4 | 128 | No | No | 122 | 36 |
| UUIDv5 | 128 | No | No | 0 | 36 |
| UUIDv6 | 128 | Yes | Yes | ~62 | 36 |
| UUIDv7 | 128 | Yes | Yes | 62 | 36 |
| ULID | 128 | Yes | Yes | 80 | 26 |
| NanoID | ~126 | No | No | ~126 | 21 |
| Snowflake | 64 | Yes | Yes | 0 | ~19 |
| KSUID | 160 | Yes | Yes | 128 | 27 |
| ObjectId | 96 | Partial | Yes | ~40 | 24 |
| XID | 96 | Yes | Yes | ~40 | 20 |
| TSID | 64 | Yes | Yes | ~22 | 13-17 |

## Detailed Comparison

### Timestamp Precision

| Type | Precision | Epoch |
|------|-----------|-------|
| UUIDv1 | 100 nanoseconds | October 15, 1582 |
| UUIDv6 | 100 nanoseconds | October 15, 1582 |
| UUIDv7 | Milliseconds | Unix epoch (1970) |
| ULID | Milliseconds | Unix epoch (1970) |
| Snowflake | Milliseconds | Custom (e.g., 2010, 2015) |
| KSUID | Seconds | May 13, 2014 |
| ObjectId | Seconds | Unix epoch (1970) |
| XID | Seconds | Unix epoch (1970) |
| TSID | Milliseconds | Unix epoch (1970) |

### Collision Resistance

| Type | Same Millisecond | Different Nodes | Notes |
|------|------------------|-----------------|-------|
| UUIDv4 | Excellent | Excellent | 122 random bits |
| UUIDv7 | Good | Excellent | 62 random bits + timestamp |
| ULID | Excellent | Excellent | Monotonic + 80 random bits |
| NanoID | Excellent | Excellent | ~126 random bits |
| Snowflake | Limited | Requires coordination | 4096/ms per worker |

### Encoding

| Type | Encoding | Case Sensitive | Alphabet |
|------|----------|----------------|----------|
| UUID | Hexadecimal | No | 0-9, a-f |
| ULID | Crockford Base32 | No | 0-9, A-Z (excluding I, L, O, U) |
| NanoID | Custom (default URL-safe) | Yes | A-Z, a-z, 0-9, -, _ |
| Snowflake | Decimal | N/A | 0-9 |
| KSUID | Base62 | Yes | 0-9, A-Z, a-z |
| ObjectId | Hexadecimal | No | 0-9, a-f |
| XID | Base32 | No | 0-9, a-v |

## Use Case Recommendations

### Database Primary Keys

| Requirement | Recommended Type |
|-------------|------------------|
| UUID compatibility required | UUIDv7 |
| Compact storage | ULID or Snowflake |
| Time-sortable queries | UUIDv7, ULID, or Snowflake |
| Maximum randomness | UUIDv4 |
| 64-bit integer column | Snowflake or TSID |

### Distributed Systems

| Requirement | Recommended Type |
|-------------|------------------|
| No coordination needed | UUIDv7, ULID, UUIDv4 |
| Very high throughput | Snowflake |
| Cross-datacenter | UUIDv7 or ULID |
| Compact wire format | Snowflake (64-bit) |

### User-Facing IDs

| Requirement | Recommended Type |
|-------------|------------------|
| Short URL slugs | NanoID (custom length) |
| Readable codes | NanoID (custom alphabet) |
| Compact but sortable | ULID |
| Standard format | UUID (canonical) |

### Security-Sensitive

| Requirement | Recommended Type |
|-------------|------------------|
| Non-sequential | UUIDv4 or NanoID |
| Hide creation time | UUIDv4 or NanoID |
| Cryptographic randomness | UUIDv4 or NanoID |

## Format Conversion Compatibility

Since ULID and UUID are both 128-bit, they can be converted between each other:

```bash
# ULID to UUID hex
idt convert 01ARZ3NDEKTSV4RRFFQ69G5FAV -f hex
# Can be stored in UUID column

# UUID to various formats
idt convert 550e8400-e29b-41d4-a716-446655440000 -f base64
```

## Storage Size Comparison

| Type | Binary (bytes) | String (chars) | Typical DB Type |
|------|----------------|----------------|-----------------|
| UUID | 16 | 36 | UUID, CHAR(36), BINARY(16) |
| ULID | 16 | 26 | CHAR(26), BINARY(16) |
| NanoID | ~16 | 21 | VARCHAR(21+) |
| Snowflake | 8 | ~19 | BIGINT |
| ObjectId | 12 | 24 | ObjectId, CHAR(24) |

## Feature Matrix

| Feature | UUIDv4 | UUIDv7 | ULID | NanoID | Snowflake |
|---------|--------|--------|------|--------|-----------|
| Time-sortable | - | Yes | Yes | - | Yes |
| Extract timestamp | - | Yes | Yes | - | Yes |
| URL-safe | - | - | Yes | Yes | Yes |
| Fits in 64-bit | - | - | - | - | Yes |
| No coordination | Yes | Yes | Yes | Yes | - |
| Customizable | - | - | - | Yes | Yes |
| RFC standard | Yes | Yes | - | - | - |
| Widely supported | Yes | Growing | Growing | Growing | Yes |

## Specification Links

| Type | Specification |
|------|---------------|
| UUID (v1-v5) | [RFC 4122](https://datatracker.ietf.org/doc/html/rfc4122) |
| UUID (v6-v8) | [RFC 9562](https://datatracker.ietf.org/doc/html/rfc9562) |
| ULID | [github.com/ulid/spec](https://github.com/ulid/spec) |
| NanoID | [github.com/ai/nanoid](https://github.com/ai/nanoid) |
| Snowflake | [Wikipedia](https://en.wikipedia.org/wiki/Snowflake_ID) |
| KSUID | [github.com/segmentio/ksuid](https://github.com/segmentio/ksuid) |
| ObjectId | [MongoDB Docs](https://www.mongodb.com/docs/manual/reference/method/ObjectId/) |
| XID | [github.com/rs/xid](https://github.com/rs/xid) |
| TypeID | [github.com/jetify-com/typeid](https://github.com/jetify-com/typeid) |
