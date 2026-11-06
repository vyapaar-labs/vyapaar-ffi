# 0001 — Python-First Quant Engine Bridge Policy

> **Status**: Proposed — not yet implemented.
> **Target Crates**: vyapaar-ffi (policy/doc-level — no new code proposed, this is a design-decision doc)
> **Depends on reading**:
> - (none external)

---

## 0. Executive Summary & Objective

Several Fincept-equivalent analytics engines that don't exist in Vyapaar yet — GARCH volatility models, HMM regime detection, copulas, panel econometrics, derivatives pricing — are the kind of statistical/numerical work commonly prototyped first in mature Python libraries (`arch`, `hmmlearn`, `statsmodels`, `QuantLib`) before, if ever, a native Rust port is justified. This doc records a standing policy so that whoever builds the first such engine doesn't reach for a subprocess shortcut: any Python-first implementation of this kind must go through `vyapaar-ffi-py`'s existing PyO3/Arrow zero-copy bridge, never a subprocess/CLI shell-out.

---

## 1. Context

`vyapaar-ffi` already has the right architecture for this via its `vyapaar-ffi-codegen` and `vyapaar-ffi-py` crates: an in-process PyO3 bridge with Arrow zero-copy memory exchange and the `py.allow_threads` GIL-release pattern (per the workspace's own `vyapaar-ffi` architecture, §6.2 of this workspace's `AGENTS.md`). A comparable desktop trading-terminal product (not Vyapaar) was found during the comparative research pass to use a `QProcess` subprocess bridge to call into Python, with 0.5-1.5s per-call cold starts and a hard concurrency cap of 3 concurrent subprocesses. That is exactly the failure mode `vyapaar-ffi`'s in-process bridge already avoids: no cold-start tax per call, and GIL release via `py.allow_threads` means Rust threads are not serialized behind a process-spawn boundary.

The risk this policy addresses is not hypothetical architecture debate — it's that a future contributor adding a GARCH or HMM engine, under time pressure, might shell out to a Python script via `std::process::Command` because it's the fastest thing to prototype, silently reintroducing the QProcess-style cold-start and concurrency ceiling that the rest of the workspace has already engineered around.

---

## 2. Component Specifications

No new types or modules. The policy itself is the artifact:

- **Mandatory bridge path**: any new Python-first statistical/numerical engine (GARCH, HMM regime detection, copula fitting, panel econometrics, derivatives pricing via `QuantLib`, or similar) that is prototyped or permanently implemented in Python MUST be called through `vyapaar-ffi-py`, using its existing PyO3 bindings and Arrow `RecordBatch`/`PyCapsule` zero-copy data exchange.
- **Forbidden pattern**: subprocess/CLI shell-outs to a Python interpreter (`std::process::Command`, `QProcess`-equivalent, or any out-of-process spawn-per-call pattern) for this class of workload are disallowed workspace-wide.
- **GIL discipline**: any such bridge call must release the GIL for the duration of the Python-side numerical work via `py.allow_threads`, matching the existing pattern already established in `vyapaar-ffi-py`, so Rust-side concurrency is not serialized behind Python execution.
- **Where this policy lives**: this doc is the canonical reference; a future engine-specific plan doc (e.g. a GARCH-volatility-engine proposal, not written here) should cite this doc by path rather than re-justifying the bridge choice.

---

## 3. Implementation Phases

### Phase 1: Policy only — no code changes
This doc exists so the policy is written down before anyone building a new quant engine crate reaches for a subprocess shortcut. No implementation, no new crate, no code change is proposed in this phase or any other for this specific doc.
