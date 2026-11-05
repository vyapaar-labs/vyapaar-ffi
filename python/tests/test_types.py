"""Tests for core domain value classes and helpers (vyapaar.types)."""
from decimal import Decimal

import pytest

from vyapaar import _vyapaar
from vyapaar import types


def test_price_roundtrip():
    p = _vyapaar.Price("2450.75")
    assert p.value == Decimal("2450.75")
    assert p.to_str() == "2450.75"
    assert _vyapaar.Price(p.to_str()).to_str() == "2450.75"


def test_price_rejects_negative():
    with pytest.raises(_vyapaar.ConversionError):
        _vyapaar.Price("-5")


def test_quantity_roundtrip():
    q = _vyapaar.Quantity("10")
    assert q.value == Decimal("10")
    assert q.to_str() == "10"


def test_to_decimal_accepts_all_inputs():
    assert types.to_decimal("2450.75") == Decimal("2450.75")
    assert types.to_decimal(Decimal("10")) == Decimal("10")
    assert types.to_decimal(_vyapaar.Price("2450.75")) == Decimal("2450.75")
    assert types.to_decimal(_vyapaar.Quantity("10")) == Decimal("10")


def test_instrument_id_roundtrip():
    iid = _vyapaar.InstrumentId("NSE", "RELIANCE")
    assert iid.to_str() == "NSE:RELIANCE"
    parsed = _vyapaar.InstrumentId.from_str("NSE:RELIANCE")
    assert parsed.exchange == "NSE"
    assert parsed.symbol == "RELIANCE"


def test_instrument_id_helper():
    assert types.instrument_id("NSE:RELIANCE").to_str() == "NSE:RELIANCE"
    assert types.instrument_id("RELIANCE").exchange == "NSE"


def test_order_side_opposite():
    assert _vyapaar.OrderSide("BUY").opposite().code == "SELL"
    assert _vyapaar.OrderSide("SELL").opposite().code == "BUY"


def test_order_request_limit_with_decimal_quantity():
    iid = _vyapaar.InstrumentId("NSE", "RELIANCE")
    order = _vyapaar.OrderRequest(
        iid,
        _vyapaar.OrderSide("BUY"),
        "LIMIT",
        Decimal("10"),
        "algo-1",
        limit_price="2450.00",
    )
    assert order.quantity == Decimal("10")
    assert order.limit_price == Decimal("2450.00")
    assert order.order_type == "LIMIT"
    assert order.algo_id == "algo-1"


def test_bar_construction():
    iid = _vyapaar.InstrumentId("NSE", "RELIANCE")
    bar = _vyapaar.Bar(
        iid, "2450.75", "2455.00", "2448.00", "2452.00", "1000",
        "2026-11-05T09:30:00+05:30",
    )
    assert bar.instrument.to_str() == "NSE:RELIANCE"
    assert bar.close == Decimal("2452.00")
    assert bar.volume == Decimal("1000")


def test_decision_from_json():
    decision = _vyapaar.Decision.from_json('{"action":"hold","reason":"x"}')
    assert decision.action == "hold"
    assert decision.to_dict()["action"] == "hold"


def test_decision_bad_json_raises_conversion_error():
    with pytest.raises(_vyapaar.ConversionError):
        _vyapaar.Decision.from_json("{nope")
