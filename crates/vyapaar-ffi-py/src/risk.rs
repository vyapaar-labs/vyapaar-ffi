//! Pre-trade risk gate pyclass: `RiskEngine` wrapping `vyapaar_risk::RiskEngine`.

use pyo3::prelude::*;
use pyo3::types::PyModule;

/// Register risk classes onto the module.
pub(crate) fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let _ = m;
    Ok(())
}
