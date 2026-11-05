//! `decimal.Decimal` <-> `rust_decimal::Decimal` conversion helpers.
//!
//! The Zero Floats Policy applies across the FFI boundary: money is always
//! exchanged as exact decimal strings. All conversion goes through the `str`
//! representation (Python `decimal.Decimal` and `rust_decimal` are both
//! round-trip stable on their exact string forms).

use pyo3::prelude::*;
use pyo3::types::PyAny;
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::ConversionError;

/// The dotted name of Python's `decimal` module.
const DECIMAL_MODULE: &str = "decimal";

/// Convert a Python `decimal.Decimal` (or any object with `__str__`) to its
/// exact decimal string form. Raises [`ConversionError`] when the string does
/// not parse as a decimal.
pub fn pydecimal_to_decimal(obj: &Bound<'_, PyAny>) -> PyResult<Decimal> {
    let s: String = obj.str()?.extract()?;
    Decimal::from_str(&s).map_err(|e| ConversionError::new_err(format!("invalid decimal: {e}")))
}

/// Convert a Python `decimal.Decimal` (or any `__str__` object) to an exact
/// decimal string.
pub fn pydecimal_to_string(obj: &Bound<'_, PyAny>) -> PyResult<String> {
    obj.str()?
        .extract()
        .map_err(|e| ConversionError::new_err(format!("could not stringify decimal value: {e}")))
}

/// Build a Python `decimal.Decimal` from an exact decimal string.
pub fn decimal_from_string<'py>(py: Python<'py>, s: &str) -> PyResult<Bound<'py, PyAny>> {
    let decimal_mod = py.import(DECIMAL_MODULE)?;
    decimal_mod.call_method1("Decimal", (s,))
}

/// Build a Python `decimal.Decimal` from a `rust_decimal::Decimal`.
pub fn decimal_to_pydecimal<'py>(py: Python<'py>, d: Decimal) -> PyResult<Bound<'py, PyAny>> {
    decimal_from_string(py, &d.to_string())
}

/// The empty decimal string is not a valid `Decimal` — this is the canonical
/// `Decimal("0")` as a Python object.
pub fn zero_pydecimal<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
    decimal_from_string(py, "0")
}
