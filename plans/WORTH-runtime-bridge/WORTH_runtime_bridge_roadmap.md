# WORTH Runtime Bridge Future Roadmap

## Purpose

This document defines the future work for the WORTH runtime bridge.

It is a future-only roadmap. It does not assume the bridge is already
productized, and it does not treat the bridge as incidental glue. It exists to
sequence the remaining work required to turn the bridge into a real causal
protocol layer between `worth-relational` and `worth-signal`.

The bridge is a general-use dual-runtime architecture, not a geometry-only
adapter. It must remain strong enough for geometry kernels, custom databases,
web and cloud platforms, traceable machine/manufacturing systems, chip
simulation, game engines, AI systems, and any other product surface that needs
authoritative truth and derived computation to stay separate without drifting
apart.

The governing rule remains:

- truth stays authoritative
- computation stays derived
- the bridge coordinates without collapsing either runtime into the other

Current cross-runtime hardening gate:

- [aspect_native_refactor.md](aspect_native_refactor.md) is the active
  crate-wide hard-break spec for replacing bridge-local JSON, raw byte,
  string-label, and digest-only authority residue with `worth-foundational`
  aspect contracts, masks, authoritative state, and patch/effect carriers.
- [../worth-query/runtime-authoritative-mutation-evidence-plan.md](../worth-query/runtime-authoritative-mutation-evidence-plan.md)
  freezes the shared Query-facing and bridge-facing authority-evidence contract
  for target identity, causality, provenance, naming/writeback evidence, and
  continuity-sensitive mutation carry-forward. Downstream write-heavy domains
  should treat that gate as the public dependency contract rather than assuming
  the seam is already implicitly closed.

The bridge's only job is to wire the runtimes together. It may translate,
route, lower, coordinate, and preserve causality across the boundary, but it
must not define truth semantics that belong to `worth-relational` or execution
semantics that belong to `worth-signal`.

## Roadmap Rules

Rules for every remaining bridge item:

- each milestone must describe a real bridge capability, not just "integration work"
- each milestone must preserve separate truth and compute ownership
- each milestone must preserve deterministic routing semantics and snapshot-backed reads
- each milestone must preserve replay from canonical truth and bridge artifacts
- each milestone must define concrete acceptance evidence through bridge harness scenarios, diagnostics artifacts, replay checks, parity checks, or certification suites
- no milestone is complete until both implementation and bridge-specific trust evidence exist
- the bridge must never become an authority for semantics owned by either parent runtime
- bridge planning, lowering, and execution must remain separate; no hot path may rediscover semantics that should have been lowered into proof-carrying bridge artifacts
- disposable bridge work may parallelize, but authoritative truth mutation and final truth publication remain serialized and canonical
- bridge APIs, policy surfaces, and configuration entrypoints must remain clean, explicit, and library-grade rather than becoming a bag of host-specific flags

## Geometry Kernel Critical Path

This section is the first build priority.

These are the bridge milestones that most directly make geometry kernels easier
to build, easier to debug, and easier to trust. They are the bridge features
that keep topology-aware truth and derived computation aligned without forcing
kernel code to devolve into manual invalidation and opaque rebuild logic.

Even here, the bridge is still only wiring. Geometry semantics, topology
authority, invariant authority, scheduling authority, and compute policy
authority remain in the parent runtimes.

If this section is weak, the kernel will inherit the classic dual-runtime
failure mode:

- topology changes route into recomputation inconsistently
- recomputation uses the wrong truth view
- lineage and identity evolution lose subscription continuity
- bridge routing falls over under large topology edits
- debugging stops at the runtime boundary instead of tracing through it

## Milestone 1: Patch-to-Invalidation and Snapshot Evaluation

### Goal

Make committed truth changes drive deterministic derived invalidation over
stable truth snapshots.

### Must Ship

- patch-to-invalidation routing as a first-class bridge surface
- snapshot-backed signal evaluation over committed truth
- deterministic invalidation routing for identical patch input
- bridge-side routing diagnostics for what patch items mapped to what invalidations
- explicit bridge read contracts that prevent live mutable truth reads during evaluation
- bridge acceptance scenarios proving stable snapshot-backed evaluation under active truth mutation

### Must Preserve

- truth remains the only authority
- signal runtime remains the owner of scheduling and execution
- no live mutable truth reads inside bridge evaluation flows
- no scheduler-shaped bridge observability

### Acceptance Evidence

This milestone is complete only when the bridge harness can prove:

- identical committed patchsets route to identical invalidation artifacts
- signal evaluation reads from stable truth snapshots rather than drifting live state
- bridge diagnostics can explain the mapping from patch surfaces to invalidated nodes
- replayed patch routing matches original routing semantics

## Milestone 2: Aspect Mapping and Fine-Grained Subscriptions

Platform integration companion:
[WORTH Signal Milestone 13.1](../WORTH_signal/milestone-13.1-plan.md) consumes
this Bridge-owned correspondence contract and governs the production cutover
through scoped Signal execution into Query-owned maintenance. That composed
path is now implemented and certified in the
[Milestone 13.1 closeout](../WORTH_signal/milestone-13.1-closeout.md); Milestone
2's local closeout remains only the Bridge-owned prerequisite.

Engineering spec: [milestone-2.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-2.md)
Shipped closeout: [milestone-2-closeout.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-2-closeout.md)
Envelope/planning hardening companion: [milestone-2-envelope-and-planning-hardening.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-2-envelope-and-planning-hardening.md)

### Goal

Preserve truth-side precision across the bridge so derived execution can depend
on exactly the changed truth surfaces instead of coarse whole-object routing.

### Must Ship

- aspect mapping layer between relational aspects and signal aspects
- field, lens, region, partition, or facet subscription shapes where the bridge needs more than whole-entity routing
- deterministic mapping semantics for aspect-aware and field-aware subscriptions
- diagnostics for unmapped, ambiguous, or suppressed aspect routing
- bridge artifact surfaces that show why a change mapped to a given signal dependency slice

### Must Preserve

- explicit aspect semantics on both sides
- no hidden truth-runtime leakage into signal dependency ownership
- deterministic mapping order and reduction
- stable snapshot-backed reads for fine-grained subscriptions

### Acceptance Evidence

This milestone is complete only when the bridge harness can prove:

- fine-grained truth changes invalidate only the intended derived surfaces
- coarse and fine subscription routes remain parity-safe with bridge diagnostics
- aspect mapping behavior is replayable and explainable

## Milestone 3: Lineage-Aware Subscription Continuity

Engineering spec: [milestone-3.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-3.md)

### Goal

Keep signal subscriptions intelligible when truth identity evolves through
replace, split, merge-like, or branch-local topology changes.

### Must Ship

- lineage-aware continuity rules for bridge subscriptions
- remapping behavior for replace and split-style truth evolution
- explicit handling for merge-like continuity and continuity rejection
- explicit handling for ambiguous continuity cases
- bridge diagnostics explaining how a subscription continued, split, merged, or failed continuity
- historical lookup support where the bridge needs historical ID resolution to preserve continuity

### Must Preserve

- truth runtime remains the owner of lineage semantics
- signal runtime remains the owner of derived node identity
- no silent subscription drift across identity evolution
- explicit failure rather than accidental continuity when mapping is ambiguous

### Acceptance Evidence

This milestone is complete only when the bridge harness can prove:

- truth identity evolution preserves or rejects subscription continuity deterministically
- topology-style replace/split flows remain traceable through bridge diagnostics
- replayed lineage-aware routing matches original continuity behavior

## Milestone 4: Historical and Branch-Aware Evaluation

Engineering spec: [milestone-4.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-4.md)
Shipped closeout: [milestone-4-closeout.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-4-closeout.md)

### Goal

Allow derived computation to evaluate intentionally against retained historical
truth and branch-local truth, not only the latest committed state.

### Must Ship

- historical snapshot evaluation contracts
- branch-aware bridge semantics for branch-local truth
- diagnostics showing what truth branch and truth version a derived run used
- parity checks between original historical evaluation and replayed historical evaluation
- bridge harness scenarios for divergent topology history and branch-local analysis

### Must Preserve

- branch identity remains explicit
- historical evaluation does not mutate truth
- no collapse of truth branches into derived-branch ownership
- canonical routing and observability over historical reads

### Acceptance Evidence

This milestone is complete only when the bridge harness can prove:

- historical and branch-local evaluation uses the intended truth surface
- branch-local truth does not leak into unrelated derived runs
- historical bridge evaluation remains replayable and diagnosable

## Milestone 5: Bridge Planning, Bulk Routing, and Parallel-Ready Scale Path

Engineering spec: [milestone-5.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-5.md)

### Goal

Make the bridge scale to large topology and geometry change sets through
explicit planning, canonical reduction, and parallel-ready preparation rather
than per-event overhead or opaque routing behavior.

### Must Ship

- planned routing and reduction artifacts for large patchsets
- bulk change propagation as a first-class bridge path
- a bridge planner that lowers patchsets, branch context, mapping context, and snapshot handles into executable bridge plans
- canonical ordering and reduction for bridge routing outputs
- packetized bridge planning for large read and routing surfaces
- explicit proofs or lowered legality markers for what bridge work may run in parallel and what must remain serial
- bridge counters for routed item count, reduction width, subscription fanout, work-packet count, fallback behavior, and serial-vs-parallel admission decisions
- performance-aware routing paths that preserve deterministic observability

### Must Preserve

- no hidden loss of bridge precision under load
- no non-deterministic reduction behavior
- no scheduler-shaped routing artifacts
- no executor rediscovery of planning semantics during hot execution
- bridge diagnostics remain bounded and structured

### Acceptance Evidence

This milestone is complete only when the bridge harness can prove:

- large patchsets route through planned bulk paths rather than per-item ad hoc handlers
- routing artifacts remain deterministic and replayable
- bridge plans lower once and execute without semantic rediscovery
- bridge counters explain the scale behavior honestly
- serial and admitted-parallel bridge preparation remain parity-safe

---

## Beyond Geometry Kernel Critical Path

Everything below this break still matters for the full product vision, but it
is less directly responsible for making a geometry kernel easy to build. This
is where the bridge roadmap expands from "make truth and recomputation sane for
kernel work" to "finish the full dual-runtime platform."

That broader platform scope still does not change the bridge's role. The bridge
becomes more capable here, not more authoritative.

## Milestone 6: Change Stream Protocol and Multi-Consumer Contracts

Engineering spec: [milestone-6.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-6.md)
Required certification suites: [test-requirements.md](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
Suites 1-3: Change Stream Checkpoint Fracture Equivalence, Multi-Consumer Coalescing Parity, Backpressure And Retention Anchor Hostility

### Goal

Turn bridge-side change consumption into a stable protocol surface rather than
one host-specific feed path.

### Must Ship

- explicit bridge-facing change stream protocol
- stream correctness semantics for ordering, resume, checkpoint, replay, idempotence, coalescing, and backpressure signaling
- bridge contracts that support more than one downstream consumer shape
- diagnostics for cursor, checkpoint, replay, coalescing, backpressure, and protocol mismatch failures

### Must Preserve

- canonical truth patch order
- deterministic interpretation of stream material
- no weakening of truth-runtime CDC semantics
- no host-specific glue becoming the public contract

### Acceptance Evidence

This milestone is complete only when bridge harness scenarios can prove:

- resumed and replayed consumption preserve bridge routing semantics
- multi-consumer protocol behavior stays deterministic
- checkpoint, resume, and coalescing behavior remain explicit and diagnosable
- protocol errors are explicit and diagnosable
- the Milestone 6 certification suites in `test-requirements.md` pass with canonical machine-checkable bundles

## Milestone 7: Reactive Source Protocol and Clean Host Surfaces

Engineering spec: [milestone-7.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-7.md)
Shipped closeout: [milestone-7-closeout.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-7-closeout.md)
Required certification suites: [test-requirements.md](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
Suites 4-6: Multi-Host Source Parity, Source Capability Rejection Boundary, Builder Surface Swap Parity

### Goal

Provide a generic bridge-grade source/read contract so computation can consume
truth-backed data without embedding relational storage details or host-specific
adapter glue.

### Must Ship

- reactive source protocol for truth-backed reads
- stable source contracts for snapshot reads, historical reads, branch reads, and field/facet reads where admitted
- clean builder/configuration entrypoints for bridge setup, source registration, policy wiring, and diagnostics selection
- explicit host adaptation points that keep database/platform/kernel-specific code outside the bridge core
- diagnostics for source contract violation, source mismatch, and unsupported source capability

### Must Preserve

- signal code does not learn truth storage internals directly
- bridge configuration mirrors subsystem boundaries rather than becoming a flat bag of flags
- source capabilities remain explicit rather than ambient
- host-specific adapters do not become accidental authority

### Acceptance Evidence

This milestone is complete only when bridge harness scenarios can prove:

- multiple host-shaped source implementations satisfy the same bridge contract
- source-backed evaluation remains parity-safe across supported read modes
- bridge setup remains explicit and comprehensible at construction sites
- the Milestone 7 certification suites in `test-requirements.md` pass with canonical machine-checkable bundles

## Milestone 8: Structural-Identity-Aware Remapping

Required certification suites: [test-requirements.md](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
Suites 7-9: Structural Match Ambiguity Torture, Structural Reuse Without Identity Fusion, Branch Comparison Drift

### Goal

Use structural identity and structural fingerprint surfaces to improve bridge
mapping, reuse, and branch comparison without fusing runtime identities.

### Must Ship

- structural-identity-aware remapping rules where the bridge can use them safely
- explicit diagnostics for structural match, structural ambiguity, and structural mismatch
- remapping artifacts that remain subordinate to truth/runtime identity ownership
- harness scenarios covering structural reuse and branch comparison behavior

### Must Preserve

- structural identity never replaces authoritative truth identity
- no accidental ID fusion across runtimes
- explicit failure or ambiguity reporting when structural signals are insufficient

### Acceptance Evidence

This milestone is complete only when bridge harness scenarios can prove:

- structural identity can assist remapping without overriding truth identity
- ambiguous structural matches are explicit and replayable
- the Milestone 8 certification suites in `test-requirements.md` pass with canonical machine-checkable bundles

## Milestone 9: Merge-Aware Bridge Semantics and Multi-Parent History Consumption

Required certification suites: [test-requirements.md](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
Suites 10-12: Merge Parent Order Determinism, Unsupported Merge Class Denial, Merge Replay And Explanation Parity

### Goal

Make the bridge explicitly understand merge-bearing truth history so merge
artifacts, causal frontiers, and schema-declared merge policies can participate
in bridge routing, remapping, explanation, and replay without being rediscovered
procedurally inside host code.

### Must Ship

- bridge consumption of ordered multi-parent truth history
- bridge-native handling of merge-bearing patch and lineage surfaces
- explicit bridge semantics for how merge ontology, causal frontier metadata, and schema-declared merge policy outcomes affect invalidation, continuity, and remapping
- diagnostics that distinguish ordinary truth evolution from merge-driven reconciliation, merge-driven continuity, merge-driven deletion, and merge-driven topology rewiring
- replay-safe bridge artifacts showing what merge inputs and merge outcomes influenced derived routing
- explicit denial or typed unsupported behavior where merge classes are not yet bridge-admitted

### Must Preserve

- bridge does not become the owner of merge authority
- truth runtime remains the owner of merge ontology, merge execution, and causal metadata
- merge meaning is consumed from canonical truth artifacts rather than re-invented inside bridge code
- explicit failure rather than heuristic branch reconciliation

### Acceptance Evidence

This milestone is complete only when bridge harness scenarios can prove:

- merge-bearing histories route deterministically through the bridge
- bridge diagnostics can explain merge-influenced invalidation and continuity behavior
- replayed bridge behavior over merge-bearing histories matches original behavior
- unsupported merge classes fail explicitly with typed bridge diagnostics
- the Milestone 9 certification suites in `test-requirements.md` pass with canonical machine-checkable bundles

## Milestone 10: Speculative Truth-Branch to Signal-Branch Coordination and Preview Flows

Engineering spec: [milestone-10.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-10.md)

Required certification suites: [test-requirements.md](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
Suites 13-15: Speculative Discard Zero-Residue, Speculative Commit Boundary Clarity, Preview Lifecycle Leak Resistance

### Goal

Coordinate speculative truth branches and speculative derived execution
branches without collapsing them into one runtime model, while also supporting
preview and non-authoritative bridge flows as first-class product surfaces.

### Must Ship

- explicit coordination rules between speculative truth branches and speculative signal branches
- preview or non-authoritative bridge flows for branch-local evaluation
- discard and commit semantics for speculative bridge outcomes
- lifecycle rules for temporary bridge resources created during preview flows
- diagnostics for speculative branch mismatch, invalid reuse, preview misuse, and branch leakage

### Must Preserve

- truth authority remains separate from speculative computation
- speculative derived state never becomes authoritative accidentally
- branch identity remains explicit end-to-end
- preview flows do not leave authoritative bridge residue

### Acceptance Evidence

This milestone is complete only when bridge harness scenarios can prove:

- speculative truth and speculative compute stay coordinated deterministically
- discard paths leave no authoritative bridge residue
- committed speculative flows become explainable and replayable
- preview and authoritative flows cannot be confused accidentally
- the Milestone 10 certification suites in `test-requirements.md` pass with canonical machine-checkable bundles

## Milestone 11: Cross-Runtime Policy Propagation and Clean Configuration Model

Engineering spec: [milestone-11.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-11.md)

Required certification suites: [test-requirements.md](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
Suites 16-18: Policy Provenance Equivalence, Illegal Policy Combination Rejection, Ambient Policy Leak Resistance

### Goal

Define how execution policy crosses the bridge so tolerance, priority, cost,
convergence, deterministic-vs-optimized mode, artifact policy, and diagnostics
policy remain explicit, composable, and cleanly configurable rather than
becoming hidden ambient state.

### Must Ship

- bridge policy model for deterministic-vs-optimized routing and evaluation requests
- explicit rules for what policy may cross from host or truth context into signal context and what must remain local to each runtime
- policy provenance artifacts that explain which policy surfaces altered bridge behavior
- clean builder-based or declaration-based configuration surfaces for bridge policy wiring
- diagnostics for illegal policy combinations, unsupported policy propagation, and policy-source ambiguity

### Must Preserve

- bridge does not become a second scheduler or a second policy authority
- no hidden ambient context across the boundary
- no flat configuration bag that ignores subsystem boundaries
- policy provenance remains replayable and machine-checkable

### Acceptance Evidence

This milestone is complete only when bridge harness scenarios can prove:

- the same bridge flow can run under deterministic and optimized policy modes without ambiguity about what changed
- policy propagation behavior is explicit, replayable, and diagnosable
- bridge configuration remains comprehensible at construction and call sites
- the Milestone 11 certification suites in `test-requirements.md` pass with canonical machine-checkable bundles

## Milestone 12: Bridge-Mediated Commit Strategies and Derived Writeback Contracts

Engineering spec: [milestone-12.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-12.md)

Required certification suites: [test-requirements.md](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
Suites 19-21: Bridge Writeback Idempotence And Diff Truth, Strategy Failure Containment, Authority Bypass Rejection

### Goal

Support bridge-mediated effect production as a first-class relational commit
strategy so signal-driven reconciliation, derived writeback, and similar
general-use workflows can participate in transaction, invariant, merge, replay,
and publication without becoming unsafe side channels.

### Must Ship

- explicit bridge contract for producing truth mutations or mutation plans from derived evaluation without giving the bridge direct authority over truth mutation
- integration with extensible commit strategies on the relational side for bridge-mediated evaluation flows
- containment rules for bridge-produced effects, including failure, panic, nondeterminism, and invariant violation handling
- replay-safe recording of bridge input context, bridge-produced mutation plan, and validation outcome
- strategy-aware bridge diagnostics that distinguish read-only bridge evaluation from bridge-mediated writeback attempts
- idempotence and output-diff-aware writeback semantics where the bridge reflects derived results into truth
- explicit denial paths for bridge flows that attempt to bypass invariant, merge, or commit authority

### Must Preserve

- serialized authority for final truth mutation
- bridge remains an effect producer or coordinator, not a parallel commit path
- replay parity for strategy-bearing histories
- invariant enforcement and merge semantics remain at the authoritative truth boundary

### Acceptance Evidence

This milestone is complete only when bridge harness scenarios can prove:

- bridge-mediated commit strategies can produce canonical replay-safe commit artifacts
- failing or divergent bridge strategies do not corrupt authoritative truth
- bridge writeback flows remain idempotent where the contract requires idempotence
- strategy-bearing and non-strategy-bearing histories remain distinguishable and diagnosable
- the Milestone 12 certification suites in `test-requirements.md` pass with canonical machine-checkable bundles

## Milestone 12b: Bridge-Native Extensible Writeback Families And Mapper Containment

Status: Complete

Engineering spec: [milestone-12b.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-12b.md)

Required certification suites: [test-requirements.md](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
Suites 22-24: Multi-Family Writeback Admission Boundary, Cross-Family Replay And Loop Isolation, Host Mapper Parity And Shadow-Protocol Rejection

### Goal

Make extensible writeback families a bridge-native protocol surface so domains
can admit multiple writeback families through one causal boundary without
teaching domain semantics to the bridge and without letting host mappers become
shadow writeback protocols.

### Must Ship

- bridge-owned writeback-family taxonomy and admission surfaces
- family-aware effect, candidate, outcome, diagnostics, and replay artifacts
- a sealed family admission boundary that avoids per-family bridge-core rewrites
- host-mapper containment rules that keep translation separate from protocol
  ownership
- at least two materially distinct admitted writeback families in
  certification-grade harness scenarios
- runtime-owned family admission, mapper, execution, and replay records with
  certification derived from those native records

### Must Preserve

- bridge remains a protocol boundary, not a domain runtime
- truth authority stays in `worth-relational`
- derived execution authority stays in `worth-signal`
- causality, idempotence, loop prevention, and replay remain bridge-owned and
  family-visible
- host mappers do not define bridge protocol semantics

### Acceptance Evidence

This milestone is complete only when bridge harness scenarios can prove:

- two admitted writeback families can coexist without aliasing replay,
  idempotence, or authority-boundary meaning
- host mapper variation that preserves family semantics yields equal canonical
  bundles
- undeclared or opaque writeback-family attempts fail explicitly before
  authority execution
- suite 22, 23, and 24 run as distinct hostile certifications rather than
  slices of one omnibus scenario
- the Milestone 12b certification suites in `test-requirements.md` pass with
  canonical machine-checkable bundles

## Milestone 13: End-to-End Causality, Failure Taxonomy, and Bridge Certification

Required certification suites: [test-requirements.md](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
Suites 25-27: End-To-End Causality Bundle Equivalence, Failure Taxonomy Localization, Certification Matrix Sufficiency
Engineering spec: [milestone-13.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-13.md)
Showcase extension companion: [milestone-13-showcase-extension.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-13-showcase-extension.md)

### Goal

Make the bridge fully certifiable as a causal protocol boundary, not just an
integration convenience layer.

### Must Ship

- end-to-end causality propagation from truth commit through invalidation to derived explanation
- bridge-native failure taxonomy
- bridge certification artifacts for routing, mapping, merge-aware evaluation, historical evaluation, preview flows, writeback flows, and replay parity
- one public bridge diagnostics entrypoint
- reusable bridge harness suites for patch-driven, historical, branch-aware, merge-bearing, strategy-bearing, parallel-admission, and failure-path scenarios
- one Rust-only reference workload that proves live high-fanout truth changes,
  speculative branch-local shocks, main-versus-speculative comparison, discard
  zero-residue, and authoritative commit promotion without requiring a UI

### Must Preserve

- separate runtime ownership on both sides
- canonical observability and replay behavior
- bridge artifacts remain machine-checkable and bounded
- the reference workload remains certification substrate rather than bridge-owned
  domain authority

### Acceptance Evidence

This milestone is complete only when the bridge harness can prove:

- end-to-end causality survives original execution and replay
- bridge failure classes are explicit and structured
- certification artifacts are sufficient to diagnose routing, merge-aware, policy-aware, and historical-evaluation failures mechanically
- the reference workload proves main-branch live propagation, speculative branch
  isolation, discard zero-residue, and commit-promotion clarity through the same
  canonical certification bundles
- the Milestone 13 certification suites in `test-requirements.md` pass with canonical machine-checkable bundles

## Milestone 14: Bridge-Native Subscription Declaration Families, Admission, and Lifecycle

Required certification suites: [test-requirements.md](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
Suites 28-30: Subscription Declaration Equivalence, Subscription Basis Binding And Rejection Boundary, Subscription Lifecycle Replay Parity

### Goal

Make subscriptions a first-class bridge protocol surface with explicit
declaration families, admission, basis binding, lifecycle, and diagnostics
instead of leaving long-lived truth-backed observation as a host-local
assembly of slices, streams, and observer handles.

### Must Ship

- one bridge-owned subscription declaration framework with extensible admitted
  declaration families
- bridge-native subscription family identity and instance identity distinct
  from slice identity, stream identity, and consumer identity
- explicit basis binding for explicit snapshot-bound and branch-head truth
  views
- subscription admission artifacts and typed rejection artifacts
- lifecycle typestates carried by declaration artifacts, admitted artifacts,
  activation-ready handles, deactivated handles, and typed rejection artifacts
- explicit rules for subscription equivalence and non-equivalence
- explicit lowering from admitted bridge declaration families into admitted
  `worth-signal` observation policies and delivery/coalescing strategies
- diagnostics explaining what truth surfaces, basis, and lowered slice set a
  subscription owns and why admission succeeded or failed
- explicit denial paths for unsupported basis and unsupported capability
  combinations

### Must Preserve

- bridge does not become the owner of query semantics
- truth runtime remains the authority for branch, history, retention, and truth
  materialization
- signal runtime remains the authority for execution scheduling and observation
  internals
- slice identity remains subordinate to subscription family and subscription
  identity rather than replacing it
- no ambient host context chooses snapshot or branch-head basis implicitly
- the bridge does not redefine `worth-signal` observation semantics; it lowers
  admitted bridge families into the existing `worth-signal` strategy substrate

### Acceptance Evidence

This milestone is complete only when bridge harness scenarios can prove:

- semantically equivalent declarations within one admitted family yield
  identical subscription admission artifacts
- non-equivalent declarations differ on canonical subscription digests
- different admitted declaration families differ mechanically in both bridge
  lowering and selected `worth-signal` strategy
- unsupported or ambiguous basis combinations fail explicitly and typed
- lifecycle transitions are replay-safe and diagnosable
- declaration identity remains stable under replay and diagnostics-tier
  variation
- the Milestone 14 certification suites in `test-requirements.md` pass with
  canonical machine-checkable bundles

## Milestone 15: Subscription Delivery Families, Continuation, and Shared Consumer Contracts

Engineering spec: [milestone-15.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-15.md)
Shipped closeout: [milestone-15-closeout.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-15-closeout.md)

Required certification suites: [test-requirements.md](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
Suites 31-34: Shared Subscription Fanout Parity, Subscription Continuation Across Identity Evolution, Subscription Resume Replay And Checkpoint Exactness, Preview Subscription Zero-Residue

### Goal

Make active subscriptions durable as bridge protocol entities over delivery,
restart, continuity, fanout, replay, and preview boundaries so a manual host
can explicitly do the same class of work that higher-level systems such as
WORTH Query will eventually automate.

### Must Ship

- subscription activation contracts tied to admitted consumer contracts
- delivery contract vocabulary for family-aware canonical member delivery,
  admitted coalesced delivery, replay or audit-oriented delivery, route-focused
  delivery, and other admitted bridge families lowered into `worth-signal`
  strategy space
- subscription checkpoint and continuation identity distinct from raw stream
  checkpoint identity where needed
- explicit resume and replay rules for active subscriptions
- shared-subscription and multi-consumer fanout contracts
- explicit continuation rules across replace, split, merge-like history where
  admitted, and branch divergence
- subscription-aware preview behavior covering preview-scoped subscriptions,
  discard semantics, promotion-boundary interaction, and zero-authoritative-
  residue guarantees
- counters and diagnostics for subscriber fanout, coalescing, continuation
  remap, replay and resume outcomes, and rejected reuse or sharing attempts
- diagnostics that explain which admitted bridge family lowered into which
  `worth-signal` observation and delivery strategy for an active subscription

### Must Preserve

- consumer pacing does not redefine subscription meaning
- stream coalescing does not redefine subscription identity
- branch identity remains explicit through continuation and replay
- preview subscriptions never become authoritative except through an explicit
  promotion boundary
- manual hosts do not need hidden host-local caches to reconstruct subscription
  meaning
- delivery families remain extensible, but only through admitted bridge
  declaration families and admitted `worth-signal` strategy lowerings

### Acceptance Evidence

This milestone is complete only when bridge harness scenarios can prove:

- equivalent shared-consumer subscriptions within one admitted family preserve
  canonical subscription meaning
- resumed subscriptions match original delivery semantics
- continuation after lineage evolution is deterministic, explicit, and typed
- preview subscription discard leaves zero authoritative residue
- shared subscriptions and separate-but-equivalent subscriptions remain
  parity-checkable
- at least two admitted subscription families remain parity-safe through
  activation, sharing, continuation, and replay
- the Milestone 15 certification suites in `test-requirements.md` pass with
  canonical machine-checkable bundles

## Milestone 16: Subscription Family Certification and End-to-End Subscription Workload

Engineering spec: [milestone-16.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/milestone-16.md)

Required certification suites: [test-requirements.md](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
Suites 35-37: End-To-End Subscription Bundle Equivalence, Subscription Failure Taxonomy Localization, Subscription Reference Workload Sufficiency

### Goal

Make bridge-native subscriptions certifiable through canonical bundles, hostile
lifecycle scenarios, and one reference workload extension that proves long-
lived ongoing observation across authoritative, historical, branch-local, and
preview flows.

### Must Ship

- one canonical subscription certification bundle shape
- bridge-native subscription failure taxonomy additions
- subscription-specific extensions to the public bridge diagnostics entrypoint
- one reference workload extension proving long-lived active subscriptions,
  shared equivalent subscriptions, branch-local subscription isolation,
  historical-basis subscription replay, preview subscription discard and
  promotion behavior, and multi-consumer continuation after restart
- offline-certifiable subscription bundle comparison rules
- certification coverage for multiple admitted bridge declaration families and
  their distinct `worth-signal` strategy lowerings

### Must Preserve

- no host logs as primary proof
- no ambient process state is required for audit
- subscription certification remains bridge-native rather than Query-only
- subscription workloads remain certification fixtures rather than bridge-owned
  domain semantics
- certification must prove family-aware lowering, not just one generic
  subscription lifecycle

### Acceptance Evidence

This milestone is complete only when bridge harness scenarios can prove:

- original execution, replay, restart, and hostile adapter variation preserve
  canonical subscription meaning where they should
- intentionally different subscription declarations differ mechanically
- illegal continuation, reuse, or basis drift fails explicitly and typed
- diagnostics richness changes retained detail only
- the subscription reference workload proves subscription behavior under
  branch, preview, replay, and fanout pressure
- the certification bundle distinguishes declaration-family variation from
  instance-level lifecycle variation
- the Milestone 16 certification suites in `test-requirements.md` pass with
  canonical machine-checkable bundles

## Milestone 17: Temporal Bridge Basis and Time-Aware Lowering

Required certification suites: [test-requirements.md](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
Suites 38-41: Temporal Bridge Basis Equivalence, Truth Patch Plus Clock Advance Replay Parity, Time-Aware Subscription Basis Rejection, Historical Truth With Temporal Wake Replay

### Goal

Make time-aware bridge flows explicit after `worth-signal` owns temporal
eligibility, scheduled wakes, previous-value access, and temporal replay.

The bridge must be able to bind a relational truth basis to a signal temporal
basis without turning `worth-relational` into a scheduler or letting bridge
host code redefine signal clock semantics.

### Must Ship

- bridge-visible temporal basis records that bind truth snapshot, branch, or
  CDC cursor evidence to signal clock basis and temporal wake evidence
- time-aware subscription admission for bridge families that depend on
  admitted `worth-signal` temporal policies
- routing and delivery records for non-patch wake causes, including flows where
  no relational commit occurred but signal time made derived work eligible
- replay rules for mixed truth-patch and clock-advance sequences
- diagnostics explaining whether a derived update came from truth change, clock
  advance, temporal wake readiness, or a combined cause
- explicit rejection paths for missing temporal basis, incompatible clock
  basis, unsupported historical temporal replay, and host-attempted temporal
  meaning redefinition

### Must Preserve

- truth runtime remains the authority for truth history, snapshots, branches,
  CDC cursors, and retention
- signal runtime remains the authority for clock domains, temporal
  eligibility, wake scheduling, previous-value access, and temporal execution
- bridge binds and certifies cross-runtime causality; it does not own clocks,
  temporal policies, or truth history
- relational commits are not required for time-only derived eligibility, but
  any truth read used by that work remains bound to an explicit truth basis
- diagnostics richness may vary, but temporal truth and truth-view basis may
  not vary with diagnostics tier

### Acceptance Evidence

This milestone is complete only when bridge harness scenarios can prove:

- equivalent truth-basis plus signal-temporal-basis inputs produce equivalent
  bridge temporal basis artifacts
- reordered host calls cannot change mixed truth-patch and clock-advance replay
  outcomes
- time-aware subscriptions fail explicitly when snapshot, branch, clock, or
  wake evidence is missing, stale, or mismatched
- historical truth evaluation with temporal wake readiness replays from
  canonical truth and signal temporal artifacts rather than ambient host time
- time-only derived updates are diagnosable without pretending a relational
  patch caused them
- the Milestone 17 certification suites in `test-requirements.md` pass with
  canonical machine-checkable bundles

## Milestone 18: Async Resource Bridge Families and Completion Causality

Required certification suites: [test-requirements.md](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
Suites 42-45: Async Source Lifecycle Bridge Parity, Out-Of-Order Completion Truth-Basis Supersession, Async Retry And Revalidation Causality, Async Completion Writeback Loop Prevention

### Goal

Make bridge-declared async and resource-backed sources lower into
`worth-signal` async/resource lifecycle truth without leaving request identity,
completion admission, retry, cancellation, or stale-completion rejection as
host-local conventions.

### Must Ship

- bridge async/resource declaration families that lower into admitted
  `worth-signal` async/resource node lifecycle surfaces
- request identity binding bridge declaration, truth-view basis, subscription
  instance where applicable, and signal async request generation
- completion admission records for fulfilled, rejected, cancelled, timed-out,
  retried, superseded, and stale-denied outcomes
- stale completion rejection when truth basis, signal generation, branch,
  subscription instance, temporal basis, or admitted family identity has moved
  on
- retry and revalidation evidence proving signal owns lifecycle scheduling
  while bridge owns cross-runtime causality and source binding
- optional writeback coordination through admitted bridge writeback families,
  including causality transfer, idempotence, and loop-prevention evidence
- diagnostics that distinguish transport failure, signal lifecycle denial,
  bridge source-family rejection, truth-basis mismatch, stale completion, and
  relational writeback rejection

### Must Preserve

- external async transport is not bridge authority
- bridge does not own retry, backoff, timeout, cancellation, or completion
  scheduling semantics
- async/resource state remains derived state unless an admitted writeback path
  commits authoritative truth through `worth-relational`
- async completions cannot mutate relational truth except through explicit
  bridge-mediated writeback admission
- out-of-order physical completion may vary, but admitted bridge completion
  meaning must remain generation-safe and replay-honest

### Acceptance Evidence

This milestone is complete only when bridge harness scenarios can prove:

- async/resource bridge declarations lower into distinct, admitted signal
  lifecycle families without host-local lifecycle invention
- out-of-order completions commit, reject, cancel, retry, or supersede exactly
  according to canonical bridge and signal evidence
- stale completions cannot publish over newer truth basis, branch basis,
  temporal basis, subscription instance, or signal generation
- retry and revalidation preserve causality and compare equal to equivalent
  no-failure control lanes where the admitted lifecycle says they should
- async completion writeback cannot create bridge-origin feedback loops or
  bypass truth authority
- the Milestone 18 certification suites in `test-requirements.md` pass with
  canonical machine-checkable bundles

## Milestone 19: Temporal/Async Subscription Certification and Reference Workload

Required certification suites: [test-requirements.md](C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
Suites 46-50: Temporal Async Subscription Bundle Equivalence, Mixed Cause Delivery Ordering Parity, Restart Resume With Clock And Inflight Basis, Temporal Async Failure Taxonomy Localization, End-To-End Temporal Async Reference Workload Sufficiency

### Goal

Close the bridge roadmap after `worth-signal` temporal and async/resource
substrates exist by certifying one end-to-end bridge story where relational
truth commits, signal clock advances, temporal wakes, async completions,
subscriptions, restart, replay, branch-local preview, promotion, and discard
all compose without a second truth model or a second scheduler.

### Must Ship

- one canonical temporal/async bridge certification bundle shape
- subscription checkpoint and resume records that include truth cursor,
  truth-view basis, signal temporal basis, async request generation, and
  subscription delivery basis where applicable
- mixed-cause delivery ordering rules for truth patches, clock advances,
  temporal wakes, async completions, retries, cancellations, and supersessions
- multi-consumer fanout over time-aware and async-backed subscriptions
- restart-safe recovery of active temporal and inflight async bridge state from
  canonical bridge and parent-runtime artifacts
- offline diagnostics for stale async completion, missing temporal basis,
  incompatible resume basis, mixed-cause replay drift, preview discard residue,
  and promotion-boundary mismatch
- one reference workload proving live truth changes, time-only updates, async
  completion, retry, cancellation, branch-local preview, promotion, discard,
  restart, and replay without relying on a UI or host-local debug logs

### Must Preserve

- bridge certification remains an integration proof, not domain authority
- no ambient process clock, host transport state, host logs, or live runtime
  memory is required for replay or offline diagnosis
- truth-side causality, signal temporal causality, and signal async lifecycle
  causality remain distinct but linkable
- consumer pacing, stream coalescing, clock advancement, and async completion
  order may affect delivery timing only through admitted bridge and signal
  artifacts
- reference workloads remain certification fixtures rather than bridge-owned
  product semantics

### Acceptance Evidence

This milestone is complete only when bridge harness scenarios can prove:

- original execution, replay, restart, and hostile adapter variation preserve
  canonical temporal/async subscription meaning where they should
- mixed truth/time/async causes deliver in the same canonical order under
  replay, even when host call order and physical async completion order vary
- stale, truncated, incompatible, or cross-branch resume basis fails explicitly
  and typed
- offline bundles are sufficient to diagnose temporal basis failures, async
  completion failures, mixed-cause delivery drift, preview residue, and
  promotion-boundary mismatches
- the reference workload proves authoritative, historical, branch-local,
  preview, time-only, async-backed, shared-consumer, restart, and replay lanes
  through one coherent bridge certification story
- the Milestone 19 certification suites in `test-requirements.md` pass with
  canonical machine-checkable bundles

## Completion Standard

The bridge roadmap is complete only when:

- all geometry-kernel-critical bridge milestones are shipped
- all broader dual-runtime bridge milestones are shipped
- bridge harness scenarios cover patch routing, aspect mapping, fine-grained subscriptions, lineage continuity, historical evaluation, branch coordination, planned bulk routing, reactive source contracts, merge-bearing history, preview flows, bridge-mediated commit strategies, and causality transfer
- bridge harness scenarios also cover first-class subscription declaration,
  admission, lifecycle, continuation, replay, fanout, preview-scoped
  subscriptions, and subscription-specific certification bundles
- those subscription scenarios prove multiple admitted subscription families
  lowered into `worth-signal`'s extensible observation and delivery substrate
- bridge harness scenarios also cover temporal-basis binding, time-aware
  subscription lowering, async/resource completion causality, mixed
  truth/time/async delivery ordering, restart-safe temporal and inflight basis
  resume, and offline certification for time-aware and async-backed bridge
  subscriptions
- bridge diagnostics and artifacts are canonical, machine-checkable, and replay-safe
- bridge configuration and policy surfaces are explicit, composable, and clean enough to serve as library-grade public contracts rather than host-local conventions
- no shipped bridge surface defines logic that belongs structurally to truth authority or compute authority

## Companion Documents

- [plans/worth-runtime-bridge/worth_runtime_bridge_vision.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/worth_runtime_bridge_vision.md)
- [plans/worth-runtime-bridge/aspect_native_refactor.md](aspect_native_refactor.md)
- [plans/worth-runtime-bridge/test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-runtime-bridge/test-requirements.md)
- [plans/worth-relational/worth_relational_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/worth_relational_roadmap.md)
- [plans/worth_signal/worth_signal_vision.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/worth_signal_vision.md)
- [plans/worth_signal/worth_signals2.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/worth_signals2.md)
