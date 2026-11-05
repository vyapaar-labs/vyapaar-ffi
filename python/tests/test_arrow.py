"""Tests for the Arrow zero-copy data bridge (vyapaar.data)."""
from decimal import Decimal

import pytest

pyarrow = pytest.importorskip("pyarrow")
import pyarrow as pa

import vyapaar
from vyapaar import _vyapaar


def make_bars():
    iid = _vyapaar.InstrumentId("NSE", "RELIANCE")
    return [
        _vyapaar.Bar(
            iid, "2450.75", "2455.00", "2448.00", "2452.00", "1000",
            "2026-11-05T09:30:00+05:30",
        ),
        _vyapaar.Bar(
            iid, "2452.00", "2460.00", "2450.00", "2458.00", "1200",
            "2026-11-05T09:35:00+05:30",
        ),
    ]


def test_bars_to_stream_capsule_imports_as_arrow():
    capsule = _vyapaar.bars_to_stream(make_bars())
    assert _vyapaar.stream_capsule_name() == "arrow_array_stream"
    reader = pa.RecordBatchReader._import_from_c_capsule(capsule)
    table = reader.read_all()
    assert table.num_rows == 2
    assert table.column_names == ["symbol", "open", "high", "low", "close", "volume", "ts"]
    assert table.column("symbol").to_pylist() == ["NSE:RELIANCE", "NSE:RELIANCE"]
    assert table.column("open").to_pylist()[0] == Decimal("2450.75000000")


def test_bars_to_arrow_helper():
    reader = vyapaar.data.bars_to_arrow(make_bars())
    assert isinstance(reader, pa.RecordBatchReader)
    table = reader.read_all()
    assert table.num_rows == 2
    assert str(table.column("open")[0]) == "2450.75000000"


def test_ticks_to_arrow():
    iid = _vyapaar.InstrumentId("NSE", "RELIANCE")
    ticks = [
        _vyapaar.Tick.ltp(
            iid, "2450.75", "100",
            "2026-11-05T09:30:00+05:30", "2026-11-05T09:30:00+05:30",
        )
    ]
    reader = vyapaar.data.ticks_to_arrow(ticks)
    table = reader.read_all()
    assert table.num_rows == 1
    assert table.column_names == ["instrument", "kind", "price", "volume", "exchange_ts", "receive_ts"]
    assert table.column("kind").to_pylist() == ["ltp"]
