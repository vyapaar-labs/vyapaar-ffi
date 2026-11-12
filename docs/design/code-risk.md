# `vyapaar-ffi` — Risk Engine PyClass Reference

Documents `crates/vyapaar-ffi-py/src/risk.rs`: the `RiskEngine` pyclass
wrapping the synchronous, non-bypassable `vyapaar_risk::RiskEngine`.

## `RiskEngine`

The gate is synchronous and non-bypassable: every Python-submitted order or AI
`Decision` is clamped here before it may reach an execution buffer. The FFI
boundary is the only wall-clock owner — timestamps are passed in as explicit
RFC3339 UTC strings (`ts_iso`) so the domain crates stay deterministic.

## Methods

| Method | Purpose |
|--------|---------|
| `new()` | Construct an empty gate (`vyapaar_risk::RiskEngine::default()`) |
| `register_algo` / `is_algo_registered` / `unregister_algo` | SEBI algo-ID registry |
| `suspend_algo(algo_id, reason)` / `reactivate_algo` | SEBI Retail Algo Circular |
| `halt_global(reason, ts_iso)` / `resume_global` | Platform-wide kill switch |
| `set_reduce_only` | Reduce-only global state |
| `is_halted` / `global_state` | `ACTIVE` / `REDUCE_ONLY` / `HALTED` |
| `set_price_band(instrument, lower, upper, reference)` | Operational circuit band |
| `check_order(order, ts_iso=None)` | Gate an `OrderRequest`; raises `RiskRefused` on refusal |
| `check_order_fast(algo_id, instrument, quantity, limit_price=None, ts_iso=None)` | Zero-allocation hot path (<15 µs target) |
| `check_decision(decision_json, ts_iso=None)` | Gate a closed-grammar AI `Decision` |
| `record_fill(fill)` | Update OTR + cumulative-turnover trackers |
| `otr_for_algo(algo_id)` | Outstanding-turnover ratio as a decimal string |
| `total_refusals` | Running refusal counter |
| `turnover_for_algo(algo_id)` | Cumulative turnover string |

## `ts_iso` Resolution

An explicit RFC3339 UTC string wins; otherwise the FFI boundary (the wall-clock
owner) supplies `Utc::now()` (`now_or_utc`). Domain crates never query the
clock.

## Risk Refusals

`RiskEngine::check_*` raises the `RiskRefused` exception whose message embeds
the machine-readable rule: `"rule=<rule>: <display>"` (see `error.rs`).

## Related

* `code-runner.md` — `StrategyRunner` owns a private `RiskEngine` for
  `submit_order`.
* `code-orders.md` — the `OrderRequest`/`FillReport` types the gate consumes.
* `vyapaar-risk` — the underlying `RiskEngine`.
