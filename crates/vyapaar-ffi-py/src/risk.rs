//! Pre-trade risk gate pyclass: `RiskEngine` wrapping `vyapaar_risk::RiskEngine`.
//!
//! The gate is synchronous and non-bypassable: every Python-submitted order or
//! AI `Decision` is clamped here before it may reach an execution buffer. The
//! FFI boundary is the only wall-clock owner — timestamps are passed in as
//! explicit RFC3339 UTC strings (`ts_iso`) so the domain crates stay
//! deterministic.

use chrono::{DateTime, Utc};
use pyo3::prelude::*;
use pyo3::types::PyModule;

use crate::orders::{FillReport, OrderRequest};
use crate::types::{InstrumentId, price_from_any, quantity_from_any};

/// Resolve the evaluation timestamp: an explicit RFC3339 UTC string wins;
/// otherwise the FFI boundary (the wall-clock owner) supplies `Utc::now()`.
fn now_or_utc(ts_iso: Option<&str>) -> PyResult<DateTime<Utc>> {
    match ts_iso {
        Some(s) => crate::error::parse_utc(s),
        None => Ok(Utc::now()),
    }
}

/// The synchronous, non-bypassable pre-trade regulatory risk gate.
#[pyclass(module = "vyapaar._vyapaar")]
#[derive(Clone)]
pub struct RiskEngine {
    pub(crate) inner: vyapaar_risk::RiskEngine,
}

#[pymethods]
impl RiskEngine {
    #[new]
    fn new() -> Self {
        Self {
            inner: vyapaar_risk::RiskEngine::default(),
        }
    }

    /// Register a SEBI algo ID with the registry.
    fn register_algo(&mut self, algo_id: &str) {
        self.inner.register_algo(algo_id);
    }

    fn is_algo_registered(&self, algo_id: &str) -> bool {
        self.inner.is_algo_registered(algo_id)
    }

    /// Suspend a registered algo ID (SEBI Retail Algo Circular).
    fn suspend_algo(&mut self, algo_id: &str, reason: &str) {
        self.inner.suspend_algo(algo_id, reason);
    }

    fn reactivate_algo(&mut self, algo_id: &str) {
        self.inner.reactivate_algo(algo_id);
    }

    fn unregister_algo(&mut self, algo_id: &str) {
        self.inner.unregister_algo(algo_id);
    }

    /// Trip the platform-wide emergency kill switch.
    fn halt_global(&mut self, reason: &str, ts_iso: &str) -> PyResult<()> {
        let ts = crate::error::parse_utc(ts_iso)?;
        self.inner.halt_global(reason, ts);
        Ok(())
    }

    fn resume_global(&mut self) {
        self.inner.resume_global();
    }

    fn set_reduce_only(&mut self) {
        self.inner.set_reduce_only();
    }

    fn is_halted(&self) -> bool {
        self.inner.global_state.is_halted()
    }

    fn global_state(&self) -> &'static str {
        match &self.inner.global_state {
            vyapaar_risk::GlobalState::Active => "ACTIVE",
            vyapaar_risk::GlobalState::ReduceOnly => "REDUCE_ONLY",
            vyapaar_risk::GlobalState::Halted { .. } => "HALTED",
        }
    }

    /// Set the operational circuit price band for a scrip.
    fn set_price_band(
        &mut self,
        instrument: &Bound<'_, InstrumentId>,
        lower: &Bound<'_, PyAny>,
        upper: &Bound<'_, PyAny>,
        reference: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let band = vyapaar_core::PriceBand::new(
            price_from_any(lower)?,
            price_from_any(upper)?,
            price_from_any(reference)?,
        );
        self.inner
            .set_price_band(instrument.borrow().inner.clone(), band);
        Ok(())
    }

    /// Synchronously gate an outgoing order before it reaches a broker.
    ///
    /// Raises [`crate::RiskRefused`] with the machine-readable refusal `rule`.
    #[pyo3(signature = (order, ts_iso=None))]
    fn check_order(
        &mut self,
        order: &Bound<'_, OrderRequest>,
        ts_iso: Option<&str>,
    ) -> PyResult<()> {
        let now = now_or_utc(ts_iso)?;
        self.inner
            .check_order(&order.borrow().inner, now)
            .map_err(|r| crate::error::risk_refused(&r))
    }

    /// Zero-allocation hot-path order gate (<15 µs target).
    #[pyo3(signature = (algo_id, instrument, quantity, limit_price=None, ts_iso=None))]
    fn check_order_fast(
        &mut self,
        algo_id: &str,
        instrument: &Bound<'_, InstrumentId>,
        quantity: &Bound<'_, PyAny>,
        limit_price: Option<&Bound<'_, PyAny>>,
        ts_iso: Option<&str>,
    ) -> PyResult<()> {
        let now = now_or_utc(ts_iso)?;
        let quantity = quantity_from_any(quantity)?;
        let limit_price = match limit_price {
            Some(lp) => Some(price_from_any(lp)?),
            None => None,
        };
        self.inner
            .check_order_fast(
                algo_id,
                &instrument.borrow().inner,
                quantity,
                limit_price,
                now,
            )
            .map_err(|r| crate::error::risk_refused(&r))
    }

    /// Gate a closed-grammar AI `Decision` (serde JSON string).
    #[pyo3(signature = (decision_json, ts_iso=None))]
    fn check_decision(&mut self, decision_json: &str, ts_iso: Option<&str>) -> PyResult<()> {
        let now = now_or_utc(ts_iso)?;
        let decision = vyapaar_core::Decision::from_json_str(decision_json)
            .map_err(crate::error::conversion_err)?;
        self.inner
            .check_decision(&decision, now)
            .map_err(|r| crate::error::risk_refused(&r))
    }

    /// Record a fill to update OTR and cumulative-turnover trackers.
    fn record_fill(&mut self, fill: &Bound<'_, FillReport>) {
        self.inner.record_fill(&fill.borrow().inner);
    }

    fn otr_for_algo(&self, algo_id: &str) -> Option<String> {
        self.inner.otr_for_algo(algo_id).map(|d| d.to_string())
    }

    fn total_refusals(&self) -> u64 {
        self.inner.total_refusals()
    }

    fn turnover_for_algo(&self, algo_id: &str) -> String {
        self.inner.turnover_for_algo(algo_id).to_string()
    }
}

/// Register risk classes onto the module.
pub(crate) fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<RiskEngine>()?;
    Ok(())
}
