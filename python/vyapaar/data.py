"""Apache Arrow zero-copy bridge helpers.

``bars_to_stream`` / ``ticks_to_stream`` return a ``PyCapsule`` implementing the
Arrow C data interface; ``_import_from_c_capsule`` wraps it as a
``pyarrow.RecordBatchReader`` with zero copy. ``pyarrow`` / ``polars`` are
imported lazily inside each function so the rest of the SDK works without them.
"""
from vyapaar import _vyapaar

_PYARROW_HINT = (
    "pyarrow is required for Arrow interop; install it with `pip install vyapaar[arrow]`"
)
_POLARS_HINT = "polars is required for DataFrame conversion; install it with `pip install polars`"


def bars_to_arrow(bars):
    """Convert ``list[Bar]`` to a ``pyarrow.RecordBatchReader`` (zero-copy)."""
    try:
        import pyarrow as pa
    except ImportError:
        raise ImportError(_PYARROW_HINT) from None
    capsule = _vyapaar.bars_to_stream(bars)
    return pa.RecordBatchReader._import_from_c_capsule(capsule)


def ticks_to_arrow(ticks):
    """Convert ``list[Tick]`` to a ``pyarrow.RecordBatchReader`` (zero-copy)."""
    try:
        import pyarrow as pa
    except ImportError:
        raise ImportError(_PYARROW_HINT) from None
    capsule = _vyapaar.ticks_to_stream(ticks)
    return pa.RecordBatchReader._import_from_c_capsule(capsule)


def bars_to_polars(bars, **kw):
    """Convert ``list[Bar]`` to a ``polars.DataFrame`` (via Arrow)."""
    try:
        import polars as pl
    except ImportError:
        raise ImportError(_POLARS_HINT) from None
    reader = bars_to_arrow(bars)
    return pl.from_arrow(reader.read_all())
