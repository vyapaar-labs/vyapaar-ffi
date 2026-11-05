//! Integration tests for the value-type, market-data, order, and decision
//! pyclasses exposed by the `vyapaar._vyapaar` module.
//!
//! The module is driven entirely through Python objects (`PyModule::new` +
//! `init_module`) because pyo3 generates Rust-private items for
//! `#[pyclass]`/`#[pyfunction]`.

use pyo3::prelude::*;
use pyo3::types::PyModule;
use vyapaar_ffi_py::init_module;

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

/// Extract the string form of a Python value.
fn py_str<'py>(value: &Bound<'py, PyAny>) -> PyResult<String> {
    value.str()?.extract()
}

#[test]
fn price_roundtrip() {
    Python::attach(|py| -> PyResult<()> {
        let m = module(py)?;
        let price_cls = m.getattr("Price")?;

        let p = price_cls.call1((decimal(py, "2450.75")?,))?;
        let to_str: String = p.call_method0("to_str")?.extract()?;
        assert_eq!(to_str, "2450.75");

        // The `value` getter is an exact `decimal.Decimal`.
        let value_str = py_str(&p.getattr("value")?)?;
        assert_eq!(value_str, "2450.75");

        let positive: bool = p.call_method0("is_positive")?.extract()?;
        assert!(positive);

        let aligned: bool = p
            .call_method1("is_tick_aligned", (decimal(py, "0.05")?,))?
            .extract()?;
        assert!(aligned);

        let not_aligned: bool = p
            .call_method1("is_tick_aligned", (decimal(py, "0.10")?,))?
            .extract()?;
        assert!(!not_aligned);

        // Negative prices are rejected at the boundary.
        let negative = price_cls.call1((decimal(py, "-1")?,));
        assert!(negative.is_err());

        Ok(())
    })
    .unwrap();
}

#[test]
fn quantity_is_lot_multiple() {
    Python::attach(|py| -> PyResult<()> {
        let m = module(py)?;
        let qty_cls = m.getattr("Quantity")?;

        let q = qty_cls.call1((decimal(py, "150")?,))?;
        let is_lot: bool = q.call_method1("is_lot_multiple", (75u64,))?.extract()?;
        assert!(is_lot);

        let not_lot: bool = q.call_method1("is_lot_multiple", (40u64,))?.extract()?;
        assert!(!not_lot);

        let to_str: String = q.call_method0("to_str")?.extract()?;
        assert_eq!(to_str, "150");

        Ok(())
    })
    .unwrap();
}

#[test]
fn currency_exchange_instrument_id() {
    Python::attach(|py| -> PyResult<()> {
        let m = module(py)?;

        let currency_cls = m.getattr("Currency")?;
        let inr = currency_cls.call1(("inr",))?;
        let code: String = inr.getattr("code")?.extract()?;
        assert_eq!(code, "INR");
        let bad = currency_cls.call1(("XXX",));
        assert!(bad.is_err());

        let exchange_cls = m.getattr("Exchange")?;
        let nfo = exchange_cls.call1(("nfo",))?;
        let ex_code: String = nfo.getattr("code")?.extract()?;
        assert_eq!(ex_code, "NFO");
        let name: String = nfo.getattr("name")?.extract()?;
        assert_eq!(name, "Nfo");

        let instrument_cls = m.getattr("InstrumentId")?;
        let inst = instrument_cls.call1(("NSE", "RELIANCE"))?;
        let to_str: String = inst.call_method0("to_str")?.extract()?;
        assert_eq!(to_str, "NSE:RELIANCE");

        let round = instrument_cls.call_method1("from_str", ("NFO:NIFTY24DEC24000CE",))?;
        let symbol: String = round.getattr("symbol")?.extract()?;
        assert_eq!(symbol, "NIFTY24DEC24000CE");
        let exchange: String = round.getattr("exchange")?.extract()?;
        assert_eq!(exchange, "NFO");

        // Invalid exchange raises.
        let bad = instrument_cls.call1(("BOGUS", "X"));
        assert!(bad.is_err());

        Ok(())
    })
    .unwrap();
}

#[test]
fn order_side_opposite() {
    Python::attach(|py| -> PyResult<()> {
        let m = module(py)?;
        let side_cls = m.getattr("OrderSide")?;

        let buy = side_cls.call1(("BUY",))?;
        let opposite = buy.call_method0("opposite")?;
        let code: String = opposite.getattr("code")?.extract()?;
        assert_eq!(code, "SELL");

        let bad = side_cls.call1(("HOLD",));
        assert!(bad.is_err());

        Ok(())
    })
    .unwrap();
}

#[test]
fn bar_roundtrip() {
    Python::attach(|py| -> PyResult<()> {
        let m = module(py)?;
        let instrument_cls = m.getattr("InstrumentId")?;
        let bar_cls = m.getattr("Bar")?;

        let inst = instrument_cls.call1(("NSE", "RELIANCE"))?;
        let bar = bar_cls.call1((
            inst,
            decimal(py, "100.5")?,
            decimal(py, "105.0")?,
            decimal(py, "99.5")?,
            decimal(py, "103.75")?,
            decimal(py, "1500")?,
            "2024-01-15T09:30:00+05:30",
        ))?;

        let open_str = py_str(&bar.getattr("open")?)?;
        assert_eq!(open_str, "100.5");

        let close_str = py_str(&bar.getattr("close")?)?;
        assert_eq!(close_str, "103.75");

        let volume_str = py_str(&bar.getattr("volume")?)?;
        assert_eq!(volume_str, "1500");

        let ts: String = bar.getattr("timestamp")?.extract()?;
        assert!(!ts.is_empty());

        let instr = bar.getattr("instrument")?;
        let instr_str: String = instr.call_method0("to_str")?.extract()?;
        assert_eq!(instr_str, "NSE:RELIANCE");

        Ok(())
    })
    .unwrap();
}

#[test]
fn tick_kinds() {
    Python::attach(|py| -> PyResult<()> {
        let m = module(py)?;
        let instrument_cls = m.getattr("InstrumentId")?;
        let tick_cls = m.getattr("Tick")?;

        let inst = instrument_cls.call1(("NSE", "RELIANCE"))?;

        let ltp = tick_cls.call_method1(
            "ltp",
            (
                inst.clone(),
                decimal(py, "2450.75")?,
                decimal(py, "100")?,
                "",
                "",
            ),
        )?;
        let kind: String = ltp.getattr("kind")?.extract()?;
        assert_eq!(kind, "ltp");

        let quote = tick_cls.call_method1(
            "quote",
            (
                inst,
                decimal(py, "2450.00")?,
                decimal(py, "2450.75")?,
                decimal(py, "100")?,
                decimal(py, "200")?,
            ),
        )?;
        let qkind: String = quote.getattr("kind")?.extract()?;
        assert_eq!(qkind, "quote");

        // An invalid `last_price` on a quote raises ConversionError.
        let kwargs = pyo3::types::PyDict::new(py);
        kwargs.set_item("last_price", "not-a-decimal")?;
        let bad_quote = tick_cls.call(
            (
                instrument_cls.call1(("NSE", "RELIANCE"))?,
                decimal(py, "2450.00")?,
                decimal(py, "2450.75")?,
                decimal(py, "100")?,
                decimal(py, "200")?,
            ),
            Some(&kwargs),
        );
        assert!(bad_quote.is_err());

        Ok(())
    })
    .unwrap();
}

#[test]
fn order_request_keyword_limits() {
    Python::attach(|py| -> PyResult<()> {
        let m = module(py)?;
        let instrument_cls = m.getattr("InstrumentId")?;
        let side_cls = m.getattr("OrderSide")?;
        let req_cls = m.getattr("OrderRequest")?;

        let inst = instrument_cls.call1(("NSE", "RELIANCE"))?;
        let side = side_cls.call1(("BUY",))?;

        let kwargs = pyo3::types::PyDict::new(py);
        kwargs.set_item("limit_price", decimal(py, "102.25")?)?;

        let req = req_cls.call(
            (
                inst.clone(),
                side.clone(),
                "LIMIT",
                decimal(py, "150")?,
                "momentum-alpha-v1",
            ),
            Some(&kwargs),
        )?;

        let cid: String = req.getattr("client_order_id")?.extract()?;
        assert!(!cid.is_empty());

        let algo: String = req.getattr("algo_id")?.extract()?;
        assert_eq!(algo, "momentum-alpha-v1");

        let ot: String = req.getattr("order_type")?.extract()?;
        assert_eq!(ot, "LIMIT");

        let lp_str = py_str(&req.getattr("limit_price")?)?;
        assert_eq!(lp_str, "102.25");

        let qty_str = py_str(&req.getattr("quantity")?)?;
        assert_eq!(qty_str, "150");

        let tif: String = req.getattr("tif")?.extract()?;
        assert_eq!(tif, "DAY");

        // A LIMIT order without a limit_price raises.
        let missing = req_cls.call(
            (inst, side, "LIMIT", decimal(py, "10")?, "momentum-alpha-v1"),
            None,
        );
        assert!(missing.is_err());

        Ok(())
    })
    .unwrap();
}

#[test]
fn order_ack_and_fill_report() {
    Python::attach(|py| -> PyResult<()> {
        let m = module(py)?;
        let instrument_cls = m.getattr("InstrumentId")?;
        let side_cls = m.getattr("OrderSide")?;
        let status_cls = m.getattr("OrderStatus")?;
        let ack_cls = m.getattr("OrderAck")?;
        let fill_cls = m.getattr("FillReport")?;

        let inst = instrument_cls.call1(("NSE", "RELIANCE"))?;
        let side = side_cls.call1(("BUY",))?;
        let status = status_cls.call1(("FILLED",))?;

        let ack = ack_cls.call1((
            "ORD-1",
            "11111111-1111-4111-8111-111111111111",
            status,
            "2024-01-15T09:30:05+00:00",
        ))?;
        let order_id: String = ack.getattr("order_id")?.extract()?;
        assert_eq!(order_id, "ORD-1");
        let status_code: String = ack.getattr("status")?.extract()?;
        assert_eq!(status_code, "FILLED");
        let ack_ts: String = ack.getattr("timestamp")?.extract()?;
        assert!(!ack_ts.is_empty());

        let fill = fill_cls.call1((
            "FILL-1",
            "ORD-1",
            "11111111-1111-4111-8111-111111111111",
            inst,
            side,
            decimal(py, "102.25")?,
            decimal(py, "150")?,
            "momentum-alpha-v1",
            "2024-01-15T09:30:05+00:00",
        ))?;
        let fill_id: String = fill.getattr("fill_id")?.extract()?;
        assert_eq!(fill_id, "FILL-1");
        let price_str = py_str(&fill.getattr("price")?)?;
        assert_eq!(price_str, "102.25");
        let qty_str = py_str(&fill.getattr("quantity")?)?;
        assert_eq!(qty_str, "150");

        Ok(())
    })
    .unwrap();
}

#[test]
fn decision_closed_grammar() {
    Python::attach(|py| -> PyResult<()> {
        let m = module(py)?;
        let decision_cls = m.getattr("Decision")?;

        let hold = decision_cls
            .call_method1("from_json", ("{\"action\":\"hold\",\"reason\":\"test\"}",))?;
        let action: String = hold.getattr("action")?.extract()?;
        assert_eq!(action, "hold");

        let place = decision_cls.call_method1(
            "from_json",
            (
                "{\"action\":\"place_order\",\"symbol\":\"RELIANCE\",\"side\":\"BUY\",\"qty\":\"150\",\"price_limit\":\"2450.75\",\"algo_id\":\"momentum-alpha-v1\"}",
            ),
        )?;
        let d = place.call_method0("to_dict")?;
        let d_action: String = d.get_item("action")?.extract()?;
        assert_eq!(d_action, "place_order");
        let d_symbol: String = d.get_item("symbol")?.extract()?;
        assert_eq!(d_symbol, "RELIANCE");
        let d_qty: String = d.get_item("qty")?.extract()?;
        assert_eq!(d_qty, "150");
        let d_lim: String = d.get_item("price_limit")?.extract()?;
        assert_eq!(d_lim, "2450.75");

        // Invalid JSON raises.
        assert!(decision_cls
            .call_method1("from_json", ("not-json",))
            .is_err());

        // `deny_unknown_fields` rejects extra keys.
        let extra = decision_cls.call_method1(
            "from_json",
            ("{\"action\":\"hold\",\"reason\":\"test\",\"bogus\":1}",),
        );
        assert!(extra.is_err());

        // Validation rejects a malformed algo_id.
        let bad_algo = decision_cls.call_method1(
            "from_json",
            (
                "{\"action\":\"place_order\",\"symbol\":\"RELIANCE\",\"side\":\"BUY\",\"qty\":\"150\",\"algo_id\":\"x\"}",
            ),
        );
        assert!(bad_algo.is_err());

        Ok(())
    })
    .unwrap();
}
