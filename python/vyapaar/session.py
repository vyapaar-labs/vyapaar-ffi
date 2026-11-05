"""NSE session windows (IST) and helpers.

Pure-Python time-window classification: local time strings are compared to the
canonical NSE session windows.
"""
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone, tzinfo

IST = timezone(timedelta(hours=5, minutes=30))


@dataclass(frozen=True)
class SessionWindow:
    """A named intra-day window with inclusive open / exclusive close."""

    name: str
    open_time: str
    close_time: str


PRE_MARKET = SessionWindow("pre-market", "09:00", "09:15")
LIVE = SessionWindow("live", "09:15", "15:30")
POST_MARKET = SessionWindow("post-market", "15:30", "16:00")

WINDOWS = (PRE_MARKET, LIVE, POST_MARKET)


def current_session(now: datetime | None = None, tz: tzinfo = IST) -> SessionWindow | None:
    """Return the :class:`SessionWindow` containing ``now`` (or ``None``)."""
    if now is None:
        now = datetime.now(tz)
    elif now.tzinfo is None:
        now = now.replace(tzinfo=tz)
    else:
        now = now.astimezone(tz)
    now_hm = f"{now.hour:02d}:{now.minute:02d}"
    for window in WINDOWS:
        if window.open_time <= now_hm < window.close_time:
            return window
    return None


def is_live(now: datetime | None = None) -> bool:
    """True when ``now`` falls inside the regular NSE live session."""
    return current_session(now) == LIVE


class Session:
    """Namespace exposing session windows and lookup helpers."""

    PRE_MARKET = PRE_MARKET
    LIVE = LIVE
    POST_MARKET = POST_MARKET

    @staticmethod
    def current(now: datetime | None = None, tz: tzinfo = IST) -> SessionWindow | None:
        return current_session(now, tz)

    @staticmethod
    def is_live(now: datetime | None = None) -> bool:
        return is_live(now)
