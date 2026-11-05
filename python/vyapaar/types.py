"""Shared helpers and re-exports for Vyapaar core domain types.

The compiled ``vyapaar._vyapaar`` extension exposes canonical value objects
(``Price``, ``Quantity``, ``InstrumentId``, ...) with exact decimal arithmetic.
This module adds small ergonomic helpers on top of that surface.
"""
from decimal import Decimal

from vyapaar import _vyapaar

# --- Re-exports of the compiled core types -----------------------------------
Price = _vyapaar.Price
Quantity = _vyapaar.Quantity
Currency = _vyapaar.Currency
Exchange = _vyapaar.Exchange
InstrumentId = _vyapaar.InstrumentId
OrderSide = _vyapaar.OrderSide
OrderStatus = _vyapaar.OrderStatus
TimeInForce = _vyapaar.TimeInForce
DepthLevel = _vyapaar.DepthLevel
Bar = _vyapaar.Bar
Tick = _vyapaar.Tick
OrderRequest = _vyapaar.OrderRequest
OrderAck = _vyapaar.OrderAck
FillReport = _vyapaar.FillReport
OrderEvent = _vyapaar.OrderEvent
Decision = _vyapaar.Decision
RiskEngine = _vyapaar.RiskEngine
StrategyRunner = _vyapaar.StrategyRunner
RiskRefused = _vyapaar.RiskRefused
ConversionError = _vyapaar.ConversionError

DEFAULT_EXCHANGE = "NSE"

__all__ = [
    "Price", "Quantity", "Currency", "Exchange", "InstrumentId",
    "OrderSide", "OrderStatus", "TimeInForce", "DepthLevel", "Bar", "Tick",
    "OrderRequest", "OrderAck", "FillReport", "OrderEvent", "Decision",
    "RiskEngine", "StrategyRunner", "RiskRefused", "ConversionError",
    "DEFAULT_EXCHANGE", "to_decimal", "instrument_id",
]


def to_decimal(value: Decimal | str | Price | Quantity) -> Decimal:
    """Coerce ``value`` into an exact ``decimal.Decimal``.

    Accepts ``Decimal``, ``str``, and the compiled ``Price``/``Quantity`` value
    objects (converted via ``.to_str()`` so the ``Price(10)`` repr pitfall never
    leaks into arithmetic).
    """
    if isinstance(value, Decimal):
        return value
    to_str = getattr(value, "to_str", None)
    if to_str is not None:
        return Decimal(to_str())
    return Decimal(str(value))


def instrument_id(symbol: str) -> InstrumentId:
    """Build an :class:`InstrumentId` from ``"EXCHANGE:SYMBOL"`` or ``"SYMBOL"``.

    Symbols without an exchange prefix default to ``NSE``.
    """
    if ":" in symbol:
        return InstrumentId.from_str(symbol)
    return InstrumentId(DEFAULT_EXCHANGE, symbol)
