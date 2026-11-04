# `vyapaar-ffi` — TDD & Engineering Roadmap

## Milestone 1: Schema & Stub Codegen (`vyapaar-ffi-codegen`) (P0)
- [ ] Port `vyapaar-ffi-codegen` from `honba-tamashii-codegen`.
- [ ] Generate `domain_schema.json` and `_vyapaar.pyi` stubs.

## Milestone 2: PyO3 Extension & Arrow Bridge (`vyapaar-ffi-py`) (P0)
- [ ] Port `vyapaar-ffi-py` from `honba-tamashii-py`.
- [ ] Verify `Decimal` round-tripping, GIL release (`py.allow_threads`), `catch_unwind` sandboxing, and Arrow PyCapsule zero-copy export.

## Milestone 3: Python `vyapaar` Package & CLI (P0)
- [ ] Port `python/honba/` to `python/vyapaar/` and `pyproject.toml` (`maturin` build configuration).
- [ ] Run Rust `cargo test --workspace` and Python `pytest`.
