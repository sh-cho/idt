# AGENTS.md

## Harness — Build, Lint, Test

Always verify if changes passed those

```bash
cargo build

# lint -> defined in .cargo/config.toml
cargo lint
cargo fmt
cargo test

# (optional, requires cargo-llvm-cov)
cargo llvm-cov
```

### Documentation (update if related/needed)

When adding new features, types, or changing behavior, update the relevant docs:

- `README.md`
- `docs/*`
- `CONTRIBUTING.md` — if changing development workflow

### Verification

After all changes:

```bash
cargo fmt
cargo lint
cargo test
```

Optionally rebuild docs:

```bash
mdbook build docs/
```
