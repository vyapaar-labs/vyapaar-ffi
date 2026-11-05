//! Order pyclasses: `OrderRequest`, `OrderAck`, `FillReport`, `OrderEvent`.
//!
//! Every order carries the SEBI-mandated `algo_id`; prices/quantities cross
//! the boundary as exact `decimal.Decimal` values.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};

use crate::decimal::decimal_to_pydecimal;
use crate::error::{conversion_err, parse_utc};
use crate::types::{
    InstrumentId, OrderSide, OrderStatus, price_from_any, quantity_from_any, status_code,
};

fn side_code(side: vyapaar_core::OrderSide) -> &'static str {
    match side {
        vyapaar_core::OrderSide::Buy => "BUY",
        vyapaar_core::OrderSide::Sell => "SELL",
    }
}

fn parse_uuid(s: &str) -> PyResult<vyapaar_core::uuid::Uuid> {
    vyapaar_core::uuid::Uuid::parse_str(s)
        .map_err(|e| conversion_err(format!("invalid client_order_id {s:?}: {e}")))
}

/// A validated order placement request.
#[pyclass(module = "vyapaar._vyapaar")]
#[derive(Clone)]
pub struct OrderRequest {
    pub(crate) inner: vyapaar_core::OrderRequest,
}

impl OrderRequest {
    /// Lift a core `OrderRequest` into its Python wrapper (public Rust API).
    #[allow(dead_code)]
    pub(crate) fn from_core(inner: vyapaar_core::OrderRequest) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl OrderRequest {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (instrument, side, order_type, quantity, algo_id, *, client_order_id=None, limit_price=None, trigger_price=None, tif="DAY", tag=None))]
    fn new(
        instrument: &Bound<'_, InstrumentId>,
        side: &Bound<'_, OrderSide>,
        order_type: &str,
        quantity: &Bound<'_, PyAny>,
        algo_id: &str,
        client_order_id: Option<&str>,
        limit_price: Option<&Bound<'_, PyAny>>,
        trigger_price: Option<&Bound<'_, PyAny>>,
        tif: &str,
        tag: Option<&str>,
    ) -> PyResult<Self> {
        let order_type = match order_type.to_uppercase().as_str() {
            "MARKET" => vyapaar_core::OrderType::Market,
            "LIMIT" => {
                let price = limit_price.ok_or_else(|| {
                    conversion_err("LIMIT order requires a keyword-only limit_price")
                })?;
                vyapaar_core::OrderType::Limit(price_from_any(price)?)
            }
            "STOP_LOSS" => {
                let trigger = trigger_price.ok_or_else(|| {
                    conversion_err("STOP_LOSS order requires a keyword-only trigger_price")
                })?;
                vyapaar_core::OrderType::StopLoss(price_from_any(trigger)?)
            }
            "STOP_LIMIT" => {
                let trigger = trigger_price.ok_or_else(|| {
                    conversion_err("STOP_LIMIT order requires a keyword-only trigger_price")
                })?;
                let limit = limit_price.ok_or_else(|| {
                    conversion_err("STOP_LIMIT order requires a keyword-only limit_price")
                })?;
                vyapaar_core::OrderType::StopLimit {
                    trigger: price_from_any(trigger)?,
                    limit: price_from_any(limit)?,
                }
            }
            other => return Err(conversion_err(format!("invalid OrderType {other:?}"))),
        };

        let client_order_id = match client_order_id {
            Some(id) => parse_uuid(id)?,
            None => vyapaar_core::uuid::Uuid::new_v4(),
        };

        let tif = match tif.to_uppercase().as_str() {
            "DAY" => vyapaar_core::TimeInForce::Day,
            "IOC" => vyapaar_core::TimeInForce::Ioc,
            "GTC" => vyapaar_core::TimeInForce::Gtc,
            other => return Err(conversion_err(format!("invalid TimeInForce {other:?}"))),
        };

        let quantity = quantity_from_any(quantity)?;

        let mut inner = vyapaar_core::OrderRequest::with_id(
            client_order_id,
            instrument.borrow().inner.clone(),
            side.borrow().inner,
            order_type,
            quantity,
            algo_id,
        );
        inner = inner.with_client_id("python");
        inner.tif = tif;
        inner.tag = tag.map(ToOwned::to_owned);
        Ok(Self { inner })
    }

    #[getter]
    fn client_order_id(&self) -> String {
        self.inner.client_order_id.to_string()
    }

    #[getter]
    fn instrument(&self) -> InstrumentId {
        InstrumentId {
            inner: self.inner.instrument.clone(),
        }
    }

    #[getter]
    fn side(&self) -> OrderSide {
        OrderSide {
            inner: self.inner.side,
        }
    }

    #[getter]
    fn order_type(&self) -> &'static str {
        match &self.inner.order_type {
            vyapaar_core::OrderType::Market => "MARKET",
            vyapaar_core::OrderType::Limit(_) => "LIMIT",
            vyapaar_core::OrderType::StopLoss(_) => "STOP_LOSS",
            vyapaar_core::OrderType::StopLimit { .. } => "STOP_LIMIT",
        }
    }

    #[getter]
    fn quantity<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        decimal_to_pydecimal(py, self.inner.quantity.0)
    }

    #[getter]
    fn limit_price<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        match self.inner.order_type.limit_price() {
            Some(price) => decimal_to_pydecimal(py, price.0).map(Some),
            None => Ok(None),
        }
    }

    #[getter]
    fn tif(&self) -> &'static str {
        match self.inner.tif {
            vyapaar_core::TimeInForce::Day => "DAY",
            vyapaar_core::TimeInForce::Ioc => "IOC",
            vyapaar_core::TimeInForce::Gtc => "GTC",
        }
    }

    #[getter]
    fn algo_id(&self) -> &str {
        &self.inner.algo_id
    }

    #[getter]
    fn tag(&self) -> Option<&str> {
        self.inner.tag.as_deref()
    }

    /// String-valued dict for JSON-ish serialization.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let dict = PyDict::new(py);
        dict.set_item("client_order_id", self.inner.client_order_id.to_string())?;
        dict.set_item("instrument", self.inner.instrument.to_string())?;
        dict.set_item("side", side_code(self.inner.side))?;
        dict.set_item("order_type", self.order_type())?;
        dict.set_item("quantity", self.inner.quantity.0.to_string())?;
        match self.inner.order_type.limit_price() {
            Some(price) => dict.set_item("limit_price", price.0.to_string())?,
            None => dict.set_item("limit_price", "")?,
        }
        dict.set_item("tif", self.tif())?;
        dict.set_item("algo_id", &self.inner.algo_id)?;
        dict.set_item("tag", self.inner.tag.clone().unwrap_or_default())?;
        Ok(dict.into_any())
    }
}

/// Acknowledgment of order placement from broker or execution engine.
#[pyclass(module = "vyapaar._vyapaar")]
#[derive(Clone)]
pub struct OrderAck {
    pub(crate) inner: vyapaar_core::OrderAck,
}

impl OrderAck {
    pub(crate) fn from_core(inner: vyapaar_core::OrderAck) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl OrderAck {
    #[new]
    fn new(
        order_id: &str,
        client_order_id: &str,
        status: &Bound<'_, OrderStatus>,
        timestamp: &str,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: vyapaar_core::OrderAck::new(
                order_id,
                parse_uuid(client_order_id)?,
                status.borrow().inner,
                parse_utc(timestamp)?,
            ),
        })
    }

    #[getter]
    fn order_id(&self) -> String {
        self.inner.order_id.clone()
    }

    #[getter]
    fn client_order_id(&self) -> String {
        self.inner.client_order_id.to_string()
    }

    #[getter]
    fn status(&self) -> &'static str {
        status_code(self.inner.status)
    }

    #[getter]
    fn timestamp(&self) -> String {
        self.inner.timestamp.to_rfc3339()
    }

    /// String-valued dict for JSON-ish serialization.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let dict = PyDict::new(py);
        dict.set_item("order_id", self.inner.order_id.clone())?;
        dict.set_item("client_order_id", self.inner.client_order_id.to_string())?;
        dict.set_item("status", status_code(self.inner.status))?;
        dict.set_item("timestamp", self.inner.timestamp.to_rfc3339())?;
        Ok(dict.into_any())
    }
}

/// Execution fill report.
#[pyclass(module = "vyapaar._vyapaar")]
#[derive(Clone)]
pub struct FillReport {
    pub(crate) inner: vyapaar_core::FillReport,
}

impl FillReport {
    /// Lift a core `FillReport` into its Python wrapper (public Rust API).
    #[allow(dead_code)]
    pub(crate) fn from_core(inner: vyapaar_core::FillReport) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl FillReport {
    #[new]
    #[allow(clippy::too_many_arguments)]
    fn new(
        fill_id: &str,
        order_id: &str,
        client_order_id: &str,
        instrument: &Bound<'_, InstrumentId>,
        side: &Bound<'_, OrderSide>,
        price: &Bound<'_, PyAny>,
        quantity: &Bound<'_, PyAny>,
        algo_id: &str,
        timestamp: &str,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: vyapaar_core::FillReport::new(
                fill_id,
                order_id,
                parse_uuid(client_order_id)?,
                instrument.borrow().inner.clone(),
                side.borrow().inner,
                price_from_any(price)?,
                quantity_from_any(quantity)?,
                algo_id,
                parse_utc(timestamp)?,
                "python",
            ),
        })
    }

    #[getter]
    fn fill_id(&self) -> String {
        self.inner.fill_id.clone()
    }

    #[getter]
    fn order_id(&self) -> String {
        self.inner.order_id.clone()
    }

    #[getter]
    fn client_order_id(&self) -> String {
        self.inner.client_order_id.to_string()
    }

    #[getter]
    fn instrument(&self) -> InstrumentId {
        InstrumentId {
            inner: self.inner.instrument.clone(),
        }
    }

    #[getter]
    fn side(&self) -> OrderSide {
        OrderSide {
            inner: self.inner.side,
        }
    }

    #[getter]
    fn price<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        decimal_to_pydecimal(py, self.inner.price.0)
    }

    #[getter]
    fn quantity<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        decimal_to_pydecimal(py, self.inner.quantity.0)
    }

    #[getter]
    fn algo_id(&self) -> &str {
        &self.inner.algo_id
    }

    #[getter]
    fn timestamp(&self) -> String {
        self.inner.timestamp.to_rfc3339()
    }

    /// String-valued dict for JSON-ish serialization.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let dict = PyDict::new(py);
        dict.set_item("fill_id", self.inner.fill_id.clone())?;
        dict.set_item("order_id", self.inner.order_id.clone())?;
        dict.set_item("client_order_id", self.inner.client_order_id.to_string())?;
        dict.set_item("instrument", self.inner.instrument.to_string())?;
        dict.set_item("side", side_code(self.inner.side))?;
        dict.set_item("price", self.inner.price.0.to_string())?;
        dict.set_item("quantity", self.inner.quantity.0.to_string())?;
        dict.set_item("algo_id", &self.inner.algo_id)?;
        dict.set_item("timestamp", self.inner.timestamp.to_rfc3339())?;
        Ok(dict.into_any())
    }
}

/// A single lifecycle event for an order (acked / filled / rejected / cancelled).
#[pyclass(module = "vyapaar._vyapaar")]
#[derive(Clone)]
pub struct OrderEvent {
    pub(crate) inner: vyapaar_core::OrderEvent,
}

impl OrderEvent {
    /// Lift a core `OrderEvent` into its Python wrapper (public Rust API).
    #[allow(dead_code)]
    pub(crate) fn from_core(inner: vyapaar_core::OrderEvent) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl OrderEvent {
    #[getter]
    fn kind(&self) -> &'static str {
        match &self.inner {
            vyapaar_core::OrderEvent::Acked(_) => "acked",
            vyapaar_core::OrderEvent::Filled(_) => "filled",
            vyapaar_core::OrderEvent::Rejected { .. } => "rejected",
            vyapaar_core::OrderEvent::Cancelled { .. } => "cancelled",
        }
    }

    /// The event payload as a string-valued dict.
    #[getter]
    fn payload<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let dict = PyDict::new(py);
        match &self.inner {
            vyapaar_core::OrderEvent::Acked(ack) => {
                dict.set_item("kind", "acked")?;
                dict.set_item("order_id", ack.order_id.clone())?;
                dict.set_item("client_order_id", ack.client_order_id.to_string())?;
                dict.set_item("status", status_code(ack.status))?;
                dict.set_item("timestamp", ack.timestamp.to_rfc3339())?;
            }
            vyapaar_core::OrderEvent::Filled(fill) => {
                dict.set_item("kind", "filled")?;
                dict.set_item("fill_id", fill.fill_id.clone())?;
                dict.set_item("order_id", fill.order_id.clone())?;
                dict.set_item("client_order_id", fill.client_order_id.to_string())?;
                dict.set_item("instrument", fill.instrument.to_string())?;
                dict.set_item("side", side_code(fill.side))?;
                dict.set_item("price", fill.price.0.to_string())?;
                dict.set_item("quantity", fill.quantity.0.to_string())?;
                dict.set_item("algo_id", &fill.algo_id)?;
                dict.set_item("timestamp", fill.timestamp.to_rfc3339())?;
            }
            vyapaar_core::OrderEvent::Rejected { order_id, reason } => {
                dict.set_item("kind", "rejected")?;
                dict.set_item("order_id", order_id.clone())?;
                dict.set_item("reason", format!("{reason:?}"))?;
            }
            vyapaar_core::OrderEvent::Cancelled { order_id } => {
                dict.set_item("kind", "cancelled")?;
                dict.set_item("order_id", order_id.clone())?;
            }
        }
        Ok(dict.into_any())
    }

    #[getter]
    fn order_id(&self) -> Option<String> {
        match &self.inner {
            vyapaar_core::OrderEvent::Acked(ack) => Some(ack.order_id.clone()),
            vyapaar_core::OrderEvent::Filled(fill) => Some(fill.order_id.clone()),
            vyapaar_core::OrderEvent::Rejected { order_id, .. } => Some(order_id.clone()),
            vyapaar_core::OrderEvent::Cancelled { order_id } => Some(order_id.clone()),
        }
    }
}

/// Register order classes onto the module.
pub(crate) fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<OrderRequest>()?;
    m.add_class::<OrderAck>()?;
    m.add_class::<FillReport>()?;
    m.add_class::<OrderEvent>()?;
    Ok(())
}
