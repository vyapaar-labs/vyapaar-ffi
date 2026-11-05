.PHONY: default build build-prod build-release build-dev prod release dev debug check check-prod check-dev test clippy fmt clean bootstrap-python codegen py-build py-test py all

# Default target is 'prod' (no debug build unless asked for specifically)
default: prod
build: prod

prod: build-prod
build-prod:
	cargo build --workspace --profile prod

release: build-release
build-release:
	cargo build --workspace --release

dev: build-dev
build-dev:
	cargo build --workspace --profile dev

# Debug build is explicit and not run unless asked
debug: build-debug
build-debug:
	cargo build --workspace

check: check-prod
check-prod:
	cargo check --workspace --profile prod

check-dev:
	cargo check --workspace --profile dev

test:
	cargo test --workspace --profile prod

clippy:
	cargo clippy --workspace --all-targets --profile prod -- -D warnings

fmt:
	cargo fmt

clean:
	cargo clean
	rm -rf target

# --- Python FFI bridge (vyapaar wheel) --------------------------------------

# Create the local virtualenv (if missing) and install the Python build/test
# toolchain. Note: PYO3_PYTHON defaults to the interpreter pinned in
# `.cargo/config.toml`; override with `PYO3_PYTHON=/path/to/python3` to build
# the extension module against another CPython (e.g. this venv).
bootstrap-python:
	@test -d .venv || python3 -m venv .venv
	.venv/bin/python3 -m pip install -q --upgrade pip maturin pytest pyarrow
	@echo "PYO3_PYTHON defaults to the value in .cargo/config.toml."
	@echo "Override with PYO3_PYTHON=/path/to/python3 to build against another CPython."

# Regenerate schema/ (domain_schema.json, openapi.json) and the
# python/vyapaar/_vyapaar.pyi type stubs from vyapaar-core wire types.
codegen:
	cargo run -q -p vyapaar-ffi-codegen --bin vyapaar-codegen
	@echo "Codegen wrote:"
	@for f in schema/domain_schema.json schema/openapi.json python/vyapaar/_vyapaar.pyi; do \
		test -f "$$f" && echo "  $$f"; \
	done

# Build the `vyapaar` extension module into the local virtualenv (editable).
py-build:
	.venv/bin/maturin develop

# Run the Python test suite (python/tests/).
py-test:
	.venv/bin/python -m pytest python/tests -q

# Build, then run the Python tests.
py: py-build py-test

# Full gate: Rust fmt/clippy/test + codegen + Python wheel + Python tests.
all: fmt clippy test codegen py-build py-test
