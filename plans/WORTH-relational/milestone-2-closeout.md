# Milestone 2 Closeout: Relational Aspect Semantics

## Status

Milestone 2 is closed as of 2026-03-19.

The runtime now treats aspects as canonical truth-layer semantics owned by
schema, lowered plans, commit-time delta computation, and durable artifacts,
rather than as payload-derived labels or projection-era convenience metadata.

## Shipped Scope

Milestone 2 delivered:

- schema-owned aspect declarations for entity and relation kinds
- deterministic per-kind `AspectPlanRevision` fingerprints
- build-time lowering into `LoweredAspectPlan`
- commit-time canonical aspect truth via `CanonicalRecordAspectDelta`
- durable patch encoding of changed aspects, structural classification, and
  degraded-precision state
- commit summaries, traces, reports, and diagnostics derived from canonical
  aspect truth
- record-local and lineage-aware aspect history surfaces driven from durable
  committed truth
- projection/query surfaces that consume declared aspect truth through
  `AspectKey`, not `ProjectionAspect`
- aspect-aware filters using `RequestedAspectSet` rather than reusing emitted
  canonical aspect sets
- crate-local DX support for aspect-heavy fixture building, invariant helpers,
  and truth-flow guidance

Before closeout, the implementation also removed two quiet semantic leaks that
would have undermined the milestone if left in place:

- mutation-side aspect-version sidecars were rewritten to consume canonical
  changed-aspect truth instead of payload-key scans
- visibility-side aspect reads stopped reconstructing aspect meaning from
  payload shape and now expose declared aspect contract or committed
  aspect-version truth directly

## Acceptance Mapping

Milestone 2 is considered closed against the roadmap because the required
acceptance surfaces are now either directly covered or explicitly deferred to
later roadmap milestones without weakening Milestone 2's truth claims.

### `Diff/CDC truth parity test`

Covered by:

- `tests::transactions::core::entity_patch_aspects_follow_declared_semantics_not_payload_keys`
- `tests::transactions::core::retained_relation_patch_only_emits_declared_lifecycle_delta_when_endpoints_and_payload_stay_same`
- `tests::publication::cdc::savepoint_residue::nested_savepoint_abandoned_aspect_work_leaves_zero_patch_cdc_history_and_lineage_residue`
- `tests::history::replay::replay_and_recovery_preserve_aspect_bearing_truth_across_a_hostile_mixed_workload`

What is proven:

- committed patch aspect sets equal canonical aspect truth
- lifecycle-only relation deltas do not inflate payload-era aspect meaning
- abandoned savepoint work leaves zero aspect residue in patch, CDC, history, or
  lineage-aware history
- recovered/replayed aspect-bearing histories stay aligned with canonical patch
  truth

### `Bulk query and traversal stress truth test`

Covered by:

- `tests::history::queries::bulk_like_aspect_history_filters_and_query_packets_stay_stable_after_recovery`
- `tests::query::projections::projection_rejects_undeclared_required_aspects`
- `tests::query::entity_scans::entity_kind_scans_can_be_partition_scoped_without_cross_partition_leakage`
- `tests::query::relation_scans::relation_kind_scans_return_only_visible_relations_of_that_kind`
- complexity visibility budget lanes under `tests::complexity::contracts::visibility_budgets`

What is proven:

- aspect-aware query/history consumers remain canonical under recovery
- projection/query surfaces fail closed on undeclared aspect requirements
- partition-scoped and historical scans stay within the promised visibility
  boundaries
- bulk-like aspect-filter consumption does not need payload rescans or a second
  aspect interpreter

### `Hostile commit/replay equivalence test`

Covered by:

- `tests::history::replay::replay_contract_success_reproduces_canonical_surfaces`
- `tests::history::replay::replay_and_recovery_preserve_aspect_bearing_truth_across_a_hostile_mixed_workload`
- `tests::durability::contracts::durability_contract_recovery_preserves_aspect_bearing_patch_truth_and_history`
- `tests::publication::observability::aspect_traces_and_diagnostics_are_stable_across_supported_execution_models`

What is proven:

- aspect-bearing committed truth survives replay and durable recovery
- canonical aspect patches, histories, and traces stay equivalent across
  equivalent histories
- supported execution modes do not introduce a second aspect-semantic outcome

### `Topology identity survival test`

Covered by:

- `tests::profiles::compiled_artifacts::compiled_artifact_rejects_stale_topology_after_later_commit`
- `tests::profiles::compiled_artifacts::chip_profile_branch_local_topology_pressure_preserves_relation_history_isolation`

What is proven:

- topology-adjacent truth remains branch-local and historically inspectable
- later commits cannot silently invalidate earlier topology-derived committed
  truth
- relation history isolation remains honest under branch-local structural
  pressure

### `Netlist rewiring identity and history test`

Covered in Milestone 2 only to the extent that the milestone honestly owns the
prerequisite truth semantics:

- `tests::profiles::compiled_artifacts::chip_profile_declared_aspect_fanout_preserves_endpoint_history_for_netlist_like_shapes`
- `tests::query::relation_scans::relation_aspects_at_version_follow_declared_contract_not_payload_shape`
- `tests::transactions::core::visibility_aspect_versions_follow_canonical_delta_truth_and_ignore_undeclared_fields`

What is proven:

- endpoint-bound relation aspects are declared, lowered, committed, published,
  and historically inspectable as part of aspect truth
- netlist-like endpoint fanout shapes do not collapse back into payload-era
  labeling
- visibility/history helper surfaces no longer re-derive endpoint aspect meaning
  from payload shape

What is explicitly not claimed by Milestone 2:

- first-class rewiring as an authoritative mutation/reconciliation capability
- domain-complete hostile rewiring certification

That capability is deferred explicitly to
[worth_relational_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/worth_relational_roadmap.md)
`Milestone 7B: Authoritative Merge Execution`.

## Additional Hardening Added Before Close

Milestone 2 closeout also includes these extra hardening lanes and QA outcomes
beyond the bare roadmap labels:

- aspect-bearing durable recovery and `AspectPlanRevision` mismatch failure
- nested savepoint fracture with explicit zero aspect residue across patch,
  CDC, history, and lineage
- commit-side aspect evaluation/emission trace publication
- schema declaration and lowering traces wired into diagnostics
- visibility-side generation safety for aspect-version slot reads
- separation between emitted canonical aspect truth and caller-supplied
  requested aspect filters
- reusable aspect-truth fixture builders, digest helpers, and invariant helpers

The closeout expectation here was certification-grade robustness, not mere API
presence.

## Explicit Deferrals

Milestone 2 intentionally does not claim ownership of full first-class rewiring
or merge/reconciliation semantics.

Those are now explicitly deferred to
[worth_relational_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/worth_relational_roadmap.md)
`Milestone 7B: Authoritative Merge Execution`, including:

- relation endpoint rewiring as a first-class authoritative capability
- richer merge/reconciliation semantics for aspect-bearing records
- persistent identity matching across branch-local non-identical record ids
- domain-grade netlist rewiring identity/history certification

Milestone 2 still guarantees the prerequisite truth foundation that later work
must consume:

- endpoint-bound aspect declaration and lowering
- endpoint-bound canonical commit-time delta computation
- durable publication/history encoding of endpoint-bound aspect truth for
  supported relation mutations
- history/query/diagnostic consumption of that truth without payload rescans

One additional closeout item remains only partially hardened rather than fully
deferred:

- byte-for-byte stability of all aspect trace and diagnostics artifacts under
  every legal scheduling variation

The current observability lanes are strong enough to close Milestone 2
honestly, but further scheduling-hostility tightening still belongs in later
certification work.

## Verification Baseline

At closeout, the crate verification baseline is:

- `cargo test -p worth-relational --lib`
- 249 tests passing

That baseline includes the adversarial aspect-history, replay, recovery,
savepoint, observability, durability, and domain-pressure lanes added during
Milestone 2 closeout.

## Operational Conclusion

Milestone 2 can be treated as closed.

The runtime now has one authoritative aspect-delta engine with downstream
durable/history/query/diagnostic consumers, and the known rewiring-oriented
gaps are explicitly owned by later roadmap milestones rather than left as soft
ambiguity.

The next product milestone is
[worth_relational_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/worth_relational_roadmap.md)
`Milestone 3: Structural Identity, Introspection, and Historical Inspection`.
