# `vyapaar-ffi` — Decimal Conversion Reference

Documents `crates/vyapaar-ffi-py/src/decimal.rs`: `decimal.Decimal` ↔ `rust_decimal::Decimal`
conversion helpers enforcing the Zero Floats Policy across the FFI boundary.

## Conversion Direction

| Direction | Function | Mechanism |
|-----------|----------|-----------|
| Python → Rust | `pydecimal_to_decimal(obj)` | Extract exact string → `Decimal::from_str` |
| Rust → Python | `decimal_to_pydecimal(py, d)` | `d.to_string()` → `decimal.Decimal(s)` |

## Helpers

| Function | Signature | Purpose |
|----------|-----------|---------|
| `object_decimal_string(obj)` | `(&Bound<PyAny>) -> PyResult<String>` | Prefers `to_str()`, falls back to `__str__` |
| `pydecimal_to_decimal(obj)` | `(&Bound<PyAny>) -> PyResult<Decimal>` | Python object → exact `Decimal` |
| `pydecimal_to_string(obj)` | `(&Bound<PyAny>) -> PyResult<String>` | Exact decimal string form |
| `decimal_from_string(py, s)` | `(Python, &str) -> PyResult<Bound<PyAny>>` | Build `decimal.Decimal` from string |
| `decimal_to_pydecimal(py, d)` | `(Python, Decimal) -> PyResult<Bound<PyAny>>` | Build `decimal.Decimal` from `Decimal` |
| `zero_pydecimal(py)` | `(Python) -> PyResult<Bound<PyAny>>` | Canonical `Decimal("0")` |

## Zero Floats Policy

Money is always exchanged as exact decimal strings. Both `decimal.Decimal` and
`rust_decimal::Decimal` are round-trip stable on their exact string forms — no
`f32`/`f64` intermediates anywhere.

## Related

- `crates/vyapaar-ffi-py/src/types.rs` — `Price`, `Quantity` pyclasses use these helpers.
- `crates/vyapaar-ffi-py/src/market.rs` — `Bar`, `Tick` use these for price/volume columns.
