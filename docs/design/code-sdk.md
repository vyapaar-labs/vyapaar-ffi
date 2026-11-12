# `vyapaar-ffi` — Python SDK Reference

Documents `python/vyapaar/`: the idiomatic Python layer on top of the compiled
`vyapaar._vyapaar` extension.

## Package Layout

| Module | Purpose |
|--------|---------|
| `__init__.py` | Public API surface: re-exports the compiled types + the pure-Python modules (`strategy`, `indicators`, `sizing`, `session`, `data`, `types`; `cli` importable as `import vyapaar.cli`) |
| `types.py` | Re-exports the compiled value objects; `to_decimal(value)` (accepts `Decimal`, `str`, `Price`/`Quantity` via `.to_str()`), `instrument_id("EXCHANGE:SYMBOL"` or `"SYMBOL"`) |
| `strategy.py` | `Strategy`, `Parameter`, `BacktestResult`, `run(strategy, bars, *, algo_id, on_start=True)` |
| `indicators.py` | `SMA`, `EMA`, `RSI` — pure-Python exact-`Decimal` indicators with `crosses_above`/`crosses_below` |
| `sizing.py` | `Sizer`, `fixed_quantity`, `fractional`, `risk_budget` — exact `Decimal` sizing, rounded down |
| `session.py` | `Session`, `SessionWindow`, `PRE_MARKET`/`LIVE`/`POST_MARKET`, `current_session`, `is_live` (IST) |
| `data.py` | Arrow bridge helpers: `bars_to_arrow`, `ticks_to_arrow`, `bars_to_polars` |
| `cli.py` | `python -m vyapaar.cli` — `version`, `backtest` (stub), `serve`/`stub` placeholders |

## `Strategy` & `run` (strategy.py)

Subclasses override `on_start`, `on_bar`, `on_tick`, `on_order_event` and
issue orders with `self.buy(symbol, quantity, *, limit_price=None, tag=None)`
/ `self.sell(...)`. `Parameter(default, min, max, step)` validates bounds;
`Parameter.int()` is the `step=1` convenience.

`run(strategy, bars, *, algo_id, on_start=True)`:

1. attaches the strategy to a compiled `StrategyRunner` and registers `algo_id`
   on its risk gate,
2. sets `strategy.indicators = vyapaar.indicators`,
3. calls `on_start()` once,
4. runs `runner.run_bars(bars)` with the GIL released per batch,
5. flushes any orders buffered during callbacks (the compiled runner holds a
   PyO3 borrow while `run_bars`/`run_ticks` execute, so runner methods cannot
   be called from inside a callback — orders are buffered and submitted after
   the batch),
6. returns a `BacktestResult` (`bars_processed`, `orders_filled`, `events`,
   `refusals`, `algo_id`).

Every order passes through the compiled `StrategyRunner.submit_order`, which
gates it through the synchronous pre-trade risk engine, simulates a fill at
the limit price, records it, and dispatches `on_order_event`.

## CLI (cli.py)

| Subcommand | Status |
|------------|--------|
| `version` | Implemented — prints SDK/API/schema versions |
| `backtest --bars-json PATH` | **Stub** — loads the JSON, prints the bar count, exits 0; execution is a placeholder (see `docs/gaps/0001-*`) |
| `serve`, `stub` | **Placeholders** — "not implemented yet" (see `docs/gaps/0001-*`) |

## Zero Floats Policy

All SDK arithmetic uses `decimal.Decimal`: indicators smooth in `Decimal`,
sizing rounds down, and `to_decimal`/`instrument_id` coerce inputs to exact
decimal before crossing into the compiled layer.

## Related

* `architecture.md` — package topology and build (`make codegen`, `py-build`).
* `code-runner.md` — the compiled `StrategyRunner` this SDK drives.
* `code-arrow.md` — the capsule bridge `data.py` wraps.
