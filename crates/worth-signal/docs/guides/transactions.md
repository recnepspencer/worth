# Transactions

Use a transaction when a change is important enough that "half applied" would
be bad.

That usually means:

- update some source nodes
- recompute related derived work
- either commit the whole thing or roll it back

## Main Surfaces

- `runtime.observe_signal_branch_basis(...)`
- `runtime.advance_signal_branch(execution, ctx, expected, |tx| ...)`
- `tx.mark_changed(...)`
- `tx.mark_changed_with_regions(...)`
- `tx.target(node).run(...)`
- `tx.target(node).read(...)`
- the returned `AdmittedSignalBranchBasis`
- `tx.rollback(...)`

## When to use them

Use a transaction when:

- partial updates would be a bug
- diagnostics and replay should match the real committed change
- failure needs to rewind cleanly

## Example

```rust
use worth_signal::facade::*;

const PRICE: Aspect = Aspect::new(0);

let mut graph = SignalGraph::new();
let source = graph.node().build();
let total = graph.node().on_demand().build();
graph.set_dependencies(total, [DependencyEdge::new(source, PRICE)])?;

let mut runtime = SignalRuntime::build_for::<()>(graph);

let basis = runtime.observe_signal_branch_basis(runtime.current_branch())?;
let serial_request = worth_execution::SerialRequest::from_memory(
    worth_execution::SerialMemoryBudget::new(runtime.runtime_policy().serial_memory_bytes),
    worth_execution::CancellationToken::new(),
    None,
);
let execution = worth_execution::ExecutionRequest::serial(&serial_request);
let _next_basis = runtime.advance_signal_branch(execution, &mut (), &basis, |tx| {
    tx.batch_changes()
        .mark(source, PRICE)
        .apply()?;
    tx.target(total).run(&|view| {
        let result = if view.node() == source {
            view.finish(NodeEvaluationResult::from_version(
                AspectVersion::from_updates([(PRICE, 1)]),
            ))
        } else {
            let version = view.read_aspect_version(source, PRICE)?;
            view.finish(NodeEvaluationResult::from_version(version))
        };
        Ok(result)
    })?;
    Ok(())
})?.into_basis();
# Ok::<(), SignalError>(())
```

## Practical Rule

If the update matters to correctness, do it in a transaction.
