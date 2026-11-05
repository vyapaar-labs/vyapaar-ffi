//! GIL-safe strategy runner pyclass: `StrategyRunner`.

use pyo3::prelude::*;
use pyo3::types::PyModule;

/// Register runner classes and functions onto the module.
pub(crate) fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let _ = m;
    Ok(())
}
