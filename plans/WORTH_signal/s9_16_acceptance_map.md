# S9.16 Acceptance Map

## Purpose

This document is the closeout map for `S9.16`.

It exists to prevent a familiar failure mode:

- implementation moves forward
- architecture language sounds right
- a few local tests pass
- but the milestone has no explicit acceptance owner, no named invariant, and no proof surface

`S9.16` is a performance-truth program. That means each sub-batch must name:

- the invariant being hardened
- the runtime counter, summary, or surface that proves it
- the tests or certification lanes that own it
- the negative-space condition that must stay impossible

This map does not replace the architecture plan in
[signal_architecture2.md](./signal_architecture2.md).
It is the implementation-side acceptance companion for that plan.

## Program Rules

- Every `S9.16.x` claim must be backed by named runtime surfaces, not narrative.
- Performance claims must prefer breadth/counter invariants over elapsed-time assertions.
- If a batch depends on future certification harness work, the gap must be named explicitly.
- Cold reconstruction may be exposed, but only through explicitly named materialization or reconstruction APIs.
- No acceptance row is complete until both success and negative-space behavior are covered.
- For work that closes `S9.9`, `S9.10`, `S9.12`, or `S9.15`, acceptance must also name the retired legacy surface, not only the new target surface.
- "Supported" means "representable through the public or owned internal execution path," not "documented as preferred."

## Implementation Control Addendum For `S9.9` / `S9.10` / `S9.12` / `S9.15`

These workstreams need tighter acceptance scaffolding than ordinary batch work
because they are substrate-replacement programs, not feature additions.

### Required Acceptance Questions

Every implementation batch touching one of these workstreams must answer all of
the following:

1. What legacy execution surface is being retired?
2. What proof-bearing type now owns admissibility?
3. What phase boundary now emits the named proof?
4. What counter proves the new path stayed bounded?
5. What exact invalid path is now impossible or rejected before construction?

If a batch cannot answer those five questions, it has not yet produced an
acceptance-quality architectural change.

### Required Migration Tracking Shape

Implementation notes or closeout notes for these workstreams must include a row
using this schema:

| Workstream | Legacy surface retired | Bridge surface remaining | Final proof-bearing surface | Counter surface | Negative-space proof |
| --- | --- | --- | --- | --- | --- |

### Required Closeout Impossibility Checks

The following must be tracked explicitly as negative-space acceptance items:

- reconstructability without bounded journal proof
- restore from snapshot-like bundles
- whole-live candidate scope on supported merge paths
- grouped concurrent apply that serializes semantic work under a parallel label
- worker writes to shared runtime surfaces
- execution beyond the configured hierarchical resource lease
- caller-asserted parallel safety or backend-minted semantic proof
- completion-order publication or worker-count-dependent deterministic output
- hidden WASM main-thread fallback or remote retry without idempotency identity
- rollback via baseline bundle semantics
- branch restore without reconstructability proof
- routine lifecycle access to heavy capture

## Acceptance Table

| Slice | Invariant | Proof surface | Current coverage | Negative space |
| --- | --- | --- | --- | --- |
| `S9.16.1` hot/cold separation | Hot operational paths run from `RuntimeArtifactState` alone | merge comparability, effect application write lanes, observer/runtime artifact access, reconstruction telemetry | `tests::merge_adoption::merge_branch_equivalent_runtime_state_ignores_retained_artifact_richness`; `tests::phase1_api::observer_exposes_runtime_and_retained_artifacts_separately`; `tests::observability::explicit_retained_and_reconstructed_artifact_apis_match_policy` | retained richness must not change merge/apply/runtime outcome |
| `S9.16.1` cold access honesty | Cold reconstruction is explicit by API name and policy result | `materialize_explanation_artifact`, `materialize_provenance_artifact`, `reconstruct_*`, `DiagnosticsAvailability`, hot-path reconstruction counter | `tests::observability::explicit_omit_policy_surfaces_unavailable_artifacts`; `tests::observability::explicit_retained_and_reconstructed_artifact_apis_match_policy`; `tests::harness_bridge::*artifact*` | ordinary observer/operational queries must not silently reconstruct broad cold views |
| `S9.16.2` shared snapshot storage | Snapshot sharing changes storage strategy, never semantic meaning | snapshot ids, snapshot artifact-retention metadata, explicit restore intents, canonical dependency snapshot update derivation, shared-node restore batch planning, restore breadth planning, dependency snapshot contracts, `SnapshotStorageStrategy`, `SnapshotDeltaRecord`, storage telemetry counters, restore behavior, checkpoint telemetry including restore breadth counters | `tests::phase1_api::shared_dependency_snapshot_reports_storage_sharing_without_implying_semantics`; `tests::phase1_api::snapshot_storage_telemetry_distinguishes_replacement_from_version_only_delta`; `tests::phase1_api::set_dep_snapshot_uses_version_only_delta_when_snapshot_shape_is_stable`; `tests::phase1_api::derive_dependency_snapshot_restore_batch_uses_version_only_delta_for_shared_shape`; `tests::phase5_state::snapshot_artifact_retention_policy_changes_richness_not_restore_truth`; `tests::phase5_state::branch_snapshot_records_explicit_artifact_retention_for_non_active_branches`; `tests::phase5_state::restore_snapshot_with_active_policy_prunes_cold_richness_without_changing_operational_truth`; `tests::phase5_state::restore_snapshot_rejects_seed_recomputation_intent_before_mutation`; `tests::phase5_state::snapshot_restore_plan_reports_shared_delta_and_coarse_requirements`; existing snapshot delta tests | pointer-sharing/backing reuse must not become identity or restore semantics |
| `S9.16.3` / Milestone 12 aspect-causal invalidation | Root mutations create unresolved recompute work; every resolved downstream aspect fact comes from the immediate dependency's atomically committed per-aspect/per-scope output delta | source recompute seeds, performed output-commit wrapper, producer-local dependency causes bound across every freshness axis, canonical pending cause store, shared pending/resolved condition input, named fintech semantic scenarios, `FreshFinancialRecompute`, `FinancialNecessityManifest`, `FinancialAspectCausalityCertificationRun` | implemented by the eight-scenario fintech causality courtroom and focused cause/condition/checkpoint/topology controls in [milestone-12-plan.md](./milestone-12-plan.md); transitive execution summaries are aspect-free and scope-free | root aspects, reachability, dirty masks, diagnostics, retained traces, stale dependency revisions, producer/consumer comparator conflation, consumer-global suppression, or generic graph-only certification must not mint or certify descendant aspect authority |
| `S9.16.3` / Milestone 13 locality-first invalidation | Invalidation breadth is bounded by realized semantic reach plus the declared indexed candidate/order granule, not all direct subscribers or reachable closure | Signal-owned Proof progression from admitted origins through executed work, current-basis work bindings, rebuildable producer/aspect/scope reverse index, six typed fintech world contracts with independent `Q/C/K/U/E/S/P` manifests, canonical ready batches, performed Signal counter receipts and executed-work observations, Foundational counter-backed evidence, three separate structural slope verdicts, same-work-stream strategy reports, `FinancialFrontierLocalityCertificationRun` | implemented and certified by [milestone-13-plan.md](./milestone-13-plan.md); reachability tests remain inherited safety evidence rather than locality proof | exact changes cannot scan disjoint direct buckets or visit disjoint descendants; root/prepared/stale facts cannot skip phases; the derived index and Foundational evidence cannot authorize Signal work; queried-candidate cost cannot be hidden as semantic work; conservative legacy scope unions, predicted counters, elapsed time, disconnected padding, or a deferred certification phase must not substitute for realized exact work |
| `S9.16.4` reuse contracts | Reuse occurs only through explicit equivalence contract and typed strategy/origin truth | `ArtifactEquivalenceContract`, `ReuseBoundaryContext`, `ReuseBasis`, `ReuseOrigin`, typed rejection taxonomy, retained certification proof count, lineage/replay/history reuse surfaces, cross-identity correspondence evidence, partial-splice composition provenance | `tests::phase3_semantics::defined_computation_evaluate_cross_identity_reuses_cached_result_via_public_api`; `tests::phase3_semantics::defined_computation_evaluate_partial_splice_uses_public_api`; `tests::phase3_semantics::cross_identity_lineage_and_history_preserve_correspondence_family`; `tests::phase3_semantics::branch_local_cross_identity_rejection_preserves_main_correspondence_and_lineage`; `tests::phase3_semantics::branch_local_partial_splice_rejection_preserves_main_mixed_provenance`; `tests::phase5_state::snapshot_restore_preserves_advanced_reuse_history_truth`; `tests::telemetry_contract::runtime_metrics_surface_exposes_typed_advanced_reuse_counts`; `tests::observability::ordinary_summary_and_history_reads_do_not_materialize_cold_artifacts`; `tests::diagnostics::diagnostics_history_and_replay_preserve_typed_advanced_reuse_origins` | reuse must not come from ad hoc field comparison, continuity-token coincidence, observer reconstruction, or retained-richness availability |
| `S9.16.5` diagnostics tiering | Tier changes richness, not semantics | `DiagnosticsTier`, `RetentionBudget`, `ReconstructionBudget`, `DiagnosticsAvailability`, replay/lineage boundaries, bounded reconstruction telemetry with retained/reconstructed/denied attribution | `tests::observability::artifact_access_counters_attribute_lane_api_and_denial_reason`; `tests::observability::ordinary_summary_surfaces_do_not_trigger_artifact_reconstruction`; `tests::observability::tier_matrix_public_observer_surfaces_preserve_truth_while_availability_changes`; `tests::observability::ordinary_observer_access_never_increments_cold_or_denial_counters_across_tiers`; `tests::observability::branch_and_snapshot_churn_respect_retention_budget_under_all_tiers`; `tests::observability::long_session_branch_churn_with_mixed_reads_keeps_bounds_and_cold_work_honest`; `tests::observability::ordinary_summary_and_history_rendering_respect_retained_detail_limits`; `tests::diagnostics::diagnostics_profiles_control_retention_bounds`; `tests::phase5_state::replay_and_lineage_overlap_stay_equivalent_across_runtime_policy_matrix`; existing branch/snapshot/history budget tests | lower tier must not trigger hidden broad reconstruction for ordinary access, exceed retained envelopes under churn, or rewrite canonical replay/lineage/reuse truth |
| `S9.16.6` / Milestones 12-13 integrated invalidation certification | Invalidation correctness and locality are certified by named financial scenarios during the implementation phase that establishes each guarantee | independent financial truth and necessity oracles, reproducible scenario cases, separate equivalence/cost verdicts, sealed causality/locality runs, typed strategy decision | implemented by [milestone-12-plan.md](./milestone-12-plan.md) and [milestone-13-plan.md](./milestone-13-plan.md) | no self-certifying oracle, generic-graph-only closeout, deferred certification, log scraping, elapsed-time-only claim, tree-algorithm assumption, or unmeasured default-strategy selection |
| `S9.17.1` / Query 9.17.6.3 Phase 1 execution authority | Parallel execution is resource-bounded, worker-local, cancellation-aware, and canonically published through the one `worth-execution` authority | execution capabilities, hierarchical resource lease, prepared batch, worker-local packet, canonical publication plan, typed execution outcome | planned by [Query Milestone 9.17.6.3](../WORTH-query/milestone-9.17.6.3.md); existing precompute/grouped-apply tests are inherited but do not prove lease enforcement | no oversubscription, per-request pools, hidden fallback, worker graph mutation, or completion-order publication |
| `S9.17.2` / Query 9.17.6.3 Phase 4 Signal graph parallelism and locality | Only settled, control-order-safe, mutation-disjoint graph work runs concurrently; candidate lookup follows the bounded `ScopePath` hierarchy | settled dependency set, control-order proof, complete graph mutation footprint, conflict groups, rewire proposals, epoch publication, `ScopePath` subscriber index | planned by [Query Milestone 9.17.6.3](../WORTH-query/milestone-9.17.6.3.md) | no stage-index-only safety, prior-graph rewiring assumption, lock-as-proof, unbounded ready queue, partial epoch, or candidate work from a sibling-disjoint subtree |
| `S9.17.3` / Query 9.17.6.3 Phase 2 structured partition work | Domain-neutral map/reduce/scan/fork-join/round/decompose work executes under stable partitions and the same hierarchical lease | stable work partitions, read/write sets, disjoint batch, deterministic reduction, scan, round, and decomposition plans, keyed/components/bisection partitioners | planned by [Query Milestone 9.17.6.3](../WORTH-query/milestone-9.17.6.3.md) | no domain/backend vocabulary, `.parallel_safe`, inner pool, worker-index identity, inferred floating associativity, or partial round |
| `S9.17.4` / Query 9.17.6.3 backend port | Serial and native execution consume the same prepared meaning through one backend port; wasm32 resolves and reports the serial posture | capability resolution, backend port, serial and native backends, serial oracle, reported execution posture | planned by [Query Milestone 9.17.6.3](../WORTH-query/milestone-9.17.6.3.md); WASM helper-worker, remote, and accelerator backends are [deferred](../deferred-work.md); existing WASM worker authority remains inherited | no closure/authority crossing the port, hidden fallback, serial fallback that changes partition or reduction semantics, or unearned device or distributed claim |

## Batch Tracking

### Substrate Completion Tracking

The following rows bind the completion spec to acceptance ownership.

| Workstream | Invariant | Required proof surface | Required counters | Negative space |
| --- | --- | --- | --- | --- |
| `S9.12` reconstructability completion | checkpoint + bounded journal + required derived rebuild is the only supported restore truth | `CheckpointBoundary`, `BoundedJournalSegment`, `RequiredDerivedRebuildSet`, `ReconstructabilityProof` | `journal_replay_span`, `journal_suffix_breadth`, `restore_authority_breadth`, `restore_required_derived_breadth`, `restore_diagnostic_richness_breadth` | no restore from snapshot bundle, no optional journal proof, no diagnostics-driven semantic rebuild |
| `S9.15` bounded merge completion | supported merge candidate construction is purely proof-driven and bounded | `MergeBoundaryWitness`, `StructuralMergeJournalSlice`, `ProofMinimalOverlapBasis`, `ConservativeOverlapExpansion`, `LoweredMergePlan` | `boundary_witness_kind`, `source_slice_breadth`, `proof_minimal_overlap_breadth`, `conservative_overlap_expansion_breadth`, `final_candidate_breadth`, `reconciliation_breadth` | no `MergeCandidateScope`, no whole-live supported candidate scope, no ambient branch-state discovery |
| `S9.9` true parallel apply completion | grouped concurrent apply is real on proof-safe static stages and all other full-parallel requests lower honestly to serial | `DisjointApplyProof`, `GroupLocalApplyPacket`, `ConcurrentApplyReductionPlan`, `LoweredApplyPlan` | `group_local_packet_breadth`, `reduction_packet_breadth`, `reduction_group_count`, `shared_surface_publication_breadth`, `parallel_admission_rejection_reason` | no fake `FullParallel`, no worker access to shared surfaces, no reduction-side semantic recomputation |
| `S9.17.1` bounded deterministic executor | all parallel and nested work consumes one strict lease and publishes canonically | `ExecutionResourceLease`, `PreparedExecutionBatch`, `WorkerLocalExecutionPacket`, `CanonicalPublicationPlan` | leased/active workers, queue width, steals, nested lease breadth, cancellations, fallbacks, publication breadth | no worker-count hint masquerading as a bound, no nested oversubscription, no schedule-derived truth |
| `S9.17.2` proof-carrying graph concurrency | graph work runs concurrently only with readiness, control-order, and complete mutation-footprint proof | `SettledDependencySet`, `ControlOrderProof`, `DisjointGraphBatch`, `OrderedConflictPartition`, `GraphEpochPublication` | work, span, critical path, conflict width, queue width, rewire proposals, epoch breadth | no topological-level-only admission, no direct shared mutation, no partial topology epoch |
| `S9.17.3` structured partition concurrency | inner-node work is declarative, domain-neutral, deterministic where requested, and resource-compositional | `StableWorkPartition`, `PartitionReadSet`, `PartitionWriteSet`, `DeterministicReductionPlan`, `SynchronousRoundPlan` | partitions, logical items, reductions, scans, barriers, rounds, bytes, peak memory | no raw spawn, backend selection, domain-specific core type, or silent reduction-order drift |
| `S9.17.4` backend port | execution capability is separate from semantic and disclosure authority; serial and native backends consume the same prepared work; returned work is readmitted before commit | resolved capability, backend port, serial oracle, reported `ExecutionPosture` | active workers, fallbacks with cause, serial-posture reports; transfer, retry, and recovery counters return with the [deferred](../deferred-work.md) remote and WASM helper-worker backends | no graph authority on workers, hidden fallback, or fake external boundary |
| `S9.10` rollback and lifecycle completion | rollback is effect-derived and lifecycle transfer is type-separated and cost-honest | `TransactionRollbackPacket`, `AuthorityTransferPacket`, `RestoreTransferPacket`, `ExplicitBranchForkPacket`, `HeavyCaptureWitness`, `BranchLifecycleTransfer` | `rollback_packet_breadth`, `rollback_packet_count_by_subsystem`, `move_transfer_count`, `explicit_fork_count`, `restore_transfer_count`, `heavy_capture_count` | no baseline-bundle rollback truth, no implicit duplication on branch switch, no raw branch-bundle restore, no routine witness construction |

### `S9.16.1` Current Batch

Targeted invariant:

- hot/cold artifact separation is real in storage lanes, write lanes, and hot-path comparability

Landed proof surfaces:

- merge comparability consumes only runtime artifact state
- effect application uses `ArtifactWriteDelta`
- merge adoption uses `ArtifactWriteDelta`
- observer exposes explicit cold assembly helpers
- cold reconstruction-capable access is named `materialize_*` instead of masquerading as a neutral getter

Named tests:

- `tests::phase1_api::observer_exposes_runtime_and_retained_artifacts_separately`
- `tests::merge_adoption::merge_branch_equivalent_runtime_state_ignores_retained_artifact_richness`
- `tests::observability::explicit_omit_policy_surfaces_unavailable_artifacts`
- `tests::observability::explicit_retained_and_reconstructed_artifact_apis_match_policy`
- `tests::harness_bridge::operational_profile_reconstructs_rich_artifacts_without_retaining_facts`

Still open for `S9.16.1`:

- audit remaining history/reuse-certification reads to ensure they are cold-only or explicitly materialized
- reduce any remaining production dependence on history-shaped setters or convenience reconstruction
- decide whether cold observational summaries need stricter naming split beyond current materialization helpers

### `S9.16.2` Pre-acceptance conditions

Before `S9.16.2` can close, it must prove:

- narrow dependency changes do not require whole snapshot clones
- retention policy changes richness only
- restore and merge semantics remain stable under shared snapshot backing

The proof must come from runtime snapshot counters and semantic equality tests, not storage anecdotes.

Current starting proof:

- `DependencySnapshot::shares_storage_with(...)` and
  `SharedDependencySnapshot::shares_storage_with(...)` make storage sharing
  explicit on the type surface
- `DependencySnapshotUpdate::storage_strategy()` distinguishes shared
  replacement from version-only delta updates
- `tests::phase1_api::shared_dependency_snapshot_reports_storage_sharing_without_implying_semantics`
  proves that shared backing is an explicit storage fact rather than an implied
  semantic identity claim
- storage telemetry now distinguishes full shared replacement boundaries from
  version-only delta boundaries, which gives later certification harness work a
  canonical measurement surface instead of log inference
- snapshot capture now records explicit explanation/provenance retention in
  `SignalSnapshotMeta.artifact_retention`, and
  `tests::phase5_state::snapshot_artifact_retention_policy_changes_richness_not_restore_truth`
  proves that snapshot richness can change while restore truth stays stable
- restore now consumes explicit `SnapshotRestoreIntent`, so “rewind active
  state,” “restore captured richness,” and “apply active runtime policy after
  restore” are no longer bundled implicitly behind one restore helper
- dependency snapshot rewrites now derive a canonical narrow update form from
  explicit previous/next snapshot contracts, which lets stable-shape rewrites
  narrow to version-only delta updates instead of defaulting to shared replacement
- restore now also has a shared-node dependency snapshot batch planning surface,
  which gives future in-place restore work a canonical delta packet instead of
  forcing it to rediscover snapshot differences from whole graph state
- restore can now emit a proof-bearing `SnapshotRestorePlan` that distinguishes
  shared-node dependency snapshot delta work from the coarse replacement reasons
  that still remain, which keeps restore-breadth honesty explicit before the
  in-place restore engine exists
- restore execution now also records shared-delta breadth and coarse-reason
  counts into canonical checkpoint telemetry, so certification can consume
  runtime counters directly instead of relying only on the planning API

### `S9.16.3` Pre-acceptance conditions

Before `S9.16.3` can close, Milestone 12 must prove:

- the existing fintech world is expanded rather than replaced by a generic
  graph-only closeout fixture
- the financial world begins from an immutable authoritative market/position
  definition with typed instruments, ownership, subscriptions, and fixed-point
  values; financial arithmetic never treats `AspectVersion` as an amount
- a causally complete baseline owns exact producer contracts and established
  dependency snapshots before a scenario mutation begins
- aspects remain producer-local across every dependency hop
- root mutation declarations remain unresolved until the source commits
- an `A -> B` intermediate translation admits a downstream
  `AspectFilter(B)` node and rejects its unmatched twin
- unresolved dependency reachability cannot masquerade as a mismatched aspect
- unchanged producer output retains its prior dependency-visible semantic
  version and emits no downstream semantic delta
- one producer delta is admitted independently by consumers with different
  dependency comparator policies
- producer output equivalence and consumer dependency comparison are separate
  configuration and authority lanes, with deterministic legacy
  output-identity upgrade evidence
- installed conditional dependency/output comparator roles remain distinct
  through lowering and runtime resolution
- multiple dependencies retain distinct causal identity
- aspect and scope correlation survives multiple producer commits while a
  consumer remains gated
- same-shaped dependency removal and recreation invalidates the earlier
  dependency revision's causes
- pending causes survive rollback, branch, checkpoint, restore, replay, and
  async composition through one canonical cause store
- incremental committed results equal an epistemically independent full
  recomputation oracle across conditions, partitions, rewiring, branch restore,
  replay, and async-capability composition
- the required quote/risk translation, heterogeneous-comparator, suppression,
  factor-collision, curve-bucket, gate-release, dependency-rewire, and
  branch/replay scenarios each own an independent necessity manifest and
  mutation probe
- Phase 1 proves the inherited named red control; every later phase passes the
  focused or financial evidence assigned to the authority it establishes, and
  the sealed causality run rejects missing, duplicate, stale, or mismatched
  evidence

Milestone 13 must then prove:

- sparse-book, partition-universe, convergent-factor, dense-market-close,
  dependency-churn, and branch/replay locality scenarios at their named lanes
- source obligations, performed dependency commits, and performed structural
  mutations enter only through their separate admitted owner forms
- each downstream dependency hop begins with the immediate producer's performed
  output commit; reachability and batch summaries cannot seed dependency work
- source-admitted, prepared-direct, committed-direct, structural-admitted,
  resolved, lowered, ready, and executed work are distinct compiler-visible
  owner forms using `worth-proof`; invalid or skipped progression is uncallable
- source, dependency-commit, and structural origins converge only after their
  own authority validates a current basis and remain distinguishable in
  execution evidence
- executable work is bound to graph instance, target node, dependency revision,
  origin generation/dependency commit ordinals, readiness epoch, and stage/order, with
  one-axis drift denial
- ready work is derived and cannot survive restore/rebind as current authority
- aspect- and partition-disjoint subscribers are rejected before enqueue
- exact changes retrieve direct candidates through producer-local aspect and
  partition/detail buckets; disjoint buckets are not scanned
- the reverse index rebuilds from authoritative topology and an index hit never
  substitutes for edge/cause admission
- each world freezes independent `Q/C/K/U/E/S/P` expectations plus exact
  admission, pre-execution, and settled observation points
- index-disjoint growth has exact zero hot-work delta; queried-bucket rejecting
  growth has exact candidate/rejection delta and zero semantic-work delta;
  semantic-frontier growth matches the exact necessity delta
- disconnected padding, aggregate fanout slopes, and wall-clock thresholds do
  not satisfy those structural verdicts
- multiple justifications collapse to one canonical work item without losing
  per-dependency cause
- entry-level direct-dirty vs maybe-stale truth, cycle preflight behavior,
  determinism, and tracing-policy independence remain preserved
- predicted and realized counters are named and tested separately
- public reachability-shaped plans/transitive-wave constructors are replaced by
  read-only predicted estimates and performed execution receipts; no alias
  preserves the former authority shape
- Signal owns observed rows, the independent financial locality manifest owns
  expectations, and `worth-foundational` canonicalizes/attaches evidence
  without acquiring operational authority
- correctness and locality verdicts remain distinct, same-work-stream strategy
  comparisons are honest, and the sealed locality run rejects incomplete proof

Reachability-only tests are inherited safety evidence, not locality proof. The
closeout proof must come from the Milestone 12 financial truth and necessity
oracles, Milestone 13 realized counters/summaries, named scenario mutation
probes, and scale-sensitive sparse/dense financial workloads. It is produced
during Milestones 12 and 13, not by a successor certification milestone.

## Relationship To Existing Certification Docs

This map should be read alongside:

- `worth_signal_adversarial_testing_matrix.md`
- `worth_signal_fintech_certification_matrix.md`
- `worth_harness_workflow_certification_design.md`

Those docs define certification philosophy and domain workflow expectations.
This map binds `S9.16` implementation batches to concrete acceptance ownership inside `worth-signal`.

For `S9.9`, `S9.10`, `S9.12`, and `S9.15`, this map should be read together
with [s9_missing_substrate_completion.md](./s9_missing_substrate_completion.md).
That document defines the proof chain and migration discipline; this map defines
the acceptance owner, counters, and negative-space obligations.
