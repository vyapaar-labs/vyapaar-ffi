# Gaps Index — `vyapaar-ffi`

Per workspace rule `.agents/rules/gap-tracking.md`:
- **Gaps** (`docs/gaps/XXXX-*.md`) are individual missing features, endpoints, or stubs.
- **Plans** (`docs/plans/XXXX-*.md`) are collections of gaps or multi-phase architectural features.
- When a gap is implemented and verified, remove the document from this
  directory and record it in the README "Closed" table below.

## Open Gaps

| # | Title | Scope | Status |
|---|-------|-------|--------|
| [0001](0001-cli-stub-subcommands.md) | `vyapaar` CLI `backtest` is a stub; `serve`/`stub` subcommands are placeholders | `python/vyapaar/cli.py` | Stub |
| [0002](0002-unused-market-data-workspace-deps.md) | Workspace declares `vyapaar-market-{finance,feed,store}` dependencies but no crate uses them | `Cargo.toml` `[workspace.dependencies]` | Declared-but-unused |

## Closed Gaps

| # | Title | Closed In |
|---|-------|-----------|
| — | *(none)* | — |

## Related

- [`docs/design/architecture.md`](../design/architecture.md) — crate topology and module map.
