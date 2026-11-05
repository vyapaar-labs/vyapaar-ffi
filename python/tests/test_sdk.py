"""Tests for the pure-Python SDK modules (parameters, sizing, sessions, indicators)."""
from datetime import datetime, timedelta, timezone
from decimal import Decimal

import pytest

import vyapaar
from vyapaar import indicators, session, sizing, strategy

IST = timezone(timedelta(hours=5, minutes=30))


# --- Parameter ---------------------------------------------------------------

def test_parameter_bounds_validation():
    p = strategy.Parameter(default=10, min=5, max=50, step=5)
    assert p.default == 10
    assert p.step == 5
    with pytest.raises(ValueError):
        strategy.Parameter(default=1, min=5, max=50)
    with pytest.raises(ValueError):
        strategy.Parameter(default=60, min=5, max=50)


def test_parameter_int_helper_and_as_dict():
    p = strategy.Parameter.int(default=20, min=10, max=30)
    assert p.as_dict() == {"default": 20, "min": 10, "max": 30, "step": 1}


# --- Sizing ------------------------------------------------------------------

def test_fixed_quantity():
    assert sizing.fixed_quantity("25") == Decimal("25")


def test_fractional_sizing():
    qty = sizing.fractional(Decimal("100000"), Decimal("0.01"), Decimal("2450"))
    assert qty == Decimal("0.40")
    with pytest.raises(ValueError):
        sizing.fractional(Decimal("0"), Decimal("0.01"), Decimal("2450"))


def test_risk_budget_sizing():
    qty = sizing.risk_budget(Decimal("100000"), Decimal("0.02"), Decimal("2450"), Decimal("2400"))
    assert qty == Decimal("40")
    with pytest.raises(ValueError):
        sizing.risk_budget(Decimal("100000"), Decimal("0.02"), Decimal("2400"), Decimal("2400"))


def test_sizer_namespace():
    assert sizing.Sizer.fixed_quantity("25") == Decimal("25")


# --- Session -----------------------------------------------------------------

def test_current_session_is_live_at_1000_ist():
    dt = datetime(2026, 11, 5, 10, 0, tzinfo=IST)
    assert session.current_session(dt) == session.LIVE
    assert session.is_live(dt) is True


def test_current_session_pre_market():
    dt = datetime(2026, 11, 5, 9, 5, tzinfo=IST)
    assert session.current_session(dt) == session.PRE_MARKET


# --- Indicators --------------------------------------------------------------

def test_sma():
    sma = indicators.SMA(3)
    assert sma.value() is None
    for price in ["10", "20", "30"]:
        sma.add(price)
    assert sma.value() == Decimal("20")
    sma.add("40")
    assert sma.value() == Decimal("30")


def test_sma_crosses_above():
    sma = indicators.SMA(2)
    sma.add("10")
    sma.add("10")
    assert sma.crosses_above(Decimal("12")) is False
    sma.add("20")
    assert sma.crosses_above(Decimal("12")) is True
    assert sma.crosses_below(Decimal("12")) is False


def test_ema():
    ema = indicators.EMA(3)
    assert ema.value() is None
    ema.add("10")
    assert ema.value() == Decimal("10")
    ema.add("20")
    assert ema.value() == Decimal("15")
    assert ema.add("30") == Decimal("22.5")


def test_rsi_wilder():
    rsi = indicators.RSI(3)
    for price in ["44.34", "44.09", "44.15", "43.61"]:
        rsi.add(price)
    value = rsi.value()
    assert value is not None
    assert value < Decimal("50")


# --- top-level package -------------------------------------------------------

def test_top_level_exports():
    assert vyapaar.Decimal is Decimal
    assert vyapaar.SCHEMA_VERSION >= 1
    assert vyapaar.Price is not None
