"""Exact-decimal position sizing helpers.

All arithmetic is ``decimal.Decimal`` (no floats) and quantities are rounded
down so a sizing calculation never risks over-allocating capital.
"""
from decimal import Decimal, ROUND_DOWN

from vyapaar.types import to_decimal


def fixed_quantity(quantity) -> Decimal:
    """Return ``quantity`` as an exact ``Decimal``."""
    return to_decimal(quantity)


def fractional(equity, risk_pct, entry_price) -> Decimal:
    """Quantity for risking ``risk_pct`` of ``equity`` at ``entry_price``.

    ``qty = (equity * risk_pct) / entry_price`` rounded down to 2 decimals.
    """
    equity = to_decimal(equity)
    risk_pct = to_decimal(risk_pct)
    entry_price = to_decimal(entry_price)
    if equity <= 0 or risk_pct <= 0 or entry_price <= 0:
        raise ValueError("equity, risk_pct, and entry_price must all be positive")
    qty = (equity * risk_pct) / entry_price
    return qty.quantize(Decimal("0.01"), rounding=ROUND_DOWN)


def risk_budget(equity, max_risk_pct, entry_price, stop_price) -> Decimal:
    """Maximum quantity given a per-trade risk budget.

    ``qty = floor(equity * max_risk_pct / |entry_price - stop_price|)``.
    """
    equity = to_decimal(equity)
    max_risk_pct = to_decimal(max_risk_pct)
    entry_price = to_decimal(entry_price)
    stop_price = to_decimal(stop_price)
    risk_per_unit = abs(entry_price - stop_price)
    if equity <= 0 or max_risk_pct <= 0 or entry_price <= 0 or stop_price <= 0:
        raise ValueError("equity, max_risk_pct, entry_price, and stop_price must all be positive")
    if risk_per_unit == 0:
        raise ValueError("entry_price must differ from stop_price")
    qty = (equity * max_risk_pct) / risk_per_unit
    return qty.to_integral_value(rounding=ROUND_DOWN)


class Sizer:
    """Namespace exposing the sizing helpers as static methods."""

    fixed_quantity = staticmethod(fixed_quantity)
    fractional = staticmethod(fractional)
    risk_budget = staticmethod(risk_budget)
