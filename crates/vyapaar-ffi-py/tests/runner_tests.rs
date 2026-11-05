//! Integration tests for the `StrategyRunner` pyclass and the `now_iso()`
//! module function.
//!
//! The module is driven entirely through Python objects (`PyModule::new` +
//! `init_module`) because pyo3 generates Rust-private items for
//! `#[pyclass]`/`#[pyfunction]`.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};
use vyapaar_ffi_py::init_module;

/// Fixed deterministic evaluation timestamp for every test.
const TS: &str = "2024-01-02T09:15:00+00:00";

/// Build a fresh `vyapaar._vyapaar` module.
fn module(py: Python<'_>) -> PyResult<Bound<'_, PyModule>> {
    let m = PyModule::new(py, "vyapaar._vyapaar")?;
    init_module(&m)?;
    Ok(m)
}

/// Build an exact `decimal.Decimal` from a decimal string.
fn decimal<'py>(py: Python<'py>, s: &str) -> PyResult<Bound<'py, PyAny>> {
    py.import("decimal")?.call_method1("Decimal", (s,))
}

/// Define a Python strategy class `SimpleStrategy` via `builtins.exec` and
/// instantiate it.
fn make_strategy<'py>(py: Python<'py>, code: &str) -> PyResult<Bound<'py, PyAny>> {
    let exec_fn = py.import("builtins")?.getattr("exec")?;
    let globals = PyDict::new(py);
    exec_fn.call1((code, globals.clone()))?;
    let cls = globals
        .get_item("SimpleStrategy")?
        .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err("SimpleStrategy not defined"))?;
    cls.call0()
}

/// Build a LIMIT `OrderRequest` (algo `momentum-alpha-v1`, qty 10) at
/// `limit_price` in Python.
fn limit_order<'py>(
    py: Python<'py>,
    m: &Bound<'py, PyModule>,
    limit_price: &str,
) -> PyResult<Bound<'py, PyAny>> {
    let inst = m.getattr("InstrumentId")?.call1(("NSE", "RELIANCE"))?;
    let side = m.getattr("OrderSide")?.call1(("BUY",))?;
    let kwargs = PyDict::new(py);
    kwargs.set_item("limit_price", decimal(py, limit_price)?)?;
    m.getattr("OrderRequest")?.call(
        (inst, side, "LIMIT", decimal(py, "10")?, "momentum-alpha-v1"),
        Some(&kwargs),
    )
}

/// A strategy with `on_bar` and `on_order_event` counters.
const COUNTING_STRATEGY: &str = "class SimpleStrategy:\n    def __init__(self):\n        self.bars = 0\n        self.events = 0\n    def on_bar(self, bar):\n        self.bars += 1\n    def on_order_event(self, evt):\n        self.events += 1\n";

#[test]
fn run_bars_calls_strategy() {
    Python::attach(|py| -> PyResult<()> {
        let m = module(py)?;
        let strategy = make_strategy(py, COUNTING_STRATEGY)?;
        let runner = m
            .getattr("StrategyRunner")?
            .call1((strategy.clone(), "momentum-alpha-v1"))?;

        let inst = m.getattr("InstrumentId")?.call1(("NSE", "RELIANCE"))?;
        let bar_cls = m.getattr("Bar")?;
        let bar1 = bar_cls.call1((
            inst.clone(),
            decimal(py, "100")?,
            decimal(py, "105")?,
            decimal(py, "99")?,
            decimal(py, "103")?,
            decimal(py, "1500")?,
            TS,
        ))?;
        let bar2 = bar_cls.call1((
            inst,
            decimal(py, "101")?,
            decimal(py, "106")?,
            decimal(py, "100")?,
            decimal(py, "104")?,
            decimal(py, "2000")?,
            TS,
        ))?;

        let checksum: i64 = runner
            .call_method1("run_bars", (vec![bar1, bar2],))?
            .extract()?;
        assert_eq!(checksum, 2, "checksum counts bars with positive close");

        let bars: i64 = strategy.getattr("bars")?.extract()?;
        assert_eq!(bars, 2, "on_bar must fire once per bar");

        Ok(())
    })
    .unwrap();
}

#[test]
fn run_ticks_calls_strategy() {
    Python::attach(|py| -> PyResult<()> {
        let m = module(py)?;
        let strategy = make_strategy(
            py,
            "class SimpleStrategy:\n    def __init__(self):\n        self.ticks = 0\n    def on_tick(self, tick):\n        self.ticks += 1\n",
        )?;
        let runner = m
            .getattr("StrategyRunner")?
            .call1((strategy.clone(), "momentum-alpha-v1"))?;

        let inst = m.getattr("InstrumentId")?.call1(("NSE", "RELIANCE"))?;
        let tick_cls = m.getattr("Tick")?;
        let tick1 = tick_cls.call_method1(
            "ltp",
            (inst.clone(), decimal(py, "2450.75")?, decimal(py, "100")?, "", ""),
        )?;
        let tick2 = tick_cls.call_method1(
            "ltp",
            (inst, decimal(py, "2451.00")?, decimal(py, "150")?, "", ""),
        )?;

        let checksum: i64 = runner
            .call_method1("run_ticks", (vec![tick1, tick2],))?
            .extract()?;
        assert_eq!(checksum, 2, "checksum counts ticks with positive price");

        let ticks: i64 = strategy.getattr("ticks")?.extract()?;
        assert_eq!(ticks, 2, "on_tick must fire once per tick");

        Ok(())
    })
    .unwrap();
}

#[test]
fn submit_order_gates_and_acks() {
    Python::attach(|py| -> PyResult<()> {
        let m = module(py)?;
        let strategy = make_strategy(py, COUNTING_STRATEGY)?;
        let runner = m
            .getattr("StrategyRunner")?
            .call1((strategy.clone(), "momentum-alpha-v1"))?;

        // The runner's internal gate starts empty: register the algo.
        runner.call_method1("register_algo", ("momentum-alpha-v1",))?;

        // A gated order returns a FILLED ack and dispatches on_order_event.
        let order = limit_order(py, &m, "2450.75")?;
        let ack = runner.call_method1("submit_order", (order, TS))?;
        let status: String = ack.getattr("status")?.extract()?;
        assert_eq!(status, "FILLED");
        let events: i64 = strategy.getattr("events")?.extract()?;
        assert_eq!(events, 1, "on_order_event must fire once per ack");

        // A global kill-switch refuses new submissions.
        runner.call_method1("halt_global", ("drill", TS))?;
        let halted: bool = runner.call_method0("is_halted")?.extract()?;
        assert!(halted);
        let order2 = limit_order(py, &m, "2450.75")?;
        let err = runner
            .call_method1("submit_order", (order2, TS))
            .expect_err("a global halt must refuse submissions");
        assert!(err.is_instance_of::<vyapaar_ffi_py::RiskRefused>(py));

        // Resuming restores order flow.
        runner.call_method1("resume_global", ())?;
        let order3 = limit_order(py, &m, "2450.75")?;
        let ack2 = runner.call_method1("submit_order", (order3, TS))?;
        let status2: String = ack2.getattr("status")?.extract()?;
        assert_eq!(status2, "FILLED");

        Ok(())
    })
    .unwrap();
}

#[test]
fn now_iso_rfc3339() {
    Python::attach(|py| -> PyResult<()> {
        let m = module(py)?;
        let s: String = m.getattr("now_iso")?.call0()?.extract()?;

        // RFC3339: contains 'T', ends with 'Z' or '+00:00', and parses.
        assert!(s.contains('T'), "RFC3339 must contain a 'T' separator: {s}");
        assert!(
            s.ends_with('Z') || s.ends_with("+00:00"),
            "RFC3339 UTC must end with Z or +00:00: {s}"
        );
        assert!(
            chrono::DateTime::parse_from_rfc3339(&s).is_ok(),
            "timestamp must parse as RFC3339: {s}"
        );

        Ok(())
    })
    .unwrap();
}
