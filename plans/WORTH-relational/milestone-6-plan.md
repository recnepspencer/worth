# Milestone 6 Engineering Spec: Lineage and Correspondence Completion

## Summary

Milestone 6 makes identity evolution truth-grade. The runtime must move from
"lineage exists and is queryable" to "lineage is an authoritative, canonical,
branch-scoped commit artifact with explicit promotion semantics, deterministic
replay equivalence, and certification-grade historical resolution."

This milestone will be built in ordered phases. Each phase produces explicit
proof-bearing types that the next phase consumes. The milestone will not be
implemented by adding behavior into the current lineage files. It will begin
with a structural split that mirrors the lineage authority lifecycle, then add
typed planning/finalization/publication/replay surfaces, then close with
certification and complexity-proof lanes.

The governing rule for the milestone is:

- advisory correspondence is not authority
- authority is decided exactly once at commit finalization
- exactly one canonical lineage artifact is published per commit
- history, replay, diagnostics, query, and summary surfaces derive from that
  artifact
- branch crossing is impossible without an explicitly authorized
  reconciliation/merge type
- all lineage cost claims must be visible at named facade/phase boundaries

## Phase 1: Structural Split and Canonical Domain Vocabulary

### Goal

Reshape lineage into responsibility-aligned subdomains before behavior
expansion so Milestone 6 does not accumulate into `lineage/data/mod.rs`,
`lineage/logic/authority.rs`, or broad lineage test files.

### Required file structure

#### Production structure

- `src/lineage/data/`
- `events.rs`
  - `LineageEventKind`
  - `LineageEventRecord`
  - canonical event ordering rules
  - lineage event equivalence helpers
- `correspondence.rs`
  - `CorrespondenceCandidate`
  - `CorrespondenceCandidateId`
  - `CorrespondencePromotionOutcome`
  - rejection-class data types
- `graph.rs`
  - `LineageNode`
  - `LineageGraphSnapshot`
  - `LineageDivergenceSummary`
- `resolution.rs`
  - `HistoricalLineageResolution`
  - `HistoricalResolutionRequest`
  - `HistoricalResolutionTrace`
- `artifacts.rs`
  - `LineageFinalizationArtifact`
  - `PublishedLineageArtifact`
  - lineage digest/equivalence basis types
- `invariants.rs`
  - lineage-specific invariant and rejection classes
- `metrics.rs`
  - lineage counters and summaries
- `mod.rs`
  - re-export only

- `src/lineage/logic/access/`
- `records.rs`
- `graph.rs`
- `resolution.rs`
- `aspect_history.rs`
- `divergence.rs`
- `mod.rs`

- `src/lineage/logic/authority/`
- `candidate_recording.rs`
- `candidate_validation.rs`
- `promotion_planning.rs`
- `promotion_execution.rs`
- `commit_finalization.rs`
- `publication.rs`
- `mod.rs`

- `src/lineage/facade.rs`
- facade only; no internal planners, validators, subsystem state, or raw runtime
  mutation helpers

- `src/logic/runtime/state/subsystems/lineage.rs`
- state and lifecycle only
- must not own promotion policy, resolution traversal policy, or publication
  logic

#### Test structure

- `src/tests/lineage/`
- `candidate_recording.rs`
- `promotion_validation.rs`
- `promotion_execution.rs`
- `historical_resolution.rs`
- `graph_queries.rs`
- `branch_locality.rs`
- `certification.rs`
- `mod.rs`

- `src/tests/domains/cad/`
- `topology_identity_survival.rs`

- `src/tests/domains/chip/`
- `netlist_rewiring_identity_history.rs`

- `src/tests/support/lineage.rs`
- generic lineage fixture/support only

- `src/tests/support/domains/cad.rs`
- CAD-only fixture/support

- `src/tests/support/domains/chip.rs`
- chip-only fixture/support

### Structural rules

- `lineage/data/mod.rs`, `lineage/logic/mod.rs`, and `lineage/facade.rs` are
  wiring files only.
- No new milestone behavior may be added to `mod.rs` files.
- No generic "lineage manager," "lineage helpers," or mixed "contracts" test
  file is allowed.
- Each file must map to one phase responsibility or one domain concept.

### Phase integrity and anti-bypass enforcement

The structural split is not sufficient by itself. Milestone 6 must also remove
the common escape hatches that let raw data bypass the phase chain.

Required enforcement rules:

- all proof-bearing lineage phase wrappers must have crate-private constructors
- transitions between phase-bearing wrappers may occur only in the owning
  lineage authority module for that phase
- raw lineage ids, raw event vectors, and raw candidate vectors must not cross
  authority boundaries directly
- no blanket `From` / `Into` conversions from raw collections into
  proof-bearing wrappers are allowed
- no convenience constructors may exist that recreate a later phase wrapper
  from weaker phase input
- internal planners, validators, and executors remain `pub(crate)` and are not
  publicly reachable except through facade-owned entrypoints
- only milestone-approved public types may leave `src/lineage/facade.rs`
- internal lineage collections remain non-public; no direct external access to
  runtime-owned candidate/event stores is allowed

Required compile-time verification:

- compile-fail tests must prove that raw ids, raw vectors, and branch-unscoped
  lineage refs cannot enter promotion, finalization, or publication APIs
- compile-fail tests must prove that later phase wrappers cannot be
  constructed outside their owning module
- compile-fail tests must prove that cross-branch lineage inputs cannot enter
  authority APIs without the explicit future reconciliation type

## Phase 2: Typed Authority Phase Chain

### Goal

Encode lineage authority as a proof-widening chain so later phases cannot
accept weaker inputs and re-decide earlier questions.

### Required phase chain

The implementation must introduce explicit types for the lineage authority
lifecycle. Exact names may vary, but the shape must be preserved.

1. `RecordedCorrespondenceCandidateSet`
- branch-scoped recorded advisory candidates only
- may contain invalid or ambiguous candidates
- not acceptable input to promotion execution

2. `ValidatedCorrespondenceCandidateSet`
- candidate references exist
- candidate branch scope is coherent
- candidate shape passes structural validation
- still not guaranteed promotion-eligible

3. `PromotionEligibleCandidateSet`
- ambiguity resolved or rejected
- candidate class is promotable
- branch authority proven
- acceptable input to promotion plan lowering

4. `LoweredPromotionPlan`
- monomorphic execution plan for promotion
- carries exact source/target lineage ids
- carries branch scope proof
- carries rejection-free promotion decision basis
- promotion execution must consume only this type

5. `FinalizedLineageEventBatch`
- create/replace/split/merge/retire/promoted-correspondence events
- canonical event ordering established
- event equivalence contract established
- rollback-safe and commit-bound

6. `LineageFinalizationArtifact`
- single canonical lineage artifact produced at commit finalization
- contains finalized event batch, event ids, branch-scoped digest basis,
  decision log, and metrics
- this is the authority artifact for lineage publication

7. `PublishedLineageArtifact`
- publication-ready lineage artifact derived directly from
  `LineageFinalizationArtifact`
- attached into canonical commit envelope/history surfaces without re-deciding
  lineage semantics

### Enforcement rules

- Promotion execution APIs must not accept raw candidates.
- Commit publication/history attachment APIs must not accept raw event vectors.
- Replay/digest/history surfaces must consume canonical lineage artifacts, not
  reconstruct lineage meaning from raw state.
- Rejection must occur before `LoweredPromotionPlan` construction.
- Later phases may enrich metrics or summaries, but may not reinterpret
  promotion legality.
- Rollback must consume lineage effect records derived from the finalized batch
  or finalization artifact itself; rollback correctness must not depend on
  ad hoc residue cleanup or re-querying the runtime for what changed.

## Phase 3: Branch Locality as a Mechanical Boundary

### Goal

Make branch-local identity evolution structurally enforced, not merely
validated by convention.

### Required representation rules

The plan must enforce branch locality with proof-bearing request/authority
types.

Required types or equivalent enforcement surfaces:

- `BranchScopedLineageRef`
- a lineage reference paired with branch scope
- raw `LineageId` must not cross into promotion/finalization APIs alone

- `HistoricalResolutionRequest`
- explicit branch-scoped resolution request
- no branch-agnostic historical resolution API

- `PromotionAuthority`
- zero-sized or sealed witness proving the caller is authorized to promote
  within branch scope
- only authority paths can construct it

- `CrossBranchLineageReconciliation`
- reserved explicit type for future merge/reconciliation work
- cross-branch lineage combination must be impossible without this class of
  type

### API rules

- `lineage_access()` read APIs may take branch-scoped requests and return
  branch-scoped results.
- `lineage_authority()` promotion/finalization APIs must require branch-scoped
  proof-bearing inputs.
- No API may accept "source lineage ids + target lineage ids + branch id" as
  unrelated loose values where invalid combinations remain representable.
- Branch divergence summaries are read surfaces only and cannot be reused as
  promotion inputs.
- Branch scope proofs must be opaque outside the owning lineage modules; the
  caller may hold them, but may not WORTH or widen them.

## Phase 4: Canonical Artifact and Derived Surface Rules

### Goal

Declare one canonical lineage artifact at commit finalization and make all
other lineage surfaces derived from it.

### Canonical artifact rule

There must be exactly one canonical lineage artifact produced during commit
finalization:

- `LineageFinalizationArtifact`

It must contain:

- finalized canonical event batch
- canonical ordered event ids
- canonical branch scope
- promotion decision log
- rejection summaries, if the commit fails before publication
- lineage digest basis
- lineage metrics for the finalization boundary

### Derived surfaces

The following must derive from `LineageFinalizationArtifact`, not recompute
lineage independently:

- commit summary lineage view
- history envelope lineage attachment
- replay lineage digest input
- durability lineage persistence
- inspection lineage event ids
- lineage diagnostics summaries
- lineage historical-resolution acceleration inputs if retained

### Decision-log contract

The decision log inside `LineageFinalizationArtifact` is a first-class
authority artifact, not optional trace metadata.

It must be:

- queryable by correspondence candidate id
- queryable by promoted lineage event id
- queryable by rejection class
- canonically ordered for digest/replay parity
- sufficient to explain why a promotion succeeded, failed, or remained
  advisory without re-running promotion logic from raw state

If a later consumer cannot answer "why did this lineage transition happen or
fail?" from the canonical lineage artifact and its decision log, the milestone
is incomplete.

### Non-goals

- no separate "history lineage artifact" with independent semantics
- no separate "replay lineage view" that is canonicalized differently
- no commit-summary lineage reconstruction from raw event vectors
- no raw event lists becoming accidental public authority surfaces

## Phase 5: Public API and Cost-Honest Read Surface Design

### Goal

Expose lineage reads and authority operations with honest cost boundaries and
explicit traversal/reconstruction semantics.

### Public API categories

The public lineage surface must distinguish these categories explicitly:

1. Cheap branch-scoped record lookup
- example class: record-to-lineage lookup
- bounded by direct lookup cost
- may expose touched-record count

2. Graph snapshot reads
- explicit graph materialization boundary
- returns a branch-scoped graph snapshot
- must expose node/event/candidate counts

3. Divergence summarization
- explicit summary/traversal boundary
- not shaped like a cheap property getter
- must expose traversed-event breadth and shared-lineage breadth

4. Historical resolution
- explicit reconstruction boundary
- request/result types must make traversal cost visible
- result includes traversed event ids and breadth metrics
- boundedness basis must be explicit in the request/result surface
- the API must state whether it is resolving from a branch-scoped lineage
  seed, retained event window, or other bounded authority basis

5. Aspect-history lineage composition
- explicit historical query boundary
- must remain distinct from direct lineage resolution

### Signature rules

- No historical-resolution or divergence API may look like an O(1) accessor.
- API naming must signal traversal/reconstruction, not "get."
- Results for traversal/reconstruction APIs must carry counters or summaries
  sufficient to explain work performed.
- If an API materializes a full branch graph, its name and result must make
  that breadth explicit.
- No historical resolution API may begin from "all lineage for branch" unless
  its type name, metrics, and diagnostics explicitly declare full-branch
  reconstruction semantics.

### Mechanical data topology and buffer lifetime assumptions

Milestone 6 must be performance-shaped, not only performance-observed. The
implementation must therefore state and follow a mechanical storage/topology
basis consistent with dominant lineage access paths.

Required assumptions:

- lineage event records must be stored in a topology aligned with dominant
  authority and read access paths rather than incidental append-only shapes
  that force broad rescans
- branch-scoped lineage traversal must not rely on full-graph filtering by
  branch on hot paths where the runtime claims bounded branch-local work
- historical resolution should prefer direct indexed lineage/event access over
  indirect repeated key-based rediscovery where that matches runtime
  conventions
- authority-phase candidate, plan, and event buffers should use
  transaction-lifetime arenas, pre-sized reusable buffers, or equivalent
  lifecycle-scoped allocation strategies where possible
- graph snapshots, divergence summaries, and historical traces must declare
  whether they are rebuilt, retained, or incrementally maintained by policy;
  this must not remain an accidental implementation detail

The milestone does not need to pre-commit to one exact storage micro-design in
the spec, but it must commit to access topology that keeps dominant lineage
operations proportional to their named authority basis rather than to whole
runtime breadth.

## Phase 6: Measurement Boundaries and Complexity Contracts

### Goal

Add named counters and proof surfaces at lineage subsystem boundaries so
Milestone 6 is compliant with the repo's performance laws, not only
semantically correct.

### Required measurement boundaries

At minimum, the milestone must expose counters for:

- correspondence candidate width
- validated candidate width
- promotion-eligible candidate width
- promotion rejection count by class
- promotion accepted count
- finalized lineage event batch width
- branch-local graph touch count
- historical traversal breadth
- divergence traversal breadth
- graph snapshot node/event/candidate counts
- publication lineage artifact width
- replay lineage digest breadth
- rollback lineage residue count
- cross-branch rejection count

### Required counter placement

- authority planning boundary
- authority finalization boundary
- lineage access facade boundary
- replay lineage verification boundary
- rollback verification boundary

### Complexity contracts

Milestone 6 must add named complexity contracts for at least:

- correspondence validation
- promotion plan lowering
- lineage finalization
- historical resolution
- branch divergence summarization
- replay lineage digest comparison

Each contract must declare:

- complexity statement
- measured counters
- verified vs debt status
- proof test name

Historical-resolution contracts must also declare their boundedness basis
explicitly, such as:

- lineage-seed-bounded
- retained-window-bounded
- full-branch-reconstruction

The contract is incomplete if it states only "reconstruction boundary" without
also naming what determines reconstruction breadth.

## Phase 7: Promotion Planning vs Execution Separation

### Goal

Prevent promotion logic from collapsing planning, validation, and mutation into
one authority blob.

### Required split

- `candidate_validation.rs`
- structural validity only
- no mutation

- `promotion_planning.rs`
- consumes `ValidatedCorrespondenceCandidateSet`
- produces `PromotionEligibleCandidateSet` and then `LoweredPromotionPlan`
- resolves ambiguity, promotion class, and branch authority before execution

- `promotion_execution.rs`
- consumes `LoweredPromotionPlan` only
- emits promoted lineage events into the finalization batch
- no rediscovery of candidate legality or branch coherence

### Enforcement rules

- `promotion_execution.rs` must not accept raw candidate ids or loose lineage
  vectors.
- Any rejection possible before mutation must occur before plan lowering
  completes.
- If execution can still fail, failure must be operational, not semantic
  revalidation.

## Phase 8: Replay Equivalence and Lineage Sameness Contract

### Goal

Make lineage replay equivalence explicit rather than emergent.

### Required equivalence contract

Milestone 6 must define canonical lineage sameness in terms of:

- canonical event ordering
- canonical source ordering
- canonical target ordering
- canonical branch-scoped identity basis
- canonical promotion identity basis
- canonical digest input shape
- canonical decision-log ordering

### Required artifact/digest surfaces

The lineage replay path must define and use explicit digest bases for:

- lineage event batch
- promotion decision log
- historical resolution outputs where certified
- branch-scoped graph export where certified

### Rules

- Replay parity cannot rely on "events happened to compare equal."
- Digest sameness must be defined from canonicalized lineage artifacts.
- Historical resolution parity must compare canonical result ordering, not
  incidental traversal order.
- Any replay-visible lineage summary must state whether it is exact-digest
  based or summary-digest based.

## Phase 9: Certification and Domain Proof

### Goal

Close the milestone against the roadmap-required certification carriers with
machine-checkable outputs.

### Generic certification

`Lineage/correspondence hardening test`

Must verify:

- advisory correspondence never becomes authority without promotion
- invalid correspondence is rejected explicitly
- ambiguous parentage is rejected explicitly
- branch-local identity evolution stays branch-local
- historical ID resolution works only through legitimate authoritative lineage
- replay preserves lineage truth exactly

Required outputs:

- `lineage_graph_export`
- `correspondence_candidate_set`
- `authoritative_promotion_log`
- `rejected_invariant_report`
- `historical_resolution_matrix`
- `lineage_boundary_counter_snapshot`

### CAD domain certification

`Topology identity survival test`

Must verify:

- topological identity survives replace/split/rebuild semantics through lineage
  authority
- branch-local topology evolution remains isolated
- restore does not fabricate derivational lineage
- relation-history and topology identity remain historically queryable

Required outputs:

- `topology_truth_snapshot_bundle`
- `topology_lineage_ancestry_graphs`
- `topology_relation_history_report`
- `branch_local_topology_parity_matrix`
- `restore_vs_recompute_distinction_report`

### Chip domain certification

`Netlist rewiring identity and history test`

Must verify:

- rewiring/replacement/bus and hierarchy edits preserve authoritative
  identity/history semantics
- correspondence remains advisory until explicit promotion
- CDC and replay describe connectivity evolution faithfully
- branch-local rewiring histories remain isolated

Required outputs:

- `connectivity_truth_snapshot_bundle`
- `hierarchical_relation_graph_digest`
- `selected_net_cell_lineage_graphs`
- `correspondence_candidate_promotion_report`
- `cdc_connectivity_parity_report`
- `branch_local_connectivity_isolation_matrix`

### Replay certification

`Hostile commit/replay equivalence test`

Milestone 6 must extend this carrier so lineage-bearing histories verify:

- `lineage_digest`
- `historical_resolution_digest`
- `promotion_decision_digest`
- `branch_scoped_lineage_query_digest`

across:

- original authoritative run
- replay from canonical commit envelopes
- replay from checkpoint plus suffix envelopes
- durable rebuild from canonical artifacts

## Important Public Interfaces and Types

### New or strengthened public types

- branch-scoped lineage request/result types
- authoritative promotion outcome and rejection types
- canonical lineage artifact type at commit boundary
- lineage metrics/counter summary types
- replay lineage digest/equivalence basis types

### Public interface constraints

- no raw planner/validator types leave the lineage namespace
- no internal runtime lineage state becomes public
- no branch-agnostic historical resolution surface
- no convenience API that hides traversal/reconstruction cost
- no public type that merges storage identity and lineage identity into one
  semantic bucket

## Assumptions and Defaults

- Milestone 6 begins with structural split before behavior expansion.
- CAD and chip carriers are in-scope from the start, not follow-on proof.
- Branch locality must be mechanically encoded, not merely validated
  procedurally.
- Exactly one canonical lineage artifact is produced per commit finalization.
- Promotion planning and promotion execution are separate phases with separate
  types.
- Replay equivalence for lineage requires an explicit sameness contract, not
  implicit structural comparison.
- Complexity counters and proof contracts are mandatory milestone scope, not
  hardening-after-closeout.
