# `vyapaar-ffi` — TDD & Engineering Roadmap

Status: **all milestones complete.**

## Milestone 1: Schema & Stub Codegen (`vyapaar-ffi-codegen`) (P0) — DONE
- [x] Port `vyapaar-ffi-codegen` from `honba-tamashii-codegen`.
- [x] Generate `domain_schema.json` and `_vyapaar.pyi` stubs.

**Done:** `vyapaar-codegen` deterministically emits `schema/domain_schema.json`, `schema/openapi.json`, and `python/vyapaar/_vyapaar.pyi` directly from `vyapaar-core` wire types; run with `make codegen` and re-generated artifacts are verified in CI.

## Milestone 2: PyO3 Extension & Arrow Bridge (`vyapaar-ffi-py`) (P0) — DONE
- [x] Port `vyapaar-ffi-py` from `honba-tamashii-py`.
- [x] Verify `Decimal` round-tripping, GIL release (`py.allow_threads`), `catch_unwind` sandboxing, and Arrow PyCapsule zero-copy export.

**Done:** value classes (`Price`, `Quantity`, `Bar`, `Tick`, `OrderRequest`, `FillReport`, `Decision`) round-trip exact `decimal.Decimal` across the FFI boundary; `RiskEngine` exposes the full pre-trade gate; `StrategyRunner` releases the GIL during Rust simulation steps and acquires it only for Python callbacks; Arrow `FFI_ArrowArrayStream` capsule export (`bars_to_stream` / `ticks_to_stream`) verified against `pyarrow`.

## Milestone 3: Python `vyapaar` Package & CLI (P0) — DONE
- [x] Port `python/honba/` to `python/vyapaar/` and `pyproject.toml` (`maturin` build configuration).
- [x] Run Rust `cargo test --workspace` and Python `pytest`.

**Done:** `pyproject.toml` builds the ABI3 `vyapaar._vyapaar` wheel via maturin; `make py-build` installs it editable into `.venv` and the `pytest` suite under `python/tests/` is green.
