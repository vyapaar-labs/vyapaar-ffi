# 0001 — `vyapaar` CLI subcommands are stubs / placeholders

**Status**: open

## What's stubbed or hardcoded

`python/vyapaar/cli.py` (`python -m vyapaar.cli`) ships three non-functional
subcommands:

| Subcommand | Behavior |
|------------|----------|
| `backtest --bars-json PATH` | **Stub** — loads the JSON file, prints `"backtest stub: N bars loaded ... (execution is a placeholder; use vyapaar.run() in Python)"`, exits 0. No backtest runs. |
| `serve` | Placeholder — prints `"'serve' is not implemented yet"`, exits 0 |
| `stub` | Placeholder — prints `"'stub' is not implemented yet"`, exits 0 |

## Why

The CLI was ported as a minimal scaffold alongside the SDK (`version` is the
only real subcommand). Real backtest execution lives in the Python SDK's
`vyapaar.run()` (through the compiled `StrategyRunner`), and a `serve`
subcommand would need a serving surface that does not exist in this repo yet.

## How to close it

1. Decide the intended surface: `serve` would presumably host the SDK over an
   HTTP/WebSocket endpoint (the compiled `vyapaar._vyapaar` extension is
   already embeddable); `backtest` should delegate to
   `vyapaar.strategy.run(strategy, bars, algo_id=...)` after loading the bars
   JSON.
2. Implement the subcommands or remove them (gap-tracking §4 — remove
   placeholders rather than leaving silently-stubbed commands).
3. Add tests under `python/tests/` for the implemented commands.

## Verification

`python -m vyapaar.cli backtest --bars-json <fixture>` runs a real backtest and
prints results; `python -m vyapaar.cli serve` starts a reachable endpoint (or
the subcommand no longer exists).
