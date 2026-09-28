# WORTH Signal Diagnostics Product Map

## Purpose

Diagnostics should be productized around user jobs, not around flat export
families.

---

## Core Jobs

The diagnostics product should be organized around:

- inspect what is happening now
- explain why this changed
- compare two runs
- inspect runtime health
- trace replay / lineage / history

---

## Canonical Entry Shape

Diagnostics should be entered through one obvious access point.

Target direction:

```rust
let diagnostics = runtime.diagnostics();
```

or the graph equivalent for graph-only usage.

Users should not need to begin by memorizing free-floating helper families.

The two main doors should be:

- `runtime.diagnostics()`
- `runtime.history()`

Inside `runtime.diagnostics()`, the first grouped paths should be:

- `why(...)` / `explain(...)`
- `compare()`
- `health_view()`
- `inspect()`

---

## Job 1: Explain Why This Changed

Primary output:

- explanation object
- optional render / summary view

What should be discoverable first:

- `explain(node)`

What should be secondary:

- raw fact and provenance types
- raw renderer entry points

Decision:

- keep `why(...)` and `explain(...)` flat on the main diagnostics object
- do not force an extra namespace just to answer the most common question

---

## Job 2: Inspect What Is Happening Now

Primary output:

- current graph summary
- latest flow / failure / rollback
- recent run history
- graph / execution / plan / report inspectors

What should be discoverable first:

- `health_view().current_now()`
- `health_view().latest_flow()`
- `health_view().recent_history()`
- `inspect().graph()`
- `inspect().execution()`
- `inspect().plan(...)`
- `inspect().report(...)`

What should be secondary:

- standalone inspector helpers
- lower-level runtime observer helpers

Decision:

- health-style reads and inspector-style reads should feel grouped
- the standalone `inspect_*` helpers can stay public, but should not define the
  first product story

---

## Job 3: Compare Two Runs

Primary output:

- comparison result with summary-first access

What should be discoverable first:

- compare reports / plans / explanations / histories through one comparison area

What should be secondary:

- raw diff types

Decision:

- `compare()` remains the grouped comparison door
- direct compare free functions stay public for tooling and test work, but the
  docs should lead with the grouped comparison object

---

## Job 4: Inspect Runtime Health

Primary output:

- health-oriented summaries:
  - planning summary
  - precompute summary
  - apply / rollback / failure summaries

What should be discoverable first:

- `health_now()`
- `health(profile)`
- `health_view()`

What should be secondary:

- low-level counters and internal diagnostic materialization details

Decision:

- keep the existing direct `health_*` convenience methods
- teach `health_view()` when the user wants a fuller "what is going on right
  now?" read instead of one summary call

---

## Job 5: Trace Replay / Lineage / History

Primary output:

- history-oriented access point
- replay slices
- lineage chains
- restore-related historical views

What should be discoverable first:

- history / replay / lineage as a coherent family

Primary entry:

- `runtime.history()`

What should be secondary:

- raw event and lineage record internals

Decision:

- replay and lineage should stay with the history story, not get flattened back
  into the main diagnostics object
- `runtime.diagnostics()` and `runtime.history()` should read like two nearby
  doors, not one crowded door

---

## Product Rule

Diagnostics should feel like:

- one premium subsystem
- a few crisp user jobs
- progressively disclosed machinery

Not:

- a taxonomy of helper families

---

## Design Requirement

The user should not need to first understand:

- compare functions
- diff functions
- render functions
- summary structs
- lineage record internals

Instead, the public diagnostics experience should lead with the jobs above and
progressively disclose the underlying machinery.

## Phase 6 Outcome

Phase 6 is done when all of the following are true:

- `runtime.diagnostics()` is the obvious first door for explain / compare /
  health / inspect work
- `runtime.history()` is the obvious first door for replay / lineage / snapshot
  work
- grouped health and inspection reads are easy to discover
- direct diagnostics exports still exist for tooling and tests, but no longer
  tell a better product story than the guided path
