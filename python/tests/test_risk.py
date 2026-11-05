"""Tests for the compiled SEBI pre-trade risk gate (vyapaar.RiskEngine)."""
import pytest

from vyapaar import _vyapaar

TS = "2026-11-05T09:30:00+05:30"
SYMBOL = "NSE:RELIANCE"


def make_order(algo_id="algo-1"):
    iid = _vyapaar.InstrumentId.from_str(SYMBOL)
    return _vyapaar.OrderRequest(
        iid, _vyapaar.OrderSide("BUY"), "LIMIT", "10", algo_id,
        limit_price="2450.00",
    )


def test_register_then_check_passes():
    engine = _vyapaar.RiskEngine()
    engine.register_algo("algo-1")
    assert engine.is_algo_registered("algo-1")
    engine.check_order(make_order(), TS)  # must not raise


def test_unregistered_algo_refused():
    engine = _vyapaar.RiskEngine()
    with pytest.raises(_vyapaar.RiskRefused):
        engine.check_order(make_order(), TS)


def test_global_halt_blocks_orders():
    engine = _vyapaar.RiskEngine()
    engine.register_algo("algo-1")
    engine.halt_global("test halt", TS)
    assert engine.is_halted()
    assert engine.global_state() == "HALTED"
    with pytest.raises(_vyapaar.RiskRefused):
        engine.check_order(make_order(), TS)
    engine.resume_global()
    assert not engine.is_halted()
    engine.check_order(make_order(), TS)


def test_price_band_enforced():
    engine = _vyapaar.RiskEngine()
    engine.register_algo("algo-1")
    iid = _vyapaar.InstrumentId.from_str(SYMBOL)
    engine.set_price_band(iid, "2400", "2500", "2450")
    engine.check_order_fast("algo-1", iid, "10", "2450.00", TS)
    with pytest.raises(_vyapaar.RiskRefused):
        engine.check_order_fast("algo-1", iid, "10", "2550.00", TS)


def test_check_decision_hold_passes():
    engine = _vyapaar.RiskEngine()
    engine.register_algo("algo-1")
    engine.check_decision('{"action":"hold","reason":"x"}', TS)


def test_total_refusals_counted():
    engine = _vyapaar.RiskEngine()
    for _ in range(3):
        with pytest.raises(_vyapaar.RiskRefused):
            engine.check_order(make_order(), TS)
    assert engine.total_refusals() == 3
