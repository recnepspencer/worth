# worth-signal Compile-Time Safety V2

> **Status:** Pre-production. Breaking changes are expected.
>
> **Parent:** [signal_architecture2.md](./signal_architecture2.md)
>
> **Goal:** Capture the compile-time safety work that is still relevant after `S1` through `S8`, and express it as an implementation plan that matches the new architecture instead of the old `R16â€“R33` backlog shape.

---

## Table of Contents

1. [What Changed Since V1](#what-changed-since-v1)
2. [What Is Already Landed or Absorbed](#what-is-already-landed-or-absorbed)
3. [Phase C1 â€” Branded Handle Safety](#phase-c1--branded-handle-safety)
4. [Phase C2 â€” Deterministic and Topology Guardrails](#phase-c2--deterministic-and-topology-guardrails)
5. [Phase C3 â€” Branch and Timeline Safety](#phase-c3--branch-and-timeline-safety)
6. [Phase C4 â€” Specialized Future Safety Layers](#phase-c4--specialized-future-safety-layers)
7. [Items We Are Explicitly Not Carrying Forward](#items-we-are-explicitly-not-carrying-forward)
8. [Sequencing](#sequencing)

---

## What Changed Since V1

The old compile-time safety doc assumed the V1 architecture:

- flat `SignalGraph`
- string-based errors
- incomplete transaction/result architecture
- no `GraphObserver`
- no contract system
- no context-aware evaluation model

That is no longer the codebase we have.

The current architecture already changed the safety baseline:

- `S1` split the graph/runtime into real subsystems
- `S2` added `NodeContract` and `ContextRequirement`
- `S3` added `EvaluationEffect` and `EvaluationVerdict`
- `S4` added `TransactionResult`
- `S5` collapsed execution paths and tightened maintenance boundaries
- `S6` already absorbed partition-aware versions, observation purity, rollback hardening, typed errors, and builder completeness
- `S8` is moving evaluation onto `EvaluationContext<'g, Ctx>`

So this document should no longer be read as â€œimplement the old `R16â€“R33` literally.â€ It should be read as â€œwhich compile-time safety ideas are still valuable under the new architecture, and what would they look like now?â€

---

## What Is Already Landed or Absorbed

These V1 items are no longer standalone future work:

| V1 Item | Current Status | Where It Landed |
| --- | --- | --- |
| `R17` `ScopedVersion` witness | Mostly absorbed | `S6.1` partition-aware version tracking in [version.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/aspect/version.rs) and [entry.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/node/entry.rs) |
| `R18` private state setters | Done | `transition_clean/dirty/maybe_stale` in [entry.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/node/entry.rs) |
| `R19` phase restriction | Redesigned and mostly absorbed | `S6.2` observer borrows plus subsystem borrow patterns |
| `R20` observation purity | Done through redesign | [observer.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/graph/runtime/observer.rs) and runtime observer surfaces |
| `R21` single source of truth | Redesigned and mostly absorbed | `EdgeTopology` ownership and `S6.3` consistency assertions |
| `R22` transactional mutation | Redesigned and mostly absorbed | `S3` effect pipeline plus `S6.4` rollback hardening |

These do not need a second compile-time roadmap. The remaining compile-time work should build on them.

---

## Phase C1 â€” Branded Handle Safety

This is the highest-value remaining compile-time phase.

### C1.1 â€” Branded `NodeRef<'g>`

**V1 source:** `R16`

### Problem

`NodeId` is still `Copy` and still appears widely in public read/evaluation/diagnostics surfaces. We did reduce the danger by:

- centralizing stale-handle checks
- adding typed `SignalError::StaleHandle`
- constraining pure reads through `GraphObserver`

But external code can still hold a `NodeId`, then later use it after unregister/rollback/compaction/topology mutation and rely on runtime rejection.

### Design

Split node identity into:

```rust
pub(crate) struct RawNodeId {
    index: u32,
    generation: u32,
}

pub struct NodeRef<'g> {
    raw: RawNodeId,
    _brand: PhantomData<&'g ()>,
}
```

The brand should be tied to the current borrow surface, not the old flat graph:

- `GraphObserver<'g>` returns and consumes `NodeRef<'g>`
- `EvaluationContext<'g, Ctx>` carries `NodeRef<'g>` for `ctx.node()`
- public graph/runtime observation APIs that produce node handles should produce branded refs

Internal storage keeps `RawNodeId`.

### Implementation Plan

1. Introduce `RawNodeId` and `NodeRef<'g>` in the node-id domain.
2. Convert public observation/explain/replay surfaces to branded node refs where the borrow naturally exists.
3. Convert `EvaluationContext<'g, Ctx>` to expose a branded current node.
4. Keep topology/storage/internal mutation APIs on raw ids.
5. Add explicit conversion points only inside the graph/runtime core.

### Files Most Likely Touched

- [node/mod.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/node/mod.rs)
- [observer.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/graph/runtime/observer.rs)
- [graph.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/data/graph/runtime/graph.rs)
- [context.rs](file:///Users/spenstar/Documents/programming/WORTH%20workspace/WORTH/crates/worth-signal/src/logic/context.rs)
- explain / diagnostics surfaces
- public facade exports

### Acceptance Criteria

- public pure-read APIs stop trafficking in naked `NodeId` where a borrow-scoped handle is possible
- using an old node handle across graph mutation becomes a compile error in normal observation/evaluation paths
- internal topology storage remains raw and efficient

### C1.2 â€” Opaque `ScopedVersion`

**V1 source:** `R17`

### Problem

The original bug is already largely fixed by `PartitionVersionMap`, but there is still one remaining compile-time gap: code can still accidentally compare version data in the wrong shape if it bypasses the intended extraction path.

### Design

Do not resurrect the old V1 item as a full phase. Instead, add a narrow opaque witness:

```rust
pub struct ScopedVersion {
    version: AspectVersion,
    _private: (),
}
```

Only `NodeEntry::scoped_version(...)` constructs it.

### Implementation Plan

1. Add the witness type in the aspect/version domain.
2. Convert meaningful-input comparison helpers to use it.
3. Do not broaden this into another parallel version model.

### Acceptance Criteria

- scope-aware comparison remains structurally forced
- no second version architecture is introduced

---

## Phase C2 â€” Deterministic and Topology Guardrails

### C2.1 â€” `DeterministicMap<K, V>`

**V1 source:** `R24`

### Problem

Deterministic output still matters in:

- planning summaries
- execution reporting
- replay/diagnostics aggregation
- performance reporting

We have improved architectural determinism, but we still rely on ordinary maps in places where iteration order should be canonical.

### Design

Introduce a small deterministic collection wrapper for ordering-sensitive surfaces:

```rust
pub struct DeterministicMap<K: Ord, V> {
    inner: HashMap<K, V>,
}
```

Only expose sorted iteration.

### Implementation Plan

1. Add `DeterministicMap<K, V>` in a small collection module.
2. Adopt it in planning/reporting/replay surfaces where ordering matters.
3. Do not replace every `HashMap` in the crate; only replace maps whose iteration becomes user-visible or semantically important.

### Acceptance Criteria

- ordering-sensitive surfaces cannot accidentally use raw hash iteration
- no performance-hostile global replacement of all maps

### C2.2 â€” Discovery/Wiring Decoupling Hardening

**V1 source:** `R29`

### Problem

`S3` and `S5` already moved us toward declarative effects and centralized commit. The remaining risk is not broad architecture anymore; it is accidental leakage of topology mutation back into evaluation-time discovery.

### Design

Keep discovered dependencies as plain data until `EdgeTopology` applies them.

### Implementation Plan

1. Audit the `EvaluationContext` dependency capture path.
2. Make sure evaluation-time discovery produces plain records, not topology mutation.
3. Make `EdgeTopology` the only place where wiring happens.

### Acceptance Criteria

- evaluation/discovery is data-only
- wiring remains a topology/apply concern
- no mid-evaluation topology mutation path reappears

### C2.3 â€” Plan Guard Newtypes

**V1 source:** `R27`, `R33`

### Problem

We still expose explicit planning/execution surfaces. That means WORTHtting optimization stages is still a plausible bug in internal or public explicit-plan flows.

### Design

Use newtype staging only if the explicit plan API remains important:

```rust
pub struct DeduplicatedPlan { ... }
pub struct CoalescedPlan { ... }
```

### Implementation Plan

1. Decide whether explicit plan execution remains a real supported surface.
2. If yes, introduce staged newtypes around optimization boundaries.
3. If no, delete broad explicit-plan entrypoints instead of hardening them.

### Acceptance Criteria

- no ambiguous middle state where optimization is â€œoptional by conventionâ€

---

## Phase C3 â€” Branch and Timeline Safety

These items are still relevant, but only if branch/timeline semantics become a serious product surface.

### C3.1 â€” Generative Branch Isolation

**V1 source:** `R25`

### Problem

We now have a cleaner `BranchManager`, but we do not yet have compile-time protection against cross-branch value/config leakage.

### Design

If branch-local config/state becomes richer, use a generative brand per branch scope.

### Implementation Plan

1. Keep `BranchManager` as the only branch-state owner.
2. When branch-local borrowed state becomes real, brand it with a branch scope lifetime.
3. Do not introduce this early if branches remain mostly serialized snapshots.

### C3.2 â€” Branded `Version<'timeline>`

**V1 source:** `R26`

### Problem

Rollback/commit/branch transitions can create timeline boundaries where cross-timeline comparisons become nonsensical.

### Design

If timeline values become explicit in APIs, brand them by timeline scope.

### Implementation Plan

1. Keep version comparison inside the graph core for now.
2. Only introduce branded timeline versions once versions become a real public or branch-level concept.

---

## Phase C4 â€” Specialized Future Safety Layers

These are still theoretically relevant, but they should stay deferred until the matching feature exists.

### C4.1 â€” Affine Topology Tokens

**V1 source:** `R23`

Still theoretically valid, but low priority now that the execution pipeline is effect-driven. Revisit only if the executor needs stronger stage-proof semantics than ordinary scheduling and tests provide.

### C4.2 â€” Linear Computational Fuel

**V1 source:** `R30`

Relevant only if we add real fixed-point or feedback-loop evaluation where non-convergence becomes a product concern.

### C4.3 â€” Branded `FrameValue<'epoch>`

**V1 source:** `R31`

Relevant only if frame/tick semantics become first-class.

### C4.4 â€” Strategy Marker Traits

**V1 source:** `R32`

Lower value now that strategy is derived from graph state. Revisit only if strategy categories become an important public typed boundary.

### C4.5 â€” Quotient Types

**V1 source:** `R28`

Still useful, but domain-side, not `worth-signal` core. This should live in geometry/analysis domains that need semantic equality, not in the signal engine itself.

---

## Items We Are Explicitly Not Carrying Forward

These old items should not be treated as active roadmap work anymore:

- `R17` as a standalone major phase
  - it was absorbed by `PartitionVersionMap`
- `R18`
  - already done
- `R32` as a near-term engine priority
  - strategy is currently state-derived; marker-typed strategy surfaces are no longer the obvious next move

If these come back, they should come back in the new architectureâ€™s language, not as literal restoration of the V1 proposal.

---

## Sequencing

Recommended order under the current architecture:

1. `C1.1` branded `NodeRef<'g>`
2. `C1.2` narrow opaque `ScopedVersion`
3. `C2.1` deterministic collections
4. `C2.2` discovery/wiring hardening
5. `C2.3` plan-guard newtypes only if explicit plan execution remains part of the supported surface
6. `C3` branch/timeline branding when branch semantics become deeper
7. `C4` only when the corresponding future features exist

If we want one immediate compile-time safety phase that still matters right now, it is `C1.1`.
