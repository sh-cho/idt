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
# Default Snowflake (Unix epoch, Twitter bit layout)
idt gen snowflake

# With machine ID
idt gen snowflake --machine-id 1

# With datacenter ID
idt gen snowflake --datacenter-id 2

# Both
idt gen snowflake --machine-id 1 --datacenter-id 2
```

### Presets

Use `--preset` to select a complete Snowflake configuration (bit layout, epoch, and timestamp resolution):

```bash
# Twitter (41t + 5dc + 5worker + 12seq, ms, Twitter epoch)
idt gen snowflake --preset twitter

# Discord (same layout as Twitter, Discord epoch)
idt gen snowflake --preset discord

# Instagram (41t + 13shard + 10seq, ms, Instagram epoch)
idt gen snowflake --preset instagram --field shard_id=42

# Sonyflake (39t + 8seq + 16machine, 10ms resolution)
idt gen snowflake --preset sonyflake --machine-id 100

# Mastodon (48t + 16seq, ms, Unix epoch)
idt gen snowflake --preset mastodon
```

### Custom Epochs (backward compatible)

You can also use `--epoch` for backward compatibility. This uses the Twitter bit layout with the specified epoch:

```bash
# Twitter epoch: Nov 4, 2010
idt gen snowflake --epoch twitter

# Discord epoch: Jan 1, 2015
idt gen snowflake --epoch discord

# Custom epoch (milliseconds since Unix epoch)
idt gen snowflake --epoch 1420070400000
```

> **Note:** `--preset` and `--epoch` cannot be used together.

### Custom Field Values

Use `--field` to set arbitrary field values based on the active layout:

```bash
# Set shard_id for Instagram layout
idt gen snowflake --preset instagram --field shard_id=42

# Set machine_id for Sonyflake (0-65535)
idt gen snowflake --preset sonyflake --field machine_id=1000
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

### Inspecting with Presets

Use `--preset` to decode with the correct bit layout, epoch, and timestamp resolution:

```bash
# Twitter Snowflake
idt inspect --preset twitter 1234567890123456789

# Discord Snowflake
idt inspect --preset discord 1474004412518240339

# Instagram Snowflake (shows shard_id in components)
idt inspect --preset instagram 3852470500357875712

# Sonyflake (10ms resolution)
idt inspect --preset sonyflake 610591162520043520

# Mastodon
idt inspect --preset mastodon 116226149176639488
```

### Inspecting with Custom Epochs (backward compatible)

You can also use `--epoch` for backward compatibility:

```bash
# Discord Snowflake
idt inspect -t snowflake --epoch discord 1474004412518240339

# Twitter Snowflake
idt inspect -t snowflake --epoch twitter 1234567890123456789

# Custom epoch in milliseconds
idt inspect -t snowflake --epoch 1420070400000 1474004412518240339
```

Without `--preset` or `--epoch`, the Unix epoch (0) and Twitter bit layout are used.

## Built-in Presets

| Preset | Layout (MSB→LSB) | Epoch (ms) | Timestamp Unit |
|--------|-------------------|------------|----------------|
| `twitter` | 41t + 5dc + 5worker + 12seq | 1288834974657 | ms |
| `discord` | 41t + 5dc + 5worker + 12seq | 1420070400000 | ms |
| `instagram` | 41t + 13shard + 10seq | 1314220021721 | ms |
| `sonyflake` | 39t + 8seq + 16machine | 1409529600000 | 10ms |
| `mastodon` | 48t + 16seq | 0 | ms |

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

Different systems use different bit layouts, epochs, and timestamp resolutions. All are supported via `--preset`:

| System | Timestamp | Other Fields | Sequence | Resolution |
|--------|-----------|-------------|----------|------------|
| Twitter | 41 | 5 DC + 5 worker | 12 | ms |
| Discord | 41 | 5 DC + 5 worker | 12 | ms |
| Instagram | 41 | 13 shard | 10 | ms |
| Sonyflake | 39 | 16 machine | 8 | 10ms |
| Mastodon | 48 | — | 16 | ms |

## Specification

- Original announcement: https://blog.twitter.com/engineering/en_us/a/2010/announcing-snowflake
- Wikipedia: https://en.wikipedia.org/wiki/Snowflake_ID
