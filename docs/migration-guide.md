# `vyapaar-ffi` — Subagent Migration & DDD Refactoring Guide

> **Tier**: Tier 4 (Depends on `vyapaar-core`, `vyapaar-risk`, `vyapaar-market`, `vyapaar-runtime`)
> **Source**: `/home/pmallapp/Devel/Finance/honba-workspace/honba-tamashii`
> **Target**: `/home/pmallapp/Devel/Finance/vyapaar-labs/vyapaar-ffi`

---

## 1. Sub-Crate & Directory Mapping

| Source (`honba-tamashii/`) | Target (`vyapaar-ffi/`) | Action |
| :--- | :--- | :--- |
| `Cargo.toml` | `Cargo.toml` | Update workspace dependencies (`vyapaar-core`, `vyapaar-risk`, `vyapaar-market-finance`, `vyapaar-market-feed`, `vyapaar-market-store`). |
| `pyproject.toml` | `pyproject.toml` | Rename package `honba` → `vyapaar`, module-name `vyapaar._vyapaar`. |
| `crates/honba-tamashii-codegen` | `crates/vyapaar-ffi-codegen` | Replace `honba` → `vyapaar` across codegen templates and tests. |
| `crates/honba-tamashii-py` | `crates/vyapaar-ffi-py` | Rename `#[pymodule] fn _honba` → `fn _vyapaar`; update imports to `vyapaar_*`. |
| `python/honba/` | `python/vyapaar/` | Rename package directory and replace `honba` / `_honba` imports with `vyapaar` / `_vyapaar`. |
| `_honba.pyi`, `__init__.pyi` | `_vyapaar.pyi`, `__init__.pyi` | Update stub files. |

---

## 2. Verification Commands
```bash
cd /home/pmallapp/Devel/Finance/vyapaar-labs/vyapaar-ffi
grep -rn "#\[cfg(test)\]" crates/*/src/ && exit 1 || true
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
```
