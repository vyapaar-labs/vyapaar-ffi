# व्यापार・桥接 vyapaar-ffi

> *Python FFI Bridge: PyO3 ABI3 bindings, Apache Arrow zero-copy memory bridge, and the `vyapaar` PyPI wheel.*

`vyapaar-ffi` compiles the canonical Rust trading core (from `vyapaar-core`, `vyapaar-ledger`, `vyapaar-risk`) into a native Python extension module via **PyO3** (ABI3, so one wheel serves CPython 3.10+). It is the "Soul" bounded context: the zero-copy bridge between the low-latency Rust plane and the Python-first developer plane.

What it provides:

- **Exact-decimal domain types** — `Price`, `Quantity`, `Currency`, `Bar`, `Tick`, `OrderRequest`, `FillReport`, `Decision`, all round-tripping as Python `decimal.Decimal` (Zero Floats Policy).
- **SEBI pre-trade risk gate** — `RiskEngine` exposes the full `vyapaar-risk` surface: algo-ID registration, price bands, OTR counters, and a synchronous, non-bypassable `check_order` before any order reaches execution.
- **Arrow zero-copy capsule bridge** — `bars_to_stream` / `ticks_to_stream` export Rust-side Arrow `FFI_ArrowArrayStream` buffers as Python `PyCapsule`s that `pyarrow` can consume with zero deserialization.
- **Smart GIL management** — long-running Rust simulation steps run without the GIL; the GIL is acquired only when invoking Python strategy callbacks (`on_bar`, `on_tick`, `on_order_event`) via `StrategyRunner`.
- **Single source of truth** — `make codegen` regenerates `schema/` and the `python/vyapaar/_vyapaar.pyi` type stubs directly from Rust types.

## Layout

| Path | Contents |
| :--- | :--- |
| `crates/vyapaar-ffi-codegen/` | Deterministic code generator: JSON Schema, OpenAPI 3.1, and `.pyi` stub emitter (binary `vyapaar-codegen`). |
| `crates/vyapaar-ffi-py/` | PyO3 extension module (ABI3) compiled into the `vyapaar` wheel. |
| `python/vyapaar/` | Python package source (`python-source`): `__init__.py` re-exports plus the compiled `_vyapaar` extension and generated `_vyapaar.pyi`. |
| `python/tests/` | Python test suite (`pytest`). |
| `schema/` | Generated artifacts: `domain_schema.json`, `openapi.json`. |
| `.cargo/config.toml` | Pins `PYO3_PYTHON` to a full CPython 3.12 for building/testing the Py crate. |

## Build & Test

```bash
# 1. One-time: create .venv and install maturin, pytest, pyarrow
make bootstrap-python

# 2. Regenerate schema/ + python/vyapaar/_vyapaar.pyi from Rust types
make codegen

# 3. Build the vyapaar extension into .venv (editable)
make py-build

# 4. Run the Python test suite
make py-test

# 5. Rust gate (fmt, clippy, workspace tests)
make check
make test
```

Everything at once: `make all` (fmt → clippy → Rust tests → codegen → wheel → pytest).

`PYO3_PYTHON` defaults to the interpreter pinned in `.cargo/config.toml`; override it in your shell to build against a different CPython (e.g. `PYO3_PYTHON=$PWD/.venv/bin/python make py-build`).

## Quick Usage

```python
from decimal import Decimal

from vyapaar import (
    Bar,
    InstrumentId,
    Price,
    Quantity,
    RiskEngine,
    StrategyRunner,
    bars_to_stream,
)

# Exact-decimal value objects — never f32/f64 (Zero Floats Policy).
price = Price("2481.35")
qty = Quantity("75")
assert price.is_tick_aligned("0.05")

# SEBI pre-trade gate: register the algo and clamp every order to a price band.
risk = RiskEngine()
risk.register_algo("algo-demo-01")
nifty = InstrumentId("NSE", "NIFTY50-INDEX")
risk.set_price_band(nifty, lower="24000", upper="25600", reference="24800")
assert not risk.is_halted()

# Drive a strategy over historical bars; Rust steps without holding the GIL.
runner = StrategyRunner(my_strategy, "algo-demo-01")
n_events = runner.run_bars(bars)

# Arrow zero-copy: export bars as a PyCapsule stream and hand it to pyarrow.
stream = bars_to_stream(bars)
import pyarrow as pa  # noqa: E402
reader = pa.RecordBatchStreamReader(stream)
table = reader.read_all()
```

## Testing

- **Rust gate**: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test --workspace`. All tests live in `tests/` parallel to `src/` — in-file `#[cfg(test)]` is forbidden and enforced in CI.
- **Python suite**: `make py-test` (runs `pytest -q python/tests` against the wheel installed by `make py-build`).
- **CI**: `.github/workflows/ci.yml` runs the Rust gate and the Python wheel build + pytest on push/PR.

## License

AGPL-3.0 — see `LICENSE`.
