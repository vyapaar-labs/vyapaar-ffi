//! # व्यापार・魂 `vyapaar-ffi-py`
//!
//! PyO3 ABI3 extension module compiled into `vyapaar._vyapaar` by maturin.
//!
//! ## Responsibilities (Tier 4, ported from `honba-tamashii-py`)
//!
//! * Expose canonical [`vyapaar_core`] types to Python as exact-decimal value
//!   objects: [`Price`], [`Quantity`], [`Money`], [`InstrumentId`], [`Bar`],
//!   [`Tick`], [`OrderRequest`], [`OrderAck`], [`FillReport`], [`Decision`].
//! * Gate every Python-submitted order through the synchronous
//!   [`vyapaar_risk::RiskEngine`] (non-bypassable pre-trade gate).
//! * Drive Python strategy callbacks (`on_bar`, `on_tick`, `on_order_event`)
//!   with the GIL held strictly at the callback boundary, releasing it
//!   (`py.detach`) around Rust-side batch work.
//! * Zero-copy export of bars/ticks into Apache Arrow `RecordBatch` streams via
//!   the PyCapsule protocol (`arrow_array_stream`).
//!
//! ## Testing
//!
//! Tests live in `crates/vyapaar-ffi-py/tests/` and drive the module entirely
//! through Python objects (`Python::attach`, `PyModule::new` + `init_module`),
//! because pyo3 generates Rust-private items for `#[pyclass]`/`#[pyfunction]`.

pub mod arrow_bridge;
pub mod decimal;
pub mod decision;
pub mod error;
pub mod market;
pub mod orders;
pub mod risk;
pub mod runner;
pub mod types;

use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::PyModule;

// Custom exceptions (raised across the FFI boundary). Registered on the module
// inside `init_module` below.
create_exception!(_vyapaar, RiskRefused, PyException);
create_exception!(_vyapaar, ConversionError, PyException);

/// Register every class and function onto a Python module.
///
/// This is the single registration point used both by the `#[pymodule]` entry
/// point and by the integration tests in `tests/`.
pub fn init_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("SCHEMA_VERSION", vyapaar_core::SCHEMA_VERSION)?;
    m.add("API_VERSION", vyapaar_core::API_VERSION)?;
    m.add("RiskRefused", m.py().get_type::<RiskRefused>())?;
    m.add("ConversionError", m.py().get_type::<ConversionError>())?;

    types::init(m)?;
    market::init(m)?;
    orders::init(m)?;
    decision::init(m)?;
    risk::init(m)?;
    runner::init(m)?;
    arrow_bridge::init(m)?;
    Ok(())
}

/// PyO3 entry point for `vyapaar._vyapaar` (maturin renames the cdylib).
#[pymodule]
fn _vyapaar(m: &Bound<'_, PyModule>) -> PyResult<()> {
    init_module(m)
}
