# validate - Validate IDs

Check if input strings are valid identifiers.

## Usage

```bash
idt validate [OPTIONS] [ID]...
```

## Arguments

| Argument | Description |
|----------|-------------|
| `ID` | ID(s) to validate (reads from stdin if omitted) |

## Options

| Option | Description |
|--------|-------------|
| `-t, --type <TYPE>` | Expected ID type (any valid if omitted) |
| `-q, --quiet` | No output, only exit code |
| `--strict` | Strict validation (reject non-canonical forms) |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All IDs are valid |
| 1 | One or more IDs are invalid |

## Examples

### Basic Validation

```bash
# Validate a UUID
idt validate 550e8400-e29b-41d4-a716-446655440000
# Output: 550e8400-e29b-41d4-a716-446655440000: valid (uuidv4)

# Validate a ULID
idt validate 01ARZ3NDEKTSV4RRFFQ69G5FAV
# Output: 01ARZ3NDEKTSV4RRFFQ69G5FAV: valid (ulid)

# Invalid ID
idt validate not-a-valid-id
# Output: not-a-valid-id: invalid
#   Error: Not a recognized ID format
```

### Type-Specific Validation

```bash
# Must be a UUID
idt validate -t uuid 550e8400-e29b-41d4-a716-446655440000
# Output: valid (uuid)

# Must be a ULID
idt validate -t ulid 01ARZ3NDEKTSV4RRFFQ69G5FAV
# Output: valid (ulid)

# Wrong type
idt validate -t ulid 550e8400-e29b-41d4-a716-446655440000
# Output: invalid (expected ulid)
```

### Assigned IDs

Validate product, publication, and financial identifiers:

```bash
# EAN-13 barcode
idt validate -t ean13 4006381333931
# Output: 4006381333931: valid (ean13)

# ISBN-13 (with or without hyphens)
idt validate -t isbn13 978-0-306-40615-7
# Output: 978-0-306-40615-7: valid (isbn13)

# ISBN-10 (check digit can be X)
idt validate -t isbn10 080442957X
# Output: 080442957X: valid (isbn10)

# ISIN (securities identifier)
idt validate -t isin US0378331005
# Output: US0378331005: valid (isin)

# EAN-8 barcode
idt validate -t ean8 96385074

# UPC-A barcode
idt validate -t upca 036000291452

# ISSN (serial publications)
idt validate -t issn 0378-5955

# ISMN (music publications)
idt validate -t ismn 979-0-060-11561-5

# ISNI (name identifiers)
idt validate -t isni "0000 0001 2103 2683"

# GTIN-14 (trade items)
idt validate -t gtin14 10614141000415

# ASIN (Amazon products)
idt validate -t asin B08N5WRWNW

# Auto-detection also works
idt validate 9780306406157
# Output: 9780306406157: valid (isbn13)
```

### Strict Mode

Strict mode rejects non-canonical forms:

```bash
# Canonical form - passes
idt validate --strict 550e8400-e29b-41d4-a716-446655440000

# Uppercase - fails strict validation
idt validate --strict 550E8400-E29B-41D4-A716-446655440000
# Output: invalid
#   Error: Non-canonical form
#   Hint: Canonical form: 550e8400-e29b-41d4-a716-446655440000
```

### Quiet Mode

For scripting, use quiet mode to check exit codes only:

```bash
# Check if valid
if idt validate -q "$ID"; then
    echo "Valid ID: $ID"
else
    echo "Invalid ID: $ID"
fi

# Validate and continue only if valid
idt validate -q "$ID" && process_id "$ID"
```

### Validating Multiple IDs

```bash
# Multiple arguments
idt validate id1 id2 id3

# From file
cat ids.txt | idt validate

# From generated IDs
idt gen uuid -n 100 | idt validate
```

### Helpful Hints

idt provides hints for common mistakes:

```bash
# UUID without dashes
idt validate 550e8400e29b41d4a716446655440000
# Output: invalid
#   Hint: Looks like UUID without dashes. Try adding dashes.

# Invalid characters in UUID
idt validate 550e8400-e29b-41d4-a716-44665544000g
# Output: invalid
#   Hint: Check for invalid characters in UUID.
```

### Structured Output (JSON, YAML, TOML)

```bash
# JSON output
idt validate 550e8400-e29b-41d4-a716-446655440000 --output json
idt validate 550e8400-e29b-41d4-a716-446655440000 --json    # shorthand
# Output: {"input":"550e8400-...","valid":true,"id_type":"uuidv4"}

# YAML output
idt validate 550e8400-e29b-41d4-a716-446655440000 --output yaml

# TOML output
idt validate 550e8400-e29b-41d4-a716-446655440000 --output toml

# Multiple IDs
idt validate id1 id2 --json
# Output: [{"input":"id1",...},{"input":"id2",...}]
```

### Batch Validation

Validate a file of IDs:

```bash
# Count valid/invalid
cat ids.txt | idt validate 2>&1 | grep -c "valid"

# Extract only valid IDs
cat ids.txt | while read id; do
    if idt validate -q "$id" 2>/dev/null; then
        echo "$id"
    fi
done
```
