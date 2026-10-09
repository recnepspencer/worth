# Compiler Targeted Rebuild

This walkthrough shows the full story in one place:

- one source file changes
- a transaction lands the update
- only the right downstream targets rerun
- diagnostics explain why the bundle moved
- replay keeps the trail

The full runnable example is:

- [`../../examples/compiler_targeted_rebuild.rs`](../../examples/compiler_targeted_rebuild.rs)

## The Setup

We have four nodes:

- `source_file`
- `symbol_index`
- `diagnostics_panel`
- `app_bundle`

The dependency shape is simple:

- `symbol_index` depends on `source_file`
- `diagnostics_panel` depends on `source_file` and `symbol_index`
- `app_bundle` depends on `source_file` and `symbol_index`

That means a source change can move several useful things at once without
turning into hand-wired glue.

## 1. Build The Graph

```rust
use worth_signal::facade::*;

const SOURCE_TEXT: Aspect = Aspect::new(0);
const SYMBOLS: Aspect = Aspect::new(1);
const DIAGNOSTICS: Aspect = Aspect::new(2);
const BUNDLE: Aspect = Aspect::new(3);

let mut graph = SignalGraph::new();
let source_file = graph.node().build();
let symbol_index = graph.node().on_demand().build();
let diagnostics_panel = graph.node().on_demand().build();
let app_bundle = graph.node().on_demand().build();

graph.set_dependencies(
    symbol_index,
    [DependencyEdge::new(source_file, SOURCE_TEXT)],
)?;
graph.set_dependencies(
    diagnostics_panel,
    [
        DependencyEdge::new(source_file, SOURCE_TEXT),
        DependencyEdge::new(symbol_index, SYMBOLS),
    ],
)?;
graph.set_dependencies(
    app_bundle,
    [
        DependencyEdge::new(source_file, SOURCE_TEXT),
        DependencyEdge::new(symbol_index, SYMBOLS),
    ],
)?;
# Ok::<(), SignalError>(())
```

This is already doing more than a toy reactive graph.
We are naming real build outputs and declaring the dependency shape once.

## 2. Boot The Runtime

```rust
# use worth_signal::facade::*;
# #[derive(Default)]
# struct BuildState {
#     source_version: u64,
#     symbols_version: u64,
#     diagnostics_version: u64,
#     bundle_version: u64,
# }
# let graph = SignalGraph::new();
let mut runtime = SignalRuntime::build_for::<BuildState>(graph);

let mut state = BuildState {
    source_version: 1,
    symbols_version: 10,
    diagnostics_version: 20,
    bundle_version: 30,
};
```

The host state stays yours.
Worth Signal owns invalidation, recompute, rollback, diagnostics, and replay.

## 3. Seed The First Clean Run

```rust
# use worth_signal::facade::*;
# #[derive(Default)]
# struct BuildState {
#     source_version: u64,
#     symbols_version: u64,
#     diagnostics_version: u64,
#     bundle_version: u64,
# }
# let mut runtime = SignalRuntime::build_for::<BuildState>(SignalGraph::new());
# let mut state = BuildState::default();
# let source_file = NodeId::new(0, 0);
# let symbol_index = NodeId::new(1, 0);
# let diagnostics_panel = NodeId::new(2, 0);
# let app_bundle = NodeId::new(3, 0);
let evaluate = |view: &mut EvaluationContext<'_, BuildState>| {
    let result = if view.node() == source_file {
        view.finish(NodeEvaluationResult::from_version(
            AspectVersion::from_updates([(SOURCE_TEXT, view.domain().source_version)]),
        ))
    } else if view.node() == symbol_index {
        let _text = view.read_aspect_version(source_file, SOURCE_TEXT)?;
        view.finish(NodeEvaluationResult::from_version(
            AspectVersion::from_updates([(SYMBOLS, view.domain().symbols_version)]),
        ))
    } else if view.node() == diagnostics_panel {
        let _text = view.read_aspect_version(source_file, SOURCE_TEXT)?;
        let _symbols = view.read_aspect_version(symbol_index, SYMBOLS)?;
        view.finish(NodeEvaluationResult::from_version(
            AspectVersion::from_updates([(DIAGNOSTICS, view.domain().diagnostics_version)]),
        ))
    } else {
        let _text = view.read_aspect_version(source_file, SOURCE_TEXT)?;
        let _symbols = view.read_aspect_version(symbol_index, SYMBOLS)?;
        view.finish(NodeEvaluationResult::from_version(
            AspectVersion::from_updates([(BUNDLE, view.domain().bundle_version)]),
        ))
    };
    Ok::<_, SignalError>(result)
};

let basis = runtime.observe_signal_branch_basis(runtime.current_branch())?;
let serial_request = worth_execution::SerialRequest::from_memory(
    worth_execution::SerialMemoryBudget::new(runtime.runtime_policy().serial_memory_bytes),
    worth_execution::CancellationToken::new(),
    None,
);
let execution = worth_execution::ExecutionRequest::serial(&serial_request);
let basis = runtime.advance_signal_branch(execution, &mut state, &basis, |tx| {
    tx.read_many(
        &[source_file, symbol_index, diagnostics_panel, app_bundle],
        &evaluate,
    )?;
    Ok(())
})?.into_basis();
```

This gives the runtime a clean baseline to compare against.

## 4. Save A Snapshot Before The Edit

```rust
let (snapshot, basis) = runtime
    .capture_signal_branch_snapshot(&basis)?
    .into_parts();
```

This matters because the runtime is not just answering "what is true now?"
It can also keep the trail around important moments.

## 5. Land The File Change In One Transaction

```rust
state.source_version += 1;
state.symbols_version += 1;
state.diagnostics_version += 1;
state.bundle_version += 1;

let _basis = runtime.advance_signal_branch(execution, &mut state, &basis, |tx| {
    tx.mark_changed(source_file, SOURCE_TEXT)?;
    tx.read_many(&[diagnostics_panel, app_bundle], &evaluate)?;
    Ok(())
})?.into_basis();
```

This is the part that tends to get messy in hand-rolled systems.
Here it stays blunt:

- mark the source change
- ask for the affected outputs
- let the runtime keep the update coherent

## 6. Read The Current Result

```rust
let versions = runtime.read_many(&[diagnostics_panel, app_bundle], &state, &evaluate)?;
assert_eq!(versions[0].get(DIAGNOSTICS), 21);
assert_eq!(versions[1].get(BUNDLE), 31);
```

The important part is not just that the bundle changed.
The important part is that the update landed as one unit.

## 7. Ask Why The Bundle Reran

```rust
let explanation = runtime.diagnostics().why(app_bundle)?;
let rendered = format!("{explanation}");
assert!(
    rendered.contains("source") || rendered.contains("upstream") || rendered.contains("Changed"),
);
```

This is one of the main lines in Worth Signal:

- the runtime did work
- the runtime can explain the work

No extra debug layer.
No separate audit pass.

## 8. Read The Replay Trail

```rust
let replay = {
    let history = runtime.history();
    let branch = history.current_branch();
    history.replay_for_branch(branch.id)
};

assert!(!replay.frames.is_empty());
assert!(
    replay
        .frames
        .iter()
        .any(|frame| frame.snapshot_id == Some(snapshot.snapshot().snapshot_id())),
);
```

Now we are past normal reactive-library territory.
The runtime is keeping:

- current truth
- why that truth changed
- and the branch-local trail around the update

## What This Walkthrough Proves

- input changes land through transactions
- downstream work stays targeted
- diagnostics are part of the runtime
- history is part of the runtime
- the system stays coherent from first update to post-hoc inspection

Read next:

- [../guides/running-the-runtime.md](../guides/running-the-runtime.md)
- [../guides/debugging-and-diagnostics.md](../guides/debugging-and-diagnostics.md)
- [../guides/snapshots-branches-and-history.md](../guides/snapshots-branches-and-history.md)
