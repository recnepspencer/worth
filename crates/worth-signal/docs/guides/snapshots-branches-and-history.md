# Snapshots, Branches, And History

Use this when "what is true right now?" is not enough.

Read these first if you need them:

- [../core-concepts/diagnostics-and-history.md](../core-concepts/diagnostics-and-history.md)
- [debugging-and-diagnostics.md](./debugging-and-diagnostics.md)

This guide covers owner-admitted snapshots and branch operations alongside
read-only replay, branch, and lineage inspection.

## Main Surfaces

- `runtime.observe_signal_branch_basis(...)`
- `runtime.capture_signal_branch_snapshot(...)`
- `runtime.restore_signal_branch(...)`
- `runtime.fork_signal_branch(...)`
- `runtime.history().branches()`
- `runtime.history().replay_for_branch(...)`
- `runtime.history().replay_for_node(...)`
- `runtime.history().lineage_for_node(...)`

## The Main Rule

Branch state changes require an owner-issued `AdmittedSignalBranchBasis`.
Observe the exact live branch first, then pass that basis to capture, restore,
fork, retirement, or advancement. `runtime.history()` is read-only
inspection; it does not issue snapshot or mutation authority.

## Capture And Inspect

```rust
use worth_signal::facade::*;

let mut runtime = SignalRuntime::build_for::<()>(SignalGraph::new());
let branch = runtime.current_branch();
let basis = runtime
    .observe_signal_branch_basis(branch)
    .expect("the live branch should admit an owner basis");
let (snapshot, captured_basis) = runtime
    .capture_signal_branch_snapshot(&basis)
    .expect("capture requires the exact live basis")
    .into_parts();
let branches = runtime.history().branches();

let _ = (snapshot, captured_basis, branches);
```

An admitted snapshot is bound to the runtime owner that captured or
reconstructed it. Serialized payload alone cannot restore a graph or runtime;
after a trust boundary, reconstruct it through
`runtime.reconstruct_signal_branch_snapshot(...)` before requesting restore.

## Replay vs lineage

- replay answers what happened over time
- lineage answers how one result changed over time

If you are starting from a runtime question, begin with `runtime.history()`.
The runtime keeps the sequence of changes, not just the latest answer. Branch
divergence and admitted restoration therefore remain inspectable after the
fact.
