# Invalidation Planning And Execution Evidence

## What This Feature Is

WORTH Signal separates an invalidation cost estimate from evidence of work the
runtime actually performed. Use the retained estimate to inspect what the
planner predicted. Use an execution receipt when reporting, comparing, or
certifying the work that really happened.

## Why You Use It

- compare the planner's forecast with performed execution without mixing them
- measure the invalidation work performed by one bounded application update
- attach performed Signal counters to a Foundational performance receipt

## Stable Entry Points

Import these types from `worth_signal::facade::adapters`:

- `InvalidationPlanningEstimate`
- `SignalInvalidationExecutionReceipt`
- `SignalInvalidationRealizedCounters`
- `InvalidationExecutionSummary`

Use `SignalRuntime::observe_invalidation_execution` for the ordinary bounded
observation. The lower-level `begin_invalidation_execution_observation` and
`finish_invalidation_execution_observation` pair is available when an update
cannot fit in one closure.

After invalidation planning, read the latest estimate through
`runtime.graph().observe().latest_invalidation_planning_estimate()`.

`FrontierPlan`, `FrontierWave*`, `TransitiveFrontier*`, and the old frontier
counter and summary types are no longer public integration surfaces.

## Core Mental Model

A planning estimate is a forecast. It describes work a planner expects, and it
cannot prove that any work occurred. A Signal execution receipt is a runtime-
created record of performed work. Its counters cannot be constructed through
the public facade.

The observation token is linear and runtime-local. A nested or concurrent
admission is rejected while a session is active; it does not supersede the
active token. Finishing on another runtime, finishing an empty observation, or
finishing a stale or duplicate token returns an error.

`InvalidationExecutionSummary` is a read-only convenience view derived from a
receipt. It is descriptive; it grants no execution authority.

## How It Executes

1. Signal starts an observation and clears the bounded performed counters.
2. The application submits its invalidation and drives the required evaluation.
3. Signal records only work performed inside that observation.
4. Finishing consumes the observation token and returns a receipt.
5. Callers may inspect realized counters, derive a summary, or attach the
   receipt to Foundational counter-backed evidence.

Checkpoint reconstruction, replay, support-report assembly, and forensic work
are not hot invalidation execution. They are retained on separate cold or
diagnostic lanes and cannot be laundered into the hot Foundational receipt.

## Small Example

```rust
use worth_signal::facade::adapters::InvalidationPerformedCounter;
use worth_signal::facade::{SignalError, SignalRuntime};

// `apply_update` is the application's normal update-and-evaluate operation.
let (_, receipt) = runtime.observe_invalidation_execution(|runtime| {
    apply_update(runtime)
})?;

let evaluated = receipt
    .realized_counters()
    .value(InvalidationPerformedCounter::NodesEvaluated);
println!("evaluated {evaluated} nodes");
# Ok::<(), SignalError>(())
```

The closure must perform invalidation work. An empty closure is rejected rather
than producing a zero-work receipt that looks authoritative.

## Real Example

```rust
use worth_signal::facade::adapters::{
    attach_foundational_invalidation_performance_receipt,
    SignalInvalidationRealizedCounters,
};

let observation = runtime.begin_invalidation_execution_observation()?;

apply_market_data_batch(&mut runtime, &market_batch)?;
settle_requested_portfolio_outputs(&mut runtime, &portfolio)?;

let receipt = runtime.finish_invalidation_execution_observation(observation)?;
let realized = *receipt.realized_counters();

// `expected` comes from an independent workload model, not from `realized`.
let expected: SignalInvalidationRealizedCounters =
    portfolio_invalidation_expectation(&market_batch, &portfolio);
let certified =
    attach_foundational_invalidation_performance_receipt(receipt, expected)?;
```

The market batch and graph outputs remain authoritative for financial meaning.
The receipt reports runtime work; it does not decide what the portfolio should
be worth. Foundational attachment checks independently expected counter rows
against the performed Signal rows.

## How It Relates To Other Features

- Use `InvalidationPlanningEstimate` to inspect the planner's prediction and
  inform later policy choices. It cannot substitute for a receipt.
- Use a receipt for realized-cost reporting and certification after execution.
- Use diagnostics sidecars for development or forensic detail. Diagnostic tier
  changes may change sidecars, but not operational receipt rows.
- Use replay and checkpoint reconstruction evidence on their dedicated lanes,
  not as part of the hot invalidation receipt.

### Cross-runtime granular invalidation

Runtime Bridge may carry a Signal execution receipt alongside an installed
granular delivery when derived Signal work actually ran. That receipt proves
only the performed Signal work and its realized counters. It does not authorize
Query maintenance, consumer disclosure, or publication.

A committed direct-truth change can lawfully reach Query without a Signal
receipt when no Signal recomputation is required. Conversely, a Query
consequence declared as Signal-derived requires current performed evidence; a
Bridge delivery, planning estimate, private reverse-index key, or copied
aspect/scope value cannot substitute for it. Query remains responsible for
impact admission and query-shaped maintenance against its current live owner.

## Inspection And Debugging

Inspect `SignalInvalidationRealizedCounters` for the fixed performed counter
rows, including examined direct edges, reverse-index probes, admitted work,
queue activity, evaluation, publication, and topology validation.

If finishing fails, check whether:

- the observation and runtime belong together
- the token is stale or duplicate after the session completed
- the bounded operation performed any instrumented invalidation work

Development and Forensic diagnostics can add node-level sidecar detail without
changing the receipt.

## Anti-Patterns

- Do not compare a planning estimate to a realized threshold as though it were
  measured execution.
- Do not copy receipt counters into the independent expected-counter model.
- Do not retain an observation token across unrelated application operations.
- Do not treat a summary or diagnostics sidecar as execution authority.
- Do not include checkpoint reconstruction, replay, support, or forensic work
  in the hot-path receipt.

## Current Limits

- The public planning estimate is read-only and intentionally exposes only a
  small scheduling-oriented view.
- The ordinary public update path plans and routes atomically. Its retained
  estimate describes that plan after the operation; it is not a pause point
  for changing the current operation's strategy.
- Receipts cover one runtime-local bounded observation; they are not portable
  execution capabilities.
- Geometry-specific cost models and parallel strategy certification remain
  domain and successor-milestone concerns.

## Related Docs

- [Signal Performance Architecture](./signal_performance_architecture.md)
- [Signal Architecture](./signal_architecture2.md)
- [Milestone 13 Plan](./milestone-13-plan.md)
- [Cross-Runtime Granular Invalidation](./milestone-13.1-plan.md)
- [Granular Live Invalidation](../../workspaces/worth-query/crates/worth-query/docs/runtime-surfaces/granular-live-invalidation.md)
