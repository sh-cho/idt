# Fuzz Testing

This directory contains [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) targets and their corpus data.

## Fuzz Targets

| Target | Description |
|---|---|
| `fuzz_detect` | Fuzzes `detect_id_type()` with arbitrary strings |
| `fuzz_parse_id` | Fuzzes `parse_id()` with no type hint |
| `fuzz_parse_id_with_hint` | Fuzzes `parse_id()` with a randomly chosen `IdKind` hint |
| `fuzz_roundtrip` | Fuzzes round-trip parsing consistency |

## Running

```bash
# Run a specific target (runs until stopped or crash found)
cargo fuzz run fuzz_detect

# Run with a time limit (seconds)
cargo fuzz run fuzz_detect -- -max_total_time=60

# Reproduce a crash
cargo fuzz run fuzz_detect fuzz/artifacts/fuzz_detect/<artifact>
```

## Corpus

Each target has a `corpus/<target>/` directory containing seed inputs. libFuzzer uses these as starting points for mutation-based fuzzing.

Corpus filenames are the **SHA-1 hash** of the file content, which is the default naming convention used by libFuzzer.

## Artifacts

Crash-reproducing inputs are written to `artifacts/<target>/` by libFuzzer. These are not checked in -- instead, the crashing inputs should be added to the corresponding `corpus/` directory so they serve as regression seeds for future runs.
