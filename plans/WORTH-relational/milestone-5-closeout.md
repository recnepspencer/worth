# Milestone 5 Closeout: Honest Schema Continuity

## Status

Milestone 5 is closed as of 2026-03-22.

The runtime now treats schema evolution, subscriber continuity, replay/recovery
verification, and schema reconciliation as authoritative runtime truth rather
than host choreography, version folklore, or best-effort compatibility
guesswork.

The semantic center shipped in this milestone is:

the runtime itself understands what kind of continuity is honest, classifies
that continuity explicitly, publishes canonical transition artifacts, and
requires CDC, replay, recovery, and reconciliation to consume those artifacts
as authority.

This is not "version awareness" in the ordinary application sense. The runtime
now owns:

- schema strata
- historical interpretation sensitivity
- continuation outcome classification
- canonical bridge/reconciliation descriptors
- descriptor-versioned replay/recovery verification
- subscriber contract-aware continuation
- fail-closed transition and reconciliation rejection

## Shipped Scope

Milestone 5 delivered:

- explicit schema strata covering:
  - structural shape
  - value domain
  - entity identity semantics
  - correspondence semantics
  - lineage semantics
  - behavioral semantics
  - publication contract
  - subscriber contract
- structured schema diff atoms with authoritative strata, publication impact,
  subscriber impact, and historical-interpretation sensitivity
- explicit continuity layers instead of one overloaded compatibility bucket:
  - reconciliation classification
  - continuation classification
  - bridgeability
  - historical interpretation sensitivity
- canonical schema transition artifacts, continuation descriptors,
  reconciliation descriptors, and boundary fingerprints
- descriptor semantics versioning and canonicalization-version authority
- commit-time schema continuity validation, lowering, publication, diagnostics,
  and fail-closed conflict classes
- replay- and durability-side descriptor-first verification with typed
  verification layers and explicit verification modes
- subscriber contract-aware continuation assessment with:
  - `ContinueUnchanged`
  - `ContinueWithTransparentBridge`
  - `ContinueWithVisibleBridge`
  - `ContinueWithContractUpgrade`
  - `RequireRenegotiation`
  - `Rejected`
- normalized continuation-proof composition with an enforced complexity ceiling
- replay-authoritative lineage and derived-index artifacts carried in canonical
  commit envelopes rather than reference-only envelope hints
- structured diagnostics and counters for schema classification, continuation
  decisions, replay/recovery verification layers, reconciliation policy, and
  descriptor-version drift
- named Milestone 5 certification suites plus upgraded hostile CDC/replay/
  durability carriers with schema-transition-bearing histories

Before closeout, the implementation also removed or tightened several paths
that would have undermined milestone honesty if left in place:

- generic compatibility is no longer an authority-bearing decision concept and
  remains only as descriptive summary metadata
- type-incompatible schema transitions now force rejected continuation rather
  than slipping through as merely subscriber-impact-shaped changes
- schema boundary fingerprints are canonical across diff ordering and now use
  an explicit 256-bit authority surface rather than a casual scalar hash
- free-form schema diff detail strings were replaced with structured
  `SchemaDiffDetail` variants
- recovery no longer stores redundant `verification_mode` and
  `verification_plan` state that could drift; the plan is authoritative and the
  mode is derived
- recovery verification counters are now preserved onto the runtime that
  survives recovery instead of being silently lost at the rebuild boundary
- test-only preparation fault injection no longer leaks across parallel worker
  threads; injected faults are carried through planned packets rather than read
  from shared global state inside worker execution

## Acceptance Mapping

Milestone 5 is considered closed against the roadmap and plan because the
required certification surfaces are now directly covered by code, diagnostics,
and machine-checkable outputs.

### `Schema evolution CDC contract test`

Covered by:

- `tests::schema::certification::schema_evolution_cdc_contract_test`
- `tests::publication::cdc::certification::cdc_certification_schema_boundary_continuation_is_explained_and_counted`
- `tests::publication::cdc::certification::diff_cdc_truth_parity_test`
- `tests::publication::cdc::subscriber_resume::subscriber_stream_mixed_boundaries_choose_strongest_supported_outcome_and_trace_each_boundary`
- `tests::publication::cdc::subscriber_resume::resumed_subscriber_stream_mixed_boundaries_choose_strongest_supported_outcome`
- `tests::publication::cdc::subscriber_failures::subscriber_stream_rejects_renegotiation_required_boundary`

What is proven:

- schema transition boundaries are first-class canonical artifacts in commit,
  CDC, replay, and durability paths
- long-running subscribers can cross harmless additive boundaries without
  renegotiation when the declared contract honestly allows it
- visible bridge boundaries are surfaced explicitly and remain semantically
  correct even if the subscriber ignores the metadata
- contract-upgrade boundaries succeed only when upgrade support was declared
- renegotiation-required boundaries fail explicitly and diagnostically
- live execution, replay, checkpoint+suffix recovery, and durable rebuild all
  preserve the same schema-transition and subscriber-continuation truth

Required machine-checkable outputs are now explicitly owned by tests:

- `schema_transition_digest`
- `schema_boundary_cdc_digest`
- `subscriber_contract_matrix`
- `transition_decision_digest`
- `descriptor_version_digest`

### `Schema reconciliation classification test`

Covered by:

- `tests::schema::certification::schema_reconciliation_classification_test`
- `tests::schema::continuity::type_incompatible_schema_transition_is_rejected_not_continued`
- `tests::schema::continuity::declared_type_incompatible_schema_transition_reports_specific_conflict_class`
- `tests::history::replay::replay_contract_reports_schema_lineage_drift_at_summary_layer_when_digest_is_unavailable`

What is proven:

- additive reconciliation is classified deterministically
- narrowing requires explicit preservation policy
- policy naming and behavior are preservation-based rather than vague
- type-incompatible and structural-incompatible transitions fail closed
- resulting lineage and reconciliation descriptors are deterministic and
  replay-stable
- descriptor-version and lineage authority survive replay and recovery

Required machine-checkable outputs are now explicitly owned by tests:

- `schema_reconciliation_digest`
- `schema_lineage_digest`
- `reconciliation_policy_matrix`
- `schema_conflict_localization_report`
- `reconciliation_replay_digest`
- `descriptor_version_digest`

### `Diff/CDC truth parity test`

Covered by:

- `tests::publication::cdc::certification::diff_cdc_truth_parity_test`
- `tests::publication::cdc::replay_parity::subscriber_stream_matches_patch_stream_for_committed_history`
- `tests::publication::cdc::recovery::subscriber_stream_recovers_from_durable_canonical_envelopes_when_checkpoint_is_not_in_memory`

What is proven:

- commit-native diffs and subscriber CDC remain parity-aligned across
  schema-transition-bearing histories
- continuation diagnostics and continuation counters are part of the certified
  truth surface, not incidental debug residue
- durable recovery preserves exact CDC suffix truth under schema boundaries

Required machine-checkable outputs are now explicitly owned by tests:

- `diff_digest`
- `cdc_digest`
- `cdc_diagnostics_digest`
- `continuation_counter_snapshot`

### `Hostile commit/replay equivalence test`

Covered by:

- `tests::history::replay::hostile_commit_replay_equivalence_test`
- `tests::history::replay::replay_and_recovery_preserve_aspect_bearing_truth_across_a_hostile_mixed_workload`
- `tests::history::replay::replay_certification_audit_drift_is_explained_and_counted`

What is proven:

- canonical commit artifacts are sufficient to reconstruct observable truth
  across schema-bearing hostile histories
- replay verification is layered, descriptor-first, and explicit about where
  drift was detected
- lineage, derived-index, patch, diagnostics, branch-head, and query surfaces
  all remain replay-authoritative rather than reference-only hints
- audit replay deepens verification without leaking rich-path cost into normal
  replay mode

Required machine-checkable outputs are now explicitly owned by tests:

- `truth_digest`
- `patch_digest`
- `lineage_digest`
- `replay_digest`
- `diagnostics_digest`
- `branch_heads_digest`
- `query_surface_digest`

### `Durable recovery and schema mismatch test`

Covered by:

- `tests::durability::contracts::durable_recovery_and_schema_mismatch_test`
- `tests::durability::contracts::durability_certification_recovery_compatibility_is_explained_and_counted`
- `tests::durability::contracts::durability_recovery_plan_reports_descriptor_version_mismatch_before_recovery`
- `tests::durability::contracts::durability_contract_failure_schema_mismatch_is_explicit`

What is proven:

- durable recovery consumes persisted continuity artifacts directly rather than
  rediscovering schema meaning from raw state
- schema mismatch, descriptor-version drift, lineage drift, and continuity
  bundle defects fail explicitly before recovery mutates truth
- compatibility diagnostics and verification-layer counters are part of the
  durable certification surface
- recovered runtimes retain the fact that they were verified

Required machine-checkable outputs are now explicitly owned by tests:

- `recovery_schema_bundle_digest`
- `recovery_compatibility_diagnostic_digest`
- `mismatch_failure_digest`

## Performance QA and Hardening

Milestone 5 closeout explicitly audited the implementation against
[docs/coding-guidelines/performance_guidelines.md](C:\Users\Esther\Documents\Programming\WORTH_workspace\WORTH\docs\coding-guidelines\performance_guidelines.md),
not just against semantic requirements.

The closeout-standard claims now backed by code and tests are:

- execution breadth remains bounded by changed schema atoms and crossed
  continuity boundaries rather than total registry or history breadth
- continuation decisions are planned and lowered once, then consumed as
  descriptors rather than rediscovered from raw schema state
- normal replay and recovery remain digest-first and do not inherit deep-path
  recomputation cost by default
- subscriber continuation proof growth is bounded structurally rather than
  merely observed after the fact
- replay-authoritative lineage and derived-index artifacts are carried forward
  structurally instead of rederived heuristically
- no broad test-only fault injection state is allowed to leak into unrelated
  concurrent worker execution

The closeout performance carrier is:

- `tests::complexity::contracts::commit_budgets::complexity_budget_milestone5_closeout_keeps_schema_cdc_and_recovery_boundary_local`

That carrier proves together that:

- schema transition classification is changed-atom bounded
- CDC continuation work stays boundary local
- normal recovery verification stays at digest parity on the hot path
- deep-path verification does not leak into the normal recovery lane

## Compiler and Type Enforcement Added Before Close

Milestone 5 closeout also converted several previously procedural guarantees
into stronger type-shaped or constructor-shaped surfaces:

- `ValidatedSchemaContinuityBundle` is the shared authority surface for commit,
  replay, and durability continuity validation
- `RecoveryVerificationPlan` is now authoritative and
  `RecoveryPlan::verification_mode()` derives from it rather than duplicating
  mutable state
- `RecoveryCompatibilityCheck::verified_at(...)` and `RecoveryPlan::new(...)`
  replace ad hoc raw struct assembly for recovery verification state
- `SubscriberContinuationAssessment::new(...)`,
  `SubscriberBoundaryAssessment::new(...)`, and
  `SubscriberRecoveryPlan::new(...)` replace raw CDC recovery field packs
- planned invariant packets now carry injected test faults directly, which
  removes a process-global side channel from worker execution

These changes matter for Milestone 5 because they turn "please keep these
parallel fields consistent" into "the compiler helps us keep these truth
surfaces coherent."

## Additional Hardening Added Before Close

Milestone 5 closeout also includes these extra hardening lanes beyond the bare
milestone headings:

- explicit rejection-side schema continuity diagnostics with per-diff-atom
  traces, not only success-side classification artifacts
- canonical bundle publication validation that rejects impossible continuity
  combinations before commit publication
- typed replay verification-layer reporting on both success and failure paths
- explicit inspection exposure for replay-authoritative lineage and
  derived-index artifacts
- CDC failure diagnostics that carry contract-capability details and
  per-boundary reasoning
- checkpoint consistency validation so subscriber checkpoints cannot drift from
  their own summarized continuity proof
- full-suite parallel fault-injection hardening so hostile certification lanes
  do not corrupt unrelated tests under concurrency

The closeout expectation here was not "schema-aware enough to demo." It was
runtime-authoritative continuity with cost-honest diagnostics, strict replay/
recovery truth, and enough compiler-shaped structure to make backsliding
harder.

## Explicit Deferrals

Milestone 5 intentionally does not claim ownership of:

- domain-complete CAD semantic rule systems
- domain-complete chip/netlist reconciliation semantics
- authoritative merge execution for arbitrary divergent truths
- host-defined schema DSLs beyond the runtime-owned schema continuity model
- product-level workload certification beyond the Milestone 5 relational
  certification carriers

Those remain deferred to later roadmap and domain milestones, especially
merge/correspondence work and domain-specific certification expansion.

Milestone 5 does guarantee the prerequisite foundation those later milestones
must consume:

- authoritative schema transition truth
- subscriber contract-aware continuation truth
- replay-/durability-authoritative continuity verification
- explicit historical-interpretation guardrails
- deterministic reconciliation classification and lineage artifacts
- performance-honest continuity execution and diagnostics

## Verification Baseline

At closeout, the crate verification baseline is:

- `cargo test -p worth-relational -- --nocapture`
- 385 tests passing

That baseline includes:

- named Milestone 5 schema certification suites
- upgraded CDC/replay/durability certification carriers with
  schema-transition-bearing histories
- complexity-contract proof lanes for schema continuity, subscriber
  continuation, and replay/recovery verification layering
- hostile observability and fault-injection suites that remain stable under
  full-suite execution

## Operational Conclusion

Milestone 5 can be treated as closed.

The runtime now has one authoritative schema continuity architecture for:

- schema transition classification
- bridge and reconciliation descriptor construction
- subscriber continuation
- replay verification
- durable recovery verification
- lineage/index continuity artifacts
- structured diagnostics and performance counters

The next work should build on this foundation rather than reopen it. Schema
continuity is no longer a planning fiction or host convention; it is a
runtime-owned truth system with certification-grade replay, recovery, and CDC
honesty.
