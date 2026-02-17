# Snowflake ID

Snowflake IDs are 64-bit identifiers designed by Twitter for distributed systems requiring high-throughput, time-sortable ID generation.

## Overview

| Property | Value |
|----------|-------|
| Bits | 64 |
| Sortable | Yes |
| Timestamp | Yes (millisecond precision) |
| Format | Decimal integer |
| Coordination | Required (machine/datacenter IDs) |

## Format

```
 1234567890123456789
 |-----------------|
   64-bit integer
```

### Bit Structure (Twitter format)

```
 0 | 41 bits timestamp | 5 bits DC | 5 bits Worker | 12 bits sequence
```

| Field | Bits | Description |
|-------|------|-------------|
| Sign | 1 | Always 0 (positive) |
| Timestamp | 41 | Milliseconds since epoch |
| Datacenter ID | 5 | 0-31 |
| Worker/Machine ID | 5 | 0-31 |
| Sequence | 12 | 0-4095 per millisecond |

## Characteristics

### High Throughput

- 4096 IDs per millisecond per worker
- 4,096,000 IDs per second per worker
- With 32 workers: ~131 million IDs/second

### Time-Sortable

IDs generated later have higher values:

```bash
$ idt gen snowflake
1234567890123456789
$ idt gen snowflake
1234567890123456790  # Greater value
```

### Compact

64 bits fits in a single long integer in most languages.

### Requires Coordination

Unlike UUIDs, Snowflake IDs require assigning machine/datacenter IDs to avoid collisions.

## Generation

```bash
# Default Snowflake
idt gen snowflake

# With machine ID
idt gen snowflake --machine-id 1

# With datacenter ID
idt gen snowflake --datacenter-id 2

# Both
idt gen snowflake --machine-id 1 --datacenter-id 2
```

### Custom Epochs

Different systems use different epochs:

```bash
# Twitter epoch (default): Nov 4, 2010
idt gen snowflake --epoch twitter

# Discord epoch: Jan 1, 2015
idt gen snowflake --epoch discord

# Custom epoch (milliseconds since Unix epoch)
idt gen snowflake --epoch 1420070400000
```

### Environment Variable

Set default epoch via environment:

```bash
export IDT_SNOWFLAKE_EPOCH=discord
idt gen snowflake
```

## Inspection

```bash
$ idt inspect 1234567890123456789
SNOWFLAKE
  1234567890123456789

  Time (UTC)          2023-01-15T12:34:56.789Z
  Local Time (+09:00) 2023-01-15T21:34:56.789+09:00
  Datacenter   1
  Worker       2
  Sequence     789

  Hex          112210f47de98115
  Base64       ESIQr0fpgRU=
  Int          1234567890123456789
```

## Common Epochs

| System | Epoch | Milliseconds |
|--------|-------|--------------|
| Twitter | Nov 4, 2010 | 1288834974657 |
| Discord | Jan 1, 2015 | 1420070400000 |
| Instagram | Jan 1, 2011 | 1293840000000 |

## Comparison with Other IDs

| Feature | Snowflake | UUID | ULID |
|---------|-----------|------|------|
| Bits | 64 | 128 | 128 |
| Sortable | Yes | No* | Yes |
| Timestamp | Yes | No* | Yes |
| Coordination | Required | No | No |
| Throughput | Very high | High | High |

*UUIDv7 has timestamp and sortability

## When to Use Snowflake

**Good for:**
- High-throughput systems (>100k IDs/second)
- Distributed databases
- Systems needing 64-bit IDs
- Real-time analytics (time-ordered)

**Consider alternatives if:**
- You can't coordinate machine IDs (use ULID)
- You need 128-bit IDs (use UUID or ULID)
- You're generating IDs client-side (use NanoID)

## Implementation Notes

### Machine ID Assignment

In production, assign machine IDs via:
- Configuration files
- Environment variables
- Service discovery (e.g., ZooKeeper, Consul)
- Database sequences

### Clock Skew

Snowflake IDs are sensitive to clock skew:
- Use NTP synchronization
- Some implementations wait for the clock to catch up
- Consider using UUIDv7 if clock skew is a concern

### Epoch Selection

Choose an epoch close to your system's start:
- Maximizes the timestamp range
- Twitter's epoch gives ~69 years from 2010
- A 2024 epoch would give ~69 years from 2024

## Variants

Different systems use slightly different bit layouts:

| System | Timestamp | DC/Worker | Sequence |
|--------|-----------|-----------|----------|
| Twitter | 41 | 10 (5+5) | 12 |
| Discord | 42 | 10 (5+5) | 12 |
| Instagram | 41 | 13 | 10 |

## Specification

- Original announcement: https://blog.twitter.com/engineering/en_us/a/2010/announcing-snowflake
- Wikipedia: https://en.wikipedia.org/wiki/Snowflake_ID
