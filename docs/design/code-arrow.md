# `vyapaar-ffi` — Apache Arrow Zero-Copy Bridge Reference

Documents `crates/vyapaar-ffi-py/src/arrow_bridge.rs`: exports `RecordBatch` streams
of bars/ticks as a `PyCapsule` implementing the Arrow C Stream Interface.

## Capsule Protocol

- **Capsule name**: `arrow_array_stream`
- **Consumer**: pyarrow (`RecordBatchReader._import_from_c_capsule`) and other Arrow-native Python libraries
- **Ownership**: FFI struct boxed on heap, owned by capsule; destructor drops boxed stream; stream's `Drop` invokes C `release` callback — private reader data freed exactly once.

## Column Schema

| Column | Arrow Type | Notes |
|--------|-----------|-------|
| `ts` | `Timestamp(Microsecond, "UTC")` | Exchange timestamp |
| `open` / `high` / `low` / `close` | `Decimal128(38, 8)` | Exact decimal price (zero-floats policy) |
| `volume` | `Decimal128(38, 8)` | Exact decimal quantity |

## Constants

| Constant | Value |
|----------|-------|
| `STREAM_NAME` | `"arrow_array_stream"` |
| `DECIMAL_PRECISION` | 38 |
| `DECIMAL_SCALE` | 8 |

## Conversion

`decimal_to_scaled_i128(d)` rescales a `rust_decimal::Decimal` to scale 8 and
returns its unscaled `i128` mantissa (truncates trailing precision beyond scale 8 — prices quoted to paise at most).

## Functions

| Function | Purpose |
|----------|---------|
| `bars_to_stream(bars)` | Export `Vec<Bar>` as Arrow stream capsule |
| `ticks_to_stream(ticks)` | Export `Vec<Tick>` as Arrow stream capsule |

## Zero-Floats Policy

All money/quantity columns are exact `Decimal128(38, 8)` scalars — zero floats.

## Related

- `crates/vyapaar-ffi-py/src/market.rs` — `Bar`, `Tick` Rust structs.
- `python/tests/test_arrow.py` — Python-side capsule consumption tests.
