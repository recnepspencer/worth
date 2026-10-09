# Observation And Effects

Worth Signal now has one runtime-backed observation story.

That matters because "reactive" can mean a lot of different things:

- dependency invalidation
- recomputation
- observer notification
- effect delivery
- diagnostics explaining why any of that happened

In Worth Signal, those are not separate subsystems stitched together later.
Observation is transaction-bounded and diagnostics-visible.

## The Contract

The runtime observation contract is:

- matching is determined against observed node sets
- delivery happens after a successful commit boundary
- one committed transaction delivers at most one boundary per observer
- rollback suppresses normal delivery
- diagnostics retain the latest observation boundary alongside the latest flow

That gives you one honest story for:

- runtime observers
- easy `watch(...)`
- easy `effect(...)`
- later embedder layers that need stable subscription semantics

## Easy Path

If you want the shortest path, use `worth_signal::easy::*`.

```rust
use worth_signal::easy::*;
use std::sync::{Arc, Mutex};

let mut app = SignalApp::new();
let count = app.input(1_i32);
let doubled = app.computed({
    let count = count;
    move |cx| cx.get(count) * 2
});

let watch_hits = Arc::new(Mutex::new(Vec::<usize>::new()));
let watch_hits_clone = Arc::clone(&watch_hits);

let watch_handle = app.watch(doubled, move |notice| {
    watch_hits_clone
        .lock()
        .expect("watch mutex poisoned")
        .push(notice.matched_nodes().len());
    assert!(notice.trigger_matched());
    assert!(notice.meaningful_change());
});

let effect_hits = Arc::new(Mutex::new(0_usize));
let effect_hits_clone = Arc::clone(&effect_hits);

let effect_handle = app.effect(doubled, move || {
    *effect_hits_clone
        .lock()
        .expect("effect mutex poisoned") += 1;
});

app.set(count, 2);

assert_eq!(app.get(doubled), 4);
assert_eq!(watch_hits.lock().unwrap().as_slice(), &[1]);
assert_eq!(*effect_hits.lock().unwrap(), 1);

assert!(app.unobserve(watch_handle));
assert!(app.unobserve(effect_handle));
# Ok::<(), worth_signal::SignalError>(())
```

The default `watch(...)` and `effect(...)` posture is meaningful-change delivery.
That means recompute without output change is suppressed on the easy path.

## Runtime Path

If you want the broader runtime surface, register observers directly:

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

let basis = runtime.observe_signal_branch_basis(runtime.current_branch())?;
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
})?.into_basis();

let latest_observation = runtime.observe().latest_observation_summary();
assert!(latest_observation.is_some());

assert!(runtime.unobserve(handle));
# Ok::<(), SignalError>(())
```

The important parts are:

- `observe_nodes(...)` registers one observer against one or more nodes
- `ObservationPolicy` decides whether touched, recomputed, or meaningful-change boundaries match
- `unobserve(...)` removes the registration for future deliveries

## Diagnostics

Observation is retained in diagnostics instead of disappearing after callbacks run.

The main doors are:

- `runtime.observe().latest_observation_summary()`
- `runtime.observe().latest_flow_diagnostics()`
- `runtime.diagnostics()`

That means you can inspect:

- which observer matched
- which nodes were observed
- which nodes actually matched the boundary
- whether the boundary was touched, recomputed, or meaningful-change
- whether delivery was committed or rollback-suppressed

The latest observation boundary is also attached to the latest flow summary so
the runtime keeps one coherent explanation of what just happened.

## Why This Matters

This is not just a convenience API.

It closes an important gap in the runtime model:

- derivation is first class
- transactional truth is first class
- observation is now first class too

That is what allows higher layers to build watchers, effects, and web-facing
subscription adapters on one honest substrate instead of inventing parallel
delivery semantics later.
