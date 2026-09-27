# WORTH Signal Canonical Surface Spec

## Purpose

This document defines the positive canonical public shape of `worth-signal`.

The library should not be remembered mainly as:

- a set of namespaces
- a cleanup project
- an export audit

It should be remembered as a small number of clear canonical flows.

---

## Product Center Of Gravity

`worth-signal` should present itself as:

- production incremental runtime for derived computation with strong diagnostics

This is the primary mental model.

Breadth across web, geometry, DSL/compiler, ML, and bridge integrations should
validate extensibility, not fragment the first impression.

---

## Canonical Public Memory Shapes

These are the things users should memorize.

## 1. Canonical Import Path

Primary path:

- `worth_signal::facade`

Policy:

- the facade is the curated public product boundary
- top-level `diagnostics` and `easy` may remain public, but they must not
  undermine the facade as the canonical import story

Target memory shape:

```rust
use worth_signal::facade::*;
```

Not:

- mixed imports from `facade`, `diagnostics`, and internal-feeling specialist
  paths for normal work
- memorizing multiple equally-official entry boundaries

## 2. Canonical Production Setup Flow

Users should have one obvious production setup flow centered on:

- `SignalGraph`
- `SignalRuntime::build_for::<Ctx>(...)`
- explicit named preset variants when needed
- deeper builder setup only for abnormal cases

Target public flow:

1. create a graph
2. declare nodes and dependencies
3. build a runtime from the graph with the recommended default
4. optionally choose a more specific preset when you mean it
5. only drop to the builder for abnormal setup
6. use transactions / reads / batch invalidation from the runtime

Target memory shape:

```rust
let graph = SignalGraph::new();

let runtime = SignalRuntime::build_for::<AppState>(graph);
```

Target refinement shape:

- graph construction remains explicit
- direct constructors own the normal setup story
- the builder remains available underneath for abnormal setup
- advanced policy remains layered under the builder instead of forcing users to
  stitch unrelated knobs manually

Architectural rule:

- runtime setup must compile into one coherent declaration boundary
- if configuration requires scattered coordination calls across subsystems, the
  setup surface is not yet productized

Target property:

- safe by default
- explicit where it matters
- no memory-based ceremony

## 3. Canonical Computation-Definition Flow

Users should have one obvious way to define runtime computations that scales
from basic to advanced use.

Target property:

- computation declaration should feel like one coherent operation
- raw lower-level control may remain underneath it

Target direction:

- graph-level node authoring stays explicit for low-level control
- runtime-level computation definition becomes the guided production path

Preferred mental split:

- `SignalGraph` for structural graph authoring
- runtime-owned computation declaration for durable computation registration

Canonical shape to drive toward:

```rust
let total = runtime.define(
    Recipe::new("price.total", tier, |ctx| { ... })
        .reads([PRICE])
        .produces([TOTAL]),
)?;
```

Meaning:

- one declaration owns identity, policy, dependency contract, and evaluation
  behavior
- scattered registration should collapse into the computation declaration itself

Raw path that may remain:

- `graph.node()...build()`
- explicit dependency wiring
- explicit prepared-plan execution

But the raw path should be clearly lower-level, not a competing â€œnormalâ€ path

## 4. Canonical Batch Invalidation Flow

Users should have one obvious batch-aware mutation / invalidation path.

Target property:

- batch-first where the runtime is semantically batch-oriented
- scalar helpers subordinate to the batch story

Canonical production shape:

```rust
runtime.transaction(&mut ctx, |tx| {
    tx.batch_changes()
        .mark(source_a, PRICE)
        .mark_regions(source_b, PRICE, regions)
        .apply()?;
    Ok(())
})?;
```

Policy:

- scalar `mark_dirty(...)` remains valid for simple cases
- the docs and examples should teach guided batch invalidation as the production
  mental model
- graph-level scalar invalidation must not define the overall product identity

## 5. Canonical Diagnostics Entry Flow

Users should have one obvious diagnostics entry flow that answers a small number
of jobs:

- explain why this changed
- compare two runs
- inspect runtime health
- trace replay / lineage / history

Canonical shape to drive toward:

```rust
let diagnostics = runtime.diagnostics();

let explanation = diagnostics.explain(node)?;
let comparison = diagnostics.compare().reports(&left, &right);
let health = diagnostics.health_now();
let replay = runtime.history().replay_for_node(node)?;
```

Required properties:

- one access point
- job-oriented operations
- render/summary helpers subordinate to those jobs
- raw diff and lineage primitives remain below the guided surface

## 6. Canonical Specialist Merge Flow

Specialists should have one obvious merge/reconciliation orchestration flow.

Target property:

- specialist
- guided
- explicitly not part of the day-one product story

Canonical shape to drive toward:

```rust
let planned = runtime.merge()
    .from(source_branch)
    .into(target_branch)
    .plan()?;

let result = planned.execute()?;
```

Meaning:

- merge planning and merge execution should feel like one guided specialist
  workflow
- raw merge plan, witness, and conflict-record types may remain underneath, but
  not as the first thing specialists have to stitch together

## 7. Canonical Role Of `easy`

This must be decided explicitly before publish.

Allowed outcomes:

1. first-15-minutes guided path
2. thin alias layer over real product API
3. subordinate demo-oriented sidecar

Ambiguity here is not acceptable.

Current provisional doctrine:

- `easy` is the explicit first-15-minutes guided path
- it should optimize for immediate comprehension and success
- it should not teach a mental model that must later be unlearned

Implication:

- `easy` should mirror the core product philosophy
- `easy` should not become a second architecture
- if it diverges too far from the production mental model, it should be reduced
  or demoted

---

## Canonical Surface Summary

If we succeed, users should remember:

1. `use worth_signal::facade::*;`
2. `SignalGraph` for graph structure
3. `SignalRuntime::builder(...)` for production setup
4. one coherent computation-definition path
5. batch-first invalidation in production flows
6. one diagnostics access point organized by jobs
7. one guided merge flow for specialists

Anything that weakens those memory shapes should be treated as DX debt.
