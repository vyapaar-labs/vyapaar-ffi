# `vyapaar-ffi` — DDD Architecture & Zero-Copy Python FFI Bridge

## 1. Bounded Context & Responsibility
`vyapaar-ffi` (Tier 4, ported from `honba-tamashii`) is the **Anti-Corruption & Zero-Copy Memory Bridge** between the Rust Core and the Python user plane (`vyapaar` PyPI wheel).

## 2. Sub-Crate & Package Topology
* `crates/vyapaar-ffi-codegen`: Generates JSON Schemas, OpenAPI 3.1 specifications, and `.pyi` type stubs from `vyapaar-core` `schemars` types.
* `crates/vyapaar-ffi-py`: PyO3 ABI3 (`py310+`) native extension module (`vyapaar._vyapaar`) exposing:
  - Canonical `Decimal`-backed `InstrumentId`, `Price`, `Quantity`, `Money`, `Bar`, `Tick`, `OrderRequest`, `Decision`.
  - Apache Arrow `RecordBatch` / `PyCapsule` (`__arrow_c_stream__`, `__arrow_c_array__`) zero-copy data bridge for Polars and PyArrow.
  - Fault-isolated `Strategy` runner with `py.allow_threads` GIL release and synchronous `vyapaar_risk::RiskEngine` gating.
* `python/vyapaar/`: Idiomatic Python SDK (`Strategy`, `Parameter`, `indicators`, `sizing`, `session`, `data`, `cli`).
