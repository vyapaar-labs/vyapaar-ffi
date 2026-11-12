# `vyapaar-ffi` — AI Decision Grammar Reference

Documents `crates/vyapaar-ffi-py/src/decision.rs`: the closed AI `Decision`
grammar pyclass.

## Closed Decision Grammar

An AI model is an **untrusted signal generator**: decisions are parsed against
the closed `vyapaar_core::Decision` serde grammar (`deny_unknown_fields`,
snake_case `action` tag) and validated before they may reach the risk gate.

## `Decision`

```python
Decision.from_json(s: str) -> Decision
```

* Raises `ConversionError` on invalid JSON, unknown fields
  (`deny_unknown_fields`), or validation failures (bad `algo_id`, zero
  quantity, empty symbol).
* `action` getter → `"place_order"` | `"modify_order"` | `"cancel_order"` |
  `"cancel_all"` | `"hold"`.
* `to_dict()` → string-valued dict with the keys available on the variant
  (`action`, `symbol`, `side`, `qty`, `price_limit`, `algo_id`,
  `rationale`, `instrument`, `new_qty`, `new_price`, `reason`, ...).

## Related

* `code-risk.md` — `RiskEngine::check_decision` gates a parsed decision before
  any execution buffer.
* `code-orders.md` — the order types a `place_order` decision maps onto.
* `vyapaar-core` — the `Decision` enum and its serde grammar.
