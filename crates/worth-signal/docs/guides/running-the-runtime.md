# Running The Runtime

This guide covers the runtime path.

Read these first if you need them:

- [../GETTING_STARTED.md](../GETTING_STARTED.md)
- [../core-concepts/runtime-and-transactions.md](../core-concepts/runtime-and-transactions.md)
- [runtime-policy.md](./runtime-policy.md)

The examples use a commerce flow:

- product price changes
- shipping and checkout depend on it
- updates should move safely
- diagnostics should explain surprising work

The same runtime shape also fits worth-native work like:

- a file change and a targeted rebuild
- a document edit and a search index refresh
- a one-sheet spreadsheet edit and a partial recompute

## 1. Build A Graph

Start by naming real things:

- product price
- shipping quote
- checkout summary

Main surfaces:

- `SignalGraph`
- `graph.node()`
- `graph.set_dependencies(...)`
- `DependencyEdge`

When the same computed thing should exist as a stable runtime-managed concept,
reach for:

- `Recipe`
- `runtime.define(...)`
- keyed families when identity comes from a stable key

## 2. Mark Input Changes Clearly

When source data changes, say so directly.

Main surfaces:

- `mark_changed(...)`
- `mark_changed_with_regions(...)`
- `tx.mark_changed(...)`
- `tx.batch_changes()`

Use:

- `mark_changed(...)` for the ordinary case
- `mark_changed_with_regions(...)` when only part of something changed
- batch changes when many updates should move together

## 3. Ask For Results Or Run Work

Now use the runtime to get the result you need or to run pending work.

Main surfaces:

- `runtime.target(node).read(...)`
- `runtime.targets(nodes).read_many(...)`
- `runtime.target(node).run(...)`
- `runtime.evaluate_dirty(...)`

Use:

- `read(...)` when you want the current result with minimum work
- `read_many(...)` when several related results belong together
- `run(...)` when you are driving execution on purpose
- `evaluate_dirty(...)` when you want to drain pending dirty work

## 4. Use Transactions For Serious Updates

Transactions are the normal answer when an update should not land halfway.

Main surfaces:

- `runtime.advance_signal_branch(execution, ...)`
- `tx.mark_changed(...)`
- `tx.target(node).run(...)`
- `tx.target(node).read(...)`

Use a transaction when:

- partial updates would be a correctness bug
- diagnostics and replay should match the real committed change
- failure needs a clean rollback

That matters just as much for build graphs and editor pipelines as it does for
commerce flows.

## 5. Start Debugging From The Guided Doors

When behavior looks wrong, start here:

- `runtime.diagnostics()`
- `runtime.history()`

Inside diagnostics, the main grouped paths are:

- `why(...)` / `explain(...)`
- `compare()`
- `health_view()`
- `inspect()`

Use history when you need:

- snapshots
- branches
- replay
- lineage

If you want the details, go to
[debugging-and-diagnostics.md](./debugging-and-diagnostics.md) and
[snapshots-branches-and-history.md](./snapshots-branches-and-history.md).

## 6. Keep The Runtime Posture Simple At First

Most users should start with the stock presets:

- `SignalRuntime::build_for::<Ctx>(graph)`
- `RuntimePolicy::development()`
- `RuntimePolicy::operational()`
- `RuntimePolicy::forensic()`

Leave the deeper tuning alone until you have a real reason to touch it.

## 7. Only Reach For Explicit Executors On Purpose

Most runtime methods already choose an executor for you.

Use explicit executor control when you are deliberately shaping execution, not
just trying to get started.

For that deeper path, read [parallel-execution.md](./parallel-execution.md).
