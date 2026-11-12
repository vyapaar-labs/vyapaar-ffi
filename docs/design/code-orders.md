# `vyapaar-ffi` — Order PyClasses Reference

Documents `crates/vyapaar-ffi-py/src/orders.rs`: order pyclasses exposing
`vyapaar_core` order types to Python. Every order carries the SEBI-mandated
`algo_id`; prices/quantities cross as exact `decimal.Decimal`.

## PyClasses

| PyClass | Purpose |
|---------|---------|
| `OrderRequest` | Validated order placement request |
| `OrderAck` | Acknowledgment from broker/execution engine |
| `FillReport` | Execution fill report |
| `OrderEvent` | Single lifecycle event (acked / filled / rejected / cancelled) |

## `OrderRequest`

```python
OrderRequest(
    instrument, side, order_type, quantity, algo_id, *,
    client_order_id=None, limit_price=None, trigger_price=None,
    tif="DAY", tag=None,
)
```

* `order_type`: `MARKET` | `LIMIT` (requires `limit_price`) |
  `STOP_LOSS` (requires `trigger_price`) |
  `STOP_LIMIT` (requires `trigger_price` + `limit_price`).
* `client_order_id`: optional UUID; a fresh v4 UUID is minted when omitted.
* `tif`: `DAY` | `IOC` | `GTC`.
* Getters: `client_order_id`, `instrument`, `side`, `order_type`, `quantity`,
  `limit_price`, `tif`, `algo_id`, `tag`; `to_dict()` for JSON-ish
  serialization.

## `OrderAck`, `FillReport`, `OrderEvent`

* `OrderAck(order_id, client_order_id, status, timestamp)` — getters
  `order_id`, `client_order_id`, `status` (uppercase snake_code, e.g.
  `PENDING_NEW`/`FILLED`), `timestamp`; `to_dict()`.
* `FillReport(fill_id, order_id, client_order_id, instrument, side, price,
  quantity, algo_id, timestamp)` — getters mirror the core `FillReport`;
  `to_dict()`.
* `OrderEvent` — read-only wrapper over `vyapaar_core::OrderEvent` with
  `kind` (`acked`/`filled`/`rejected`/`cancelled`), `order_id`, and a
  string-valued `payload` dict.

All orders carry a client id (`with_client_id("python")`) so the core wire
contract stays intact while the Python caller is attributed.

## Related

* `code-pyclass.md` — `InstrumentId`/`OrderSide`/`OrderStatus` value objects.
* `code-decision.md` — the closed AI `Decision` grammar that can also place orders.
* `code-risk.md` — `RiskEngine::check_order` gates every `OrderRequest`.
