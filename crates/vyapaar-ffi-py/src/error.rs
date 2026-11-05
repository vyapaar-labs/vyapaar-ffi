//! Conversion and risk-refusal error helpers shared by every FFI module.

use pyo3::PyErr;
use vyapaar_risk::RiskRefusal;

/// Convenience alias so modules can `use crate::error::*`.
pub(crate) type Result<T> = std::result::Result<T, PyErr>;

/// Build the `ConversionError` exception for a failed Rust<->Python conversion.
pub(crate) fn conversion_err(msg: impl std::fmt::Display) -> PyErr {
    crate::ConversionError::new_err(msg.to_string())
}

/// Build the `RiskRefused` exception carrying the refusal rule and message.
///
/// The message embeds the machine-readable `rule` so Python callers can switch
/// on it: `"rule=<rule>: <display>"`.
pub(crate) fn risk_refused(refusal: &RiskRefusal) -> PyErr {
    crate::RiskRefused::new_err(format!("rule={}: {}", refusal.rule(), refusal))
}

/// Chrono `DateTime<Utc>` parsing errors surface as `ConversionError`.
pub(crate) fn parse_utc(iso: &str) -> Result<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(iso)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|e| conversion_err(format!("invalid RFC3339 UTC timestamp {iso:?}: {e}")))
}
