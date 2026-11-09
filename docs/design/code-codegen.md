# `vyapaar-ffi` — Codegen Reference

Documents `crates/vyapaar-ffi-codegen/src/lib.rs`: deterministic code generator producing
three artifacts from the canonical `vyapaar_core` wire types.

## Generated Artifacts

| Artifact | Function | Description |
|----------|----------|-------------|
| JSON Schema | `domain_schema()` | Combined JSON Schema document (draft 2020-12) |
| OpenAPI 3.1 | `openapi_document()` | OpenAPI document for REST surface |
| Python stubs | `pyi_stubs()` | `.pyi` type stubs for `vyapaar._vyapaar` |

## Determinism

- Pure generators perform **no I/O** — identical inputs always produce identical output.
- The single sanctioned I/O entry point is `write_all()`, driven by the `vyapaar-codegen` CLI binary (`main.rs`).
- Artifacts are committed and diffable — any wire type change bumps `SCHEMA_VERSION` (mirrors `vyapaar_core::SCHEMA_VERSION`).

## Wire Types Covered

`OrderRequest`, `OrderAck`, `FillReport`, `OrderEvent`, `CancelRequest`, `ModifyRequest`,
`FeeBreakdown`, `Bar`, `Tick`, and all `vyapaar_core` wire types.

## Verification

Run with `make codegen` — re-generated artifacts verified in CI.

## Related

- `vyapaar-core/src/lib.rs` — `SCHEMA_VERSION: u32 = 8`, `API_VERSION: &str = "v1"`.
- `vyapaar-core/src/types.rs` — `Price`, `Quantity`, `Money` (schema sources).
- `python/vyapaar/_vyapaar.pyi` — generated type stubs.
