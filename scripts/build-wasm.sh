#!/bin/bash
set -e
wasm-pack build crates/idt-wasm --target web --out-dir ../../docs/src/playground/pkg --no-typescript
