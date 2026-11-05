//! GIL-safe strategy runner pyclass: `StrategyRunner` + `now_iso()`.
//!
//! The runner holds the Python strategy object and a private, synchronous
//! [`vyapaar_risk::RiskEngine`] that cannot be bypassed: every `submit_order`
//! is gated, simulated-fill recorded, and dispatched back through the strategy's
//! `on_order_event` callback with the GIL held strictly at the callback
//! boundary. Batch `run_bars` / `run_ticks` work releases the GIL
//! (`py.detach`) around pure-Rust aggregation and re-acquires it per callback.

use chrono::{DateTime, Utc};
use pyo3::prelude::*;
use pyo3::types::PyModule;

use crate::market::{Bar, Tick};
use crate::orders::{OrderAck, OrderRequest};
use crate::types::{InstrumentId, price_from_any};

/// Resolve the evaluation timestamp: an explicit RFC3339 UTC string wins;
/// otherwise the FFI boundary (the wall-clock owner) supplies `Utc::now()`.
fn now_or_utc(ts_iso: Option<&str>) -> PyResult<DateTime<Utc>> {
    match ts_iso {
        Some(s) => crate::error::parse_utc(s),
        None => Ok(Utc::now()),
    }
}

/// Drives a Python strategy object through the synchronous risk gate.
#[pyclass(module = "vyapaar._vyapaar")]
pub struct StrategyRunner {
    /// The Python strategy object (`on_bar` / `on_tick` / `on_order_event`).
    strategy: Py<PyAny>,
    /// Synchronous, non-bypassable pre-trade gate owned by this runner.
    risk: vyapaar_risk::RiskEngine,
    /// The runner's SEBI algo ID (informational identity of the strategy).
    pub(crate) algo_id: String,
}

#[pymethods]
impl StrategyRunner {
    #[new]
    fn new(strategy: Py<PyAny>, algo_id: &str) -> Self {
        Self {
            strategy,
            risk: vyapaar_risk::RiskEngine::default(),
            algo_id: algo_id.to_owned(),
        }
    }

    /// The runner's SEBI algo ID.
    #[getter]
    fn algo_id(&self) -> &str {
        &self.algo_id
    }

    /// Register a SEBI algo ID on the internal gate.
    fn register_algo(&mut self, algo_id: &str) {
        self.risk.register_algo(algo_id);
    }

    /// Trip the internal platform-wide emergency kill switch.
    fn halt_global(&mut self, reason: &str, ts_iso: &str) -> PyResult<()> {
        let ts = crate::error::parse_utc(ts_iso)?;
        self.risk.halt_global(reason, ts);
        Ok(())
    }

    fn resume_global(&mut self) {
        self.risk.resume_global();
    }

    fn set_reduce_only(&mut self) {
        self.risk.set_reduce_only();
    }

    fn is_halted(&self) -> bool {
        self.risk.global_state.is_halted()
    }

    /// Set a scrip price band on the internal gate.
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
        self.risk
            .set_price_band(instrument.borrow().inner.clone(), band);
        Ok(())
    }

    /// Gate an order through the internal risk engine (no dispatch).
    fn check_order(
        &mut self,
        order: &Bound<'_, OrderRequest>,
        ts_iso: Option<&str>,
    ) -> PyResult<()> {
        let now = now_or_utc(ts_iso)?;
        self.risk
            .check_order(&order.borrow().inner, now)
            .map_err(|r| crate::error::risk_refused(&r))
    }

    /// Gate, simulate a fill at the limit price, record it, ack it as FILLED,
    /// and dispatch `on_order_event` to the Python strategy.
    fn submit_order<'py>(
        &mut self,
        py: Python<'py>,
        order: &Bound<'py, OrderRequest>,
        ts_iso: Option<&str>,
    ) -> PyResult<OrderAck> {
        let now = now_or_utc(ts_iso)?;
        let core = order.borrow().inner.clone();

        // 1. Synchronous pre-trade gate (non-bypassable).
        self.risk
            .check_order(&core, now)
            .map_err(|r| crate::error::risk_refused(&r))?;

        // 2. Simulated fill at the limit price (zero for market orders).
        let fill_price = core
            .order_type
            .limit_price()
            .unwrap_or(vyapaar_core::Price::ZERO);
        let fill = vyapaar_core::FillReport::new(
            format!("fill-{}", core.client_order_id),
            format!("order-{}", core.client_order_id),
            core.client_order_id,
            core.instrument.clone(),
            core.side,
            fill_price,
            core.quantity,
            core.algo_id.clone(),
            now,
            "python",
        );
        self.risk.record_fill(&fill);

        // 3. Simulated acknowledgement: the paper order fills immediately.
        let ack = OrderAck::from_core(vyapaar_core::OrderAck::new(
            format!("order-{}", core.client_order_id),
            core.client_order_id,
            vyapaar_core::OrderStatus::Filled,
            now,
        ));

        // 4. Dispatch to the Python strategy with the GIL held (callback
        //    boundary). An exception in the strategy callback is a real error.
        if self.strategy.bind(py).getattr("on_order_event").is_ok() {
            let ack_bound = Bound::new(py, ack.clone())?;
            self.strategy
                .bind(py)
                .call_method1("on_order_event", (ack_bound.as_any(),))?;
        }

        Ok(ack)
    }

    /// Run a batch of bars: pure-Rust aggregation with the GIL released, then
    /// `on_bar` dispatch per bar with the GIL re-acquired.
    fn run_bars<'py>(&mut self, py: Python<'py>, bars: Vec<Bound<'py, Bar>>) -> PyResult<i64> {
        let core_bars: Vec<vyapaar_core::Bar> =
            bars.iter().map(|b| b.borrow().inner.clone()).collect();

        // GIL-free Rust-side work: checksum = count of bars with positive close.
        let checksum: i64 =
            py.detach(move || core_bars.iter().filter(|b| b.close.is_positive()).count() as i64);

        for bar in &bars {
            if self.strategy.bind(py).getattr("on_bar").is_ok() {
                self.strategy
                    .bind(py)
                    .call_method1("on_bar", (bar.as_any(),))?;
            }
        }

        Ok(checksum)
    }

    /// Run a batch of ticks: pure-Rust aggregation with the GIL released, then
    /// `on_tick` dispatch per tick with the GIL re-acquired.
    fn run_ticks<'py>(&mut self, py: Python<'py>, ticks: Vec<Bound<'py, Tick>>) -> PyResult<i64> {
        let core_ticks: Vec<vyapaar_core::Tick> =
            ticks.iter().map(|t| t.borrow().inner.clone()).collect();

        // GIL-free Rust-side work: checksum = count of ticks with positive price.
        let checksum: i64 = py.detach(move || {
            core_ticks
                .iter()
                .filter(|t| match t {
                    vyapaar_core::Tick::Ltp { price, .. } => price.is_positive(),
                    vyapaar_core::Tick::Quote { last_price, .. } => {
                        last_price.as_ref().is_some_and(|p| p.is_positive())
                    }
                    vyapaar_core::Tick::Depth { last_price, .. } => {
                        last_price.as_ref().is_some_and(|p| p.is_positive())
                    }
                })
                .count() as i64
        });

        for tick in &ticks {
            if self.strategy.bind(py).getattr("on_tick").is_ok() {
                self.strategy
                    .bind(py)
                    .call_method1("on_tick", (tick.as_any(),))?;
            }
        }

        Ok(checksum)
    }
}

/// Current UTC timestamp as an RFC3339 string. The FFI boundary is the only
/// wall-clock owner; domain crates stay deterministic.
#[pyfunction]
fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

/// Register runner classes and functions onto the module.
pub(crate) fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<StrategyRunner>()?;
    m.add_function(wrap_pyfunction!(now_iso, m)?)?;
    Ok(())
}
