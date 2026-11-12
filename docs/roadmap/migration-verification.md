# `vyapaar-ffi` — Migration Verification Milestone

Verification commands for the `honba-tamashii → vyapaar-ffi` migration:

```bash
# Run from the repository root
grep -rn "#\[cfg(test)\]" crates/*/src/ && exit 1 || true
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
```

## Migration Milestone Status

| Milestone | Status |
|-----------|--------|
| Schema & Stub Codegen (`vyapaar-ffi-codegen`) | DONE — generates `domain_schema.json`, `openapi.json`, `_vyapaar.pyi` |
| PyO3 Extension & Arrow Bridge (`vyapaar-ffi-py`) | DONE — Decimal round-trips, GIL release, Arrow PyCapsule |
| Python `vyapaar` Package & CLI | DONE — maturin ABI3 wheel, `pytest` green |
| Zero inline `#[cfg(test)]` in `src/` | Complete |
| `cargo fmt --check` | Clean |
| `cargo clippy --all-targets -- -D warnings` | Clean |
| `cargo test --workspace` | Passing |
| `pytest -q` | Passing |
