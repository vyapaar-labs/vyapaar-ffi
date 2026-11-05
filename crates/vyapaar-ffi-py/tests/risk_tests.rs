//! Integration tests for the `RiskEngine` pre-trade risk gate pyclass.
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

/// Build a fresh `RiskEngine` instance.
fn risk_engine<'py>(m: &Bound<'py, PyModule>) -> PyResult<Bound<'py, PyAny>> {
    m.getattr("RiskEngine")?.call0()
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

/// Whether `err` is a `RiskRefused` exception.
fn is_risk_refused(py: Python<'_>, err: &PyErr) -> bool {
    err.is_instance_of::<vyapaar_ffi_py::RiskRefused>(py)
}

#[test]
fn register_and_check() {
    Python::attach(|py| -> PyResult<()> {
        let m = module(py)?;
        let risk = risk_engine(&m)?;

        risk.call_method1("register_algo", ("momentum-alpha-v1",))?;

        let order = limit_order(py, &m, "2450.75")?;
        risk.call_method1("check_order", (order, TS))?;

        let registered: bool = risk
            .call_method1("is_algo_registered", ("momentum-alpha-v1",))?
            .extract()?;
        assert!(registered);

        let refusals: u64 = risk.call_method0("total_refusals")?.extract()?;
        assert_eq!(refusals, 0);

        Ok(())
    })
    .unwrap();
}

#[test]
fn unregistered_refused() {
    Python::attach(|py| -> PyResult<()> {
        let m = module(py)?;
        let risk = risk_engine(&m)?;

        let order = limit_order(py, &m, "2450.75")?;
        let err = risk
            .call_method1("check_order", (order, TS))
            .expect_err("an unregistered algo must be refused");
        assert!(is_risk_refused(py, &err));

        let refusals: u64 = risk.call_method0("total_refusals")?.extract()?;
        assert!(refusals >= 1);

        Ok(())
    })
    .unwrap();
}

#[test]
fn halt_blocks() {
    Python::attach(|py| -> PyResult<()> {
        let m = module(py)?;
        let risk = risk_engine(&m)?;

        risk.call_method1("register_algo", ("momentum-alpha-v1",))?;
        risk.call_method1("halt_global", ("drill", TS))?;

        let halted: bool = risk.call_method0("is_halted")?.extract()?;
        assert!(halted);

        let state: String = risk.call_method0("global_state")?.extract()?;
        assert_eq!(state, "HALTED");

        let order = limit_order(py, &m, "2450.75")?;
        let err = risk
            .call_method1("check_order", (order, TS))
            .expect_err("a global kill-switch must refuse registered algos too");
        assert!(is_risk_refused(py, &err));

        Ok(())
    })
    .unwrap();
}

#[test]
fn price_band() {
    Python::attach(|py| -> PyResult<()> {
        let m = module(py)?;
        let risk = risk_engine(&m)?;

        risk.call_method1("register_algo", ("momentum-alpha-v1",))?;
        let inst = m.getattr("InstrumentId")?.call1(("NSE", "RELIANCE"))?;
        risk.call_method1("set_price_band", (inst, "2400", "2500", "2450"))?;

        // A limit price inside the band passes.
        let ok_order = limit_order(py, &m, "2450")?;
        risk.call_method1("check_order", (ok_order, TS))?;

        // A limit price outside the band is refused (PriceBandViolation).
        let bad_order = limit_order(py, &m, "2600")?;
        let err = risk
            .call_method1("check_order", (bad_order, TS))
            .expect_err("a limit price above the band must be refused");
        assert!(is_risk_refused(py, &err));

        Ok(())
    })
    .unwrap();
}

#[test]
fn decision_gate() {
    Python::attach(|py| -> PyResult<()> {
        let m = module(py)?;
        let risk = risk_engine(&m)?;
        risk.call_method1("register_algo", ("momentum-alpha-v1",))?;

        // `hold` decisions always pass the gate.
        risk.call_method1(
            "check_decision",
            ("{\"action\":\"hold\",\"reason\":\"x\"}", TS),
        )?;

        // A `place_order` for a registered algo passes.
        let place = "{\"action\":\"place_order\",\"symbol\":\"RELIANCE\",\"side\":\"BUY\",\"qty\":\"10\",\"algo_id\":\"momentum-alpha-v1\"}";
        risk.call_method1("check_decision", (place, TS))?;

        // A fresh (empty) gate refuses the same decision: algo not registered.
        let risk2 = risk_engine(&m)?;
        let err = risk2
            .call_method1("check_decision", (place, TS))
            .expect_err("an unregistered algo decision must be refused");
        assert!(is_risk_refused(py, &err));

        Ok(())
    })
    .unwrap();
}
