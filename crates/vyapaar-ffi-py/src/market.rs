//! Market-data pyclasses: `DepthLevel`, `Bar`, `Tick`.
//!
//! Prices/quantities cross the boundary as exact `decimal.Decimal`; timestamps
//! are RFC3339 UTC strings (an empty string means the deterministic Unix epoch,
//! matching the Nautilus-style "zero time" convention).

use chrono::{DateTime, Utc};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};

use crate::decimal::decimal_to_pydecimal;
use crate::error::parse_utc;
use crate::types::{InstrumentId, price_from_any, quantity_from_any};

/// The deterministic "zero" timestamp used when a caller omits one.
fn epoch_ts() -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp(0, 0).expect("the Unix epoch is a valid timestamp")
}

/// Parse an RFC3339 timestamp, treating the empty string as the Unix epoch.
fn parse_ts(iso: &str) -> PyResult<DateTime<Utc>> {
    if iso.is_empty() {
        Ok(epoch_ts())
    } else {
        parse_utc(iso)
    }
}

/// One level of the L2/L3 order book.
#[pyclass(module = "vyapaar._vyapaar")]
#[derive(Clone)]
pub struct DepthLevel {
    pub(crate) inner: vyapaar_core::DepthLevel,
}

#[pymethods]
impl DepthLevel {
    #[new]
    #[pyo3(signature = (price, quantity, orders_count=None))]
    fn new(
        price: &Bound<'_, PyAny>,
        quantity: &Bound<'_, PyAny>,
        orders_count: Option<u32>,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: vyapaar_core::DepthLevel {
                price: price_from_any(price)?,
                quantity: quantity_from_any(quantity)?,
                orders_count,
            },
        })
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
    fn orders_count(&self) -> Option<u32> {
        self.inner.orders_count
    }
}

/// Standard OHLCV bar / candle.
#[pyclass(module = "vyapaar._vyapaar")]
#[derive(Clone)]
pub struct Bar {
    pub(crate) inner: vyapaar_core::Bar,
}

impl Bar {
    /// Lift a core `Bar` into its Python wrapper (public Rust API).
    #[allow(dead_code)]
    pub(crate) fn from_core(inner: vyapaar_core::Bar) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl Bar {
    #[new]
    #[pyo3(signature = (instrument, open, high, low, close, volume, timestamp))]
    fn new(
        instrument: &Bound<'_, InstrumentId>,
        open: &Bound<'_, PyAny>,
        high: &Bound<'_, PyAny>,
        low: &Bound<'_, PyAny>,
        close: &Bound<'_, PyAny>,
        volume: &Bound<'_, PyAny>,
        timestamp: &str,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: vyapaar_core::Bar {
                instrument: instrument.borrow().inner.clone(),
                open: price_from_any(open)?,
                high: price_from_any(high)?,
                low: price_from_any(low)?,
                close: price_from_any(close)?,
                volume: quantity_from_any(volume)?,
                timestamp: parse_utc(timestamp)?,
            },
        })
    }

    #[getter]
    fn instrument(&self) -> InstrumentId {
        InstrumentId {
            inner: self.inner.instrument.clone(),
        }
    }

    #[getter]
    fn open<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        decimal_to_pydecimal(py, self.inner.open.0)
    }

    #[getter]
    fn high<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        decimal_to_pydecimal(py, self.inner.high.0)
    }

    #[getter]
    fn low<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        decimal_to_pydecimal(py, self.inner.low.0)
    }

    #[getter]
    fn close<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        decimal_to_pydecimal(py, self.inner.close.0)
    }

    #[getter]
    fn volume<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        decimal_to_pydecimal(py, self.inner.volume.0)
    }

    #[getter]
    fn timestamp(&self) -> String {
        self.inner.timestamp.to_rfc3339()
    }

    /// String-valued dict for JSON-ish serialization.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let dict = PyDict::new(py);
        dict.set_item("instrument", self.inner.instrument.to_string())?;
        dict.set_item("open", self.inner.open.0.to_string())?;
        dict.set_item("high", self.inner.high.0.to_string())?;
        dict.set_item("low", self.inner.low.0.to_string())?;
        dict.set_item("close", self.inner.close.0.to_string())?;
        dict.set_item("volume", self.inner.volume.0.to_string())?;
        dict.set_item("timestamp", self.inner.timestamp.to_rfc3339())?;
        Ok(dict.into_any())
    }
}

/// Canonical typed tick: LTP, best bid/ask quote, or full L2 depth.
#[pyclass(module = "vyapaar._vyapaar")]
#[derive(Clone)]
pub struct Tick {
    pub(crate) inner: vyapaar_core::Tick,
}

impl Tick {
    /// Lift a core `Tick` into its Python wrapper (public Rust API).
    #[allow(dead_code)]
    pub(crate) fn from_core(inner: vyapaar_core::Tick) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl Tick {
    #[getter]
    fn kind(&self) -> &'static str {
        match &self.inner {
            vyapaar_core::Tick::Ltp { .. } => "ltp",
            vyapaar_core::Tick::Quote { .. } => "quote",
            vyapaar_core::Tick::Depth { .. } => "depth",
        }
    }

    #[staticmethod]
    #[pyo3(signature = (instrument, price, volume, exchange_ts, receive_ts))]
    fn ltp(
        instrument: &Bound<'_, InstrumentId>,
        price: &Bound<'_, PyAny>,
        volume: &Bound<'_, PyAny>,
        exchange_ts: &str,
        receive_ts: &str,
    ) -> PyResult<Tick> {
        Ok(Tick {
            inner: vyapaar_core::Tick::Ltp {
                instrument: instrument.borrow().inner.clone(),
                price: price_from_any(price)?,
                volume: quantity_from_any(volume)?,
                exchange_ts: parse_ts(exchange_ts)?,
                receive_ts: parse_ts(receive_ts)?,
            },
        })
    }

    #[staticmethod]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (instrument, bid, ask, bid_qty, ask_qty, last_price=None, exchange_ts="", receive_ts=""))]
    fn quote(
        instrument: &Bound<'_, InstrumentId>,
        bid: &Bound<'_, PyAny>,
        ask: &Bound<'_, PyAny>,
        bid_qty: &Bound<'_, PyAny>,
        ask_qty: &Bound<'_, PyAny>,
        last_price: Option<&Bound<'_, PyAny>>,
        exchange_ts: &str,
        receive_ts: &str,
    ) -> PyResult<Tick> {
        // A `last_price` that is present but not an exact decimal raises.
        let last_price = match last_price {
            Some(lp) => Some(price_from_any(lp)?),
            None => None,
        };
        Ok(Tick {
            inner: vyapaar_core::Tick::Quote {
                instrument: instrument.borrow().inner.clone(),
                bid: price_from_any(bid)?,
                ask: price_from_any(ask)?,
                bid_qty: quantity_from_any(bid_qty)?,
                ask_qty: quantity_from_any(ask_qty)?,
                last_price,
                exchange_ts: parse_ts(exchange_ts)?,
                receive_ts: parse_ts(receive_ts)?,
                band: None,
            },
        })
    }

    /// String-valued dict for JSON-ish serialization.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let dict = PyDict::new(py);
        match &self.inner {
            vyapaar_core::Tick::Ltp {
                instrument,
                price,
                volume,
                exchange_ts,
                receive_ts,
            } => {
                dict.set_item("kind", "ltp")?;
                dict.set_item("instrument", instrument.to_string())?;
                dict.set_item("price", price.0.to_string())?;
                dict.set_item("volume", volume.0.to_string())?;
                dict.set_item("exchange_ts", exchange_ts.to_rfc3339())?;
                dict.set_item("receive_ts", receive_ts.to_rfc3339())?;
            }
            vyapaar_core::Tick::Quote {
                instrument,
                bid,
                ask,
                bid_qty,
                ask_qty,
                last_price,
                exchange_ts,
                receive_ts,
                ..
            } => {
                dict.set_item("kind", "quote")?;
                dict.set_item("instrument", instrument.to_string())?;
                dict.set_item("bid", bid.0.to_string())?;
                dict.set_item("ask", ask.0.to_string())?;
                dict.set_item("bid_qty", bid_qty.0.to_string())?;
                dict.set_item("ask_qty", ask_qty.0.to_string())?;
                match last_price {
                    Some(lp) => dict.set_item("last_price", lp.0.to_string())?,
                    None => dict.set_item("last_price", "")?,
                }
                dict.set_item("exchange_ts", exchange_ts.to_rfc3339())?;
                dict.set_item("receive_ts", receive_ts.to_rfc3339())?;
            }
            vyapaar_core::Tick::Depth {
                instrument,
                bids,
                asks,
                last_price,
                exchange_ts,
                receive_ts,
            } => {
                dict.set_item("kind", "depth")?;
                dict.set_item("instrument", instrument.to_string())?;
                dict.set_item("bid_levels", bids.len())?;
                dict.set_item("ask_levels", asks.len())?;
                match last_price {
                    Some(lp) => dict.set_item("last_price", lp.0.to_string())?,
                    None => dict.set_item("last_price", "")?,
                }
                dict.set_item("exchange_ts", exchange_ts.to_rfc3339())?;
                dict.set_item("receive_ts", receive_ts.to_rfc3339())?;
            }
        }
        Ok(dict.into_any())
    }
}

/// Register market-data classes onto the module.
pub(crate) fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<DepthLevel>()?;
    m.add_class::<Bar>()?;
    m.add_class::<Tick>()?;
    Ok(())
}
