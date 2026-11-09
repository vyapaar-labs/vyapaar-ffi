# `vyapaar-ffi` — Strategy Runner Reference

Documents `crates/vyapaar-ffi-py/src/runner.rs`: GIL-safe `StrategyRunner` pyclass driving
Python strategy objects through a synchronous, non-bypassable `vyapaar_risk::RiskEngine`.

## `StrategyRunner`

```rust
#[pyclass(module = "vyapaar._vyapaar")]
pub struct StrategyRunner {
    strategy: Py<PyAny>,          // Python strategy object (on_bar / on_tick / on_order_event)
    risk: vyapaar_risk::RiskEngine, // Non-bypassable pre-trade gate
    algo_id: String,              // SEBI algo ID for the strategy
}
```

### Methods

| Method | Purpose |
|--------|---------|
| `new(strategy, algo_id)` | Create runner with strategy + algo ID |
| `algo_id()` | Return the runner's SEBI algo ID |
| `register_algo(algo_id)` | Register a SEBI algo ID on the internal gate |
| `kill_switch()` | Trip platform-wide emergency kill switch |
| `submit_order(...)` | Submit order through mandatory risk gate |
| `run_bars(bars, ...)` | Batch bar processing (GIL released during Rust aggregation) |
| `run_ticks(ticks, ...)` | Batch tick processing (GIL released during Rust aggregation) |

## GIL Management

- **Batch work** (`run_bars`, `run_ticks`): releases GIL via `py.detach` around pure-Rust aggregation; re-acquires per Python callback.
- **Callbacks** (`on_bar`, `on_tick`, `on_order_event`): GIL held strictly at the callback boundary.
- **Risk gate**: `vyapaar_risk::RiskEngine` is synchronous — every `submit_order` is gated before any broker dispatch.

## Zero Floats Policy

All order prices/quantities cross as exact `decimal.Decimal` — no floats.

## Related

- `crates/vyapaar-ffi-py/src/risk.rs` — `RiskEngine` pyclass.
- `crates/vyapaar-ffi-py/src/orders.rs` — `OrderRequest`, `OrderAck` pyclasses.
- `crates/vyapaar-ffi-py/src/market.rs` — `Bar`, `Tick` pyclasses.
- `vyapaar_risk::RiskEngine` — pre-trade risk gate (in `vyapaar-risk`).
