# व्यापार・桥接 vyapaar-ffi

> *Python FFI Bridge: Apache Arrow zero-copy memory bridge, PyO3 bindings, and PyPI distribution wheel.*

Zero-overhead Python bindings to Rust domain types, indicators, and execution gateways.

## Subcrates

- `vyapaar-ffi-py` — PyO3 extension module compiling to the `vyapaar` wheel
- `vyapaar-ffi-codegen` — Code generator for Python stubs and Arrow schema definitions

## Build & Test

```bash
make prod
make check
make test
```

License: AGPL-3.0.
