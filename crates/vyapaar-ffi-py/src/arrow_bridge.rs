//! Apache Arrow zero-copy bridge.
//!
//! Exports [`RecordBatch`] streams of bars/ticks as a `PyCapsule` implementing
//! the Arrow C Stream Interface (capsule name `arrow_array_stream`). Consumers
//! (pyarrow and other Arrow-native Python libraries) import the stream with
//! `RecordBatchReader._import_from_c_capsule`.
//!
//! All money/quantity columns are exact `Decimal128(38, 8)` scalars (zero-floats
//! policy); timestamps are `Timestamp(Microsecond, "UTC")`. The FFI struct is
//! boxed on the heap and owned by the capsule; its destructor drops the boxed
//! stream, whose own `Drop` invokes the C `release` callback, freeing the
//! private reader data exactly once.

use std::ffi::c_void;
use std::sync::Arc;

use arrow::array::{
    ArrayRef, Decimal128Array, Decimal128Builder, StringArray, TimestampMicrosecondArray,
};
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use arrow::error::ArrowError;
use arrow::ffi_stream::FFI_ArrowArrayStream;
use arrow::record_batch::{RecordBatch, RecordBatchIterator, RecordBatchReader};
use pyo3::exceptions::PyMemoryError;
use pyo3::ffi::{PyCapsule_GetPointer, PyCapsule_New, PyObject as CPyObject};
use pyo3::prelude::*;
use pyo3::types::PyModule;
use rust_decimal::Decimal;

use crate::error::conversion_err;
use crate::market::{Bar, Tick};
use vyapaar_core::{Price, Quantity};

/// The PyCapsule name implementing the Arrow C Stream Interface.
const STREAM_NAME: &str = "arrow_array_stream";

/// Precision of every decimal column in the exported streams.
const DECIMAL_PRECISION: u8 = 38;
/// Scale of every decimal column in the exported streams.
const DECIMAL_SCALE: i8 = 8;

/// Rescale an exact decimal to scale 8 and return its unscaled `i128` mantissa.
///
/// `rust_decimal::Decimal` keeps a coefficient + scale; Arrow stores decimals
/// as a fixed-scale integer mantissa. `rescale` never fails and truncates
/// trailing precision beyond scale 8 (prices are quoted to paise at most).
fn decimal_to_scaled_i128(d: &Decimal) -> i128 {
    let mut scaled = *d;
    scaled.rescale(DECIMAL_SCALE as u32);
    scaled.mantissa()
}

/// `Price` -> scale-8 unscaled `i128` mantissa.
fn price_to_i128(p: &Price) -> i128 {
    decimal_to_scaled_i128(&p.0)
}

/// `Quantity` -> scale-8 unscaled `i128` mantissa.
fn quantity_to_i128(q: &Quantity) -> i128 {
    decimal_to_scaled_i128(&q.0)
}

/// Build a non-nullable scale-8 `Decimal128Array`.
fn decimal_array(values: Vec<i128>) -> Decimal128Array {
    Decimal128Array::from(values)
        .with_precision_and_scale(DECIMAL_PRECISION, DECIMAL_SCALE)
        .expect("DECIMAL_PRECISION/DECIMAL_SCALE are always valid")
}

/// Arrow schema for bar streams: symbol + OHLCV + UTC microsecond timestamp.
fn bar_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("symbol", DataType::Utf8, false),
        Field::new(
            "open",
            DataType::Decimal128(DECIMAL_PRECISION, DECIMAL_SCALE),
            false,
        ),
        Field::new(
            "high",
            DataType::Decimal128(DECIMAL_PRECISION, DECIMAL_SCALE),
            false,
        ),
        Field::new(
            "low",
            DataType::Decimal128(DECIMAL_PRECISION, DECIMAL_SCALE),
            false,
        ),
        Field::new(
            "close",
            DataType::Decimal128(DECIMAL_PRECISION, DECIMAL_SCALE),
            false,
        ),
        Field::new(
            "volume",
            DataType::Decimal128(DECIMAL_PRECISION, DECIMAL_SCALE),
            false,
        ),
        Field::new(
            "ts",
            DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into())),
            false,
        ),
    ]))
}

/// Build ONE `RecordBatch` of bars from core domain bars.
///
/// Columns: `symbol` (Utf8), `open`/`high`/`low`/`close`/`volume`
/// (`Decimal128(38, 8)`), `ts` (`Timestamp(Microsecond, "UTC")`).
pub fn record_batch_for_bars(bars: &[vyapaar_core::Bar]) -> Result<RecordBatch, ArrowError> {
    let schema = bar_schema();

    let symbols: Vec<String> = bars.iter().map(|b| b.instrument.to_string()).collect();
    let opens: Vec<i128> = bars.iter().map(|b| price_to_i128(&b.open)).collect();
    let highs: Vec<i128> = bars.iter().map(|b| price_to_i128(&b.high)).collect();
    let lows: Vec<i128> = bars.iter().map(|b| price_to_i128(&b.low)).collect();
    let closes: Vec<i128> = bars.iter().map(|b| price_to_i128(&b.close)).collect();
    let volumes: Vec<i128> = bars.iter().map(|b| quantity_to_i128(&b.volume)).collect();
    let ts: Vec<i64> = bars
        .iter()
        .map(|b| b.timestamp.timestamp_micros())
        .collect();

    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(symbols)),
        Arc::new(decimal_array(opens)),
        Arc::new(decimal_array(highs)),
        Arc::new(decimal_array(lows)),
        Arc::new(decimal_array(closes)),
        Arc::new(decimal_array(volumes)),
        Arc::new(TimestampMicrosecondArray::from(ts).with_timezone("UTC")),
    ];

    RecordBatch::try_new(schema, columns)
}

/// Arrow schema for tick streams: instrument + kind + nullable price + volume +
/// exchange/receive UTC microsecond timestamps.
fn tick_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("instrument", DataType::Utf8, false),
        Field::new("kind", DataType::Utf8, false),
        Field::new(
            "price",
            DataType::Decimal128(DECIMAL_PRECISION, DECIMAL_SCALE),
            true,
        ),
        Field::new(
            "volume",
            DataType::Decimal128(DECIMAL_PRECISION, DECIMAL_SCALE),
            false,
        ),
        Field::new(
            "exchange_ts",
            DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into())),
            false,
        ),
        Field::new(
            "receive_ts",
            DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into())),
            false,
        ),
    ]))
}

/// Kind tag of a tick: `"ltp"`, `"quote"`, or `"depth"`.
fn tick_kind(tick: &vyapaar_core::Tick) -> &'static str {
    match tick {
        vyapaar_core::Tick::Ltp { .. } => "ltp",
        vyapaar_core::Tick::Quote { .. } => "quote",
        vyapaar_core::Tick::Depth { .. } => "depth",
    }
}

/// Exchange timestamp of a tick as microseconds since the Unix epoch.
fn tick_exchange_ts_micros(tick: &vyapaar_core::Tick) -> i64 {
    match tick {
        vyapaar_core::Tick::Ltp { exchange_ts, .. }
        | vyapaar_core::Tick::Quote { exchange_ts, .. }
        | vyapaar_core::Tick::Depth { exchange_ts, .. } => exchange_ts.timestamp_micros(),
    }
}

/// Volume carried by a tick: LTP traded volume, best bid + ask quantities for a
/// quote, and `Decimal::ZERO` for depth ticks.
fn tick_volume(tick: &vyapaar_core::Tick) -> Decimal {
    match tick {
        vyapaar_core::Tick::Ltp { volume, .. } => volume.0,
        vyapaar_core::Tick::Quote {
            bid_qty, ask_qty, ..
        } => bid_qty.0 + ask_qty.0,
        vyapaar_core::Tick::Depth { .. } => Decimal::ZERO,
    }
}

/// Build ONE `RecordBatch` of ticks from core domain ticks.
///
/// Columns: `instrument` (Utf8), `kind` (Utf8), `price`
/// (`Decimal128(38, 8)` nullable — null when the tick has no price), `volume`
/// (`Decimal128(38, 8)` — `0` when absent), `exchange_ts` and `receive_ts`
/// (`Timestamp(Microsecond, "UTC")`).
pub fn record_batch_for_ticks(ticks: &[vyapaar_core::Tick]) -> Result<RecordBatch, ArrowError> {
    let schema = tick_schema();

    let instruments: Vec<String> = ticks.iter().map(|t| t.instrument().to_string()).collect();
    let kinds: Vec<&str> = ticks.iter().map(tick_kind).collect();

    let mut price_builder = Decimal128Builder::new();
    for tick in ticks {
        price_builder.append_option(tick.last_price().map(|p| price_to_i128(&p)));
    }
    let prices = price_builder
        .finish()
        .with_precision_and_scale(DECIMAL_PRECISION, DECIMAL_SCALE)
        .expect("DECIMAL_PRECISION/DECIMAL_SCALE are always valid");

    let volumes: Vec<i128> = ticks
        .iter()
        .map(|t| decimal_to_scaled_i128(&tick_volume(t)))
        .collect();
    let exchange_ts: Vec<i64> = ticks.iter().map(tick_exchange_ts_micros).collect();
    let receive_ts: Vec<i64> = ticks
        .iter()
        .map(|t| t.receive_ts().timestamp_micros())
        .collect();

    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(instruments)),
        Arc::new(StringArray::from(kinds)),
        Arc::new(prices),
        Arc::new(decimal_array(volumes)),
        Arc::new(TimestampMicrosecondArray::from(exchange_ts).with_timezone("UTC")),
        Arc::new(TimestampMicrosecondArray::from(receive_ts).with_timezone("UTC")),
    ];

    RecordBatch::try_new(schema, columns)
}

/// Capsule destructor: drops the boxed `FFI_ArrowArrayStream`, whose own `Drop`
/// invokes the C `release` callback freeing the private reader data.
unsafe extern "C" fn destroy_stream(capsule: *mut CPyObject) {
    if capsule.is_null() {
        return;
    }
    let ptr = unsafe { PyCapsule_GetPointer(capsule, c"arrow_array_stream".as_ptr()) };
    if !ptr.is_null() {
        drop(unsafe { Box::from_raw(ptr as *mut FFI_ArrowArrayStream) });
    }
}

/// Export a `RecordBatch` as an `arrow_array_stream` `PyCapsule` (Arrow C
/// Stream Interface), transferring ownership of the boxed stream to the capsule.
pub(crate) fn export_stream_capsule<'py>(
    py: Python<'py>,
    batch: RecordBatch,
) -> PyResult<Py<PyAny>> {
    let schema = batch.schema();
    let reader: Box<dyn RecordBatchReader + Send> = Box::new(RecordBatchIterator::new(
        vec![Ok(batch)].into_iter(),
        schema,
    ));

    let stream = FFI_ArrowArrayStream::new(reader);
    let raw = Box::into_raw(Box::new(stream)) as *mut c_void;

    let capsule_ptr =
        unsafe { PyCapsule_New(raw, c"arrow_array_stream".as_ptr(), Some(destroy_stream)) };
    if capsule_ptr.is_null() {
        // `PyCapsule_New` failed and never took ownership; reclaim the boxed
        // stream so the reader does not leak.
        drop(unsafe { Box::from_raw(raw as *mut FFI_ArrowArrayStream) });
        return Err(PyMemoryError::new_err("PyCapsule_New failed"));
    }

    let capsule = unsafe { Bound::from_owned_ptr(py, capsule_ptr) };
    Ok(capsule.unbind())
}

/// The PyCapsule name implementing the Arrow C Stream Interface.
#[pyfunction]
fn stream_capsule_name() -> &'static str {
    STREAM_NAME
}

/// Export a list of [`Bar`] objects as an `arrow_array_stream` PyCapsule.
#[pyfunction]
fn bars_to_stream<'py>(py: Python<'py>, bars: Vec<Bound<'py, Bar>>) -> PyResult<Py<PyAny>> {
    let core: Vec<vyapaar_core::Bar> = bars.iter().map(|b| b.borrow().inner.clone()).collect();
    let batch = record_batch_for_bars(&core).map_err(conversion_err)?;
    export_stream_capsule(py, batch)
}

/// Export a list of [`Tick`] objects as an `arrow_array_stream` PyCapsule.
#[pyfunction]
fn ticks_to_stream<'py>(py: Python<'py>, ticks: Vec<Bound<'py, Tick>>) -> PyResult<Py<PyAny>> {
    let core: Vec<vyapaar_core::Tick> = ticks.iter().map(|t| t.borrow().inner.clone()).collect();
    let batch = record_batch_for_ticks(&core).map_err(conversion_err)?;
    export_stream_capsule(py, batch)
}

/// Register the arrow bridge functions onto the module.
pub(crate) fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(stream_capsule_name, m)?)?;
    m.add_function(wrap_pyfunction!(bars_to_stream, m)?)?;
    m.add_function(wrap_pyfunction!(ticks_to_stream, m)?)?;
    Ok(())
}
