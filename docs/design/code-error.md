# `vyapaar-ffi` — Error & Exception Reference

Documents `crates/vyapaar-ffi-py/src/error.rs` and the module-level exceptions
in `lib.rs`.

## Exceptions (registered on `vyapaar._vyapaar`)

| Exception | Purpose |
|-----------|---------|
| `RiskRefused` | Pre-trade risk gate refused an order (`check_order`, `check_order_fast`, `check_decision`, `submit_order`) |
| `ConversionError` | FFI boundary conversion failure (bad decimal, bad timestamp, invalid enum code, invalid JSON grammar) |

Both subclass `PyException` and are created via `pyo3::create_exception!` in
`lib.rs`, then added to the module inside `init_module`.

## Helpers (`error.rs`)

| Helper | Purpose |
|--------|---------|
| `Result<T>` | Alias for `std::result::Result<T, PyErr>` |
| `conversion_err(msg)` | Build a `ConversionError` for a failed conversion |
| `risk_refused(refusal)` | Build a `RiskRefused` embedding the machine-readable rule: `"rule=<rule>: <display>"` |
| `parse_utc(iso)` | Parse RFC3339 → `DateTime<Utc>`, mapping errors to `ConversionError` |

## Related

* `code-risk.md` — where `RiskRefused` is raised.
* `code-pyclass.md` — where `ConversionError` is raised on invalid values.
