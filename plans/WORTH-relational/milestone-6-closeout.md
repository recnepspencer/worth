# Milestone 6 Closeout: Lineage and Correspondence Completion

## Status

Milestone 6 is closed as of 2026-03-23.

The runtime now treats identity evolution as authoritative truth rather than
as a side graph, advisory annotation, or replay-time reconstruction trick.

The semantic center shipped in this milestone is:

lineage authority is decided exactly once through a proof-typed promotion and
finalization pipeline, published as one canonical lineage artifact per commit,
and then consumed by history, replay, durability, inspection, certification,
and domain workflows as derived truth.

This is not "lineage exists" in the ordinary product sense. The runtime now
owns:

- explicit lineage event and decision artifacts
- advisory-to-authoritative correspondence promotion
- branch-local identity evolution as a mechanical boundary
- typed authority phase transitions from recorded candidate to published
  artifact
- canonical lineage digest bases and replay sameness rules
- explicit historical-resolution and graph-traversal surfaces with visible
  work basis
- durability and replay verification over lineage-bearing canonical artifacts
- lineage-bearing certification workflows across generic, CAD, chip, and
  fintech carriers

## Shipped Scope

Milestone 6 delivered:

- a structural split of lineage into responsibility-aligned data, access,
  authority, and certification modules
- explicit lineage domain vocabularies for:
  - events
  - correspondence candidates and outcomes
  - branch-scoped graph snapshots
  - historical-resolution requests, traces, and digest bases
  - finalization, publication, and checkpoint artifacts
- a proof-typed authority chain covering:
  - recorded correspondence candidate
  - validated candidate
  - promotion-eligible candidate
  - lowered promotion plan
  - execution-authorized promotion plan
  - finalized lineage event batch
  - lineage finalization artifact
  - published lineage artifact
- advisory correspondence that remains non-authoritative until explicit
  promotion and fail-closed rejection for invalid, stale, ambiguous, or
  branch-incoherent promotion attempts
- branch-local lineage resolution, branch-local promotion, and branch-local
  divergence surfaces that do not accept raw cross-branch truth combinations
- one canonical lineage artifact produced at commit finalization and attached
  into canonical commit envelopes, replay, durability, and inspection
- lineage decision-log query surfaces by candidate id, event id, and rejection
  class
- cost-honest read APIs for:
  - record lookup
  - graph snapshot materialization
  - branch divergence summarization
  - historical lineage resolution
  - lineage-aware aspect history composition
- explicit lineage digest-basis and authority-basis types consumed by replay,
  recovery, and certification
- durability checkpoints and recovery validation that preserve and verify
  lineage artifact basis rather than treating lineage as loosely packed raw
  vectors
- lineage-specific measurement boundaries and complexity counters for planning,
  finalization, publication, historical resolution, divergence, graph
  materialization, and replay parity
- domain certification carriers proving lineage truth through:
  - generic correspondence hardening
  - CAD topology identity survival
  - chip/netlist rewiring identity history
  - lineage-bearing fintech workflow certification

Before closeout, the implementation also removed or tightened several paths
that would have undermined milestone honesty if left in place:

- the remaining guarded lineage/replay `.expect()` paths were removed from
  production authority/replay execution
- public proof-like digest/result bags were sealed so callers can no longer
  WORTH "proof happened" surfaces without passing through the proving path
- `CorrespondenceResolution` and `CorrespondencePromotionOutcome` were
  tightened so impossible state combinations cannot be assembled as public
  option bags
- historical lineage resolution no longer scans full branch event history for
  every query; the runtime now maintains a branch-local source-lineage event
  index and traverses only reachable events
- replay lineage parity no longer trusts stale stored digest metadata; it
  compares canonical observed lineage artifact basis directly
- certified fintech workflows no longer "happen to replay"; they now include
  explicit lineage-bearing promotion before replay capture and require lineage
  authority to be present
- lineage access/runtime bypasses and public raw authority artifact fields were
  sealed behind faÃ§ade-owned access surfaces and read-only accessors

## Phase Completion Map

Milestone 6 is considered closed against the engineering plan because each
phase now has a concrete implementation surface rather than only a named goal.

### Phase 1: Structural Split and Canonical Vocabulary

Closed by:

- `src/lineage/data/*`
- `src/lineage/logic/access/*`
- `src/lineage/logic/authority/*`
- `src/lineage/facade.rs`
- `src/tests/lineage/*`

What is proven:

- lineage responsibilities are split by phase/domain rather than collected
  into monolithic lineage files
- faÃ§ade and module visibility now enforce the intended public surface
- lineage data modules expose named concepts rather than generic manager/helper
  blobs

### Phase 2: Typed Authority Phase Chain

Closed by:

- `src/lineage/logic/authority/phase_types.rs`
- `src/lineage/logic/authority/candidate_validation.rs`
- `src/lineage/logic/authority/promotion_planning.rs`
- `src/lineage/logic/authority/promotion_execution.rs`
- `src/lineage/logic/authority/commit_finalization.rs`
- `src/lineage/logic/authority/promotion_commit.rs`

What is proven:

- later lineage authority phases cannot be entered from weaker inputs
- execution consumes only plan-shaped proof types, not raw candidate ids or
  loose lineage vectors
- finalization and publication consume canonical lineage artifact inputs
  rather than rediscovering legality from runtime state

### Phase 3: Branch Locality as Mechanical Boundary

Closed by:

- branch-scoped lineage request/result types
- branch-scoped promotion and resolution APIs
- `tests::lineage::branch_locality::historical_lineage_resolution_is_branch_local_under_divergent_replacements`
- `tests::history::replay::replay_contract_preserves_branch_local_relation_integrity_truth_after_rejected_feature_attempt`

What is proven:

- branch-local lineage evolution stays branch-local
- cross-branch truth is not representable as ordinary promotion input
- historical lineage resolution is always explicit about branch scope

### Phase 4: Canonical Artifact and Derived Surface Rules

Closed by:

- `LineageFinalizationArtifact`
- `PublishedLineageArtifact`
- checkpoint lineage artifacts and basis validation
- commit inspection lineage projection from artifact basis

What is proven:

- exactly one canonical lineage artifact is produced per commit finalization
- history, replay, durability, and inspection derive from that artifact rather
  than parallel reinterpretation of raw event vectors
- the lineage decision log is an authority artifact and can answer why a
  promotion succeeded, failed, or remained advisory without rerunning the
  planner

### Phase 5: Public API and Cost-Honest Read Surfaces

Closed by:

- explicit graph, divergence, and historical-resolution request/result types
- graph/divergence/historical-resolution breadth metrics and digest bases
- `tests::lineage::resolution::*`
- `tests::lineage::branch_locality::*`

What is proven:

- traversal-heavy lineage APIs are named as traversal/materialization
  boundaries rather than cheap getters
- read surfaces expose boundedness basis and work breadth
- historical resolution is lineage-seed-bounded and now mechanically follows
  reachable events rather than scanning unrelated branch history

### Phase 6: Measurement Boundaries and Complexity Contracts

Closed by:

- `tests::complexity::contracts::lineage_budgets::*`
- lineage counters in performance instrumentation

What is proven:

- candidate validation, promotion planning, finalization, publication,
  graph materialization, divergence, historical resolution, and replay parity
  all have named counters and proof lanes
- historical resolution explicitly reports lineage-seed-bounded work
- publication and replay lineage breadth are visible and testable rather than
  assumed

### Phase 7: Promotion Planning vs Execution Separation

Closed by:

- `tests::lineage::promotion_validation::*`
- `tests::lineage::promotion_execution::*`

What is proven:

- validation, planning, and execution are distinct phase boundaries
- semantic rejection occurs before execution
- execution failures after planning are operational failures, not hidden
  revalidation paths

### Phase 8: Replay Equivalence and Lineage Sameness

Closed by:

- replay lineage authority basis types
- lineage event batch and decision-log digest bases
- replay parity tests over durable log, checkpoint basis, and history fallback

What is proven:

- replay-visible lineage sameness is defined from canonical lineage artifacts
  and canonical digest inputs
- deep replay parity uses artifact comparison bases rather than accidental
  equality of raw collections
- audit mode and normal mode remain explicit about exact-vs-fallback lineage
  authority

### Phase 9: Certification and Domain Proof

Closed by:

- generic lineage/correspondence certification
- CAD topology identity survival
- chip/netlist rewiring identity history
- lineage-bearing fintech workflow certification
- hostile replay/recovery equivalence lanes

What is proven:

- authoritative lineage survives replay, checkpoint+suffix replay, and durable
  rebuild
- domain workflows exercise lineage truth instead of relying on replay-only
  business state
- certification surfaces now prove lineage authority presence or absence
  honestly

## Acceptance Mapping

Milestone 6 is considered closed against the roadmap and plan because the
required certification surfaces are now directly covered by code, metrics, and
machine-checkable outputs.

### `Lineage/correspondence hardening test`

Covered by:

- `tests::lineage::certification::lineage_correspondence_hardening_tracks_advisory_promotion_and_rejection_artifacts`
- `tests::lineage::candidate_recording::lineage_candidate_recording_try_promote_returns_rejected_resolution`
- `tests::lineage::promotion_validation::lineage_promotion_validation_invalid_references_do_not_promote`
- `tests::lineage::promotion_validation::lineage_promotion_validation_rejects_commit_branch_mismatch`
- `tests::lineage::promotion_validation::lineage_promotion_validation_rejects_stale_commit_anchor`
- `tests::lineage::promotion_validation::lineage_promotion_validation_resolves_anchor_truth_from_history_not_caller_shape`
- `tests::lineage::promotion_execution::lineage_promotion_execution_stays_advisory_until_promoted`
- `tests::lineage::promotion_execution::lineage_promotion_execution_reports_operational_anchor_drift_after_plan_lowering`
- `tests::lineage::branch_locality::historical_lineage_resolution_is_branch_local_under_divergent_replacements`
- `tests::lineage::resolution::historical_lineage_resolution_follows_replace_events`
- `tests::lineage::resolution::historical_lineage_resolution_does_not_scan_unrelated_branch_events`

What is proven:

- advisory correspondence never becomes authority without promotion
- invalid, stale, ambiguous, and branch-incoherent candidates fail explicitly
- promotion authority resolves anchor truth from history rather than trusting
  caller-shaped inputs
- branch-local identity evolution stays branch-local
- historical ID resolution follows legitimate authoritative lineage only
- historical resolution work is honest and no longer scans unrelated branch
  history

Required machine-checkable outputs are now explicitly owned by tests:

- `lineage_graph_export`
- `correspondence_candidate_set`
- `authoritative_promotion_log`
- `rejected_invariant_report`
- `historical_resolution_matrix`
- `lineage_boundary_counter_snapshot`

### `Topology identity survival test`

Covered by:

- `tests::domains::cad::topology_identity_survival::topology_identity_survival_preserves_reidentification_truth_across_recovery`

What is proven:

- topological identity survives replace/update workflows through authoritative
  lineage rather than storage-id continuity
- historical lineage resolution and lineage-aware aspect history remain
  queryable after recovery
- recovery does not fabricate derivational lineage
- recovered commit truth remains identical to authoritative pre-recovery truth

Required machine-checkable outputs are now explicitly owned by tests:

- `topology_truth_snapshot_bundle`
- `topology_lineage_ancestry_graphs`
- `topology_relation_history_report`
- `branch_local_topology_parity_matrix`
- `restore_vs_recompute_distinction_report`

### `Netlist rewiring identity and history test`

Covered by:

- `tests::domains::chip::netlist_rewiring_identity_history::netlist_rewiring_identity_history_preserves_exact_lineage_truth`

What is proven:

- rewiring/replacement identity remains authoritative and replay-stable
- correspondence stays advisory until explicit promotion
- replay exposes exact canonical lineage digest authority for promoted
  lineage-bearing commits
- checkpoint/recovery preserves the canonical promotion envelope exactly

Required machine-checkable outputs are now explicitly owned by tests:

- `connectivity_truth_snapshot_bundle`
- `hierarchical_relation_graph_digest`
- `selected_net_cell_lineage_graphs`
- `correspondence_candidate_promotion_report`
- `cdc_connectivity_parity_report`
- `branch_local_connectivity_isolation_matrix`

### `Hostile commit/replay equivalence test`

Covered by:

- `tests::history::replay::hostile_commit_replay_equivalence_test`
- `tests::history::replay::replay_and_recovery_preserve_aspect_bearing_truth_across_a_hostile_mixed_workload`
- `tests::history::replay::replay_contract_preserves_metadata_only_promotion_commit_truth_and_recovery`
- `tests::history::replay::replay_contract_reports_lineage_event_drift_at_digest_layer_when_artifacts_are_tampered`
- `tests::history::replay::replay_contract_reports_lineage_decision_log_drift_at_digest_layer_when_artifacts_are_tampered`
- `tests::history::replay::replay_contract_uses_checkpoint_canonical_basis_in_audit_mode_when_durable_log_tail_is_absent`
- `tests::history::replay::replay_contract_rejects_history_envelope_fallback_basis_in_audit_mode`

What is proven:

- canonical lineage artifacts are sufficient to reproduce lineage-bearing truth
  across original execution, replay, checkpoint+suffix replay, and durable
  rebuild
- replay-visible lineage sameness is defined from canonical event/decision
  digest bases rather than accidental equality
- tampering with lineage events or decision logs is caught explicitly at the
  replay parity boundary
- normal-mode history fallback and audit-mode authoritative-basis requirements
  remain distinct and explicit

Required machine-checkable outputs are now explicitly owned by tests:

- `lineage_digest`
- `historical_resolution_digest`
- `promotion_decision_digest`
- `branch_scoped_lineage_query_digest`

## Performance QA and Hardening

Milestone 6 closeout explicitly audited the implementation against
[docs/coding-guidelines/performance_guidelines.md](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\docs\coding-guidelines\performance_guidelines.md),
not just against semantic requirements.

The closeout-standard claims now backed by code and tests are:

- candidate validation, promotion planning, finalization, publication, graph
  materialization, divergence, historical resolution, and replay parity all
  have named lineage counters
- historical resolution is lineage-seed-bounded and no longer performs a full
  branch sweep on the hot path
- graph and divergence APIs expose their breadth explicitly and do not disguise
  full-branch work as cheap getters
- replay lineage parity is artifact-shaped and basis-aware rather than relying
  on raw vector lengths or ambient runtime hints
- checkpoint lineage basis and recovery validation remain canonical and
  deterministic across durable rebuilds

The closeout performance carrier is:

- `tests::complexity::contracts::lineage_budgets::*`

That carrier proves together that:

- historical resolution work matches its reported boundedness basis
- graph snapshots expose node/event/candidate materialization breadth
- divergence reports event/node breadth honestly
- candidate validation and promotion planning report candidate widths
- finalization/publication report lineage artifact width
- replay lineage parity reports authority-basis selection and digest width

## Compiler and Type Enforcement Added Before Close

Milestone 6 closeout also converted several previously procedural guarantees
into stronger type-shaped or constructor-shaped surfaces:

- proof-bearing authority phase types are sealed inside lineage authority
  modules
- `ExecutionAuthorizedPromotionPlan` now separates branch-head authorization
  from earlier lowering phases
- replay lineage authority, graph digest basis, historical-resolution digest
  basis, and finalization/publication digest basis types are sealed and exposed
  through read-only accessors rather than public proof-looking bags
- `CorrespondenceResolution` and `CorrespondencePromotionOutcome` are now
  stage-shaped surfaces instead of public option bags or type aliases
- important lineage artifact/data fields were tightened from public bags to
  accessor-shaped authority/read surfaces
- `LineageAccess` no longer exposes crate-wide runtime bypass through its read
  surface
- lineage faÃ§ade and data re-exports are now explicit rather than blind glob
  leaks

These changes matter for Milestone 6 because they make "the type must encode
what has been proven" true across the lineage authority chain and its derived
surfaces, not only inside the planner.

## Additional Hardening Added Before Close

Milestone 6 closeout also includes these extra hardening lanes beyond the bare
milestone headings:

- replay authority now fails explicitly instead of panicking if replayed
  envelopes or promised lineage surfaces are missing
- replay digesting uses deterministic streaming SHA-256 instead of
  panic-backed temporary serialization
- durability checkpoints now validate canonical lineage artifact basis before
  recovery mutates runtime truth
- commit inspection exposes lineage digest basis and artifact counters from the
  canonical artifact rather than reinterpreting raw lineage slices
- domain certification workflows now require lineage-bearing promotion where
  lineage authority is part of the certified contract
- event deduplication, divergence intersection, graph materialization, and
  decision-log canonical sorting were tightened so performance claims better
  match actual implementation

The closeout expectation here was not "lineage is queryable enough to demo."
It was authoritative identity evolution with proof-typed promotion,
artifact-derived replay/durability/inspection surfaces, branch-local
enforcement, and certification-grade domain proof.

## Explicit Deferrals

Milestone 6 intentionally does not claim ownership of:

- arbitrary cross-branch lineage reconciliation or merge-time identity
  semantics beyond reserved future reconciliation types
- domain-complete CAD merge semantics
- domain-complete chip/netlist merge or reconciliation semantics
- user-facing lineage product tooling beyond the runtime, certification, and
  faÃ§ade surfaces shipped here
- custom invariant families and domain-specific structural invariant authoring
  beyond what Milestone 6 depends on from earlier milestones

Milestone 6 does guarantee the prerequisite foundation those later milestones
must consume:

- authoritative lineage/correspondence promotion semantics
- branch-local lineage isolation
- canonical lineage finalization/publication artifacts
- replay-/durability-authoritative lineage verification
- cost-honest lineage read surfaces
- proof-typed phase transitions through the lineage authority lifecycle

## Verification Baseline

At closeout, the crate verification baseline is:

- `cargo test -p worth-relational -- --nocapture`
- 436 tests enumerated for `worth-relational`

That baseline includes:

- named lineage certification suites
- upgraded hostile replay/recovery parity lanes for lineage-bearing histories
- domain certification carriers for CAD, chip, and fintech flows
- complexity-contract proof lanes for historical resolution, graph breadth,
  promotion planning, finalization/publication, and replay lineage parity
- compile-fail and visibility enforcement coverage around the proof-shaped
  phase boundaries

## Operational Conclusion

Milestone 6 can be treated as closed.

The runtime now has one authoritative lineage architecture for:

- correspondence recording, validation, planning, and promotion
- canonical lineage event finalization and publication
- branch-local graph and historical-resolution reads
- replay lineage parity and digest authority
- durability checkpoint and recovery lineage verification
- lineage-bearing domain certification
- cost-honest lineage counters and proof tests

The next work should build on this foundation rather than reopen it. Lineage is
no longer advisory bookkeeping or replay theater; it is a runtime-owned truth
system with certification-grade authority, replay, recovery, and domain proof.
