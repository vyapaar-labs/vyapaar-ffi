# `vyapaar-ffi` — Historical Migration Map (honba-tamashii → vyapaar-ffi)

> **Tier**: Tier 4 (Depends on `vyapaar-core`, `vyapaar-risk`, `vyapaar-market`, `vyapaar-runtime`)
> **Source**: `honba-workspace/honba-tamashii`
> **Target**: `vyapaar-ffi`
> **Status**: Migration complete (all milestones done).

## Sub-Crate & Directory Mapping

| Source (`honba-tamashii/`) | Target (`vyapaar-ffi/`) | Action |
| :--- | :--- | :--- |
| `Cargo.toml` | `Cargo.toml` | Update workspace dependencies (`vyapaar-core`, `vyapaar-risk`, `vyapaar-market-finance`, `vyapaar-market-feed`, `vyapaar-market-store`). |
| `pyproject.toml` | `pyproject.toml` | Rename package `honba` → `vyapaar`, module-name `vyapaar._vyapaar`. |
| `crates/honba-tamashii-codegen` | `crates/vyapaar-ffi-codegen` | Replace `honba` → `vyapaar` across codegen templates and tests. |
| `crates/honba-tamashii-py` | `crates/vyapaar-ffi-py` | Rename `#[pymodule] fn _honba` → `fn _vyapaar`; update imports to `vyapaar_*`. |
| `python/honba/` | `python/vyapaar/` | Rename package directory and replace `honba` / `_honba` imports with `vyapaar` / `_vyapaar`. |
| `_honba.pyi`, `__init__.pyi` | `_vyapaar.pyi`, `__init__.pyi` | Update stub files. |

## Verification Commands

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
| `vyapaar-ffi-codegen` ported (JSON Schema + OpenAPI + `.pyi` stubs) | Complete |
| `vyapaar-ffi-py` ported (PyO3 ABI3 extension `vyapaar._vyapaar`) | Complete |
| `python/vyapaar/` SDK ported (`Strategy`, `Parameter`, `indicators`, `sizing`, `session`, `data`, `cli`) | Complete |
| `pyproject.toml` (maturin ABI3 build config) | Complete |
| Decimal round-tripping verified | Complete |
| GIL release (`py.detach`/`py.allow_threads`) verified | Complete |
| Arrow PyCapsule zero-copy export verified | Complete |
| `cargo test --workspace` | Passing |
| `pytest` under `python/tests/` | Passing |
| Zero inline `#[cfg(test)]` in `src/` | Complete |
| `cargo fmt --check` | Clean |
| `cargo clippy --all-targets -- -D warnings` | Clean |
