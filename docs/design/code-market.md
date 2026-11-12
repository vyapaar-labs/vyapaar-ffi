# `vyapaar-ffi` — Market-Data PyClasses Reference

Documents `crates/vyapaar-ffi-py/src/market.rs`: `DepthLevel`, `Bar`, and
`Tick` pyclasses exposing the canonical `vyapaar_core` market types. Prices and
quantities cross as exact `decimal.Decimal`; timestamps are RFC3339 UTC
strings (an empty string means the deterministic Unix epoch, the
Nautilus-style "zero time" convention).

## PyClasses

| PyClass | Purpose |
|---------|---------|
| `DepthLevel` | One level of the L2/L3 order book (`price`, `quantity`, `orders_count`) |
| `Bar` | Standard OHLCV bar / candle (`instrument`, `open`, `high`, `low`, `close`, `volume`, `timestamp`) |
| `Tick` | Canonical typed tick: LTP, best bid/ask quote, or full L2 depth |

## `Tick` Constructors

* `Tick.ltp(instrument, price, volume, exchange_ts, receive_ts)` —
  last-traded-price tick.
* `Tick.quote(instrument, bid, ask, bid_qty, ask_qty, last_price=None,
  exchange_ts="", receive_ts="")` — best bid/ask quote. A present-but-non
  decimal `last_price` raises `ConversionError`; the core `band` field is
  always `None` from Python.
* `Tick.kind` getter → `"ltp"` / `"quote"` / `"depth"`.
* `Tick.to_dict()` — string-valued dict (empty string for absent
  `last_price`).

`Bar` and `Tick` expose public Rust `from_core` lifters so Rust code can wrap
core values; Python normally constructs them directly. `Bar.to_dict()` returns
a string-valued dict for JSON-ish serialization.

## Zero Floats Policy

All price/quantity fields are validated exact `decimal.Decimal` via
`crate::types::price_from_any`/`quantity_from_any` — no floats.

## Related

* `code-arrow.md` — `bars_to_stream`/`ticks_to_stream` export these to Arrow.
* `code-pyclass.md` — `InstrumentId` value object used by `Bar`/`Tick`.
