# Milestone 3 Closeout: Provenance-Preserving Inspection

## Status

Milestone 3 is closed as of 2026-03-19.

The runtime now treats structural identity, graph introspection, recent
mutation inspection, retention/reclaim inspection, and historical inspection as
first-class truth capabilities over canonical subsystem artifacts rather than as
ad hoc debug surfaces or convenience views.

The semantic center shipped in this milestone is:

inspection is read-only composition over canonical subsystem truths, and public
inspection results preserve origin, access path, resolution context, and
availability honestly.

That contract is semantic rather than mechanically uniform. Narrow
single-origin reads may use lighter specialized result types, but composed
inspection outputs now preserve provenance and availability explicitly instead
of flattening multiple truth families into one ambiguous payload.

## Shipped Scope

Milestone 3 delivered:

- a public `inspection` faÃ§ade namespace with read-only
  `RelationalRuntime::inspection_access(&self)` entry
- an explicit inspection truth contract built around origin, access path,
  resolution context, availability, and narrow degradation semantics
- removal of the misleading mutable retention observer path in favor of
  explicit `retention_authority(&mut self)` authority naming
- structural identity evidence and comparison surfaces that preserve permanent
  separation between storage identity, lineage identity, and structural
  evidence
- request-shaped, scope-explicit graph introspection over graph summaries,
  kind summaries, connectivity summaries, and neighbor inspection
- explicit recent-commit, branch-head, and transaction-staging inspection
  surfaces
- branch-explicit historical inspection with retained-only and canonical
  reconstruction modes
- historical record inspection that keeps record truth, lineage context, aspect
  history, structural evidence, and retention availability independently
  truthful
- explicit retention inspection surfaces for lifecycle state, pin state,
  reclaim eligibility, and historical availability
- complexity-contract and certification lanes for inspection cost honesty,
  replay/recovery parity, branch-locality, reclaim correctness, and hostile
  provenance preservation

Before closeout, the implementation also removed several quiet semantic leaks
that would have undermined the milestone if left in place:

- historical inspection in `RetainedOnly` mode no longer leaks structural
  evidence by implicitly reconstructing through version reads after record truth
  correctly fails closed
- structural identity no longer reads sidecars through reused current slots for
  historical or stale entity ids
- recent-commit inspection no longer double-counts inspection work in
  complexity reporting
- branch-head pin maintenance and live-history trimming now respect the same
  branch-local retention truth instead of drifting under trimming pressure
- test fault-injection helpers no longer corrupt later certification runs under
  parallel execution

## Acceptance Mapping

Milestone 3 is considered closed against the roadmap because the required
acceptance surfaces are now either directly covered or explicitly deferred to
later roadmap milestones without weakening Milestone 3's truth claims.

### `Hostile commit/replay equivalence test`

Covered by:

- `tests::history::replay::replay_contract_success_reproduces_canonical_surfaces`
- `tests::inspection::inspection_truth_bundle_recovery_parity_holds_for_current_and_historical_surfaces`
- `tests::durability::contracts::durability_contract_recovery_preserves_inspection_truth_bundle`
- `tests::publication::observability::harness_phase8_fault_injection_soak_does_not_corrupt_following_certification_runs`

What is proven:

- inspection surfaces remain replay-equivalent across live, recovered, and
  canonical reconstruction paths
- current, historical, and branch-local inspection outputs preserve the same
  truth contract after durable recovery
- certification/fault-injection lanes do not create scheduler-shaped inspection
  semantics

### `Snapshot pinning and reclaim correctness test`

Covered by:

- `tests::durability::retention::retention_plan_reports_snapshot_pinned_records_before_release`
- `tests::durability::retention::retention_plan_turns_deleted_records_reclaimable_after_snapshot_release`
- `tests::durability::retention::retention_plan_reports_branch_pinned_deleted_records_when_sibling_branch_lags`
- `tests::durability::retention::retention_inspection_reports_exact_branch_pin_counts_for_lagging_deleted_records`
- `tests::durability::contracts::durability_contract_recovery_rebuilds_branch_pinned_retention_from_branch_heads`
- `tests::inspection::historical_inspection_stays_branch_local_under_divergence_and_reclaim_pressure`

What is proven:

- retention inspection distinguishes lifecycle state, pin state, reclaim
  eligibility, and historical availability
- branch-local lag and snapshot pinning keep the correct records retained
  without hidden authority repair during reads
- recovered runtimes rebuild branch-pin truth from canonical branch heads rather
  than drifting from live retention state

### `Lineage/correspondence hardening test`

Covered by:

- `tests::inspection::structural_identity_comparison_only_uses_fingerprint_truth`
- `tests::inspection::structural_identity_comparison_distinguishes_equal_mismatch_and_family_mismatch`
- `tests::inspection::structural_identity_historical_scope_does_not_leak_reused_slot_sidecars`
- `tests::inspection::historical_record_inspection_keeps_subresults_separate_when_retained_only_blocks_record_truth`
- `tests::lineage::historical_resolution::historical_lineage_resolution_is_branch_local_under_divergent_replacements`

What is proven:

- structural identity never promotes storage identity or lineage continuity into
  structural sameness
- branch-local lineage resolution remains explanatory context rather than a
  substitute for record truth or structural evidence
- historical inspection sub-results degrade independently instead of collapsing
  unavailable record truth into a vague composed answer

### `Bulk query and traversal stress truth test`

Covered by:

- `tests::inspection::graph_summary_is_scope_explicit_and_canonical`
- `tests::complexity::contracts::inspection_budgets::complexity_budget_graph_summary_reports_explicit_inspection_work`
- `tests::complexity::contracts::inspection_budgets::complexity_budget_structural_identity_distinguishes_direct_lookup_from_broad_query`
- `tests::complexity::contracts::inspection_budgets::complexity_budget_kind_summary_reports_request_shaped_scope`
- `tests::complexity::contracts::inspection_budgets::complexity_budget_connectivity_summary_reports_broad_traversal_work_explicitly`
- `tests::complexity::contracts::inspection_budgets::complexity_budget_neighbor_inspection_uses_adjacency_not_relation_materialization`
- `tests::complexity::contracts::inspection_budgets::complexity_budget_commit_inspection_reads_are_index_explicit_and_bounded`

What is proven:

- graph, kind, connectivity, structural-identity, and recent-commit inspection
  surfaces are request-shaped and scope-explicit rather than fake-cheap getters
- broad traversal work is measured and surfaced honestly
- current-scope fast paths do not need full read-view materialization for the
  narrowed hot inspection cases

### `Topology identity survival test`

Covered by:

- `tests::profiles::compiled_artifacts::compiled_artifact_rejects_stale_topology_after_later_commit`
- `tests::profiles::compiled_artifacts::chip_profile_branch_local_topology_pressure_preserves_relation_history_isolation`
- `tests::inspection::historical_relation_inspection_reconstructs_record_truth_without_inventing_lineage`
- `tests::inspection::current_graph_surfaces_match_version_and_snapshot_scopes_for_same_truth`

What is proven:

- topology-adjacent inspection remains historically inspectable and branch-local
- later commits do not silently rewrite earlier topology-bearing truth
- relation historical reconstruction does not invent lineage or collapse
  topology truth into current storage identity

### `Netlist rewiring identity and history test`

Covered in Milestone 3 to the extent that the milestone honestly owns the
required inspection and provenance substrate:

- `tests::inspection::historical_inspection_matrix_keeps_entity_and_relation_subresults_honest_across_modes`
- `tests::inspection::historical_relation_inspection_keeps_direct_commit_history_when_retained_only_blocks_record_truth`
- `tests::inspection::recent_commit_inspection_and_branch_head_reads_stay_branch_local`
- `tests::profiles::compiled_artifacts::chip_profile_branch_local_topology_pressure_preserves_relation_history_isolation`

What is proven:

- branch-local relation/history inspection remains truthful under retained-only
  and reconstructed historical modes
- recent mutation and branch-head inspection stay envelope-driven and
  branch-scoped rather than becoming a second event feed
- netlist-like relation histories remain inspectable without implying merge or
  rewiring reconciliation semantics the runtime does not yet own

What is explicitly not claimed by Milestone 3:

- first-class authoritative rewiring or reconciliation execution
- merge-time identity matching across branch-local non-identical record ids
- domain-complete rewiring certification beyond truthful inspection of committed
  and historical artifacts

That capability remains deferred to
[worth_relational_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/worth_relational_roadmap.md)
`Milestone 7B: Authoritative Merge Execution`.

## Additional Hardening Added Before Close

Milestone 3 closeout also includes these extra hardening lanes and QA outcomes
beyond the bare roadmap labels:

- cross-surface parity for `Current`, `Version(current_version)`, and
  `Snapshot` inspection of the same truth
- inspection-truth-bundle parity across live, durable recovery, and historical
  reconstruction paths
- structural identity hostile coverage for missing fingerprint, family mismatch,
  entity vs relation evidence, branch divergence, slot reuse, and recovery
- branch-explicit historical inspection under divergence plus reclaim pressure
- transaction inspection proofs that staged topology and lineage-affecting
  intents never preview committed graph/history truth
- merge-commit and recent-commit inspection proofs that commit inspection stays
  canonical-envelope projected rather than becoming a synthetic story layer
- current-scope graph and connectivity fast paths with complexity counters that
  prove zero full-record materialization on the narrowed hot paths
- shared test-only fault-injection locking and poison recovery so hostile
  certification lanes remain stable under parallel execution

The closeout expectation here was certification-grade robustness, not mere API
presence.

## Explicit Deferrals

Milestone 3 intentionally does not claim ownership of merge execution,
authoritative correspondence promotion completion, or rewiring reconciliation.

Those remain deferred to later roadmap milestones, including
[worth_relational_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/worth_relational_roadmap.md)
`Milestone 6: Correspondence and Merge Foundations` and
[worth_relational_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/worth_relational_roadmap.md)
`Milestone 7B: Authoritative Merge Execution`, including:

- authoritative promotion of correspondence into merged identity
- merge reconciliation semantics for divergent committed truths
- first-class rewiring as an authoritative mutation capability
- domain-complete merge/reconciliation certification for topology and netlist
  workloads

Milestone 3 still guarantees the prerequisite truth foundation that later work
must consume:

- observer-only composition over canonical subsystem truths
- explicit provenance and availability boundaries for composed inspection
  results
- structural identity as evidence rather than authority
- branch-local and replay-equivalent historical inspection
- explicit retention/reclaim truth products
- cost-honest graph and recent-mutation inspection surfaces

## Verification Baseline

At closeout, the crate verification baseline is:

- `cargo test -p worth-relational --lib`
- 282 tests passing

That baseline includes the hostile inspection, replay, recovery, retention,
reclaim, topology-pressure, complexity-contract, and certification harness
lanes added during Milestone 3 closeout.

## Operational Conclusion

Milestone 3 can be treated as closed.

The runtime now has one inspection truth architecture for structural identity,
graph introspection, recent mutation, retention/reclaim truth, and historical
inspection, with explicit provenance and availability boundaries and no hidden
mutation during observer flows.

The next product milestone is
[worth_relational_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/worth_relational_roadmap.md)
`Milestone 4: Relation Integrity and Schema Contracts`.
