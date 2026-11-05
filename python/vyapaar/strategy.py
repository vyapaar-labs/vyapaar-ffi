"""High-level strategy authoring and backtesting for the Vyapaar SDK.

Pure-Python ``Strategy`` subclasses implement ``on_start`` / ``on_bar`` /
``on_tick`` / ``on_order_event`` and issue orders through ``self.buy`` /
``self.sell``. ``run()`` wires a strategy to the compiled ``StrategyRunner`` so
every order is validated by the SEBI pre-trade risk gate.

Note: the compiled ``StrategyRunner`` holds a PyO3 borrow on the strategy while
``run_bars``/``run_ticks`` execute, so runner methods cannot be invoked from
inside a callback. Orders issued inside ``on_bar`` are therefore buffered and
flushed by ``run()`` *after* the bars finish, when it is safe to submit.
"""
import logging
from dataclasses import dataclass
from typing import Any, Optional

from vyapaar import _vyapaar
from vyapaar import indicators as indicators_module
from vyapaar.types import instrument_id, to_decimal


@dataclass
class Parameter:
    """Declarative strategy hyper-parameter with bounds validation."""

    default: Any
    min: Optional[Any] = None
    max: Optional[Any] = None
    step: Optional[Any] = None

    def __post_init__(self) -> None:
        if self.min is not None and self.default < self.min:
            raise ValueError(f"default {self.default} must be >= min {self.min}")
        if self.max is not None and self.default > self.max:
            raise ValueError(f"default {self.default} must be <= max {self.max}")

    @classmethod
    def int(cls, default, min=None, max=None) -> "Parameter":
        """Integer parameter helper (``step=1``)."""
        return cls(default, min=min, max=max, step=1)

    def as_dict(self) -> dict:
        return {
            "default": self.default,
            "min": self.min,
            "max": self.max,
            "step": self.step,
        }


def _build_order(symbol, side_code, quantity, limit_price, algo_id, tag):
    """Construct a canonical ``OrderRequest`` from SDK-level arguments."""
    order_type = "LIMIT" if limit_price is not None else "MARKET"
    quantity_str = str(to_decimal(quantity))
    limit = str(to_decimal(limit_price)) if limit_price is not None else None
    return _vyapaar.OrderRequest(
        instrument_id(symbol),
        _vyapaar.OrderSide(side_code),
        order_type,
        quantity_str,
        algo_id,
        limit_price=limit,
        tag=tag,
    )


class Strategy:
    """Base class for Vyapaar strategies.

    Subclasses override the lifecycle callbacks and issue orders with
    ``self.buy(...)`` / ``self.sell(...)``.
    """

    def __init__(self) -> None:
        self._runner: Optional[_vyapaar.StrategyRunner] = None
        self._algo_id: Optional[str] = None
        self.logger = logging.getLogger(self.__class__.__name__)
        self.indicators: Any = None  # set by run()
        self._in_callback = False
        self._pending_orders = []
        self._orders_filled = 0
        self._refusals = 0
        self._events = 0

    # -- lifecycle callbacks --------------------------------------------------
    def on_start(self) -> None:
        """Called once before bars are processed."""

    def on_bar(self, bar: _vyapaar.Bar) -> None:
        """Called for every bar in the backtest feed."""

    def on_tick(self, tick: _vyapaar.Tick) -> None:
        """Called for every tick in the feed."""

    def on_order_event(self, event: Any) -> None:
        """Called for each order acknowledgement emitted by the runner."""
        self._events += 1

    # -- order entry ----------------------------------------------------------
    def buy(self, symbol: str, quantity, *, limit_price=None, tag=None):
        """Place a BUY order; returns an :class:`OrderAck` (or ``None`` while a
        backtest run is in progress -- acks arrive after ``run()`` returns)."""
        self._require_runner()
        order = _build_order(symbol, "BUY", quantity, limit_price, self._algo_id, tag)
        return self._submit(order)

    def sell(self, symbol: str, quantity, *, limit_price=None, tag=None):
        """Place a SELL order; returns an :class:`OrderAck` (or ``None`` while a
        backtest run is in progress -- acks arrive after ``run()`` returns)."""
        self._require_runner()
        order = _build_order(symbol, "SELL", quantity, limit_price, self._algo_id, tag)
        return self._submit(order)

    # -- internals ------------------------------------------------------------
    def _require_runner(self) -> None:
        if self._runner is None or self._algo_id is None:
            raise RuntimeError(
                "strategy is not attached to a runner; pass it to vyapaar.run(...)"
            )

    def _submit(self, order):
        if self._in_callback:
            # The compiled runner borrows the strategy while run_bars/run_ticks
            # execute, so buffer the order and let run() flush it afterwards.
            self._pending_orders.append(order)
            return None
        try:
            ack = self._runner.submit_order(order, _vyapaar.now_iso())
        except _vyapaar.RiskRefused:
            self._refusals += 1
            raise
        self._orders_filled += 1
        return ack


@dataclass
class BacktestResult:
    """Summary of a single ``run()`` backtest."""

    bars_processed: int
    orders_filled: int
    events: int
    refusals: int
    algo_id: str

    def summary(self) -> str:
        return (
            f"Backtest(algo={self.algo_id}): bars={self.bars_processed} "
            f"orders_filled={self.orders_filled} events={self.events} "
            f"refusals={self.refusals}"
        )


def run(strategy: Strategy, bars, *, algo_id: str = "default-algo", on_start: bool = True) -> BacktestResult:
    """Run ``strategy`` over ``bars`` through the compiled ``StrategyRunner``."""
    strategy._algo_id = algo_id
    strategy._pending_orders = []
    strategy._orders_filled = 0
    strategy._refusals = 0
    strategy._events = 0
    runner = _vyapaar.StrategyRunner(strategy, algo_id)
    strategy._runner = runner
    runner.register_algo(algo_id)
    strategy.indicators = indicators_module
    if on_start:
        strategy.on_start()
    try:
        strategy._in_callback = True
        bars_processed = runner.run_bars(bars)
    finally:
        strategy._in_callback = False
    # Flush buffered orders now that the runner is no longer borrowed.
    for order in strategy._pending_orders:
        try:
            runner.submit_order(order, _vyapaar.now_iso())
        except _vyapaar.RiskRefused:
            strategy._refusals += 1
            raise
        strategy._orders_filled += 1
    return BacktestResult(
        bars_processed=bars_processed,
        orders_filled=strategy._orders_filled,
        events=strategy._events,
        refusals=strategy._refusals,
        algo_id=algo_id,
    )
