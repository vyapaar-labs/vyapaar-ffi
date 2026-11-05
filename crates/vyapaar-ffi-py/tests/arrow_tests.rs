//! Integration tests for the Apache Arrow zero-copy bridge
//! (`arrow_bridge::record_batch_*` helpers and the `bars_to_stream` /
//! `ticks_to_stream` PyCapsule exports implementing the Arrow C Stream
//! Interface, consumable by pyarrow via
//! `RecordBatchReader._import_from_c_capsule`).

use std::ffi::CStr;
use std::sync::Arc;

use arrow::array::{Decimal128Array, StringArray};
use arrow::datatypes::{DataType, TimeUnit};
use arrow::ffi_stream::{ArrowArrayStreamReader, FFI_ArrowArrayStream};
use pyo3::exceptions::PyRuntimeError;
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

/// Build an `InstrumentId` pyobject.
fn instrument<'py>(
    m: &Bound<'py, PyModule>,
    exchange: &str,
    symbol: &str,
) -> PyResult<Bound<'py, PyAny>> {
    let cls = m.getattr("InstrumentId")?;
    cls.call1((exchange, symbol))
}

/// Build a `Bar` pyobject.
fn bar<'py>(
    py: Python<'py>,
    m: &Bound<'py, PyModule>,
    exchange: &str,
    symbol: &str,
    open: &str,
    ts: &str,
) -> PyResult<Bound<'py, PyAny>> {
    let cls = m.getattr("Bar")?;
    cls.call1((
        instrument(m, exchange, symbol)?,
        decimal(py, open)?,
        decimal(py, open)?,
        decimal(py, open)?,
        decimal(py, open)?,
        decimal(py, "1500")?,
        ts,
    ))
}

/// Build an `Tick.ltp(...)` pyobject.
#[allow(clippy::too_many_arguments)]
fn ltp_tick<'py>(
    py: Python<'py>,
    m: &Bound<'py, PyModule>,
    exchange: &str,
    symbol: &str,
    price: &str,
    volume: &str,
    exchange_ts: &str,
    receive_ts: &str,
) -> PyResult<Bound<'py, PyAny>> {
    let cls = m.getattr("Tick")?;
    cls.call_method1(
        "ltp",
        (
            instrument(m, exchange, symbol)?,
            decimal(py, price)?,
            decimal(py, volume)?,
            exchange_ts,
            receive_ts,
        ),
    )
}

/// Re-import the capsule's C stream and return the first `RecordBatch` without
/// taking ownership away from the capsule for longer than this call: the boxed
/// stream memory stays capsule-owned, the reader only moves the FFI struct out.
fn first_batch_from_capsule(
    capsule: &Bound<'_, PyAny>,
) -> PyResult<arrow::record_batch::RecordBatch> {
    // Pass the capsule's own name: CPython 3.12 requires the name to match
    // rather than accepting NULL here.
    let ptr = unsafe {
        pyo3::ffi::PyCapsule_GetPointer(capsule.as_ptr(), c"arrow_array_stream".as_ptr())
    };
    if ptr.is_null() {
        let e = PyErr::fetch(capsule.py());
        panic!("PyCapsule_GetPointer failed: {e}");
    }
    let stream = unsafe { FFI_ArrowArrayStream::from_raw(ptr as *mut FFI_ArrowArrayStream) };
    let mut reader = ArrowArrayStreamReader::try_new(stream)
        .map_err(|e| PyRuntimeError::new_err(format!("stream import failed: {e}")))?;
    let batch = reader
        .next()
        .expect("expected at least one batch")
        .map_err(|e| PyRuntimeError::new_err(format!("batch read failed: {e}")))?;
    Ok(batch)
}

/// Read a single `Decimal128(38, 8)` value out of a batch column.
fn decimal128_value(batch: &arrow::record_batch::RecordBatch, col: usize, row: usize) -> i128 {
    batch
        .column(col)
        .as_any()
        .downcast_ref::<Decimal128Array>()
        .unwrap_or_else(|| panic!("column {col} is not a Decimal128Array"))
        .value(row)
}

/// Read a single Utf8 value out of a batch column.
fn utf8_value(batch: &arrow::record_batch::RecordBatch, col: usize, row: usize) -> &str {
    batch
        .column(col)
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap_or_else(|| panic!("column {col} is not a StringArray"))
        .value(row)
}

#[test]
fn bars_stream_roundtrip() {
    Python::attach(|py| -> PyResult<()> {
        let m = module(py)?;

        let bar1 = bar(
            py,
            &m,
            "NSE",
            "RELIANCE",
            "2450.75",
            "2024-01-15T09:30:00+05:30",
        )?;
        let bar2 = bar(py, &m, "NSE", "TCS", "150.25", "2024-01-15T09:31:00+05:30")?;

        let list = pyo3::types::PyList::new(py, [bar1, bar2])?;
        let capsule = m.getattr("bars_to_stream")?.call1((list,))?;

        // It is a PyCapsule named `arrow_array_stream`.
        assert_eq!(capsule.get_type().name()?.to_str()?, "PyCapsule");
        let name = unsafe { CStr::from_ptr(pyo3::ffi::PyCapsule_GetName(capsule.as_ptr())) };
        assert_eq!(name.to_str().unwrap(), "arrow_array_stream");

        // Re-import the C stream interface and read the exported batch.
        let batch = first_batch_from_capsule(&capsule)?;
        assert_eq!(batch.num_rows(), 2);
        assert_eq!(batch.num_columns(), 7);

        assert_eq!(utf8_value(&batch, 0, 0), "NSE:RELIANCE");
        assert_eq!(utf8_value(&batch, 0, 1), "NSE:TCS");
        // open == 2450.75 at scale 8.
        assert_eq!(decimal128_value(&batch, 1, 0), 245075000000_i128);
        assert_eq!(decimal128_value(&batch, 1, 1), 15025000000_i128);
        // close mirrors open in this fixture.
        assert_eq!(decimal128_value(&batch, 4, 0), 245075000000_i128);
        assert_eq!(decimal128_value(&batch, 4, 1), 15025000000_i128);

        // ts is a UTC microsecond timestamp (2024-01-15T09:30:00+05:30 == 04:00 UTC).
        let ts = batch
            .column(6)
            .as_any()
            .downcast_ref::<arrow::array::TimestampMicrosecondArray>()
            .expect("ts column is a TimestampMicrosecondArray");
        assert_eq!(ts.value(0), 1705291200000000_i64);

        Ok(())
    })
    .unwrap();
}

#[test]
fn ticks_stream_roundtrip() {
    Python::attach(|py| -> PyResult<()> {
        let m = module(py)?;

        let tick1 = ltp_tick(
            py,
            &m,
            "NSE",
            "RELIANCE",
            "2450.75",
            "100",
            "2024-01-15T09:30:00+05:30",
            "2024-01-15T09:30:01+05:30",
        )?;
        let tick2 = ltp_tick(
            py,
            &m,
            "NSE",
            "TCS",
            "150.25",
            "50",
            "2024-01-15T09:30:02+05:30",
            "2024-01-15T09:30:03+05:30",
        )?;

        let list = pyo3::types::PyList::new(py, [tick1, tick2])?;
        let capsule = m.getattr("ticks_to_stream")?.call1((list,))?;

        assert_eq!(capsule.get_type().name()?.to_str()?, "PyCapsule");

        let batch = first_batch_from_capsule(&capsule)?;
        assert_eq!(batch.num_rows(), 2);
        assert_eq!(batch.num_columns(), 6);

        assert_eq!(utf8_value(&batch, 0, 0), "NSE:RELIANCE");
        assert_eq!(utf8_value(&batch, 0, 1), "NSE:TCS");
        assert_eq!(utf8_value(&batch, 1, 0), "ltp");
        assert_eq!(utf8_value(&batch, 1, 1), "ltp");
        assert_eq!(decimal128_value(&batch, 2, 0), 245075000000_i128);
        assert_eq!(decimal128_value(&batch, 2, 1), 15025000000_i128);
        assert_eq!(decimal128_value(&batch, 3, 0), 10000000000_i128);
        assert_eq!(decimal128_value(&batch, 3, 1), 5000000000_i128);

        Ok(())
    })
    .unwrap();
}

#[test]
fn stream_capsule_name_is_contract() {
    Python::attach(|py| -> PyResult<()> {
        let m = module(py)?;
        let name: String = m.getattr("stream_capsule_name")?.call0()?.extract()?;
        assert_eq!(name, "arrow_array_stream");
        Ok(())
    })
    .unwrap();
}

#[test]
fn single_bar_batch_schema() {
    use chrono::{DateTime, Utc};
    use vyapaar_core::{Bar, Exchange, InstrumentId, Price, Quantity};

    let core_bar = Bar {
        instrument: InstrumentId::new(Exchange::Nse, "RELIANCE"),
        open: Price::from_rupees("2450.75").unwrap(),
        high: Price::from_rupees("2451.50").unwrap(),
        low: Price::from_rupees("2449.00").unwrap(),
        close: Price::from_rupees("2450.75").unwrap(),
        volume: Quantity::from_u64(1500),
        timestamp: DateTime::parse_from_rfc3339("2024-01-15T09:30:00+05:30")
            .unwrap()
            .with_timezone(&Utc),
    };

    let batch =
        vyapaar_ffi_py::arrow_bridge::record_batch_for_bars(&[core_bar]).expect("bar batch builds");
    assert_eq!(batch.num_rows(), 1);
    let schema = batch.schema();

    assert_eq!(schema.field(0).name(), "symbol");
    assert_eq!(schema.field(0).data_type(), &DataType::Utf8);
    assert_eq!(schema.field(1).name(), "open");
    assert_eq!(schema.field(1).data_type(), &DataType::Decimal128(38, 8));
    assert_eq!(schema.field(5).name(), "volume");
    assert_eq!(schema.field(5).data_type(), &DataType::Decimal128(38, 8));
    assert_eq!(schema.field(6).name(), "ts");
    match schema.field(6).data_type() {
        DataType::Timestamp(TimeUnit::Microsecond, tz) => {
            assert_eq!(tz.as_deref(), Some("UTC"));
        }
        other => panic!("expected Timestamp(Microsecond, Some(\"UTC\")), got {other:?}"),
    }

    // Field types agree with the concrete arrays.
    assert_eq!(batch.column(0).data_type(), &DataType::Utf8);
    assert_eq!(batch.column(1).data_type(), &DataType::Decimal128(38, 8));
    assert_eq!(
        batch.column(6).data_type(),
        &DataType::Timestamp(TimeUnit::Microsecond, Some(Arc::from("UTC")))
    );

    // The tick schema marks `price` nullable.
    let ltp = vyapaar_core::Tick::Ltp {
        instrument: InstrumentId::new(Exchange::Nse, "RELIANCE"),
        price: Price::from_rupees("2450.75").unwrap(),
        volume: Quantity::from_u64(100),
        exchange_ts: DateTime::parse_from_rfc3339("2024-01-15T09:30:00+05:30")
            .unwrap()
            .with_timezone(&Utc),
        receive_ts: DateTime::parse_from_rfc3339("2024-01-15T09:30:01+05:30")
            .unwrap()
            .with_timezone(&Utc),
    };
    let tick_batch =
        vyapaar_ffi_py::arrow_bridge::record_batch_for_ticks(&[ltp]).expect("tick batch builds");
    assert!(tick_batch.schema().field(2).is_nullable());
    assert_eq!(
        tick_batch.schema().field(2).data_type(),
        &DataType::Decimal128(38, 8)
    );
}
