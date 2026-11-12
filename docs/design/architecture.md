# `vyapaar-ffi` — DDD Architecture & Zero-Copy Python FFI Bridge

> **Subsystem**: Soul (Tier 4, ported from `honba-tamashii`). The
> Anti-Corruption & Zero-Copy Memory Bridge between the Rust core and the
> Python user plane (`vyapaar` PyPI wheel).

## 1. Bounded Context & Responsibility

`vyapaar-ffi` compiles the canonical Rust trading core (`vyapaar-core`,
`vyapaar-risk`) into a native Python extension module via **PyO3** (ABI3, one
wheel serves CPython 3.10+). It owns four responsibilities:

1. **Exact-decimal value objects** — `Price`, `Quantity`, `Currency`,
   `Exchange`, `InstrumentId`, `OrderSide`, `TimeInForce`, `OrderStatus`,
   `DepthLevel`, `Bar`, `Tick`, `OrderRequest`, `OrderAck`, `FillReport`,
   `OrderEvent`, `Decision` cross the boundary as Python `decimal.Decimal`
   values (Zero Floats Policy).
2. **SEBI pre-trade risk gate** — `RiskEngine` exposes the full
   `vyapaar_risk::RiskEngine` surface (algo-ID registry, price bands, OTR
   counters, kill switch, `check_order`/`check_order_fast`/`check_decision`).
3. **GIL-safe strategy runner** — `StrategyRunner` drives Python strategy
   callbacks (`on_bar`, `on_tick`, `on_order_event`) with the GIL held
   strictly at the callback boundary, releasing it (`py.detach`) around
   Rust-side batch work.
4. **Arrow zero-copy bridge** — `bars_to_stream`/`ticks_to_stream` export
   Rust-side Arrow `FFI_ArrowArrayStream` buffers as Python `PyCapsule`s
   (`arrow_array_stream`) that `pyarrow` consumes with zero deserialization.

## 2. Workspace Topology

Two crates plus the Python package:

| Crate / Package | Role | Public Surface |
| :--- | :--- | :--- |
| `crates/vyapaar-ffi-codegen` | Deterministic code generator | `domain_schema()` (JSON Schema), `openapi_document()` (OpenAPI 3.1), `pyi_stubs()`, `write_all()`; binary `vyapaar-codegen` (`main.rs`) |
| `crates/vyapaar-ffi-py` | PyO3 ABI3 extension (`vyapaar._vyapaar`) | modules `types`, `market`, `orders`, `decision`, `risk`, `runner`, `arrow_bridge`, `decimal`, `error`; exceptions `RiskRefused`, `ConversionError`; `init_module()` registers everything |
| `python/vyapaar/` | Idiomatic Python SDK | `Strategy`, `Parameter`, `run`, `BacktestResult`, indicators (`SMA`/`EMA`/`RSI`), `Sizer`/`fixed_quantity`/`fractional`/`risk_budget`, `Session`/`SessionWindow`, Arrow helpers (`bars_to_arrow`, `ticks_to_arrow`, `bars_to_polars`), `instrument_id`/`to_decimal`, `cli` |

## 3. FFI Boundary Contracts

* **Zero floats**: every price/quantity crosses as an exact decimal string —
  `decimal.Decimal` ↔ `rust_decimal::Decimal` round-trips through
  `crate::decimal` helpers; no `f32`/`f64` anywhere on the boundary.
* **Explicit timestamps**: domain crates stay deterministic — the FFI
  boundary is the only wall-clock owner. Callers pass RFC3339 UTC strings
  (`ts_iso`); `now_iso()` is the sanctioned boundary clock.
* **Closed AI decision grammar**: `Decision::from_json` parses against
  `vyapaar_core::Decision` (`deny_unknown_fields`, snake_case `action` tag) —
  an AI model is an untrusted signal generator and cannot bypass the gate.
* **Non-bypassable risk**: every `RiskEngine::check_order` /
  `check_order_fast` / `check_decision` and every `StrategyRunner::submit_order`
  runs the synchronous `vyapaar_risk::RiskEngine` before anything may reach an
  execution buffer.

## 4. GIL Management

* Batch work (`run_bars`, `run_ticks`) releases the GIL via `py.detach`
  around pure-Rust aggregation; the GIL is re-acquired per Python callback.
* `#[pyclass]` items are private by default in pyo3 0.30 — the public Rust
  `from_core` lifters exist for the SDK, and tests drive the module entirely
  through Python objects (`Python::attach` + `init_module`).

## 5. Codegen & Single Source of Truth

`make codegen` runs the `vyapaar-codegen` binary, which writes:

* `schema/domain_schema.json` — combined JSON Schema (draft 2020-12),
  version `vyapaar_core::SCHEMA_VERSION` (= 8),
* `schema/openapi.json` — OpenAPI 3.1 doc whose `components.schemas` is cloned
  from the domain schema so the two can never drift,
* `python/vyapaar/_vyapaar.pyi` — the generated type-stub contract for the
  extension module.

Artifacts are committed and diffable; any wire-type change bumps the schema
version. `pyproject.toml` builds the ABI3 `vyapaar._vyapaar` wheel via maturin
(`module-name = "vyapaar._vyapaar"`, `python-source = "python"`).

## 6. Module Map (per-module design docs)

| Rust module / Python module | Doc |
|-----------------------------|-----|
| `types.rs` (value objects) | [code-pyclass.md](code-pyclass.md) |
| `market.rs` (`Bar`/`Tick`/`DepthLevel`) | [code-market.md](code-market.md) |
| `orders.rs` (`OrderRequest`/`OrderAck`/`FillReport`/`OrderEvent`) | [code-orders.md](code-orders.md) |
| `decision.rs` (`Decision`) | [code-decision.md](code-decision.md) |
| `risk.rs` (`RiskEngine`) | [code-risk.md](code-risk.md) |
| `runner.rs` (`StrategyRunner`, `now_iso`) | [code-runner.md](code-runner.md) |
| `arrow_bridge.rs` (PyCapsule export) | [code-arrow.md](code-arrow.md) |
| `decimal.rs` (Decimal conversion) | [code-decimal.md](code-decimal.md) |
| `error.rs` (`RiskRefused`/`ConversionError`) | [code-error.md](code-error.md) |
| `vyapaar-ffi-codegen` | [code-codegen.md](code-codegen.md) |
| `python/vyapaar/*` (SDK) | [code-sdk.md](code-sdk.md) |
| historical `honba-tamashii` → `vyapaar-ffi` migration | [code-migration-map.md](code-migration-map.md) |

## 7. Testing

* Rust: `crates/vyapaar-ffi-codegen/tests/codegen_tests.rs` pins the generated
  artifact contracts; `crates/vyapaar-ffi-py/tests/` (`value_types_tests`,
  `risk_tests`, `runner_tests`, `arrow_tests`) drive the module through
  Python objects. Zero `#[cfg(test)]` in `src/` (enforced in CI).
* Python: `python/tests/` (`test_types`, `test_risk`, `test_strategy`,
  `test_sdk`, `test_arrow`) via `pytest`.
* CI (`.github/workflows/ci.yml`): Rust gate (fmt/clippy/test) + Python wheel
  (`maturin develop` + `pytest`). `PYO3_PYTHON` must point at a CPython with
  dev headers (repo-local `.cargo/config.toml` pins CPython 3.12).

## 8. Known Gaps

* CLI subcommands `backtest` (stub), `serve`, `stub` are placeholders — see
  `docs/gaps/0001-cli-stub-subcommands.md`.
* Workspace declares `vyapaar-market-{finance,feed,store}` dependencies but no
  crate uses them — see `docs/gaps/0002-unused-market-data-workspace-deps.md`.

## Related

* `vyapaar-core` — canonical wire types and `SCHEMA_VERSION`/`API_VERSION`.
* `vyapaar-risk` — the pre-trade risk gate exposed through `RiskEngine`.
* `vyapaar-market` — market-data crates declared at the workspace level for a
  future data bridge (currently unused).
