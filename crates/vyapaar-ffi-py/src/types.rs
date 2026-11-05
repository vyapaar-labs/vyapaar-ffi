//! Canonical value-object pyclasses: `Price`, `Quantity`, `Currency`,
//! `Exchange`, `InstrumentId`, `OrderSide`, `TimeInForce`, `OrderStatus`.
//!
//! Money crosses the FFI boundary exclusively as exact `decimal.Decimal`
//! values (Zero Floats Policy); every price/quantity is converted through the
//! exact decimal-string helpers in [`crate::decimal`].

use std::str::FromStr;

use pyo3::prelude::*;
use pyo3::types::PyModule;

use crate::decimal::{decimal_to_pydecimal, pydecimal_to_decimal};
use crate::error::conversion_err;

/// Whether `other` exposes a `to_str()` method returning `s`.
fn same_to_str(s: &str, other: &Bound<'_, PyAny>) -> bool {
    other
        .call_method0("to_str")
        .is_ok_and(|v| v.extract::<String>().is_ok_and(|v| v == s))
}

/// Whether `other` exposes a `code` attribute equal to `code`.
fn same_code(code: &str, other: &Bound<'_, PyAny>) -> bool {
    other
        .getattr("code")
        .is_ok_and(|c| c.extract::<String>().is_ok_and(|c| c == code))
}

/// Parse a Python `decimal.Decimal` (or any `__str__` object) into a validated
/// core `Price`. Raises [`crate::ConversionError`] on non-decimal or negative
/// values.
pub(crate) fn price_from_any(value: &Bound<'_, PyAny>) -> PyResult<vyapaar_core::Price> {
    let d = pydecimal_to_decimal(value)?;
    vyapaar_core::Price::try_new(d).map_err(conversion_err)
}

/// Parse a Python `decimal.Decimal` (or any `__str__` object) into a validated
/// core `Quantity`. Raises [`crate::ConversionError`] on non-decimal or
/// negative values.
pub(crate) fn quantity_from_any(value: &Bound<'_, PyAny>) -> PyResult<vyapaar_core::Quantity> {
    let d = pydecimal_to_decimal(value)?;
    vyapaar_core::Quantity::try_new(d).map_err(conversion_err)
}

/// Map a core `OrderStatus` to its uppercase snake_code wire form
/// (`"PENDING_NEW"`, `"PARTIALLY_FILLED"`, ...).
pub(crate) fn status_code(status: vyapaar_core::OrderStatus) -> &'static str {
    match status {
        vyapaar_core::OrderStatus::PendingNew => "PENDING_NEW",
        vyapaar_core::OrderStatus::New => "NEW",
        vyapaar_core::OrderStatus::PartiallyFilled => "PARTIALLY_FILLED",
        vyapaar_core::OrderStatus::Filled => "FILLED",
        vyapaar_core::OrderStatus::Cancelled => "CANCELLED",
        vyapaar_core::OrderStatus::Rejected => "REJECTED",
        vyapaar_core::OrderStatus::Expired => "EXPIRED",
    }
}

/// Exact-decimal price value object.
#[pyclass(module = "vyapaar._vyapaar")]
#[derive(Clone)]
pub struct Price {
    pub(crate) inner: vyapaar_core::Price,
}

#[pymethods]
impl Price {
    #[new]
    #[pyo3(signature = (value))]
    fn new(value: &Bound<'_, PyAny>) -> PyResult<Self> {
        let d = pydecimal_to_decimal(value)?;
        let inner = vyapaar_core::Price::try_new(d).map_err(conversion_err)?;
        Ok(Self { inner })
    }

    /// The exact decimal value as a Python `decimal.Decimal`.
    #[getter]
    fn value<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        decimal_to_pydecimal(py, self.inner.0)
    }

    /// Exact decimal string form.
    fn to_str(&self) -> String {
        self.inner.0.to_string()
    }

    fn is_positive(&self) -> bool {
        self.inner.is_positive()
    }

    fn is_tick_aligned(&self, tick: &Bound<'_, PyAny>) -> PyResult<bool> {
        let tick = pydecimal_to_decimal(tick)?;
        Ok(self.inner.is_tick_aligned(tick))
    }

    fn __str__(&self) -> String {
        format!("Price({})", self.inner.0)
    }

    fn __repr__(&self) -> String {
        format!("Price({})", self.inner.0)
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        same_to_str(&self.to_str(), other)
    }
}

/// Exact-decimal quantity value object.
#[pyclass(module = "vyapaar._vyapaar")]
#[derive(Clone)]
pub struct Quantity {
    pub(crate) inner: vyapaar_core::Quantity,
}

#[pymethods]
impl Quantity {
    #[new]
    #[pyo3(signature = (value))]
    fn new(value: &Bound<'_, PyAny>) -> PyResult<Self> {
        let d = pydecimal_to_decimal(value)?;
        let inner = vyapaar_core::Quantity::try_new(d).map_err(conversion_err)?;
        Ok(Self { inner })
    }

    /// The exact decimal value as a Python `decimal.Decimal`.
    #[getter]
    fn value<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        decimal_to_pydecimal(py, self.inner.0)
    }

    /// Exact decimal string form.
    fn to_str(&self) -> String {
        self.inner.0.to_string()
    }

    fn is_positive(&self) -> bool {
        self.inner.is_positive()
    }

    fn is_lot_multiple(&self, lot_size: u64) -> bool {
        self.inner.is_lot_multiple(lot_size)
    }

    fn __str__(&self) -> String {
        format!("Quantity({})", self.inner.0)
    }

    fn __repr__(&self) -> String {
        format!("Quantity({})", self.inner.0)
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        same_to_str(&self.to_str(), other)
    }
}

/// ISO 4217 currency.
#[pyclass(module = "vyapaar._vyapaar")]
#[derive(Clone)]
pub struct Currency {
    pub(crate) inner: vyapaar_core::Currency,
}

#[pymethods]
impl Currency {
    #[new]
    fn new(code: &str) -> PyResult<Self> {
        let inner = vyapaar_core::Currency::from_code(code).map_err(conversion_err)?;
        Ok(Self { inner })
    }

    #[getter]
    fn code(&self) -> &'static str {
        self.inner.code()
    }

    #[staticmethod]
    fn from_code(code: &str) -> PyResult<Currency> {
        Self::new(code)
    }

    fn __str__(&self) -> &'static str {
        self.inner.code()
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        same_code(self.inner.code(), other)
    }
}

/// Indian market exchange venue.
#[pyclass(module = "vyapaar._vyapaar")]
#[derive(Clone)]
pub struct Exchange {
    pub(crate) inner: vyapaar_core::Exchange,
}

#[pymethods]
impl Exchange {
    #[new]
    fn new(code: &str) -> PyResult<Self> {
        let inner = vyapaar_core::Exchange::from_str(code).map_err(conversion_err)?;
        Ok(Self { inner })
    }

    #[getter]
    fn code(&self) -> &'static str {
        self.inner.as_str()
    }

    #[getter]
    fn name(&self) -> String {
        format!("{:?}", self.inner)
    }

    #[staticmethod]
    fn from_code(code: &str) -> PyResult<Exchange> {
        Self::new(code)
    }

    fn __str__(&self) -> &'static str {
        self.inner.as_str()
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        same_code(self.inner.as_str(), other)
    }
}

/// Canonical instrument identifier (`EXCHANGE:SYMBOL`).
#[pyclass(module = "vyapaar._vyapaar")]
#[derive(Clone)]
pub struct InstrumentId {
    pub(crate) inner: vyapaar_core::InstrumentId,
}

#[pymethods]
impl InstrumentId {
    #[new]
    #[pyo3(signature = (exchange, symbol, token=None, isin=None))]
    fn new(
        exchange: &str,
        symbol: &str,
        token: Option<String>,
        isin: Option<String>,
    ) -> PyResult<Self> {
        let ex = vyapaar_core::Exchange::from_str(exchange).map_err(conversion_err)?;
        let mut inner = vyapaar_core::InstrumentId::new(ex, symbol);
        if let Some(token) = token {
            inner = inner.with_token(token);
        }
        if let Some(isin) = isin {
            inner = inner.with_isin(isin);
        }
        Ok(Self { inner })
    }

    #[getter]
    fn exchange(&self) -> &'static str {
        self.inner.exchange.as_str()
    }

    #[getter]
    fn symbol(&self) -> &str {
        &self.inner.symbol
    }

    #[getter]
    fn token(&self) -> Option<&str> {
        self.inner.token.as_deref()
    }

    #[getter]
    fn isin(&self) -> Option<&str> {
        self.inner.isin.as_deref()
    }

    fn to_str(&self) -> String {
        self.inner.to_string()
    }

    #[staticmethod]
    fn from_str(s: &str) -> PyResult<InstrumentId> {
        let (exchange, symbol) = s.split_once(':').ok_or_else(|| {
            conversion_err(format!(
                "invalid InstrumentId {s:?}: expected \"EXCHANGE:SYMBOL\""
            ))
        })?;
        let ex = vyapaar_core::Exchange::from_str(exchange).map_err(conversion_err)?;
        Ok(InstrumentId {
            inner: vyapaar_core::InstrumentId::new(ex, symbol),
        })
    }
}

/// Order side: strictly `BUY` or `SELL`.
#[pyclass(module = "vyapaar._vyapaar")]
#[derive(Clone)]
pub struct OrderSide {
    pub(crate) inner: vyapaar_core::OrderSide,
}

#[pymethods]
impl OrderSide {
    #[new]
    fn new(code: &str) -> PyResult<Self> {
        let inner = match code.to_uppercase().as_str() {
            "BUY" => vyapaar_core::OrderSide::Buy,
            "SELL" => vyapaar_core::OrderSide::Sell,
            other => return Err(conversion_err(format!("invalid OrderSide {other:?}"))),
        };
        Ok(Self { inner })
    }

    #[getter]
    fn code(&self) -> &'static str {
        match self.inner {
            vyapaar_core::OrderSide::Buy => "BUY",
            vyapaar_core::OrderSide::Sell => "SELL",
        }
    }

    fn opposite(&self) -> Self {
        Self {
            inner: self.inner.opposite(),
        }
    }

    fn __str__(&self) -> &'static str {
        match self.inner {
            vyapaar_core::OrderSide::Buy => "BUY",
            vyapaar_core::OrderSide::Sell => "SELL",
        }
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        same_code(self.code(), other)
    }
}

/// Order time-in-force.
#[pyclass(module = "vyapaar._vyapaar")]
#[derive(Clone)]
pub struct TimeInForce {
    pub(crate) inner: vyapaar_core::TimeInForce,
}

#[pymethods]
impl TimeInForce {
    #[new]
    fn new(code: &str) -> PyResult<Self> {
        let inner = match code.to_uppercase().as_str() {
            "DAY" => vyapaar_core::TimeInForce::Day,
            "IOC" => vyapaar_core::TimeInForce::Ioc,
            "GTC" => vyapaar_core::TimeInForce::Gtc,
            other => return Err(conversion_err(format!("invalid TimeInForce {other:?}"))),
        };
        Ok(Self { inner })
    }

    #[getter]
    fn code(&self) -> &'static str {
        match self.inner {
            vyapaar_core::TimeInForce::Day => "DAY",
            vyapaar_core::TimeInForce::Ioc => "IOC",
            vyapaar_core::TimeInForce::Gtc => "GTC",
        }
    }
}

/// Lifecycle status of an order.
#[pyclass(module = "vyapaar._vyapaar")]
#[derive(Clone)]
pub struct OrderStatus {
    pub(crate) inner: vyapaar_core::OrderStatus,
}

#[pymethods]
impl OrderStatus {
    #[new]
    fn new(code: &str) -> PyResult<Self> {
        let inner = match code.to_uppercase().as_str() {
            "PENDING_NEW" => vyapaar_core::OrderStatus::PendingNew,
            "NEW" => vyapaar_core::OrderStatus::New,
            "PARTIALLY_FILLED" => vyapaar_core::OrderStatus::PartiallyFilled,
            "FILLED" => vyapaar_core::OrderStatus::Filled,
            "CANCELLED" => vyapaar_core::OrderStatus::Cancelled,
            "REJECTED" => vyapaar_core::OrderStatus::Rejected,
            "EXPIRED" => vyapaar_core::OrderStatus::Expired,
            other => return Err(conversion_err(format!("invalid OrderStatus {other:?}"))),
        };
        Ok(Self { inner })
    }

    #[getter]
    fn code(&self) -> &'static str {
        status_code(self.inner)
    }

    #[staticmethod]
    fn from_code(code: &str) -> PyResult<OrderStatus> {
        Self::new(code)
    }
}

/// Register every value-type class onto the module.
pub(crate) fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Price>()?;
    m.add_class::<Quantity>()?;
    m.add_class::<Currency>()?;
    m.add_class::<Exchange>()?;
    m.add_class::<InstrumentId>()?;
    m.add_class::<OrderSide>()?;
    m.add_class::<TimeInForce>()?;
    m.add_class::<OrderStatus>()?;
    Ok(())
}
