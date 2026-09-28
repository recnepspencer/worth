# WORTH Runtime Bridge Vision

## Thesis

WORTH does not want one fused runtime that mixes truth storage and derived
computation into a single opaque system.

It wants two strong runtimes with a principled bridge between them:

- a truth runtime that owns identity, mutation, history, diffs, lineage, and
  authoritative graph state
- a computation runtime that owns invalidation, recomputation, scheduling,
  convergence, and runtime self-inspection
- a bridge layer that turns truth changes, truth history, and truth snapshots
  into deterministic derived execution without collapsing the runtimes together

The bridge is not glue code. It is the causal protocol boundary that keeps the
system decoupled while still allowing precise, large-scale propagation from
truth to computation.

The bridge exists because the two runtimes are separate libraries. Its job is
to bring them together cleanly for hosts that use both, while preserving their
separate ownership boundaries, their separate identity models, and their
separate execution semantics.

## What This Bridge Is For

The bridge exists for systems where truth and computation must remain separate,
but still move together with high precision.

It is meant to support:

- geometry kernels that need truth-preserving topology/history on one side and
  selective, explainable recomputation on the other
- chip-design and simulation systems that need replayable connectivity truth,
  branch-local analysis, and large-scale derived evaluation over stable
  snapshots
- AI systems that need speculative truth branches, speculative computation
  branches, and causal explanation across both
- interactive editors and workflow systems that need exact change routing
  without collapsing into hand-written invalidation code
- future multi-consumer platforms where one truth runtime may feed multiple
  derived systems through stable protocol contracts

The real breakthrough is not “signals can read relational data.” The
breakthrough is that truth history, identity evolution, patch streams,
snapshots, and derived execution can stay separate and still behave like one
coherent, auditable system.

## Why This Bridge Is Different

These are the architectural bets that make the bridge first-class:

- dual-graph architecture
- patch-to-invalidation bridge
- aspect mapping layer
- lineage-aware subscription continuity
- structural-identity-aware mapping
- snapshot evaluation
- historical and time-travel evaluation
- bulk change propagation
- planned routing and reduction artifacts
- change stream protocol
- reactive source protocol
- relational-key to signal-node mapping
- field / lens subscriptions
- branch-aware bridge semantics
- speculative truth-branch to signal-branch coordination
- end-to-end causality propagation
- bridge diagnostics and certification artifacts

The innovation is not only “truth changes trigger compute.” The innovation is a
protocol boundary that lets truth and computation remain separate systems while
still composing precisely, efficiently, historically, and causally.

## Mission

The bridge exists to answer these questions natively:

- How do committed truth changes become signal invalidations without hard-coding
  one runtime into the other?
- How do signal computations read stable truth snapshots without taking
  ownership of truth storage?
- How do relational aspects map onto signal aspects without lossy translation?
- How do identity evolution and lineage events preserve subscription
  continuity?
- How do structural identity and structural fingerprints participate in
  remapping and reuse without fusing the runtimes?
- How do large diff batches propagate efficiently and deterministically?
- How do truth IDs stay decoupled from signal node IDs while still supporting
  stable linkage?
- How do fine-grained field/facet dependencies remain precise at the bridge
  boundary?
- How do historical truth and branch-local truth feed replayable derived
  computation?
- How do speculative truth branches and speculative compute branches stay
  coordinated without becoming one runtime?

If the bridge is weak, truth and computation either fuse into one brittle
system or devolve into manual invalidation code and leaky abstractions. The
bridge therefore needs to be designed as a standalone integration architecture
between libraries, not as internal glue inside one host application.

## Architectural Model

### Runtime stack

| Layer | Responsibility | Owns |
| --- | --- | --- |
| Truth runtime | authoritative graph state | identity, mutation, history, diffs, lineage |
| Derived runtime | reactive execution | invalidation, recomputation, conditions, scheduling, convergence |
| Bridge / integration | coordination boundary | protocols, mapping, routing, snapshot-backed reads, causality transfer |

### Dual-graph rule

WORTH keeps two graphs on purpose:

- truth graph: the authoritative structural/versioned model
- computation graph: the dependency DAG for derived work

The bridge coordinates them. It does not erase the distinction.

### Dual-branch rule

When the host supports branching on both sides, the bridge must preserve that
distinction too:

- truth branches are branches of authoritative or speculative state
- computation branches are branches of derived execution state

The bridge may coordinate them, but it must not collapse them into one opaque
branch model.

## Ownership Boundaries

### What the bridge owns

- change feed contracts
- patch-to-invalidation routing
- aspect mapping semantics
- lineage-aware continuity rules
- structural-identity-aware remapping rules
- snapshot-backed and historical read contracts
- truth-key to signal-node mapping
- fine-grained subscription shapes
- bulk routing and reduction policy surfaces
- causality transfer contracts
- bridge-native diagnostics and certification artifacts

### What the bridge does not own

- truth mutation or truth storage
- signal scheduling internals
- domain-specific semantic scoring
- numeric compute logic
- permanent ID fusion across runtimes
- kernel-specific adaptation logic as the definition of the bridge

### Structural rule

The bridge translates, coordinates, and preserves causality. It does not become
a second truth runtime or a second scheduler.

## Principles

1. Truth and computation remain separate systems.
2. Change propagation starts from authoritative patch feeds, not ad hoc callbacks.
3. Aspects remain explicit at the bridge boundary.
4. Snapshot-backed evaluation is mandatory for determinism.
5. Truth IDs and signal node IDs remain decoupled.
6. Lineage and identity evolution must remain intelligible across the boundary.
7. Structural identity may inform mapping, but must not replace runtime identity ownership.
8. Bulk propagation is a first-class scale problem, not an optimization after the fact.
9. Fine-grained subscriptions belong in the bridge contract, not by leaking truth internals into signal.
10. Branch-aware coordination matters as much as current-state coordination.
11. Historical evaluation is a product surface, not a debugging accident.
12. Protocol boundaries matter as much as runtime internals.
13. The bridge must be library-grade architecture, not a special-case WORTH adapter.
14. End-to-end causality must survive the boundary from truth commit through invalidation and recomputation.
15. Diagnostics are a first-class bridge contract.
16. The bridge harness is first-class infrastructure.

## Pillars

### Dual-Runtime Foundation

#### Dual-graph architecture

Why it is first-class:
It preserves the distinction between authoritative state and derived
computation.

What coupling problem it prevents:
Without dual-graph architecture, the system drifts toward a fused runtime where
truth changes and reactive execution become inseparable.

Boundary ownership:
Truth runtime owns structural state. Signal runtime owns dependency scheduling.
The bridge coordinates between them.

#### Explicit truth/compute separation

Why it is first-class:
Each runtime can optimize for its real job rather than carrying the other
runtime’s constraints.

What coupling problem it prevents:
Fused designs make truth storage harder to branch and make compute scheduling
harder to reason about.

Boundary ownership:
The bridge enforces separation by making protocol surfaces explicit instead of
implicit shared state.

### Change Propagation

#### Patch-to-invalidation bridge

Why it is first-class:
Truth commits should drive recomputation automatically through structured
patchsets.

What coupling problem it prevents:
Manual invalidation logic becomes fragmented, lossy, and impossible to audit at
scale.

Boundary ownership:
Truth emits structured patch data; bridge translates it into signal
invalidations; signal executes the resulting recompute work.

#### Bulk change propagation

Why it is first-class:
Large truth diffs must propagate efficiently into the computation runtime.

What coupling problem it prevents:
Per-change invalidation routing creates overhead cliffs and hides the real
scale model.

Boundary ownership:
The bridge owns routing strategies for large patchsets, but not signal executor
internals.

#### Planned routing and reduction artifacts

Why it is first-class:
Large bridge flows should be planned and reduced deterministically, not routed
through ad hoc per-item handlers.

What coupling problem it prevents:
Without explicit routing plans and reduction artifacts, bridge behavior becomes
opaque, hard to certify, and expensive to scale.

Boundary ownership:
The bridge owns the routing/reduction contract; runtimes own the data they
produce and consume.

#### Deterministic invalidation routing

Why it is first-class:
The same diff must drive the same invalidation set and ordering semantics every
time.

What coupling problem it prevents:
Nondeterministic routing undermines replay, debugging, and trust in downstream
results.

Boundary ownership:
The bridge contract defines mapping and routing semantics; the signal runtime
preserves deterministic execution semantics after invalidation arrives.

### Semantic Mapping

#### Aspect mapping layer

Why it is first-class:
Relational aspects and signal aspects are distinct spaces that must be mapped
intentionally.

What coupling problem it prevents:
Without a dedicated mapping layer, aspect translation becomes scattered and
inconsistent.

Boundary ownership:
Truth runtime owns relational aspects; signal runtime owns signal aspects; the
bridge owns the mapping contract.

#### Lineage-aware subscription continuity

Why it is first-class:
Subscriptions must survive replace, split, merge-like truth evolution without
pretending identity never changed.

What coupling problem it prevents:
Without continuity rules, truth evolution causes either silent subscription loss
or incorrect stale linkage.

Boundary ownership:
Truth runtime owns lineage semantics; the bridge owns continuity and remapping
rules; signal runtime owns recomputation over the resulting subscriptions.

#### Structural-identity-aware mapping

Why it is first-class:
Structural identity and fingerprints can improve remapping, reuse, and branch
comparison across the boundary.

What coupling problem it prevents:
Without a dedicated bridge concept, structural identity leaks into both
runtimes inconsistently or is ignored entirely.

Boundary ownership:
Truth runtime may expose structural identity surfaces; the bridge decides how
they participate in mapping; signal runtime remains independent.

#### Relational-key to signal-node mapping

Why it is first-class:
Truth IDs and signal IDs should not collapse into one namespace.

What coupling problem it prevents:
Hard-coupled IDs make each runtime hostage to the storage/indexing choices of
the other.

Boundary ownership:
The bridge owns mapping and lookup. Truth and signal each keep their own
identity models.

#### Field / lens subscriptions

Why it is first-class:
Signals often depend on a specific field, facet, or region, not the whole truth
object.

What coupling problem it prevents:
Whole-entity subscriptions create coarse invalidation and throw away the value
of precise truth diffs and aspect-aware history.

Boundary ownership:
The bridge exposes fine-grained subscription shapes; truth provides the
field/facet view; signal consumes mapped invalidation.

### Read and History Contracts

#### Snapshot evaluation

Why it is first-class:
Signal computation must read stable truth state while writes continue elsewhere.

What coupling problem it prevents:
Direct live reads from mutable truth state make computation nondeterministic and
fragile.

Boundary ownership:
Truth runtime provides snapshots; the bridge supplies snapshot-backed read
surfaces; signal evaluates against them.

#### Historical and time-travel evaluation

Why it is first-class:
Derived computation should be able to evaluate against retained historical truth
intentionally, not only against “latest snapshot.”

What coupling problem it prevents:
Without a first-class contract, historical analysis becomes an ad hoc replay
hack rather than a trustworthy product surface.

Boundary ownership:
Truth runtime owns historical truth retention; the bridge owns historical read
contracts; signal runtime consumes them for replay, diagnosis, and comparison.

#### Branch-aware bridge semantics

Why it is first-class:
Branch-local truth and branch-local computation must stay coordinated without
being flattened into a current-state-only integration model.

What coupling problem it prevents:
Without branch-aware semantics, speculative or divergent work becomes hard to
reason about and easy to misroute.

Boundary ownership:
Truth runtime owns truth branches; signal runtime owns execution branches; the
bridge owns coordination rules between them.

#### Reactive source protocol

Why it is first-class:
Signals need a generic way to consume truth-backed data without embedding
storage internals.

What coupling problem it prevents:
If signal code must know relational storage details directly, the layering
collapses.

Boundary ownership:
The bridge owns the read contract; the truth runtime implements it; the signal
runtime consumes it.

### Speculation and Branch Coordination

#### Speculative truth-branch to signal-branch coordination

Why it is first-class:
When hosts explore alternate truth branches, derived computation needs a
coherent speculative execution model too.

What coupling problem it prevents:
Without explicit coordination, speculative workflows either duplicate too much
logic or silently mix speculative and authoritative derived state.

Boundary ownership:
Truth runtime owns speculative truth branches; signal runtime owns speculative
execution branches; the bridge owns correspondence and lifecycle rules between
them.

#### Preview and non-authoritative evaluation flows

Why it is first-class:
Not every bridge flow should require authoritative publication first.

What coupling problem it prevents:
Without preview semantics, interactive and AI systems are forced to choose
between unsafe live coupling and expensive full commit/replay loops.

Boundary ownership:
The bridge owns preview-flow contracts; truth and signal runtimes remain owners
of their own speculative state.

### Protocol Surfaces

#### Change stream protocol

Why it is first-class:
Patch feeds need a stable contract if multiple downstream systems are going to
consume them.

What coupling problem it prevents:
Ad hoc internal event shapes hardwire consumers to implementation details.

Boundary ownership:
Truth runtime produces the stream; the bridge defines the consumption contract;
downstream systems subscribe through it.

#### Stream correctness semantics

Why it is first-class:
Continuous integration between runtimes needs more than just an event shape; it
needs correct delivery semantics.

What coupling problem it prevents:
Without explicit ordering, cursor/checkpoint, replay/resume, idempotence, and
coalescing semantics, the bridge becomes hard to reason about under failure and
scale.

Boundary ownership:
Truth runtime provides deterministic stream material; the bridge defines how
consumers resume, checkpoint, and interpret the stream; the signal runtime
consumes it consistently.

#### Stable bridge contracts for host integrations

Why it is first-class:
Kernel and future integrations need a stable coordination surface.

What coupling problem it prevents:
Without explicit contracts, every integration path invents its own mapping and
propagation logic.

Boundary ownership:
The bridge defines the contract shapes; host systems adapt to them explicitly.

### Observability and Trust

#### End-to-end causality propagation

Why it is first-class:
The bridge should preserve causal identity across truth commit, patch emission,
invalidaton routing, and derived recomputation.

What coupling problem it prevents:
Without explicit causality propagation, provenance fragments at the runtime
boundary and explanations become local rather than end-to-end.

Boundary ownership:
Truth runtime owns commit provenance; the bridge carries and maps causal
references; signal runtime consumes them for explanation and trace surfaces.

#### Bridge-native diagnostics and failure taxonomy

Why it is first-class:
Change routing, mapping, historical evaluation, and causality transfer will
fail in distinct ways that must be visible as first-class bridge failures.

What coupling problem it prevents:
Without explicit diagnostics and failure classes, bridge behavior becomes
operationally opaque right where two complex runtimes meet.

Boundary ownership:
The bridge owns bridge-specific diagnostics, failure classes, and audit
surfaces; runtimes own their local diagnostics.

#### Bridge certification artifacts

Why it is first-class:
Serious systems will need machine-checkable artifacts showing what routed,
mapped, replayed, and recomputed across the boundary.

What coupling problem it prevents:
Without explicit artifacts, certification becomes narrative instead of
mechanical.

Boundary ownership:
The bridge owns bridge-level artifacts; truth and signal runtimes contribute
their own local artifacts.

## Roadmap Direction

This file is a vision document, not the execution roadmap. But the future work
should be derivable from it.

The highest-signal bridge programs are:

- dual-graph and dual-branch contract definition
- patch-to-invalidation and bulk routing productization
- aspect and lineage-aware mapping
- snapshot, historical, and branch-aware evaluation contracts
- speculative truth-branch to signal-branch coordination
- end-to-end causality transfer and replay parity
- bridge diagnostics, failure taxonomy, and certification artifacts

If a capability is named here and not yet built, it is roadmap work.

If a capability is built but not yet proven under replay, history, branch, and
failure scenarios, it is certification work.

## Non-goals

- turning the bridge into a fused truth+compute runtime
- requiring signal code to understand truth storage internals directly
- collapsing truth IDs and signal IDs into one handle space
- treating patch-to-invalidation, historical evaluation, or branch coordination as optional later polish
- making the bridge a grab bag of bespoke kernel glue

## Companion Documents

- [plans/worth-relational/worth_relational_vision.md](../WORTH-relational/WORTH_relational_vision.md)
- [plans/worth_signal/worth_signal_vision.md](../WORTH_signal/WORTH_signal_vision.md)
- [plans/worth_signal/worth_signals2.md](../WORTH_signal/WORTH_signals2.md)

Dual-graph architecture, lineage-aware continuity, snapshot and historical
evaluation, speculative branch coordination, and end-to-end causality are what
make this bridge more than “integration code.” If those are weak, the system
either fuses into a monolith or falls back to manual invalidation and leaky
abstractions. The bridge should therefore be designed as a strong coordination
layer between generic libraries that need to work together without losing their
separate strengths.
