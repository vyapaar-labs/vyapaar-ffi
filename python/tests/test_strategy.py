"""Tests for the high-level Strategy SDK and vyapaar.run backtest."""
from decimal import Decimal

import pytest

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
        _vyapaar.Bar(
            iid, "2458.00", "2470.00", "2455.00", "2465.00", "900",
            "2026-11-05T09:40:00+05:30",
        ),
    ]


class MAStrategy(vyapaar.Strategy):
    """Buys whenever close trades above the threshold."""

    def __init__(self, threshold="2455.00"):
        super().__init__()
        self.threshold = Decimal(threshold)
        self.buy_calls = 0

    def on_bar(self, bar):
        if bar.close > self.threshold:
            self.buy_calls += 1
            self.buy("NSE:RELIANCE", quantity="10", limit_price="2450.00")


def test_run_backtest_returns_summary():
    bars = make_bars()
    strategy = MAStrategy()
    result = vyapaar.run(strategy, bars, algo_id="ma-test-algo")
    assert result.bars_processed == 3
    assert result.orders_filled == 2
    assert result.events == result.orders_filled
    assert result.algo_id == "ma-test-algo"
    assert strategy.buy_calls == 2
    assert "Backtest" in result.summary()


def test_buy_without_runner_raises():
    strategy = vyapaar.Strategy()
    with pytest.raises(RuntimeError):
        strategy.buy("NSE:RELIANCE", quantity="10")


class RefusingStrategy(vyapaar.Strategy):
    """Points orders at an algo id that was never registered with the risk gate."""

    def on_start(self):
        self._algo_id = "ghost-algo"

    def on_bar(self, bar):
        self.buy("NSE:RELIANCE", quantity="10")


def test_run_raises_riskrefused_for_unregistered_algo():
    strategy = RefusingStrategy()
    with pytest.raises(_vyapaar.RiskRefused):
        vyapaar.run(strategy, make_bars(), algo_id="refusal-test-algo")
    assert strategy._refusals == 1
