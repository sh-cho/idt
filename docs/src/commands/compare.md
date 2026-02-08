# compare - Compare IDs

Compare two identifiers to understand their relationship in terms of binary ordering, lexicographic ordering, and chronological ordering (for time-based IDs).

## Usage

```bash
idt compare [OPTIONS] <ID1> <ID2>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `ID1` | First ID to compare |
| `ID2` | Second ID to compare |

## Options

| Option | Description |
|--------|-------------|
| `-t, --type <TYPE>` | ID type (auto-detect if omitted) |

## Comparison Types

| Comparison | Description |
|------------|-------------|
| Binary | Byte-by-byte comparison of raw ID data |
| Lexicographic | String comparison of canonical forms |
| Chronological | Time comparison (for time-based IDs) |

## Examples

### Basic Comparison

```bash
idt compare 019c04e5-6118-7b22-95cb-a10e84dad469 019c04e5-6119-7000-8000-000000000000
```

Output:
```
Comparing IDs:
  ID 1:                019c04e5-6118-7b22-95cb-a10e84dad469
  ID 2:                019c04e5-6119-7000-8000-000000000000

Comparison Results:
  Binary:              ID1 < ID2
  Lexicographic:       ID1 < ID2
  Chronological:       ID1 is older (created before ID2)
  Time difference:     1.00 seconds
```

### Comparing Different Types

```bash
idt compare 01ARZ3NDEKTSV4RRFFQ69G5FAV 01ARZ3NDEKTSV4RRFFQ69G5FAW
```

When comparing IDs of different types, idt warns you:

```bash
idt compare 550e8400-e29b-41d4-a716-446655440000 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

Output:
```
Comparing IDs:
  ID 1:                550e8400-e29b-41d4-a716-446655440000
  ID 2:                01ARZ3NDEKTSV4RRFFQ69G5FAV

  Warning: Different types! (uuidv4 vs ulid)

Comparison Results:
  Binary:              ID1 > ID2
  Lexicographic:       ID1 > ID2
```

### Time-Based Comparison

For IDs with timestamps (UUIDv1, UUIDv6, UUIDv7, ULID, Snowflake), idt shows chronological comparison:

```bash
# Generate two UUIDv7s with a delay
ID1=$(idt gen uuidv7)
sleep 2
ID2=$(idt gen uuidv7)
idt compare "$ID1" "$ID2"
```

Output:
```
Comparison Results:
  Binary:              ID1 < ID2
  Lexicographic:       ID1 < ID2
  Chronological:       ID1 is older (created before ID2)
  Time difference:     2.00 seconds
```

### Comparison Symbols

The output uses comparison symbols:

| Symbol | Meaning |
|--------|---------|
| `<` | ID1 is less than ID2 |
| `>` | ID1 is greater than ID2 |
| `=` | IDs are equal |

### JSON Output

```bash
idt compare id1 id2 --json
```

Output:
```json
{
  "id1": "019c04e5-6118-7b22-95cb-a10e84dad469",
  "id2": "019c04e5-6119-7000-8000-000000000000",
  "type1": "uuidv7",
  "type2": "uuidv7",
  "binary_order": "less",
  "lexicographic_order": "less",
  "chronological_order": "less",
  "time_diff_ms": 1000,
  "timestamp1": 1706450267416,
  "timestamp2": 1706450268416
}
```

### Use Cases

**Verify sortability:**
```bash
# Check if time-sortable IDs maintain order
idt compare "$OLDER_ID" "$NEWER_ID"
# Should show: ID1 < ID2 for all comparisons
```

**Debug ordering issues:**
```bash
# When IDs aren't sorting as expected
idt compare "$ID_A" "$ID_B" --json | jq '.binary_order, .lexicographic_order'
```

**Time difference calculation:**
```bash
# Find time between two events
idt compare "$START_ID" "$END_ID" --json | jq '.time_diff_ms'
```
