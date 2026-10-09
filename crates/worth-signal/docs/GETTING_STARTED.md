# Getting Started

This page shows the smallest useful Worth Signal setup with the main runtime
surface.

If you want the shortest path first, start with `worth_signal::easy`.
If you want to learn the broader runtime surface right away, start here.

The example is a checkout summary that depends on product price. We update the
price in a transaction, read the current result, and keep diagnostics ready for
"why did this rerun?"

This is the same system you can use for smaller cases like a todo count or file
preview, and for bigger cases like targeted rebuilds, branch history, and
runtime diagnostics.

It is also the same system that now owns observation.
You can register runtime observers directly, or use `watch(...)` / `effect(...)`
on the short path, and both sit on the same commit-bounded delivery model.

## The Bigger Story

The small example below is the shape.
The bigger version of the same shape looks like this:

- one source file changes
- a transaction updates the build session
- symbol indexing and the right bundle rerun
- diagnostics explain why the bundle moved
- replay keeps the trail after the update lands

That full version lives here:

- [`../examples/compiler_targeted_rebuild.rs`](../examples/compiler_targeted_rebuild.rs)
- [walkthroughs/compiler-targeted-rebuild.md](./walkthroughs/compiler-targeted-rebuild.md)

## What You Are Working With

The main pieces are:

- `SignalGraph`
- `SignalRuntime`
- `runtime.advance_signal_branch(execution, ...)`
- `runtime.observe_nodes(...)`
- `runtime.target(node).read(...)`
- `runtime.diagnostics()`

## First Run

```rust
use worth_signal::facade::*;

const PRICE: Aspect = Aspect::new(0);

let mut graph = SignalGraph::new();
let product_price = graph.node().build();
let checkout_summary = graph.node().on_demand().build();

graph.set_dependencies(
    checkout_summary,
    [DependencyEdge::new(product_price, PRICE)],
)?;

let mut runtime = SignalRuntime::build_for::<()>(graph);

let basis = runtime
    .observe_signal_branch_basis(runtime.current_branch())
    .expect("current branch should admit an owner basis");
let serial_request = worth_execution::SerialRequest::from_memory(
    worth_execution::SerialMemoryBudget::new(runtime.runtime_policy().serial_memory_bytes),
    worth_execution::CancellationToken::new(),
    None,
);
let execution = worth_execution::ExecutionRequest::serial(&serial_request);
let _next_basis = runtime.advance_signal_branch(execution, &mut (), &basis, |tx| {
    tx.mark_changed(product_price, PRICE)?;
    tx.target(checkout_summary).run(&|view| {
        let result = if view.node() == product_price {
            view.finish(NodeEvaluationResult::from_version(
                AspectVersion::from_updates([(PRICE, 1)]),
            ))
        } else {
            let version = view.read_aspect_version(product_price, PRICE)?;
            view.finish(NodeEvaluationResult::from_version(version))
        };
        Ok(result)
    })?;
    Ok(())
})
.expect("admitted branch advance should succeed")
.into_basis();

let version = runtime.target(checkout_summary).read(&(), &|view| {
    let price_version = view.read_aspect_version(product_price, PRICE)?;
    Ok(view.finish(NodeEvaluationResult::from_version(price_version)))
})?;

let diagnostics = runtime.diagnostics();

let _ = (version, diagnostics);
# Ok::<(), SignalError>(())
```

## Observation In The Same Runtime

Once you want a callback boundary, do not invent one outside the runtime.
Register an observer through the same system:

```rust
use worth_signal::facade::*;

struct CounterListener;

impl ObservationListener<(), (), (), (), ()> for CounterListener {
    fn on_observation(
        &self,
        _ctx: ObservationReadContext<'_, (), (), (), (), ()>,
        notice: &ObservationNotice<'_>,
    ) {
        assert!(notice.trigger_matched());
        assert!(notice.meaningful_change());
    }
}

let mut graph = SignalGraph::new();
let source = graph.node().build();
let derived = graph.node().on_demand().build();
graph.set_dependencies(derived, [DependencyEdge::new(source, ASPECT_A)])?;

let mut runtime = SignalRuntime::build_for::<()>(graph);

let handle = runtime.observe_nodes(
    ObservationPolicy::meaningful_change(),
    [derived],
    Box::new(CounterListener),
);

let basis = runtime
    .observe_signal_branch_basis(runtime.current_branch())
    .expect("current branch should admit an owner basis");
let serial_request = worth_execution::SerialRequest::from_memory(
    worth_execution::SerialMemoryBudget::new(runtime.runtime_policy().serial_memory_bytes),
    worth_execution::CancellationToken::new(),
    None,
);
let execution = worth_execution::ExecutionRequest::serial(&serial_request);
let _next_basis = runtime.advance_signal_branch(execution, &mut (), &basis, |tx| {
    tx.mark_changed(source, ASPECT_A)?;
    tx.target(derived).run(&|view| {
        let version = view.read_aspect_version(source, ASPECT_A)?;
        Ok(view.finish(NodeEvaluationResult::from_version(version)))
    })?;
    Ok(())
})
.expect("admitted branch advance should succeed");

let latest_observation = runtime.observe().latest_observation_summary();
assert!(latest_observation.is_some());

assert!(runtime.unobserve(handle));
# Ok::<(), SignalError>(())
```

## What Happened

- `product_price` is a source node
- `checkout_summary` is a computed node
- `tx.mark_changed(...)` tells the runtime the source changed
- `tx.target(...).run(...)` computes the affected work inside the transaction
- `runtime.observe_nodes(...)` registers commit-bounded observation on the same runtime
- `runtime.target(...).read(...)` asks for the current result
- `runtime.diagnostics()` and `runtime.observe().latest_observation_summary()` give you the main debugging doors

## What To Read Next

- [API_OVERVIEW.md](./API_OVERVIEW.md) for the map
- [guides/observation-and-effects.md](./guides/observation-and-effects.md) for commit-bounded observation, `watch(...)`, and `effect(...)`
- [core-concepts/README.md](./core-concepts/README.md) for the fundamentals
- [guides/running-the-runtime.md](./guides/running-the-runtime.md) for the runtime path
- [guides/debugging-and-diagnostics.md](./guides/debugging-and-diagnostics.md) for the main debugging flow
- [guides/snapshots-branches-and-history.md](./guides/snapshots-branches-and-history.md) for replay, snapshots, and branch history
- [walkthroughs/easy-task-board.md](./walkthroughs/easy-task-board.md) for the shortest path
- [walkthroughs/compiler-targeted-rebuild.md](./walkthroughs/compiler-targeted-rebuild.md) for the full runtime story

If you want the shortest path for a smaller setup, start with `worth_signal::easy`.
