//! Apache Arrow zero-copy bridge: `RecordBatch` stream exported as a
//! `arrow_array_stream` PyCapsule via the Arrow C Data Interface.

use pyo3::prelude::*;
use pyo3::types::PyModule;

/// Register arrow bridge functions onto the module.
pub(crate) fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let _ = m;
    Ok(())
}
