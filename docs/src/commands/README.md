# Commands Overview

idt provides six main commands for working with identifiers:

| Command | Alias | Description |
|---------|-------|-------------|
| [gen](./gen.md) | `g` | Generate new IDs |
| [inspect](./inspect.md) | `i` | Analyze and decode IDs |
| [convert](./convert.md) | `c` | Convert between formats |
| [validate](./validate.md) | `v` | Check if input is valid |
| [compare](./compare.md) | - | Compare two IDs |
| [info](./info.md) | - | Show ID type information |

## Global Options

These options work with all commands:

| Option | Description |
|--------|-------------|
| `-j, --json` | Output in JSON format |
| `-p, --pretty` | Pretty-print JSON output |
| `--no-color` | Disable colored output |
| `-h, --help` | Show help information |
| `-V, --version` | Show version |

## Command Aliases

For faster typing, use command aliases:

```bash
idt g uuid      # Same as: idt gen uuid
idt i <ID>      # Same as: idt inspect <ID>
idt c <ID> -f hex   # Same as: idt convert <ID> -f hex
idt v <ID>      # Same as: idt validate <ID>
```

## Reading from stdin

Most commands that accept IDs can read from stdin:

```bash
# Pipe from another command
idt gen uuid | idt inspect

# Read from file
cat ids.txt | idt validate

# Here-string
idt inspect <<< "550e8400-e29b-41d4-a716-446655440000"
```

## Exit Codes

Commands use standard exit codes:

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (invalid input, validation failure, etc.) |

This makes idt suitable for use in scripts:

```bash
if idt validate -q "$ID"; then
    echo "Valid ID"
else
    echo "Invalid ID"
fi
```
