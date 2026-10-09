# Document Partial Recompute

This walkthrough shows the region-aware case:

- one paragraph in the summary section changes
- the summary index refreshes
- the appendix index does not
- diagnostics keep the changed region visible

The runnable example is:

- [`../../examples/document_partial_recompute.rs`](../../examples/document_partial_recompute.rs)

Run it with:

```text
cargo run -p worth-signal --example document_partial_recompute
```

## The Dependency Shape

```rust
use worth_signal::facade::*;

const DOCUMENT: Aspect = Aspect::new(0);

fn summary_scope() -> PartitionSubscription {
    PartitionSubscription::whole_partition("summary")
}

fn appendix_scope() -> PartitionSubscription {
    PartitionSubscription::whole_partition("appendix")
}

let mut graph = SignalGraph::new();
let document = graph.node().partitioned_output().build();
let summary_index = graph.node().on_demand().build();
let appendix_index = graph.node().on_demand().build();

graph.set_dependencies(
    summary_index,
    [DependencyEdge::with_partition_scope(document, DOCUMENT, summary_scope())],
)?;
graph.set_dependencies(
    appendix_index,
    [DependencyEdge::with_partition_scope(document, DOCUMENT, appendix_scope())],
)?;
# Ok::<(), SignalError>(())
```

This is the key move:

- summary indexing subscribes to summary changes
- appendix indexing subscribes to appendix changes

That is how one local edit stays local.

## Read Through The Scope

Each index reads the document through its own section:

```rust
let _summary = view.read_partitioned_aspect_version(document, DOCUMENT, summary_scope())?;
```

Dependencies captured during evaluation are what the runtime keeps.
A plain `read_aspect_version` records an unscoped dependency, and then every
section edit reaches every index. The partitioned read keeps the scope.

## Mark A Region Change

```rust
let basis = runtime.observe_signal_branch_basis(runtime.current_branch())?;
let serial_request = worth_execution::SerialRequest::from_memory(
    worth_execution::SerialMemoryBudget::new(runtime.runtime_policy().serial_memory_bytes),
    worth_execution::CancellationToken::new(),
    None,
);
let execution = worth_execution::ExecutionRequest::serial(&serial_request);
let _next_basis = runtime.advance_signal_branch(execution, &mut state, &basis, |tx| {
    tx.mark_changed_with_regions(document, DOCUMENT, &[ChangedRegion::new("summary")])?;
    tx.read_many(&[document, summary_index, appendix_index], &evaluate)?;
    Ok(())
})?.into_basis();
```

That one line is doing real work.
It tells the runtime this was not a whole-document change.
It was a summary-only change.

The transaction reads the document itself as well as both indexes, so the new
document version and its changed region exist before the indexes are checked.

## Check The Result

```rust
let versions = runtime.read_many(&[summary_index, appendix_index], &state, &evaluate)?;
assert_eq!(versions[0].get(SUMMARY_INDEX), 101);
assert_eq!(versions[1].get(APPENDIX_INDEX), 200);
```

The summary index moved.
The appendix index stayed put.

That is the kind of selective recompute people actually care about.

## Ask For The Explanation

```rust
let explanation = runtime.diagnostics().explain(summary_index)?;
assert!(format!("{explanation}").contains("summary"));

let appendix = runtime.diagnostics().explain(appendix_index)?;
assert!(format!("{appendix}").contains("ScopeUntouched"));
```

The changed region is not lost after recompute.
The runtime can still tell you which section moved, and why the appendix
index did not rerun.

Read next:

- [../core-concepts/aspects-and-dependencies.md](../core-concepts/aspects-and-dependencies.md)
- [../guides/defining-computation.md](../guides/defining-computation.md)
