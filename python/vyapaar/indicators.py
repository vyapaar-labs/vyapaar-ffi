"""Pure-Python technical indicators with exact ``Decimal`` arithmetic.

No floating-point math anywhere: all prices are stored and smoothed as
``decimal.Decimal``. ``SMA``/``EMA`` also track the previous value so
``crosses_above``/``crosses_below`` can be evaluated incrementally.
"""
from collections import deque
from decimal import Decimal

from vyapaar.types import to_decimal


def _series_value(other):
    """Resolve ``other`` (Decimal, number, or indicator with ``.value()``)."""
    if other is None:
        return None
    if isinstance(other, Decimal):
        return other
    value = getattr(other, "value", None)
    if value is not None:
        return value()
    return other


def _crosses_above(prev, cur, other_value) -> bool:
    if prev is None or cur is None or other_value is None:
        return False
    return prev <= other_value and cur > other_value


def _crosses_below(prev, cur, other_value) -> bool:
    if prev is None or cur is None or other_value is None:
        return False
    return prev >= other_value and cur < other_value


class SMA:
    """Simple moving average of the last ``period`` values."""

    def __init__(self, period: int) -> None:
        if period <= 0:
            raise ValueError("period must be positive")
        self.period = period
        self._history = deque()
        self._previous = None

    def add(self, price: Decimal | str) -> Decimal | None:
        self._previous = self.value()
        self._history.append(to_decimal(price))
        while len(self._history) > self.period:
            self._history.popleft()
        return self.value()

    def value(self) -> Decimal | None:
        if not self._history:
            return None
        return sum(self._history, Decimal("0")) / Decimal(self.period)

    def crosses_above(self, other) -> bool:
        return _crosses_above(self._previous, self.value(), _series_value(other))

    def crosses_below(self, other) -> bool:
        return _crosses_below(self._previous, self.value(), _series_value(other))


class EMA:
    """Exponential moving average with ``Decimal`` alpha smoothing."""

    def __init__(self, period: int) -> None:
        if period <= 0:
            raise ValueError("period must be positive")
        self.period = period
        self._alpha = Decimal(2) / Decimal(period + 1)
        self._value: Decimal | None = None
        self._previous: Decimal | None = None

    def add(self, price: Decimal | str) -> Decimal | None:
        self._previous = self._value
        price = to_decimal(price)
        if self._value is None:
            self._value = price
        else:
            self._value = self._alpha * price + (Decimal(1) - self._alpha) * self._value
        return self._value

    def value(self) -> Decimal | None:
        return self._value

    def crosses_above(self, other) -> bool:
        return _crosses_above(self._previous, self._value, _series_value(other))

    def crosses_below(self, other) -> bool:
        return _crosses_below(self._previous, self._value, _series_value(other))


class RSI:
    """Relative Strength Index with Wilder smoothing (``Decimal``)."""

    def __init__(self, period: int = 14) -> None:
        if period <= 0:
            raise ValueError("period must be positive")
        self.period = period
        self._last: Decimal | None = None
        self._deltas = deque()
        self._avg_gain: Decimal | None = None
        self._avg_loss: Decimal | None = None
        self._previous: Decimal | None = None

    def add(self, price: Decimal | str) -> Decimal | None:
        self._previous = self.value()
        price = to_decimal(price)
        if self._last is not None:
            diff = price - self._last
            gain = diff if diff > 0 else Decimal("0")
            loss = -diff if diff < 0 else Decimal("0")
            if self._avg_gain is None:
                self._deltas.append((gain, loss))
                if len(self._deltas) >= self.period:
                    self._avg_gain = sum(g for g, _ in self._deltas) / Decimal(self.period)
                    self._avg_loss = sum(l for _, l in self._deltas) / Decimal(self.period)
                    self._deltas.clear()
            else:
                smooth = Decimal(self.period - 1) / Decimal(self.period)
                self._avg_gain = self._avg_gain * smooth + gain / Decimal(self.period)
                self._avg_loss = self._avg_loss * smooth + loss / Decimal(self.period)
        self._last = price
        return self.value()

    def value(self) -> Decimal | None:
        if self._avg_gain is None:
            return None
        if self._avg_loss == 0:
            return Decimal("100") if self._avg_gain > 0 else Decimal("50")
        rs = self._avg_gain / self._avg_loss
        return Decimal("100") - (Decimal("100") / (Decimal("1") + rs))
