"""Vyapaar Python SDK (Tier 4 FFI).

Re-exports the compiled ``vyapaar._vyapaar`` extension surface plus the
pure-Python high-level API (``Strategy``, ``run``, indicators, sizing, session
windows, and Arrow bridge helpers).
"""
from decimal import Decimal

from vyapaar import _vyapaar
from vyapaar._vyapaar import (
    SCHEMA_VERSION,
    API_VERSION,
    __version__,
    RiskRefused,
    ConversionError,
    Price,
    Quantity,
    Currency,
    Exchange,
    InstrumentId,
    OrderSide,
    TimeInForce,
    OrderStatus,
    DepthLevel,
    Bar,
    Tick,
    OrderRequest,
    OrderAck,
    FillReport,
    OrderEvent,
    Decision,
    RiskEngine,
    StrategyRunner,
    now_iso,
    bars_to_stream,
    ticks_to_stream,
    stream_capsule_name,
)

# Pure-Python API (submodules importable as `vyapaar.strategy`, ...).
# `cli` is deliberately not imported eagerly so `python -m vyapaar.cli`
# executes the module cleanly; it stays importable as `import vyapaar.cli`.
from vyapaar import strategy, indicators, sizing, session, data
from vyapaar.strategy import Strategy, Parameter, BacktestResult, run
from vyapaar.sizing import Sizer, fixed_quantity, fractional, risk_budget
from vyapaar.session import (
    Session,
    SessionWindow,
    PRE_MARKET,
    LIVE,
    POST_MARKET,
    current_session,
    is_live,
)
from vyapaar.types import instrument_id, to_decimal
from vyapaar.indicators import SMA, EMA, RSI

__all__ = [
    "SCHEMA_VERSION", "API_VERSION", "__version__",
    "RiskRefused", "ConversionError",
    "Price", "Quantity", "Currency", "Exchange", "InstrumentId",
    "OrderSide", "TimeInForce", "OrderStatus",
    "DepthLevel", "Bar", "Tick",
    "OrderRequest", "OrderAck", "FillReport", "OrderEvent",
    "Decision", "RiskEngine", "StrategyRunner",
    "now_iso", "bars_to_stream", "ticks_to_stream", "stream_capsule_name",
    "Decimal",
    "Strategy", "Parameter", "BacktestResult", "run",
    "Sizer", "fixed_quantity", "fractional", "risk_budget",
    "Session", "SessionWindow", "PRE_MARKET", "LIVE", "POST_MARKET",
    "current_session", "is_live",
    "instrument_id", "to_decimal",
    "SMA", "EMA", "RSI",
    "strategy", "indicators", "sizing", "session", "data", "cli",
]
