# P-C merge cleanup reds

Observed at e787b51b3e; retained under Delivery tracks C1/C2/C4. These85 cases include3 audited incomplete campaigns. Do not infer final-source green or rerun coverage.

## `worth-court-relational-certification::worth_court_relational_certification$content::durable_publication::content_durable_artifact_cancellation_after_original_allocation_never_calls_medium`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text

thread 'content::durable_publication::content_durable_artifact_cancellation_after_original_allocation_never_calls_medium' (38868) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\content\durable_publication.rs:255:5:
assertion `left == right` failed
  left: 1559
 right: 0
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-court-relational-certification::worth_court_relational_certification$content::durable_publication::content_durable_artifact_memory_refuses_before_the_medium_and_selected_commit`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text

thread 'content::durable_publication::content_durable_artifact_memory_refuses_before_the_medium_and_selected_commit' (68808) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\content\durable_publication.rs:182:5:
assertion `left == right` failed
  left: 1559
 right: 0
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-court-relational-certification::worth_court_relational_certification$content::durable_publication::content_durable_artifact_original_work_exhaustion_is_exact_and_pre_effect`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text

thread 'content::durable_publication::content_durable_artifact_original_work_exhaustion_is_exact_and_pre_effect' (40124) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\content\durable_publication.rs:316:5:
assertion `left == right` failed
  left: Some(67107305)
 right: Some(67108864)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-court-relational-certification::worth_court_relational_certification$content::historical_kind::custody::original_kind_custody_survives_successor_basis_checkpoint_tail_and_reopen`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text

thread 'content::historical_kind::custody::original_kind_custody_survives_successor_basis_checkpoint_tail_and_reopen' (46140) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\content\historical_kind\custody.rs:74:5:
assertion `left == right` failed
  left: 34066
 right: 8576
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-court-relational-certification::worth_court_relational_certification$content::historical_kind::funding::historical_kind_reconstruction_preserves_literal_original_stops_and_releases_scratch`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text

thread 'content::historical_kind::funding::historical_kind_reconstruction_preserves_literal_original_stops_and_releases_scratch' (76884) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\content\historical_kind\funding.rs:70:36:
called `Result::unwrap()` on an `Err` value: Execution { denial: CommitExecutionDenial { kind: Cause(WorkExhausted), partition_identity: None }, context: ErrorContext { subsystem: Transaction, operation: Commit, affected_records: [], version_context: None, suggested_fix: None }, commit_log: CommitLog { events: [PhaseStarted(DraftPreparation), StructuralSummary, PhaseCompleted(DraftPreparation), PhaseStarted(InvariantPreCheck), InvariantEvaluated { execution_point: CommitBoundary, observation_kind: Committed, disposition: Executed, preparation_selected_mode: None, preparation_parallel_legality: None, preparation_parallel_profitability: None, preparation_serial_selection_reason: None, consumed_groups: InvariantGroupSet { mask: 377 }, applicable_groups: InvariantGroupSet { mask: 337 }, result_count: 0, advisory_count: 0, violation_count: 0, blocking_violation: false, publication_violation: false }, PhaseCompleted(InvariantPreCheck), PhaseStarted(AuthoritativeMutation), InvariantEvaluated { execution_point: MutationSensitive, observation_kind: Speculative, disposition: Executed, preparation_selected_mode: Some(Serial), preparation_parallel_legality: Some(RequiresSerial), preparation_parallel_profitability: Some(NotProfitable), preparation_serial_selection_reason: Some(NoLease), consumed_groups: InvariantGroupSet { mask: 317 }, applicable_groups: InvariantGroupSet { mask: 277 }, result_count: 2, advisory_count: 0, violation_count: 0, blocking_violation: false, publication_violation: false }, PhaseCompleted(AuthoritativeMutation), PhaseStarted(HistoryResolution), HistoryResolved, PhaseCompleted(HistoryResolution), PhaseStarted(InvariantPostCheck), InvariantEvaluated { execution_point: SnapshotPublication, observation_kind: Speculative, disposition: Executed, preparation_selected_mode: None, preparation_parallel_legality: None, preparation_parallel_profitability: None, preparation_serial_selection_reason: None, consumed_groups: InvariantGroupSet { mask: 66 }, applicable_groups: InvariantGroupSet { mask: 66 }, result_count: 0, advisory_count: 0, violation_count: 0, blocking_violation: false, publication_violation: false }, PhaseCompleted(InvariantPostCheck), PhaseStarted(ArtifactAssembly), PatchBudgetEvaluated, CommitRejected { phase: ArtifactAssembly, diagnostic_code: None, publication_stage: None, detail: "commit preparation execution stopped: CommitExecutionDenial { kind: Cause(WorkExhausted), partition_identity: None }" }], running_summary: CommitSummary { phase_count: 6, structural_summary: Some(CommitStructuralSummary { invariant_groups: InvariantGroupSet { mask: 343 }, commit_topology: GraphMutation, touched_partitions: [PartitionId(7)], bulk_entity_slots_reserved: 0, bulk_relation_slots_reserved: 0, descriptor_semantics_version: DescriptorSemanticsVersion(1) }), history_summary: Some(CommitHistorySummary { target_branch: "main", requested_merge_parent_count: 0, effective_merge_parent_count: 0, parent_count: 1, merge_base_count: 0, had_previous_branch_head: true }), patch_budget_summary: Some(CommitPatchBudgetSummary { patch_record_count: 1, max_patch_records_per_commit: None }), change_summary: None, aspect_summary: None, publication_summary: None, invariant_result_count: 2, invariant_advisory_count: 0, invariant_violation_count: 0 } } }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-court-relational-certification::worth_court_relational_certification$relational::validated_proposal_memory::unfunded_validated_proposal_backing_refuses_and_releases_original_custody`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text

thread 'relational::validated_proposal_memory::unfunded_validated_proposal_backing_refuses_and_releases_original_custody' (81328) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\relational\validated_proposal_memory.rs:61:5:
CommitExecutionDenial { kind: Cause(PolicyMemoryExhausted { requested: 4000, admitted: 1280, ancestor: 0 }), partition_identity: None }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-relational::worth_relational$authority::commit::pipeline::request_checkpoints::work_account::one_work_account_bounds_each_entry_with_its_original_canonical_evidence`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text

thread 'authority::commit::pipeline::request_checkpoints::work_account::one_work_account_bounds_each_entry_with_its_original_canonical_evidence' (908) panicked at crates\worth-relational\src\authority\commit\pipeline\request_checkpoints\work_account.rs:148:9:
assertion `left == right` failed: Ordinary: every original SDK charge settles into the same account
  left: 8059
 right: 8595
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-relational::worth_relational$authority::commit::pipeline::request_checkpoints::work_account::one_work_account_bounds_the_same_proposal_on_every_entry`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text
native commit work invoice: actual=8595 settled=8059

thread 'authority::commit::pipeline::request_checkpoints::work_account::one_work_account_bounds_the_same_proposal_on_every_entry' (79204) panicked at crates\worth-relational\src\authority\commit\pipeline\request_checkpoints\work_account.rs:175:5:
assertion `left == right` failed: the authoritative account settles all SDK work
  left: 8059
 right: 8595
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-relational::worth_relational$durability::log::native_checkpoint_codec::kind_authentication_tests::forged_cross_region_kind_commitment_is_refused_before_member_reads`

CLEANUP-malformed-input-boundary; owner: root owning court / recorded cleanup.

The failing assertion compares ordering or typed malformed-input diagnostics in a fault court.

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text

thread 'durability::log::native_checkpoint_codec::kind_authentication_tests::forged_cross_region_kind_commitment_is_refused_before_member_reads' (54632) panicked at crates\worth-relational\src\durability\log\native_checkpoint_codec\kind_authentication_tests.rs:155:5:
assertion `left == right` failed
  left: 1
 right: 0
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-relational::worth_relational$history::commit::envelope_residency::memory_publication_tests::memory_frame_quota_and_cancel_refuse_before_position_or_head`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text

thread 'history::commit::envelope_residency::memory_publication_tests::memory_frame_quota_and_cancel_refuse_before_position_or_head' (76412) panicked at crates\worth-relational\src\history\commit\envelope_residency\memory_publication_tests.rs:154:9:
the actual frame reservation must refuse
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-relational::worth_relational$storage::overlay::partition_content::digest_index::carrier_tests::inline_digest_headers_measure_empty_singleton_dense_and_retained_owners`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text
DIGEST_HEADER_MEASUREMENT {"axis":"record_history_u64","scenario":"empty","retained_roots":1,"old_map_nodes":0,"new_map_nodes":0,"old_map_node_bytes":null,"new_map_node_bytes":null,"old_unique_header_arcs":0,"header_arc_bytes":32,"shared_child_owners":0,"shared_child_bytes":0,"old_total_bytes":0,"new_total_bytes":0,"new_minus_old_bytes":0}

thread 'storage::overlay::partition_content::digest_index::carrier_tests::inline_digest_headers_measure_empty_singleton_dense_and_retained_owners' (43796) panicked at crates\worth-relational\src\storage\overlay\partition_content\digest_index\carrier_tests.rs:87:9:
assertion `left == right` failed
  left: 80
 right: 32
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-relational::worth_relational$storage::substrate::record_arena::values::metadata::tests::metadata_controls_follow_actual_history_snapshots_and_refund`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text

thread 'storage::substrate::record_arena::values::metadata::tests::metadata_controls_follow_actual_history_snapshots_and_refund' (3228) panicked at crates\worth-relational\src\storage\substrate\record_arena\values\metadata\tests.rs:123:5:
assertion `left == right` failed
  left: 504
 right: 360
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-relational::worth_relational$storage::substrate::record_arena::values::metadata::tests::metadata_controls_refuse_before_history_edits_with_typed_cause`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text

thread 'storage::substrate::record_arena::values::metadata::tests::metadata_controls_refuse_before_history_edits_with_typed_cause' (69496) panicked at crates\worth-relational\src\storage\substrate\record_arena\values\metadata\tests.rs:64:5:
assertion failed: matches!(denied, Err(Cause::PolicyMemoryExhausted { requested, .. }) if
    requested == bill)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-relational::worth_relational$storage::substrate::record_arena::values::metadata::tests::metadata_footer_preserves_legacy_named_and_compact_wire`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text

thread 'storage::substrate::record_arena::values::metadata::tests::metadata_footer_preserves_legacy_named_and_compact_wire' (70948) panicked at crates\worth-relational\src\storage\substrate\record_arena\values\metadata\tests.rs:225:5:
assertion `left == right` failed
  left: 352
 right: 256
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-relational::worth_relational$storage::substrate::record_arena::values::metadata::tests::moved_metadata_retains_only_its_fixed_bill_and_ordinary_clone_is_independent`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text

thread 'storage::substrate::record_arena::values::metadata::tests::moved_metadata_retains_only_its_fixed_bill_and_ordinary_clone_is_independent' (82924) panicked at crates\worth-relational\src\storage\substrate\record_arena\values\metadata\tests.rs:155:5:
assertion `left == right` failed: only the fixed metadata bill survives the unique move
  left: 168
 right: 120
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-relational::worth_relational$storage::substrate::record_arena::values::tests::escaped_extra_carrier_keeps_only_its_own_bill_after_node_drop`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text

thread 'storage::substrate::record_arena::values::tests::escaped_extra_carrier_keeps_only_its_own_bill_after_node_drop' (79056) panicked at crates\worth-relational\src\storage\substrate\record_arena\values\tests.rs:318:5:
assertion `left == right` failed
  left: 385
 right: 337
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-relational::worth_relational$storage::substrate::record_arena::values::tests::funded_extra_refuses_copy_and_replacement_before_growth`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text

thread 'storage::substrate::record_arena::values::tests::funded_extra_refuses_copy_and_replacement_before_growth' (79180) panicked at crates\worth-relational\src\storage\substrate\record_arena\values\tests.rs:169:5:
assertion `left == right` failed
  left: 10600
 right: 10552
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-relational::worth_relational$storage::substrate::record_arena::values::tests::funded_extra_shares_actual_row_history_and_private_holds_until_last_drop`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text

thread 'storage::substrate::record_arena::values::tests::funded_extra_shares_actual_row_history_and_private_holds_until_last_drop' (14116) panicked at crates\worth-relational\src\storage\substrate\record_arena\values\tests.rs:116:5:
assertion `left == right` failed
  left: 368
 right: 320
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-relational::worth_relational$storage::substrate::record_arena::values::tests::unique_extra_move_and_borrowed_export_preserve_original_funding`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text

thread 'storage::substrate::record_arena::values::tests::unique_extra_move_and_borrowed_export_preserve_original_funding' (28780) panicked at crates\worth-relational\src\storage\substrate\record_arena\values\tests.rs:204:5:
assertion `left == right` failed: the whole unique extra moved, including its footer
  left: 375
 right: 327
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-relational::worth_relational$storage::substrate::shared_map::custody_tests::map_cow_owns_only_actual_nodes_and_unique_replacement_refunds_nothing_early`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text

thread 'storage::substrate::shared_map::custody_tests::map_cow_owns_only_actual_nodes_and_unique_replacement_refunds_nothing_early' (47568) panicked at crates\worth-relational\src\storage\substrate\shared_map\custody_tests.rs:19:5:
assertion `left == right` failed
  left: 20480
 right: 14336
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-relational::worth_relational$storage::substrate::shared_map::tests::inline_header_footprints_remove_only_actual_header_arcs`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text

thread 'storage::substrate::shared_map::tests::inline_header_footprints_remove_only_actual_header_arcs' (18924) panicked at crates\worth-relational\src\storage\substrate\shared_map\tests.rs:157:5:
assertion `left == right` failed
  left: 24
 right: 72
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-relational::worth_relational$tests::durability::contracts::branch_root_shared_target_binding::checkpoint_rejects_later_shared_root_cell_with_relabeled_schema_root`

CLEANUP-malformed-input-boundary; owner: root owning court / recorded cleanup.

The failing assertion compares ordering or typed malformed-input diagnostics in a fault court.

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text

thread 'tests::durability::contracts::branch_root_shared_target_binding::checkpoint_rejects_later_shared_root_cell_with_relabeled_schema_root' (76856) panicked at crates\worth-relational\src\tests\durability\contracts\branch_root_content_binding.rs:320:5:
unexpected denial: DurabilityError { class: CorruptCheckpoint, detail: "branch cell `shared-root-sibling` target does not match immutable commit artifact `1`: target version/parents/roots = 1/[]/RelationalBranchRootDescriptor { truth_root: CanonicalDigestId([240, 8, 49, 175, 98, 98, 7, 71, 71, 108, 109, 33, 35, 88, 194, 3, 159, 176, 242, 209, 96, 10, 218, 83, 5, 164, 202, 92, 168, 48, 140, 166]), schema_root: CanonicalDigestId([197, 97, 197, 233, 123, 140, 54, 30, 146, 28, 253, 137, 138, 5, 65, 1, 73, 142, 153, 85, 71, 242, 107, 77, 240, 100, 71, 241, 106, 117, 106, 86]) }, artifact = 1/[]/RelationalBranchRootDescriptor { truth_root: CanonicalDigestId([240, 8, 49, 175, 98, 98, 7, 71, 71, 108, 109, 33, 35, 88, 194, 3, 159, 176, 242, 209, 96, 10, 218, 83, 5, 164, 202, 92, 168, 48, 140, 166]), schema_root: CanonicalDigestId([133, 97, 197, 233, 123, 140, 54, 30, 146, 28, 253, 137, 138, 5, 65, 1, 73, 142, 153, 85, 71, 242, 107, 77, 240, 100, 71, 241, 106, 117, 106, 86]) }", history_drift_class: None, authority_continuity_mismatch: None, storage_unavailable: None, context: ErrorContext { subsystem: Durability, operation: Recover, affected_records: [], version_context: None, suggested_fix: Some(RepairDurableStore) } }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-relational::worth_relational$tests::durability::contracts::branch_root_shared_target_binding::checkpoint_rejects_later_shared_root_cell_with_relabeled_truth_root`

CLEANUP-malformed-input-boundary; owner: root owning court / recorded cleanup.

The failing assertion compares ordering or typed malformed-input diagnostics in a fault court.

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text

thread 'tests::durability::contracts::branch_root_shared_target_binding::checkpoint_rejects_later_shared_root_cell_with_relabeled_truth_root' (48676) panicked at crates\worth-relational\src\tests\durability\contracts\branch_root_content_binding.rs:320:5:
unexpected denial: DurabilityError { class: CorruptCheckpoint, detail: "branch cell `shared-root-sibling` target does not match immutable commit artifact `1`: target version/parents/roots = 1/[]/RelationalBranchRootDescriptor { truth_root: CanonicalDigestId([38, 15, 14, 121, 126, 225, 71, 108, 186, 49, 130, 35, 63, 67, 175, 227, 107, 1, 134, 220, 80, 161, 183, 155, 119, 64, 147, 173, 64, 213, 172, 200]), schema_root: CanonicalDigestId([133, 97, 197, 233, 123, 140, 54, 30, 146, 28, 253, 137, 138, 5, 65, 1, 73, 142, 153, 85, 71, 242, 107, 77, 240, 100, 71, 241, 106, 117, 106, 86]) }, artifact = 1/[]/RelationalBranchRootDescriptor { truth_root: CanonicalDigestId([166, 15, 14, 121, 126, 225, 71, 108, 186, 49, 130, 35, 63, 67, 175, 227, 107, 1, 134, 220, 80, 161, 183, 155, 119, 64, 147, 173, 64, 213, 172, 200]), schema_root: CanonicalDigestId([133, 97, 197, 233, 123, 140, 54, 30, 146, 28, 253, 137, 138, 5, 65, 1, 73, 142, 153, 85, 71, 242, 107, 77, 240, 100, 71, 241, 106, 117, 106, 86]) }", history_drift_class: None, authority_continuity_mismatch: None, storage_unavailable: None, context: ErrorContext { subsystem: Durability, operation: Recover, affected_records: [], version_context: None, suggested_fix: Some(RepairDurableStore) } }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-relational::worth_relational$visibility::materialization::read_records::projection::field_revisions::restore_tests::checkpoint_restore_rejects_incomplete_field_revision_column`

CLEANUP-malformed-input-boundary; owner: root owning court / recorded cleanup.

The failing assertion compares ordering or typed malformed-input diagnostics in a fault court.

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-relational.log`.

```text

thread 'visibility::materialization::read_records::projection::field_revisions::restore_tests::checkpoint_restore_rejects_incomplete_field_revision_column' (54632) panicked at crates\worth-relational\src\visibility\materialization\read_records\projection\field_revisions\restore_tests.rs:324:5:
DurabilityError { class: CorruptCheckpoint, detail: "record arena SoA column lengths differ", history_drift_class: None, authority_continuity_mismatch: None, storage_unavailable: None, context: ErrorContext { subsystem: Durability, operation: Recover, affected_records: [], version_context: None, suggested_fix: Some(RepairDurableStore) } }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-court-relational-certification::worth_court_relational_certification$cost::relational::relational_certification::retained_entity_selection::equal_payload_jumps_charge_the_same_receiving_owner_work_at_near_and_distant_history_nodes`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-root-cost.log`.

```text

thread 'cost::relational::relational_certification::retained_entity_selection::equal_payload_jumps_charge_the_same_receiving_owner_work_at_near_and_distant_history_nodes' (79024) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\cost\relational\relational_certification\retained_entity_selection.rs:52:5:
assertion `left == right` failed: actual request charged work
  left: 7095
 right: 7094
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-court-relational-certification::worth_court_relational_certification$cost::relational::checkpoint_stage::checkpoint_stage_charges_the_documented_rate_for_each_encoded_byte`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-root-cost.log`.

```text

thread 'cost::relational::checkpoint_stage::checkpoint_stage_charges_the_documented_rate_for_each_encoded_byte' (67228) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\cost\relational\checkpoint_stage.rs:91:9:
assertion `left == right` failed: rows=0
  left: 2628
 right: 1108
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-court-relational-certification::worth_court_relational_certification$cost::relational::checkpoint_stage::empty_checkpoint_memory_boundary_is_its_bytes_plus_one_stage`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-root-cost.log`.

```text

thread 'cost::relational::checkpoint_stage::empty_checkpoint_memory_boundary_is_its_bytes_plus_one_stage' (54684) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\cost\relational\checkpoint_stage.rs:215:9:
leased=false: Err(Request(PolicyMemoryExhausted { requested: 2628, admitted: 724, ancestor: 0 }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-court-relational-certification::worth_court_relational_certification$cost::relational::checkpoint_stage::checkpoint_capture_sets_up_exactly_one_stage_whatever_the_row_count`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-root-cost.log`.

```text

thread 'cost::relational::checkpoint_stage::checkpoint_capture_sets_up_exactly_one_stage_whatever_the_row_count' (51568) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\cost\relational\checkpoint_stage.rs:135:9:
assertion `left == right` failed: rows=0
  left: ExecutionScopeSetupCountsForTest { stage_setups: 14, limit_derivations: 14, admissions: 14 }
 right: ExecutionScopeSetupCountsForTest { stage_setups: 1, limit_derivations: 1, admissions: 1 }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-court-relational-certification::worth_court_relational_certification$cost::relational::checkpoint_stage::empty_checkpoint_work_ceiling_bounds_the_whole_capture`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-root-cost.log`.

```text

thread 'cost::relational::checkpoint_stage::empty_checkpoint_work_ceiling_bounds_the_whole_capture' (53592) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\cost\relational\checkpoint_stage.rs:169:9:
assertion `left == right` failed: ceiling=2376
  left: 550
 right: 585
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-court-relational-certification::worth_court_relational_certification$process::serial::relational::leased_preparation::preimage::preimage_endpoint_slots`

CLEANUP-C2-resource-accounting; owner: root owning court / recorded cleanup.

Exact child exit101: actual1024-endpoint commit exceeds remaining64MiB fixture grant by1118 bytes before COW growth assertions; no overflow. Evidence: pa-merge-preimage-child-exit.log; milestone C2 backing/work-budget and growth courts.

Log: `target/lane-evidence/pc-merge-root-tier2/test-cost/root/run-root-process.log`.

```text

thread 'process::serial::relational::leased_preparation::preimage::preimage_endpoint_slots' (28392) panicked at crates\worth-testkit-relational\src\test_process\declarations.rs:16:5:
isolated court process::serial::relational::leased_preparation::preimage::preimage_endpoint_slots failed:

running 1 test

```

## `worth-query-court-query-decl-host::worth_query_court_query_decl_host$process::certification::program_example_authority::execution_memory::execution_memory_is_shared_by_installed_worlds_and_reselected_on_checkpoint_reopen`

CLEANUP-C2-memory-ledger; owner: query owning court / recorded cleanup.

Exact existing child probe fails expected process-capacity denial after checkpoint reopen; actual variant not printed, no overflow evidence.

Log: `target/lane-evidence/pc-merge-query-tier2/test-cost/query/run-query-process.log`.

```text

thread 'process::certification::program_example_authority::execution_memory::execution_memory_is_shared_by_installed_worlds_and_reselected_on_checkpoint_reopen' (69200) panicked at crates\worth-query-court-query-decl-host\tests\worth_query_court_query_decl_host\process\certification\program_example_authority\execution_memory\mod.rs:18:9:

running 1 test

```

## `worth-query-court-query-decl-host::worth_query_court_query_decl_host$process::host::durable_retry::request_stops::original_directory_receipt_preserves_cancellation_at_every_consult`

CLEANUP-C1-incomplete-consult-cancellation-sweep; owner: query owning court / recorded cleanup.

Cancellation at every consult; stopped exact own busy child after>10min. Incomplete, not passed; abort evidence retained.

Log: `target/lane-evidence/pc-merge-query-tier2/test-cost/query/run-query-process.log`.

```text
test process::host::durable_retry::request_stops::original_directory_receipt_preserves_cancellation_at_every_consult has been running for over 60 seconds

```

## `worth-query::worth_query$domain_installation::operation_execution::workflow_parallel_progression::frontier::tests::dispatch::reversed_completion_still_applies_canonical_effects_and_receipts`

CLEANUP-C2-cost-or-growth; owner: query owning court / recorded cleanup.

Complete original SDK invoice45717 versus1992.

Log: `target/lane-evidence/pc-merge-query-tier2/test-cost/query/run-query-rest.log`.

```text

thread 'domain_installation::operation_execution::workflow_parallel_progression::frontier::tests::dispatch::reversed_completion_still_applies_canonical_effects_and_receipts' (12156) panicked at crates\worth-query\src\domain_installation\operation_execution\workflow_execution\progression\frontier\tests\dispatch.rs:277:5:
assertion `left == right` failed: complete original SDK invoice
  left: 45717
 right: 1992
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-query::worth_query$domain_installation::operation_execution::workflow_parallel_progression::frontier::tests::dispatch::stops::admitted_opening_refuses_the_owned_maps_declared_reservation`

CLEANUP-C2-cost-or-growth; owner: query owning court / recorded cleanup.

Declared owned-map reservation3184 versus1002.

Log: `target/lane-evidence/pc-merge-query-tier2/test-cost/query/run-query-rest.log`.

```text

thread 'domain_installation::operation_execution::workflow_parallel_progression::frontier::tests::dispatch::stops::admitted_opening_refuses_the_owned_maps_declared_reservation' (43244) panicked at C:\programming\worth worktees\worktrees\m3-host-home\crates\worth-proof\src\transition\outcomes.rs:99:36:
called TransitionOutcome::unwrap on Denied(WorthQueryWorkflowStartDenial { kind: ManagedRun("original request refused immutable lowering storage: Admission(MemoryExhausted(Capacity { requested: 3184, admitted: 1002, level: Policy { ancestor: 0 } }))"), counters: WorthQueryWorkflowRunCounters { runtime_authority_checks: 0, stage_index_lookups: 0, stage_admission_checks: 0, predecessor_checks: 0, predecessor_receipt_lookups: 0, required_capability_checks: 0, required_domain_checks: 0, graph_read_contacts: 0, touch_effect_contacts: 0, effect_receipt_checks: 0, commit_admission_contacts: 0, invariant_checks: 0, parallel_admission_checks: 0, stage_executor_contacts: 0, computation_charged_work: 0, output_contract_checks: 0, terminal_contract_checks: 0, consumption_contacts: 0, unrelated_run_scans: 0, conditional_request_admission_checks: 0, conditional_contract_lookups: 0, conditional_dependency_observation_reads: 0, conditional_dependency_checks: 0, conditional_semantic_reads: 0, conditional_condition_checks: 0, conditional_condition_deferrals: 0, conditional_temporal_deferrals: 0, conditional_on_demand_deferrals: 0, conditional_comparator_checks: 0, conditional_compute_contacts: 0, conditional_output_version_reads: 0, conditional_runtime_dependency_edges_captured: 0, conditional_application_contacts: 0, conditional_semantic_classifications: 0, conditional_reverted_clean_outcomes: 0, conditional_semantic_changes: 0, conditional_reuse_checks: 0, conditional_decisions_delivered: 0 } })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-query-court-decl-execution-package-archive::worth_query_court_decl_execution_package_archive$domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::retained_recovery::checkpoint_inspection_compares_query_header_count_and_output_metadata_bytes`

CLEANUP-C1-recovery-or-fault; owner: query owning court / recorded cleanup.

Authenticated metadata fault uses stale v13 offsets; attacks Native bytes and hits work denial before comparison.

Log: `target/lane-evidence/pc-merge-query-tier2/test-cost/query/run-query-rest.log`.

```text

thread 'domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::retained_recovery::checkpoint_inspection_compares_query_header_count_and_output_metadata_bytes' (48308) panicked at crates\worth-query-court-decl-execution-package-archive\tests\worth_query_court_decl_execution_package_archive\domain_computation\primary_graph\output_lineage\ownership_oracle\optional_aspects\retained_recovery\mod.rs:273:22:
called `Result::unwrap()` on an `Err` value: Native(Request(WorkExhausted))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-query-court-decl-host::worth_query_court_decl_host$certification::document_history::group::recovery::an_unpublished_first_member_resumes_once_then_completes_the_one_group_step`

CLEANUP-C1-recovery-or-fault; owner: query owning court / recorded cleanup.

Cancellation injected between owner effect and product movement; unpublished group recovery assertion fails.

Log: `target/lane-evidence/pc-merge-query-tier2/test-cost/query/run-query-rest.log`.

```text

thread 'certification::document_history::group::recovery::an_unpublished_first_member_resumes_once_then_completes_the_one_group_step' (62204) panicked at crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\recovery.rs:155:9:
assertion failed: matches!(execution.completed_source(0).unwrap().evidence(),
    WorthQuerySourceCompletion::Recovered { publication: Some(_), .. })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-query-court-decl-host::worth_query_court_decl_host$certification::document_history::recording::recovery::unpublished_candidate_keeps_its_original_recovery_until_one_recorded_selection`

CLEANUP-C1-recovery-or-fault; owner: query owning court / recorded cleanup.

Cancellation injected between owner effect and product movement; original retained recovery assertion fails.

Log: `target/lane-evidence/pc-merge-query-tier2/test-cost/query/run-query-rest.log`.

```text

thread 'certification::document_history::recording::recovery::unpublished_candidate_keeps_its_original_recovery_until_one_recorded_selection' (71332) panicked at crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\recording\recovery.rs:119:9:
keep the original owner-issued recovered publication, without replaying the handler
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-query-court-decl-host::worth_query_court_decl_host$producer_succession::history_courts::checkpoint_keeps_exact_per_route_retirement_and_recovers_the_surviving_generator`

CLEANUP-C2-cost-or-growth; owner: query owning court / recorded cleanup.

Coordinator explicitly holds exact per-route retirement shrink asC2; retained roots0 versus4.

Log: `target/lane-evidence/pc-merge-query-tier2/test-cost/query/run-query-rest.log`.

```text

thread 'producer_succession::history_courts::checkpoint_keeps_exact_per_route_retirement_and_recovers_the_surviving_generator' (58664) panicked at crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\producer_succession\checkpoint.rs:143:9:
assertion `left == right` failed: all retained roots: before={CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 0, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 1, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 2, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 3, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 4, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 5, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 6, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 7, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 8, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 9, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 10, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 11, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 12, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 13, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 14, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 15, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 16, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 17, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 18, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 0, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 1, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 2, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 3, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 4, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 5, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 6, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 7, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 8, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 9, generation: 1 }}, after={CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 0, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 1, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 2, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 3, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 4, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 5, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 6, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 7, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 8, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 9, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 10, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 11, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 12, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 13, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 14, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 15, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 16, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 17, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 18, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 6, slot: 0, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 7, slot: 0, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 8, slot: 0, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 0, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 1, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 2, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 3, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 4, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 5, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 6, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 7, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 8, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 9, generation: 1 }}
  left: 0
 right: 4
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-query-court-decl-host-package-archive-replay-application-graph::worth_query_court_decl_host_package_archive_replay_application_graph$certification::application_graph::restored_primary_backend::the_first_production_write_after_restore_answers_as_before_the_capture`

CLEANUP-C1-recovery-or-fault; owner: query owning court / recorded cleanup.

CommitThenDecline adapter injects post-Native-effect refusal twice before capture; WrongImmutableTarget before cold first-write proof.

Log: `target/lane-evidence/pc-merge-query-tier2/test-cost/query/run-query-rest.log`.

```text

thread 'certification::application_graph::restored_primary_backend::the_first_production_write_after_restore_answers_as_before_the_capture' (22960) panicked at crates\worth-query-court-decl-host-package-archive-replay-application-graph\tests\worth_query_court_decl_host_package_archive_replay_application_graph\certification\application_graph\restored_primary_backend\mod.rs:22:18:
the host captures: WorldOriginals(Component(Basis(WrongImmutableTarget)))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-query-court-execution-primary-graph::worth_query_court_execution_primary_graph$domain_computation::primary_graph::tests::application_attempt::unwind_custody::world_unwind_retains_and_releases_the_exact_query_idempotency_slot`

CLEANUP-C1-recovery-or-fault; owner: query owning court / recorded cleanup.

Injected unwind custody cleanup leaves Query idempotency slot1, expected0.

Log: `target/lane-evidence/pc-merge-query-tier2/test-cost/query/run-query-rest.log`.

```text

thread 'domain_computation::primary_graph::tests::application_attempt::unwind_custody::world_unwind_retains_and_releases_the_exact_query_idempotency_slot' (26744) panicked at C:\programming\worth worktees\worktrees\m3-host-home\crates\worth-runtime-world\src\lifecycle\operation_control.rs:164:17:
injected unwind before product comparison
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'domain_computation::primary_graph::tests::application_attempt::unwind_custody::world_unwind_retains_and_releases_the_exact_query_idempotency_slot' (26744) panicked at crates\worth-query-court-execution-primary-graph\tests\worth_query_court_execution_primary_graph\domain_computation\primary_graph\tests\application_attempt\unwind_custody\mod.rs:139:5:
assertion `left == right` failed: cleanup releases Query's exact recovery-indexed slot
  left: 1
 right: 0

```

## `worth-query-court-execution-primary-graph::worth_query_court_execution_primary_graph$domain_computation::primary_graph::tests::checkpoint_allocation::leased_checkpoint_retains_one_frame_charge_through_native_handoff_and_reopen`

CLEANUP-C2-cost-or-growth; owner: query owning court / recorded cleanup.

Leased checkpoint allocation child first fails zero-memory denial category; actual variant not printed, later stale offsets unreached; no overflow evidence.

Log: `target/lane-evidence/pc-merge-query-tier2/test-cost/query/run-query-rest.log`.

```text

thread 'domain_computation::primary_graph::tests::checkpoint_allocation::leased_checkpoint_retains_one_frame_charge_through_native_handoff_and_reopen' (46932) panicked at crates\worth-query-testkit-execution-primary-graph-application-attempt-2\src\domain_computation\primary_graph\bootstrap_publication\execution_refusals\mod.rs:59:5:

running 1 test

```

## `worth-query-court-query-decl-host::worth_query_court_query_decl_host$certification::application_graph::canonical_transition::related::companion_transition::capture::companion_post_ack_capture_refusal_repairs_same_capsule_with_historical_roster`

CLEANUP-C1-post-ACK-deliberate-refusal; owner: query owning court / recorded cleanup.

Benign restored-model setup/post-restore mutation reads canonical commit/version through resident-only lookup; cold catalog evidence is available through original-request demand.

Log: `target/lane-evidence/pc-merge-query-tier2/test-cost/query/run-query-rest.log`.

```text

thread 'certification::application_graph::canonical_transition::related::companion_transition::capture::companion_post_ack_capture_refusal_repairs_same_capsule_with_historical_roster' (64316) panicked at crates\worth-query-court-query-decl-host\tests\worth_query_court_query_decl_host\certification\application_graph\canonical_transition\related\companion_transition\capture\mod.rs:43:9:
exact native allocation cause retained
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-query-court-query-decl-host::worth_query_court_query_decl_host$certification::application_graph::canonical_transition::related::settlement::related_acknowledged_successor_repairs_capture_without_reauthoring_or_old_decode`

CLEANUP-C1-post-ACK-deliberate-refusal; owner: query owning court / recorded cleanup.

Benign restored-model setup/post-restore mutation reads canonical commit/version through resident-only lookup; cold catalog evidence is available through original-request demand.

Log: `target/lane-evidence/pc-merge-query-tier2/test-cost/query/run-query-rest.log`.

```text

thread 'certification::application_graph::canonical_transition::related::settlement::related_acknowledged_successor_repairs_capture_without_reauthoring_or_old_decode' (57144) panicked at crates\worth-query-court-query-decl-host\tests\worth_query_court_query_decl_host\certification\application_graph\canonical_transition\related\settlement\mod.rs:44:9:
capture refusal remains precise
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-query-court-query-decl-host::worth_query_court_query_decl_host$host::application_home::recovered_custody::continued_recovered_custody_closes_cooperatively_and_resumes_after_reopen`

CLEANUP-C1-recovery-or-fault; owner: query owning court / recorded cleanup.

Injected unwind recovery reaches cross-incarnation attempt identity inequality before cooperative close; held ProductBasisStale not reached.

Log: `target/lane-evidence/pc-merge-query-tier2/test-cost/query/run-query-rest.log`.

```text

thread 'host::application_home::recovered_custody::continued_recovered_custody_closes_cooperatively_and_resumes_after_reopen' (48352) panicked at C:\programming\worth worktees\worktrees\m3-host-home\crates\worth-runtime-world\src\lifecycle\operation_control.rs:164:17:
injected unwind before product comparison
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'host::application_home::recovered_custody::continued_recovered_custody_closes_cooperatively_and_resumes_after_reopen' (48352) panicked at crates\worth-query-court-query-decl-host\tests\worth_query_court_query_decl_host\host\application_home\recovered_custody\disposition.rs:27:5:
assertion `left == right` failed
  left: CompositePublicationAttemptIdentity { owner: RuntimeWorldOwnerIdentity { scope: Home(DurableWorldOwnerIdentity { home: HomeIdentity([181, 222, 246, 128, 236, 234, 239, 10, 211, 11, 200, 249, 242, 5, 41, 141]), slot: WorldOwnerSlotIdentity(0) }), instance: EphemeralWorldOwnerIdentity(1), home_fence: Some(HomeIncarnationFence { incarnation: 2 }) }, ordinal: 1 }
 right: CompositePublicationAttemptIdentity { owner: RuntimeWorldOwnerIdentity { scope: Home(DurableWorldOwnerIdentity { home: HomeIdentity([181, 222, 246, 128, 236, 234, 239, 10, 211, 11, 200, 249, 242, 5, 41, 141]), slot: WorldOwnerSlotIdentity(0) }), instance: EphemeralWorldOwnerIdentity(0), home_fence: Some(HomeIncarnationFence { incarnation: 1 }) }, ordinal: 1 }

```

## `worth-query-court-query-decl-host::worth_query_court_query_decl_host$host::application_home::recovered_custody::recovered_original_custody_reaches_service_and_continues_without_reexecution`

CLEANUP-C1-recovery-or-fault; owner: query owning court / recorded cleanup.

Injected unwind recovery reaches cross-incarnation attempt identity inequality before continuing; held ProductBasisStale not reached.

Log: `target/lane-evidence/pc-merge-query-tier2/test-cost/query/run-query-rest.log`.

```text

thread 'host::application_home::recovered_custody::recovered_original_custody_reaches_service_and_continues_without_reexecution' (61132) panicked at C:\programming\worth worktees\worktrees\m3-host-home\crates\worth-runtime-world\src\lifecycle\operation_control.rs:164:17:
injected unwind before product comparison
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'host::application_home::recovered_custody::recovered_original_custody_reaches_service_and_continues_without_reexecution' (61132) panicked at crates\worth-query-court-query-decl-host\tests\worth_query_court_query_decl_host\host\application_home\recovered_custody\disposition.rs:27:5:
assertion `left == right` failed
  left: CompositePublicationAttemptIdentity { owner: RuntimeWorldOwnerIdentity { scope: Home(DurableWorldOwnerIdentity { home: HomeIdentity([69, 157, 119, 233, 232, 238, 199, 232, 19, 181, 220, 75, 111, 151, 20, 186]), slot: WorldOwnerSlotIdentity(0) }), instance: EphemeralWorldOwnerIdentity(1), home_fence: Some(HomeIncarnationFence { incarnation: 2 }) }, ordinal: 1 }
 right: CompositePublicationAttemptIdentity { owner: RuntimeWorldOwnerIdentity { scope: Home(DurableWorldOwnerIdentity { home: HomeIdentity([69, 157, 119, 233, 232, 238, 199, 232, 19, 181, 220, 75, 111, 151, 20, 186]), slot: WorldOwnerSlotIdentity(0) }), instance: EphemeralWorldOwnerIdentity(0), home_fence: Some(HomeIncarnationFence { incarnation: 1 }) }, ordinal: 1 }

```

## `worth-query-court-query-decl-host::worth_query_court_query_decl_host$host::application_home::recovered_custody::recovered_original_custody_reaches_service_and_release_stays_released`

CLEANUP-C1-recovery-or-fault; owner: query owning court / recorded cleanup.

Injected unwind recovery reaches cross-incarnation attempt identity inequality before releasing; held ProductBasisStale not reached.

Log: `target/lane-evidence/pc-merge-query-tier2/test-cost/query/run-query-rest.log`.

```text

thread 'host::application_home::recovered_custody::recovered_original_custody_reaches_service_and_release_stays_released' (10528) panicked at C:\programming\worth worktees\worktrees\m3-host-home\crates\worth-runtime-world\src\lifecycle\operation_control.rs:164:17:
injected unwind before product comparison
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'host::application_home::recovered_custody::recovered_original_custody_reaches_service_and_release_stays_released' (10528) panicked at crates\worth-query-court-query-decl-host\tests\worth_query_court_query_decl_host\host\application_home\recovered_custody\disposition.rs:27:5:
assertion `left == right` failed
  left: CompositePublicationAttemptIdentity { owner: RuntimeWorldOwnerIdentity { scope: Home(DurableWorldOwnerIdentity { home: HomeIdentity([228, 82, 248, 169, 33, 199, 113, 87, 149, 182, 220, 34, 38, 74, 207, 183]), slot: WorldOwnerSlotIdentity(0) }), instance: EphemeralWorldOwnerIdentity(1), home_fence: Some(HomeIncarnationFence { incarnation: 2 }) }, ordinal: 1 }
 right: CompositePublicationAttemptIdentity { owner: RuntimeWorldOwnerIdentity { scope: Home(DurableWorldOwnerIdentity { home: HomeIdentity([228, 82, 248, 169, 33, 199, 113, 87, 149, 182, 220, 34, 38, 74, 207, 183]), slot: WorldOwnerSlotIdentity(0) }), instance: EphemeralWorldOwnerIdentity(0), home_fence: Some(HomeIncarnationFence { incarnation: 1 }) }, ordinal: 1 }

```

## `worth-query-court-execution-primary-graph-application-attempt::worth_query_court_execution_primary_graph_application_attempt$domain_computation::primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::execution::authority_cause::tests::authority_evidence_exposes_request_resources_by_value_through_the_commit_door`

CLEANUP-C2-incomplete-charge-prefix-sweep; owner: query owning court / recorded cleanup.

All unique charged prefixes are replayed at required-1; stopped exact own process after prolonged busy execution. Incomplete, not passed or an assertion result; abort-v2 evidence retained.

Log: `target/lane-evidence/pc-merge-query-tier2/test-cost/query/run-query-rest.log`.

```text
authority charge probes=7832
authority probe=0, required=1, reached=0
authority witness probe=0, required=1
authority witness probe=1, required=34
authority witness probe=2, required=37
authority witness probe=3, required=70
authority witness probe=4, required=154
authority witness probe=5, required=156
authority witness probe=6, required=195
authority witness probe=7, required=217
authority witness probe=8, required=218
authority witness probe=9, required=219
authority witness probe=10, required=252
authority witness probe=11, required=255
authority witness probe=12, required=288
authority witness probe=13, required=363
authority witness probe=14, required=365
authority witness probe=15, required=401
authority witness probe=16, required=423
authority witness probe=17, required=424
authority witness probe=18, required=425
authority witness probe=19, required=426
authority witness probe=20, required=427
authority witness probe=21, required=428
authority witness probe=22, required=461
authority witness probe=23, required=464
authority witness probe=24, required=497
authority witness probe=25, required=562
authority witness probe=26, required=564
authority witness probe=27, required=605
authority witness probe=28, required=616
authority witness probe=29, required=617
authority witness probe=30, required=618
authority witness probe=31, required=619
authority witness probe=36, required=3034
authority witness probe=37, required=3067
authority witness probe=38, required=3070
authority witness probe=39, required=3103
authority witness probe=40, required=3187
authority witness probe=41, required=3189
authority witness probe=42, required=3228
authority witness probe=43, required=3250
authority witness probe=44, required=3251
authority witness probe=45, required=3252
authority witness probe=46, required=3285
authority witness probe=47, required=3288
authority witness probe=48, required=3321
authority witness probe=49, required=3396
authority witness probe=50, required=3398
authority witness probe=51, required=3434
authority witness probe=52, required=3456
authority witness probe=53, required=3457
authority witness probe=54, required=3458
authority witness probe=55, required=3459
authority witness probe=56, required=3460
authority witness probe=57, required=3461
authority witness probe=58, required=3494
authority witness probe=59, required=3497
authority witness probe=60, required=3530
authority witness probe=61, required=3595
authority witness probe=62, required=3597
authority witness probe=63, required=3638
authority witness probe=64, required=3649
authority witness probe=65, required=3650
authority witness probe=66, required=3651
authority witness probe=67, required=3652
authority probe=100, required=4272, reached=64
test domain_computation::primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::execution::authority_cause::tests::authority_evidence_exposes_request_resources_by_value_through_the_commit_door has been running for over 60 seconds
authority probe=200, required=6222, reached=64
authority witness probe=219, required=6335
authority witness probe=220, required=6368
authority witness probe=221, required=6371
authority witness probe=222, required=6404
authority witness probe=223, required=6488
authority witness probe=224, required=6490
authority witness probe=225, required=6529
authority witness probe=226, required=6551
authority witness probe=227, required=6552
authority witness probe=228, required=6553
authority witness probe=229, required=6586
authority witness probe=230, required=6589
authority witness probe=231, required=6622
authority witness probe=232, required=6697
authority witness probe=233, required=6699
authority witness probe=234, required=6735
authority witness probe=235, required=6757
authority witness probe=236, required=6758
authority witness probe=237, required=6759
authority witness probe=238, required=6760
authority witness probe=239, required=6761
authority witness probe=240, required=6762
authority witness probe=241, required=6795
authority witness probe=242, required=6798
authority witness probe=243, required=6831
authority witness probe=244, required=6896
authority witness probe=245, required=6898
authority witness probe=246, required=6939
authority witness probe=247, required=6950
authority witness probe=248, required=6951
authority witness probe=249, required=6952
authority witness probe=250, required=6953
authority probe=300, required=7631, reached=96
authority probe=400, required=9847, reached=96
authority probe=500, required=15099, reached=96
authority probe=600, required=15632, reached=96
authority probe=700, required=16149, reached=96
authority probe=800, required=16764, reached=96
authority probe=900, required=17251, reached=96
authority probe=1000, required=17956, reached=96
authority probe=1100, required=18437, reached=96
authority probe=1200, required=18912, reached=96
authority probe=1300, required=19383, reached=96
authority probe=1400, required=19703, reached=96
authority probe=1500, required=19894, reached=96
authority probe=1600, required=20174, reached=96
authority probe=1700, required=20729, reached=96
authority probe=1800, required=21545, reached=96
authority probe=1900, required=22393, reached=96
authority probe=2000, required=23320, reached=96
authority probe=2100, required=24232, reached=96
authority probe=2200, required=25094, reached=96
authority probe=2300, required=26038, reached=96
authority probe=2400, required=26853, reached=96
authority probe=2500, required=27752, reached=96
authority probe=2600, required=28576, reached=96
authority probe=2700, required=29487, reached=96
authority probe=2800, required=30420, reached=96
authority probe=2900, required=31288, reached=96
authority probe=3000, required=32170, reached=96
authority probe=3100, required=33041, reached=96
authority probe=3200, required=33944, reached=96
authority probe=3300, required=34821, reached=96
authority probe=3400, required=35737, reached=96
authority probe=3500, required=36639, reached=96
authority probe=3600, required=37549, reached=96
authority probe=3700, required=38429, reached=96
authority probe=3800, required=39343, reached=96
authority probe=3900, required=40270, reached=96
authority probe=4000, required=41214, reached=96
authority probe=4100, required=42126, reached=96
authority probe=4200, required=42810, reached=96
authority probe=4300, required=45034, reached=96
authority probe=4400, required=45402, reached=96
authority probe=4500, required=45882, reached=96
authority probe=4600, required=46345, reached=96
authority probe=4700, required=47203, reached=96
authority probe=4800, required=47693, reached=96
authority probe=4900, required=48240, reached=96
authority probe=5000, required=48630, reached=96
authority probe=5100, required=49028, reached=96
authority probe=5200, required=49400, reached=96
authority probe=5300, required=49775, reached=96
authority probe=5400, required=50172, reached=96
authority probe=5500, required=50559, reached=96
authority probe=5600, required=50956, reached=96
authority probe=5700, required=51258, reached=96
authority probe=5800, required=51440, reached=96
authority probe=5900, required=51789, reached=96
authority probe=6000, required=52236, reached=96
authority probe=6100, required=52723, reached=96
authority probe=6200, required=53382, reached=96
authority probe=6300, required=57019, reached=96
authority probe=6400, required=60408, reached=96

```

## `worth-query-court-decl-host-package-archive-replay-application-graph::worth_query_court_decl_host_package_archive_replay_application_graph$certification::application_graph::relation_laws::logical::locality::logical_local_rule_business_acquisitions_ignore_ten_and_ten_thousand_unrelated_cohorts`

CLEANUP-C2-cost-or-growth; owner: query owning court / recorded cleanup.

Ten-versus-ten-thousand cohort growth journey expires30000ms candidate during setup.

Log: `target/lane-evidence/pc-merge-query-tier2/test-cost/query/run-query-rest.log`.

```text
test certification::application_graph::relation_laws::logical::locality::logical_local_rule_business_acquisitions_ignore_ten_and_ten_thousand_unrelated_cohorts has been running for over 60 seconds

thread 'certification::application_graph::relation_laws::logical::locality::logical_local_rule_business_acquisitions_ignore_ten_and_ten_thousand_unrelated_cohorts' (37768) panicked at crates\worth-query-testkit-certification-application-graph\src\certification\application_graph\relation_laws\installation.rs:37:59:
called `Result::unwrap()` on an `Err` value: Publication(WorthQueryPrimaryGraphInstallationDenial { kind: RelationalCommitRejected, subject: "Relational rejected the application seed transaction: branch publication deferred: CandidateLifetimeExpired { maximum_lifetime_millis: 30000 }", cause: Some(NativeValidation(WorthQueryInvariantExecutionFailure { kind: ProviderRejected, posture: Denied, detail: "Relational rejected the installed proposed-state invariant", custom_invariant: None, native: None })) })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-query-bank-court-external-rail-server::worth_query_bank_court_external_rail_server$cost::server::ordinary_reads::canonical_work_scale::policy_fact_fanout_does_not_leak_into_closed_disclosure_evidence`

CLEANUP-C2-cost-or-growth; owner: bank owning court / recorded cleanup.

Explicit fanout/linear growth court is denied by original seed-memory quota during large-population setup, before the work-bound assertion. Grants and assertions unchanged.

Log: `target/lane-evidence/pc-merge-bank-tier2/test-cost/bank/run-bank-cost.log`.

```text

thread 'cost::server::ordinary_reads::canonical_work_scale::policy_fact_fanout_does_not_leak_into_closed_disclosure_evidence' (43328) panicked at crates\worth-query-bank-testkit-server\src\support\mod.rs:79:34:
bank test runtime should install: ApplicationInstallation(ConditionalPublication(WorthQueryConditionalRuntimeInstallationDenial { kind: PrimaryGraphPublication, subject: "primary graph installation denied: ExecutionDenied { kind: ExecutionResource { denial: MemoryLimit { requested: 48762168, admitted: 48762167, level: Policy }, partition_identity: None, policy_ancestor: Some(0) } } (Relational rejected the application seed transaction: commit preparation execution stopped: CommitExecutionDenial { kind: Cause(PolicyMemoryExhausted { requested: 48762168, admitted: 48762167, ancestor: 0 }), partition_identity: None })" }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-query-bank-court-external-rail-server::worth_query_bank_court_external_rail_server$cost::server::ordinary_reads::canonical_work_scale::result_and_graph_fanout_is_visible_only_as_closed_public_work`

CLEANUP-C2-cost-or-growth; owner: bank owning court / recorded cleanup.

Explicit fanout/linear growth court is denied by original seed-memory quota during large-population setup, before the work-bound assertion. Grants and assertions unchanged.

Log: `target/lane-evidence/pc-merge-bank-tier2/test-cost/bank/run-bank-cost.log`.

```text

thread 'cost::server::ordinary_reads::canonical_work_scale::result_and_graph_fanout_is_visible_only_as_closed_public_work' (10184) panicked at crates\worth-query-bank-testkit-server\src\support\mod.rs:79:34:
bank test runtime should install: ApplicationInstallation(ConditionalPublication(WorthQueryConditionalRuntimeInstallationDenial { kind: PrimaryGraphPublication, subject: "primary graph installation denied: ExecutionDenied { kind: ExecutionResource { denial: MemoryLimit { requested: 48762168, admitted: 48762167, level: Policy }, partition_identity: None, policy_ancestor: Some(0) } } (Relational rejected the application seed transaction: commit preparation execution stopped: CommitExecutionDenial { kind: Cause(PolicyMemoryExhausted { requested: 48762168, admitted: 48762167, ancestor: 0 }), partition_identity: None })" }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-query-bank-court-external-rail-server::worth_query_bank_court_external_rail_server$cost::server::ordinary_reads::locality::account_discovery_root_union_work_grows_at_most_linearly`

CLEANUP-C2-cost-or-growth; owner: bank owning court / recorded cleanup.

Explicit fanout/linear growth court is denied by original seed-memory quota during large-population setup, before the work-bound assertion. Grants and assertions unchanged.

Log: `target/lane-evidence/pc-merge-bank-tier2/test-cost/bank/run-bank-cost.log`.

```text

thread 'cost::server::ordinary_reads::locality::account_discovery_root_union_work_grows_at_most_linearly' (45040) panicked at crates\worth-query-bank-testkit-server\src\support\mod.rs:79:34:
bank test runtime should install: ApplicationInstallation(ConditionalPublication(WorthQueryConditionalRuntimeInstallationDenial { kind: PrimaryGraphPublication, subject: "primary graph installation denied: ExecutionDenied { kind: ExecutionResource { denial: MemoryLimit { requested: 8440, admitted: 562, level: Policy }, partition_identity: None, policy_ancestor: Some(0) } } (Relational rejected the application seed transaction: commit preparation execution stopped: CommitExecutionDenial { kind: Cause(PolicyMemoryExhausted { requested: 8440, admitted: 562, ancestor: 0 }), partition_identity: None })" }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-query-bank-court-external-rail-server::worth_query_bank_court_external_rail_server$server::ordinary_reads::pending_payments::empty_guarded_frontier_work_grows_at_most_linearly`

CLEANUP-C2-cost-or-growth; owner: bank owning court / recorded cleanup.

Explicit fanout/linear growth court is denied by original seed-memory quota during large-population setup, before the work-bound assertion. Grants and assertions unchanged.

Log: `target/lane-evidence/pc-merge-bank-tier2/test-cost/bank/run-bank.log`.

```text

thread 'server::ordinary_reads::pending_payments::empty_guarded_frontier_work_grows_at_most_linearly' (8360) panicked at crates\worth-query-bank-testkit-server\src\support\mod.rs:79:34:
bank test runtime should install: ApplicationInstallation(ConditionalPublication(WorthQueryConditionalRuntimeInstallationDenial { kind: PrimaryGraphPublication, subject: "primary graph installation denied: ExecutionDenied { kind: ExecutionResource { denial: MemoryLimit { requested: 61963880, admitted: 61961528, level: Policy }, partition_identity: None, policy_ancestor: Some(0) } } (Relational rejected the application seed transaction: commit preparation execution stopped: CommitExecutionDenial { kind: Cause(PolicyMemoryExhausted { requested: 61963880, admitted: 61961528, ancestor: 0 }), partition_identity: None })" }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-store-court-application::worth_store_court_application$process::application::host_oracle::continuation::bounded_host_oracle_seed_one_continues_across_real_process_reopens`

CLEANUP-C1-host-abandonment-convergence; owner: Host oracle / Native-Query recovery.

Reference-prefix entity meaning diverges at step25 reopen after abandonment; comparator remains unchanged.

Log: `target/lane-evidence/pc-merge-store-tier2/run-selected-native.log`.

```text
HOST_ORACLE seed=1 step=0 command=Open
HOST_ORACLE seed=1 step=1 command=Initialize
HOST_ORACLE seed=1 step=2 command=Mutate { branch: Main, key: 10451216379200822465, value: 28529 }
HOST_ORACLE seed=1 step=3 command=Retry { key: 10451216379200822465, value: 28529 }
HOST_ORACLE seed=1 step=4 command=Retry { key: 10451216379200822465, value: 28530 }
HOST_ORACLE seed=1 step=5 command=Mutate { branch: Main, key: 10451216379200822466, value: 90590 }
HOST_ORACLE seed=1 step=6 command=LoseResponse { key: 10451216379200822467, value: 28531 }
HOST_ORACLE seed=1 step=7 command=Reopen
HOST_ORACLE seed=1 step=8 command=Retry { key: 10451216379200822467, value: 28531 }
HOST_ORACLE seed=1 step=9 command=ConcurrentSameKey { key: 10451216379200822468, value: 28535 }
INACTIVE seed=1 step=10 owner=P-C: reviewed codex/m3-p3-lifecycle sibling-route merge command=CreateBranch(Left)
INACTIVE seed=1 step=11 owner=P-C: reviewed codex/m3-p3-lifecycle sibling-route merge command=CreateBranch(Right)
HOST_ORACLE seed=1 step=12 command=MoveProduct { branch: Main, key: 10451216379200822469, value: 80235 }
HOST_ORACLE seed=1 step=13 command=Mutate { branch: Main, key: 10451216379200822470, value: 68761 }
INACTIVE seed=1 step=14 owner=P-C: reviewed codex/m3-p3-lifecycle sibling-route merge command=MoveProduct { branch: Left, key: 10451216379200822471, value: 30048 }
INACTIVE seed=1 step=15 owner=P-C: reviewed codex/m3-p3-lifecycle sibling-route merge command=Mutate { branch: Left, key: 10451216379200822472, value: 67045 }
INACTIVE seed=1 step=16 owner=P-C: reviewed codex/m3-p3-lifecycle sibling-route merge command=MoveProduct { branch: Right, key: 10451216379200822473, value: 60533 }
INACTIVE seed=1 step=17 owner=P-C: reviewed codex/m3-p3-lifecycle sibling-route merge command=Mutate { branch: Right, key: 10451216379200822474, value: 56520 }
HOST_ORACLE seed=1 step=18 command=SettlementFailure { key: 10451216379200822475, value: 28532 }
HOST_ORACLE seed=1 step=19 command=Reopen
HOST_ORACLE seed=1 step=20 command=ContinueCustody { key: 10451216379200822475, value: 28532 }
HOST_ORACLE seed=1 step=21 command=AbandonCaller { key: 10451216379200822476, value: 28533 }

thread 'process::application::host_oracle::continuation::bounded_host_oracle_seed_one_continues_across_real_process_reopens' (6320) panicked at C:\programming\worth worktees\worktrees\m3-host-home\crates\worth-runtime-world\src\lifecycle\operation_control.rs:164:17:
injected unwind before product comparison
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
HOST_ORACLE seed=1 step=22 command=Reopen
HOST_ORACLE seed=1 step=23 command=ReleaseCustody
HOST_ORACLE seed=1 step=24 command=AbandonCaller { key: 10451216379200822477, value: 28534 }
HOST_ORACLE seed=1 step=25 command=Reopen

thread 'process::application::host_oracle::continuation::bounded_host_oracle_seed_one_continues_across_real_process_reopens' (6320) panicked at crates\worth-store-court-application\tests\worth_store_court_applicatio
[Projection abbreviated here; full original output remains in the cited log.]
, 199, 137, 52, 77, 213, 139, 14, 13, 101, 166, 59, 193, 73, 57, 149, 173, 199, 216, 240, 230, 3, 98, 237, 70])]), ("source", []), ("operation", [CanonicalDigestId([209, 64, 92, 6, 37, 30, 36, 115, 163, 84, 243, 131, 218, 230, 93, 110, 72, 206, 129, 59, 109, 85, 125, 81, 87, 152, 47, 136, 243, 180, 0, 109])]), ("scope", [CanonicalDigestId([83, 253, 172, 142, 71, 107, 232, 82, 217, 19, 38, 192, 12, 148, 56, 115, 51, 129, 117, 217, 165, 69, 5, 195, 94, 161, 120, 78, 26, 130, 71, 201]), CanonicalDigestId([84, 255, 71, 185, 55, 68, 0, 144, 205, 10, 52, 125, 160, 25, 110, 17, 32, 152, 202, 124, 228, 4, 189, 26, 252, 144, 234, 52, 57, 25, 103, 243]), CanonicalDigestId([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1])]), ("precondition", []), ("input", []), ("proposal", []), ("conditional-definition", []), ("mutation-binding", [CanonicalDigestId([50, 89, 131, 247, 144, 49, 63, 199, 47, 206, 53, 131, 243, 201, 205, 239, 98, 187, 169, 109, 187, 135, 214, 177, 89, 18, 252, 170, 48, 72, 234, 210])])], outcome: 6, emitted: 0, settlement: 1 }, IdempotencyProjection { entity: RecordId { partition_id: PartitionId(7), local_slot: LocalSlot(0), generation: Generation(1), _marker: PhantomData<worth_relational::identity::data::record_ids::EntityDomain> }, created: VersionId(9), key: CanonicalDigestId([96, 89, 255, 71, 44, 7, 41, 185, 255, 12, 170, 201, 121, 252, 111, 229, 22, 22, 206, 160, 212, 204, 98, 0, 165, 80, 228, 251, 235, 121, 243, 191]), intent: [("root", [CanonicalDigestId([58, 59, 216, 178, 130, 118, 48, 181, 34, 200, 4, 31, 142, 125, 1, 232, 145, 179, 98, 201, 37, 53, 65, 82, 91, 7, 84, 234, 152, 127, 133, 51])]), ("source", []), ("operation", [CanonicalDigestId([209, 64, 92, 6, 37, 30, 36, 115, 163, 84, 243, 131, 218, 230, 93, 110, 72, 206, 129, 59, 109, 85, 125, 81, 87, 152, 47, 136, 243, 180, 0, 109])]), ("scope", [CanonicalDigestId([83, 253, 172, 142, 71, 107, 232, 82, 217, 19, 38, 192, 12, 148, 56, 115, 51, 129, 117, 217, 165, 69, 5, 195, 94, 161, 120, 78, 26, 130, 71, 201]), CanonicalDigestId([84, 255, 71, 185, 55, 68, 0, 144, 205, 10, 52, 125, 160, 25, 110, 17, 32, 152, 202, 124, 228, 4, 189, 26, 252, 144, 234, 52, 57, 25, 103, 243]), CanonicalDigestId([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1])]), ("precondition", []), ("input", []), ("proposal", []), ("conditional-definition", []), ("mutation-binding", [CanonicalDigestId([50, 89, 131, 247, 144, 49, 63, 199, 47, 206, 53, 131, 243, 201, 205, 239, 98, 187, 169, 109, 187, 135, 214, 177, 89, 18, 252, 170, 48, 72, 234, 210])])], outcome: 7, emitted: 0, settlement: 1 }, IdempotencyProjection { entity: RecordId { partition_id: PartitionId(8), local_slot: LocalSlot(0), generation: Generation(1), _marker: PhantomData<worth_relational::identity::data::record_ids::EntityDomain> }, created: VersionId(10), key: CanonicalDigestId([53, 129, 22, 131, 30, 54, 53, 223, 35, 126, 126, 88, 49, 215, 91, 132, 16, 104, 223, 117, 253, 255, 145, 163, 62, 58, 174, 19, 200, 169, 119, 142]), intent: [("root", [CanonicalDigestId([104, 222, 132, 47, 213, 70, 235, 215, 169, 106, 249, 176, 131, 51, 57, 163, 235, 156, 18, 2, 226, 63, 147, 171, 163, 252, 190, 65, 41, 113, 47, 57])]), ("source", []), ("operation", [CanonicalDigestId([209, 64, 92, 6, 37, 30, 36, 115, 163, 84, 243, 131, 218, 230, 93, 110, 72, 206, 129, 59, 109, 85, 125, 81, 87, 152, 47, 136, 243, 180, 0, 109])]), ("scope", [CanonicalDigestId([83, 253, 172, 142, 71, 107, 232, 82, 217, 19, 38, 192, 12, 148, 56, 115, 51, 129, 117, 217, 165, 69, 5, 195, 94, 161, 120, 78, 26, 130, 71, 201]), CanonicalDigestId([84, 255, 71, 185, 55, 68, 0, 144, 205, 10, 52, 125, 160, 25, 110, 17, 32, 152, 202, 124, 228, 4, 189, 26, 252, 144, 234, 52, 57, 25, 103, 243]), CanonicalDigestId([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1])]), ("precondition", []), ("input", []), ("proposal", []), ("conditional-definition", []), ("mutation-binding", [CanonicalDigestId([50, 89, 131, 247, 144, 49, 63, 199, 47, 206, 53, 131, 243, 201, 205, 239, 98, 187, 169, 109, 187, 135, 214, 177, 89, 18, 252, 170, 48, 72, 234, 210])])], outcome: 8, emitted: 0, settlement: 1 }, IdempotencyProjection { entity: RecordId { partition_id: PartitionId(9), local_slot: LocalSlot(0), generation: Generation(1), _marker: PhantomData<worth_relational::identity::data::record_ids::EntityDomain> }, created: VersionId(11), key: CanonicalDigestId([171, 120, 101, 36, 57, 32, 172, 37, 79, 231, 255, 208, 101, 235, 238, 188, 158, 162, 61, 236, 249, 79, 140, 175, 230, 194, 139, 225, 102, 16, 47, 238]), intent: [("root", [CanonicalDigestId([115, 161, 63, 141, 30, 32, 95, 93, 212, 227, 105, 14, 116, 17, 221, 197, 63, 159, 39, 127, 131, 248, 87, 49, 253, 11, 135, 31, 43, 53, 104, 63])]), ("source", []), ("operation", [CanonicalDigestId([209, 64, 92, 6, 37, 30, 36, 115, 163, 84, 243, 131, 218, 230, 93, 110, 72, 206, 129, 59, 109, 85, 125, 81, 87, 152, 47, 136, 243, 180, 0, 109])]), ("scope", [CanonicalDigestId([83, 253, 172, 142, 71, 107, 232, 82, 217, 19, 38, 192, 12, 148, 56, 115, 51, 129, 117, 217, 165, 69, 5, 195, 94, 161, 120, 78, 26, 130, 71, 201]), CanonicalDigestId([84, 255, 71, 185, 55, 68, 0, 144, 205, 10, 52, 125, 160, 25, 110, 17, 32, 152, 202, 124, 228, 4, 189, 26, 252, 144, 234, 52, 57, 25, 103, 243]), CanonicalDigestId([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1])]), ("precondition", []), ("input", []), ("proposal", []), ("conditional-definition", []), ("mutation-binding", [CanonicalDigestId([50, 89, 131, 247, 144, 49, 63, 199, 47, 206, 53, 131, 243, 201, 205, 239, 98, 187, 169, 109, 187, 135, 214, 177, 89, 18, 252, 170, 48, 72, 234, 210])])], outcome: 9, emitted: 0, settlement: 1 }], history_complete: false, history_crossed_retired: true, entities: [GraphEntityProjection { entity_id: RecordId { partition_id: PartitionId(0), local_slot: LocalSlot(1), generation: Generation(1), _marker: PhantomData<worth_relational::identity::data::record_ids::EntityDomain> }, lineage_id: Some(LineageId(2)), kind_name: "Account", lifecycle: Live, created_at_version: VersionId(2), retired_at_version: None, authoritative_aspect_state: Object {"AccountIdentity": Object {"identity": Number(2439057472), "revision": Number(1), "value": Object {"Struct": Object {"AccountNumber": Object {"String": Object {"Raw": String("account-001")}}, "Balance": Object {"UInt64": Number(28533)}}}}} }, GraphEntityProjection { entity_id: RecordId { partition_id: PartitionId(0), local_slot: LocalSlot(2), generation: Generation(1), _marker: PhantomData<worth_relational::identity::data::record_ids::EntityDomain> }, lineage_id: Some(LineageId(3)), kind_name: "ExternalMapping", lifecycle: Live, created_at_version: VersionId(2), retired_at_version: None, authoritative_aspect_state: Object {"ExternalIdentity": Object {"identity": Number(2439057470), "revision": Number(1), "value": Object {"Struct": Object {"ExternalIdentityField": Object {"String": Object {"Raw": String("22:https://issuer.example7:subject")}}, "MappingStatusField": Object {"Bool": Bool(true)}}}}} }, GraphEntityProjection { entity_id: RecordId { partition_id: PartitionId(0), local_slot: LocalSlot(3), generation: Generation(1), _marker: PhantomData<worth_relational::identity::data::record_ids::EntityDomain> }, lineage_id: Some(LineageId(4)), kind_name: "Principal", lifecycle: Live, created_at_version: VersionId(2), retired_at_version: None, authoritative_aspect_state: Object {"PrincipalIdentity": Object {"identity": Number(2439057471), "revision": Number(1), "value": Object {"Struct": Object {"PrincipalIdentityField": Object {"UInt64": Number(1)}}}}} }], relations: [GraphRelationProjection { relation_id: RecordId { partition_id: PartitionId(0), local_slot: LocalSlot(0), generation: Generation(1), _marker: PhantomData<worth_relational::identity::data::record_ids::RelationDomain> }, source: RecordId { partition_id: PartitionId(0), local_slot: LocalSlot(2), generation: Generation(1), _marker: PhantomData<worth_relational::identity::data::record_ids::EntityDomain> }, target: RecordId { partition_id: PartitionId(0), local_slot: LocalSlot(3), generation: Generation(1), _marker: PhantomData<worth_relational::identity::data::record_ids::EntityDomain> }, kind_name: "MappingTarget", lifecycle: Live, created_at_version: VersionId(2), retired_at_version: None, authoritative_aspect_state: Null }], head: HeadProjection { owner: MemoryOnly { observation: 0 }, branch: "primary", incarnation: 0, generation: 7, commit: 7 }, occurrences: [OccurrenceProjection { owner: MemoryOnly { observation: 0 }, commit: 7, parent: Some(6), provenance: Publication(6), relational_publication: Some(RelationalCommitIdentity { commit_id: CommitId(10), version_id: VersionId(10), authoring_branch: BranchId("main") }), relational_fork: None, relational_changed: true, signal_changed: false, signal_movement: None }], custody: [] }

```

## `worth-store-court-application::worth_store_court_application$process::application::host_oracle::continuation::bounded_host_oracle_seed_zero_continues_across_real_process_reopens`

CLEANUP-C1-host-abandonment-convergence; owner: Host oracle / Native-Query recovery.

Reference-prefix entity meaning diverges at step25 reopen after abandonment; comparator remains unchanged.

Log: `target/lane-evidence/pc-merge-store-tier2/run-selected-native.log`.

```text
HOST_ORACLE seed=0 step=0 command=Open
HOST_ORACLE seed=0 step=1 command=Initialize
HOST_ORACLE seed=0 step=2 command=Mutate { branch: Main, key: 16294208416658607535, value: 55710 }
HOST_ORACLE seed=0 step=3 command=Retry { key: 16294208416658607535, value: 55710 }
HOST_ORACLE seed=0 step=4 command=Retry { key: 16294208416658607535, value: 55711 }
HOST_ORACLE seed=0 step=5 command=Mutate { branch: Main, key: 16294208416658607536, value: 45679 }
HOST_ORACLE seed=0 step=6 command=LoseResponse { key: 16294208416658607537, value: 55712 }
HOST_ORACLE seed=0 step=7 command=Reopen
HOST_ORACLE seed=0 step=8 command=Retry { key: 16294208416658607537, value: 55712 }
HOST_ORACLE seed=0 step=9 command=ConcurrentSameKey { key: 16294208416658607538, value: 55716 }
INACTIVE seed=0 step=10 owner=P-C: reviewed codex/m3-p3-lifecycle sibling-route merge command=CreateBranch(Left)
INACTIVE seed=0 step=11 owner=P-C: reviewed codex/m3-p3-lifecycle sibling-route merge command=CreateBranch(Right)
HOST_ORACLE seed=0 step=12 command=MoveProduct { branch: Main, key: 16294208416658607539, value: 42444 }
HOST_ORACLE seed=0 step=13 command=Mutate { branch: Main, key: 16294208416658607540, value: 94747 }
INACTIVE seed=0 step=14 owner=P-C: reviewed codex/m3-p3-lifecycle sibling-route merge command=MoveProduct { branch: Left, key: 16294208416658607541, value: 62090 }
INACTIVE seed=0 step=15 owner=P-C: reviewed codex/m3-p3-lifecycle sibling-route merge command=Mutate { branch: Left, key: 16294208416658607542, value: 6913 }
INACTIVE seed=0 step=16 owner=P-C: reviewed codex/m3-p3-lifecycle sibling-route merge command=MoveProduct { branch: Right, key: 16294208416658607543, value: 46940 }
INACTIVE seed=0 step=17 owner=P-C: reviewed codex/m3-p3-lifecycle sibling-route merge command=Mutate { branch: Right, key: 16294208416658607544, value: 23299 }
HOST_ORACLE seed=0 step=18 command=SettlementFailure { key: 16294208416658607545, value: 55713 }
HOST_ORACLE seed=0 step=19 command=Reopen
HOST_ORACLE seed=0 step=20 command=ContinueCustody { key: 16294208416658607545, value: 55713 }
HOST_ORACLE seed=0 step=21 command=AbandonCaller { key: 16294208416658607546, value: 55714 }

thread 'process::application::host_oracle::continuation::bounded_host_oracle_seed_zero_continues_across_real_process_reopens' (82568) panicked at C:\programming\worth worktees\worktrees\m3-host-home\crates\worth-runtime-world\src\lifecycle\operation_control.rs:164:17:
injected unwind before product comparison
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
HOST_ORACLE seed=0 step=22 command=Reopen
HOST_ORACLE seed=0 step=23 command=ReleaseCustody
HOST_ORACLE seed=0 step=24 command=AbandonCaller { key: 16294208416658607547, value: 55715 }
HOST_ORACLE seed=0 step=25 command=Reopen

thread 'process::application::host_oracle::continuation::bounded_host_oracle_seed_zero_continues_across_real_process_reopens' (82568) panicked at crates\worth-store-court-application\tests\worth_store_court_applica
[Projection abbreviated here; full original output remains in the cited log.]
215, 15, 211, 10, 197, 120, 187, 103, 252, 233, 203, 218, 78, 42, 2, 131, 100, 45, 183, 76, 148, 99, 116, 228, 32, 208])]), ("source", []), ("operation", [CanonicalDigestId([209, 64, 92, 6, 37, 30, 36, 115, 163, 84, 243, 131, 218, 230, 93, 110, 72, 206, 129, 59, 109, 85, 125, 81, 87, 152, 47, 136, 243, 180, 0, 109])]), ("scope", [CanonicalDigestId([83, 253, 172, 142, 71, 107, 232, 82, 217, 19, 38, 192, 12, 148, 56, 115, 51, 129, 117, 217, 165, 69, 5, 195, 94, 161, 120, 78, 26, 130, 71, 201]), CanonicalDigestId([84, 255, 71, 185, 55, 68, 0, 144, 205, 10, 52, 125, 160, 25, 110, 17, 32, 152, 202, 124, 228, 4, 189, 26, 252, 144, 234, 52, 57, 25, 103, 243]), CanonicalDigestId([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1])]), ("precondition", []), ("input", []), ("proposal", []), ("conditional-definition", []), ("mutation-binding", [CanonicalDigestId([50, 89, 131, 247, 144, 49, 63, 199, 47, 206, 53, 131, 243, 201, 205, 239, 98, 187, 169, 109, 187, 135, 214, 177, 89, 18, 252, 170, 48, 72, 234, 210])])], outcome: 6, emitted: 0, settlement: 1 }, IdempotencyProjection { entity: RecordId { partition_id: PartitionId(7), local_slot: LocalSlot(0), generation: Generation(1), _marker: PhantomData<worth_relational::identity::data::record_ids::EntityDomain> }, created: VersionId(9), key: CanonicalDigestId([71, 9, 163, 62, 131, 141, 199, 35, 58, 103, 84, 131, 33, 9, 22, 181, 21, 176, 246, 6, 220, 141, 250, 12, 133, 167, 72, 13, 237, 112, 52, 35]), intent: [("root", [CanonicalDigestId([248, 118, 180, 173, 74, 89, 92, 196, 211, 13, 183, 95, 172, 110, 195, 44, 9, 55, 62, 92, 41, 222, 24, 159, 142, 9, 232, 2, 53, 158, 227, 150])]), ("source", []), ("operation", [CanonicalDigestId([209, 64, 92, 6, 37, 30, 36, 115, 163, 84, 243, 131, 218, 230, 93, 110, 72, 206, 129, 59, 109, 85, 125, 81, 87, 152, 47, 136, 243, 180, 0, 109])]), ("scope", [CanonicalDigestId([83, 253, 172, 142, 71, 107, 232, 82, 217, 19, 38, 192, 12, 148, 56, 115, 51, 129, 117, 217, 165, 69, 5, 195, 94, 161, 120, 78, 26, 130, 71, 201]), CanonicalDigestId([84, 255, 71, 185, 55, 68, 0, 144, 205, 10, 52, 125, 160, 25, 110, 17, 32, 152, 202, 124, 228, 4, 189, 26, 252, 144, 234, 52, 57, 25, 103, 243]), CanonicalDigestId([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1])]), ("precondition", []), ("input", []), ("proposal", []), ("conditional-definition", []), ("mutation-binding", [CanonicalDigestId([50, 89, 131, 247, 144, 49, 63, 199, 47, 206, 53, 131, 243, 201, 205, 239, 98, 187, 169, 109, 187, 135, 214, 177, 89, 18, 252, 170, 48, 72, 234, 210])])], outcome: 7, emitted: 0, settlement: 1 }, IdempotencyProjection { entity: RecordId { partition_id: PartitionId(8), local_slot: LocalSlot(0), generation: Generation(1), _marker: PhantomData<worth_relational::identity::data::record_ids::EntityDomain> }, created: VersionId(10), key: CanonicalDigestId([48, 169, 247, 139, 143, 174, 186, 72, 66, 43, 150, 150, 90, 50, 131, 186, 210, 201, 149, 66, 13, 237, 76, 178, 247, 253, 197, 174, 111, 147, 119, 100]), intent: [("root", [CanonicalDigestId([61, 47, 227, 18, 103, 166, 74, 149, 141, 202, 1, 217, 207, 214, 2, 79, 227, 32, 141, 106, 23, 237, 16, 212, 195, 16, 72, 236, 94, 77, 63, 161])]), ("source", []), ("operation", [CanonicalDigestId([209, 64, 92, 6, 37, 30, 36, 115, 163, 84, 243, 131, 218, 230, 93, 110, 72, 206, 129, 59, 109, 85, 125, 81, 87, 152, 47, 136, 243, 180, 0, 109])]), ("scope", [CanonicalDigestId([83, 253, 172, 142, 71, 107, 232, 82, 217, 19, 38, 192, 12, 148, 56, 115, 51, 129, 117, 217, 165, 69, 5, 195, 94, 161, 120, 78, 26, 130, 71, 201]), CanonicalDigestId([84, 255, 71, 185, 55, 68, 0, 144, 205, 10, 52, 125, 160, 25, 110, 17, 32, 152, 202, 124, 228, 4, 189, 26, 252, 144, 234, 52, 57, 25, 103, 243]), CanonicalDigestId([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1])]), ("precondition", []), ("input", []), ("proposal", []), ("conditional-definition", []), ("mutation-binding", [CanonicalDigestId([50, 89, 131, 247, 144, 49, 63, 199, 47, 206, 53, 131, 243, 201, 205, 239, 98, 187, 169, 109, 187, 135, 214, 177, 89, 18, 252, 170, 48, 72, 234, 210])])], outcome: 8, emitted: 0, settlement: 1 }, IdempotencyProjection { entity: RecordId { partition_id: PartitionId(9), local_slot: LocalSlot(0), generation: Generation(1), _marker: PhantomData<worth_relational::identity::data::record_ids::EntityDomain> }, created: VersionId(11), key: CanonicalDigestId([216, 81, 194, 195, 247, 116, 26, 53, 106, 245, 6, 133, 156, 144, 181, 141, 117, 44, 53, 212, 128, 3, 163, 231, 100, 206, 174, 186, 208, 104, 146, 127]), intent: [("root", [CanonicalDigestId([20, 139, 98, 66, 219, 243, 44, 45, 148, 190, 76, 58, 150, 134, 86, 7, 13, 225, 190, 25, 119, 61, 48, 70, 194, 37, 36, 33, 142, 16, 172, 88])]), ("source", []), ("operation", [CanonicalDigestId([209, 64, 92, 6, 37, 30, 36, 115, 163, 84, 243, 131, 218, 230, 93, 110, 72, 206, 129, 59, 109, 85, 125, 81, 87, 152, 47, 136, 243, 180, 0, 109])]), ("scope", [CanonicalDigestId([83, 253, 172, 142, 71, 107, 232, 82, 217, 19, 38, 192, 12, 148, 56, 115, 51, 129, 117, 217, 165, 69, 5, 195, 94, 161, 120, 78, 26, 130, 71, 201]), CanonicalDigestId([84, 255, 71, 185, 55, 68, 0, 144, 205, 10, 52, 125, 160, 25, 110, 17, 32, 152, 202, 124, 228, 4, 189, 26, 252, 144, 234, 52, 57, 25, 103, 243]), CanonicalDigestId([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1])]), ("precondition", []), ("input", []), ("proposal", []), ("conditional-definition", []), ("mutation-binding", [CanonicalDigestId([50, 89, 131, 247, 144, 49, 63, 199, 47, 206, 53, 131, 243, 201, 205, 239, 98, 187, 169, 109, 187, 135, 214, 177, 89, 18, 252, 170, 48, 72, 234, 210])])], outcome: 9, emitted: 0, settlement: 1 }], history_complete: false, history_crossed_retired: true, entities: [GraphEntityProjection { entity_id: RecordId { partition_id: PartitionId(0), local_slot: LocalSlot(1), generation: Generation(1), _marker: PhantomData<worth_relational::identity::data::record_ids::EntityDomain> }, lineage_id: Some(LineageId(2)), kind_name: "Account", lifecycle: Live, created_at_version: VersionId(2), retired_at_version: None, authoritative_aspect_state: Object {"AccountIdentity": Object {"identity": Number(2439057472), "revision": Number(1), "value": Object {"Struct": Object {"AccountNumber": Object {"String": Object {"Raw": String("account-001")}}, "Balance": Object {"UInt64": Number(55714)}}}}} }, GraphEntityProjection { entity_id: RecordId { partition_id: PartitionId(0), local_slot: LocalSlot(2), generation: Generation(1), _marker: PhantomData<worth_relational::identity::data::record_ids::EntityDomain> }, lineage_id: Some(LineageId(3)), kind_name: "ExternalMapping", lifecycle: Live, created_at_version: VersionId(2), retired_at_version: None, authoritative_aspect_state: Object {"ExternalIdentity": Object {"identity": Number(2439057470), "revision": Number(1), "value": Object {"Struct": Object {"ExternalIdentityField": Object {"String": Object {"Raw": String("22:https://issuer.example7:subject")}}, "MappingStatusField": Object {"Bool": Bool(true)}}}}} }, GraphEntityProjection { entity_id: RecordId { partition_id: PartitionId(0), local_slot: LocalSlot(3), generation: Generation(1), _marker: PhantomData<worth_relational::identity::data::record_ids::EntityDomain> }, lineage_id: Some(LineageId(4)), kind_name: "Principal", lifecycle: Live, created_at_version: VersionId(2), retired_at_version: None, authoritative_aspect_state: Object {"PrincipalIdentity": Object {"identity": Number(2439057471), "revision": Number(1), "value": Object {"Struct": Object {"PrincipalIdentityField": Object {"UInt64": Number(1)}}}}} }], relations: [GraphRelationProjection { relation_id: RecordId { partition_id: PartitionId(0), local_slot: LocalSlot(0), generation: Generation(1), _marker: PhantomData<worth_relational::identity::data::record_ids::RelationDomain> }, source: RecordId { partition_id: PartitionId(0), local_slot: LocalSlot(2), generation: Generation(1), _marker: PhantomData<worth_relational::identity::data::record_ids::EntityDomain> }, target: RecordId { partition_id: PartitionId(0), local_slot: LocalSlot(3), generation: Generation(1), _marker: PhantomData<worth_relational::identity::data::record_ids::EntityDomain> }, kind_name: "MappingTarget", lifecycle: Live, created_at_version: VersionId(2), retired_at_version: None, authoritative_aspect_state: Null }], head: HeadProjection { owner: MemoryOnly { observation: 0 }, branch: "primary", incarnation: 0, generation: 7, commit: 7 }, occurrences: [OccurrenceProjection { owner: MemoryOnly { observation: 0 }, commit: 7, parent: Some(6), provenance: Publication(6), relational_publication: Some(RelationalCommitIdentity { commit_id: CommitId(10), version_id: VersionId(10), authoring_branch: BranchId("main") }), relational_fork: None, relational_changed: true, signal_changed: false, signal_movement: None }], custody: [] }

```

## `worth-store-court-application::worth_store_court_application$process::application::no_pending_owners::transition_catalog_readmission::a_completed_transition_occurrence_restores_target_contracts_and_continues_only_its_missing_suffix`

CLEANUP-C2-zero-budget-transition-boundary; owner: Store transition court.

Explicit zero-memory admission now refuses earlier at authenticated schema image; normal B reopen is green.

Log: `target/lane-evidence/pc-merge-store-tier2/run-selected-native.log`.

```text

thread 'process::application::no_pending_owners::transition_catalog_readmission::a_completed_transition_occurrence_restores_target_contracts_and_continues_only_its_missing_suffix' (62068) panicked at crates\worth-store-court-application\tests\worth_store_court_application\process\application\no_pending_owners\transition_catalog_readmission\request_controls.rs:45:18:
the original zero budget must refuse backing: TargetMeaning(SchemaImage(Request(PolicyMemoryExhausted { requested: 400, admitted: 0, ancestor: 0 })))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-store-court-application::worth_store_court_application$process::application::query_readmission::bootstrap_publication::retired_format::retired_bootstrap_completion_format_refuses_existing_home_without_replaying_seeds`

CLEANUP-C1-retired-format-refusal-shape; owner: store owning court / recorded cleanup.

Deliberately altered completion record is correctly refused by D8 quarantined primary-graph denial; court expects old ExistingQueryAttachment shape. No service conversion restored.

Log: `target/lane-evidence/pc-merge-store-tier2/run-selected-native.log`.

```text

thread 'process::application::query_readmission::bootstrap_publication::retired_format::retired_bootstrap_completion_format_refuses_existing_home_without_replaying_seeds' (29424) panicked at crates\worth-store-court-application\tests\worth_store_court_application\process\application\query_readmission\bootstrap_publication\retired_format.rs:103:9:
actual selected Query completion must refuse: WorthQueryApplicationHomeOpenRefused("primary graph installation denied: CheckpointRecoveryRejected (Home bootstrap completion format is unsupported)")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-store-court-application::worth_store_court_application$process::application::world_occurrence_readmission::late_history_refusal_retains_actual_partial_root_caps_and_home_lease_until_cleanup`

CLEANUP-C1-injected-recovery-custody; owner: Host recovery cleanup.

Injected late-history refusal exercises partial opening cleanup, not a graceful service journey.

Log: `target/lane-evidence/pc-merge-store-tier2/run-selected-native.log`.

```text

thread 'process::application::world_occurrence_readmission::late_history_refusal_retains_actual_partial_root_caps_and_home_lease_until_cleanup' (68776) panicked at crates\worth-store-court-application\tests\worth_store_court_application\process\application\world_occurrence_readmission.rs:349:5:
assertion failed: matches!(refused.into_home().close(&execution),
    ApplicationHomeCloseOutcome::Closed
    {
        checkpoint:
        Err(ApplicationHomeCheckpointDenial::WorldSectionRequired), ..
    })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-store-court-application::worth_store_court_application$process::relational::relational_home::incarnation::anchor_replay::wal_replays_the_thirty_second_anchor_advance_after_process_death`

CLEANUP-C1-C2-anchor-growth-kill; owner: store owning court / recorded cleanup.

32-anchor/high-generation process-death control hits AnchorTableFull before WAL pause; no successful kill/replay proof.

Log: `target/lane-evidence/pc-merge-store-tier2/run-selected-native.log`.

```text

thread 'process::relational::relational_home::incarnation::anchor_replay::wal_replays_the_thirty_second_anchor_advance_after_process_death' (80508) panicked at crates\worth-store-court-application\tests\worth_store_court_application\support\killed_child\declarations.rs:62:13:
child process::relational::relational_home::incarnation::anchor_replay::high_anchor_child ended without parking: 
running 1 test
test process::relational::relational_home::incarnation::anchor_replay::high_anchor_child ... FAILED

failures:

failures:
    process::relational::relational_home::incarnation::anchor_replay::high_anchor_child

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 192 filtered out; finished in 2.95s



thread 'process::relational::relational_home::incarnation::anchor_replay::high_anchor_child' (58772) panicked at crates\worth-store-court-application\tests\worth_store_court_application\support\incarnation\anchor_replay\declarations.rs:34:10:
called `Result::unwrap()` on an `Err` value: AnchorTableFull
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-store-court-application::worth_store_court_application$process::relational::relational_home::open_refusals::a_checkpoint_record_store_accepts_but_relational_cannot_readmit_refuses`

CLEANUP-C1-worldless-damage-fixture; owner: store owning court / recorded cleanup.

Damage/refusal court stops at benign WorldSectionRequired setup before fault assertion. Cleanup must compose actual World and preserve chosen checkpoint/tail/abort; no fake bytes or graceful-close substitution.

Log: `target/lane-evidence/pc-merge-store-tier2/run-selected-native.log`.

```text

thread 'process::relational::relational_home::open_refusals::a_checkpoint_record_store_accepts_but_relational_cannot_readmit_refuses' (82108) panicked at crates\worth-store-court-application\tests\worth_store_court_application\support\durable_run\declarations.rs:111:36:
home checkpoint: WorldSectionRequired
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-store-court-application::worth_store_court_application$process::relational::relational_home::open_refusals::a_damaged_checkpoint_in_the_store_files_refuses_before_service`

CLEANUP-C1-worldless-damage-fixture; owner: store owning court / recorded cleanup.

Damage/refusal court stops at benign WorldSectionRequired setup before fault assertion. Cleanup must compose actual World and preserve chosen checkpoint/tail/abort; no fake bytes or graceful-close substitution.

Log: `target/lane-evidence/pc-merge-store-tier2/run-selected-native.log`.

```text

thread 'process::relational::relational_home::open_refusals::a_damaged_checkpoint_in_the_store_files_refuses_before_service' (22104) panicked at crates\worth-store-court-application\tests\worth_store_court_application\support\durable_run\declarations.rs:111:36:
home checkpoint: WorldSectionRequired
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-store-court-application::worth_store_court_application$process::relational::relational_home::open_refusals::a_damaged_journal_entry_in_the_store_files_refuses_before_service`

CLEANUP-C1-worldless-damage-fixture; owner: store owning court / recorded cleanup.

Damage/refusal court stops at benign WorldSectionRequired setup before fault assertion. Cleanup must compose actual World and preserve chosen checkpoint/tail/abort; no fake bytes or graceful-close substitution.

Log: `target/lane-evidence/pc-merge-store-tier2/run-selected-native.log`.

```text

thread 'process::relational::relational_home::open_refusals::a_damaged_journal_entry_in_the_store_files_refuses_before_service' (51648) panicked at crates\worth-store-court-application\tests\worth_store_court_application\support\durable_run\declarations.rs:111:36:
home checkpoint: WorldSectionRequired
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-store-court-application::worth_store_court_application$process::relational::relational_home::open_refusals::a_journal_entry_store_accepts_but_relational_cannot_readmit_refuses`

CLEANUP-C1-worldless-damage-fixture; owner: store owning court / recorded cleanup.

Damage/refusal court stops at benign WorldSectionRequired setup before fault assertion. Cleanup must compose actual World and preserve chosen checkpoint/tail/abort; no fake bytes or graceful-close substitution.

Log: `target/lane-evidence/pc-merge-store-tier2/run-selected-native.log`.

```text

thread 'process::relational::relational_home::open_refusals::a_journal_entry_store_accepts_but_relational_cannot_readmit_refuses' (11800) panicked at crates\worth-store-court-application\tests\worth_store_court_application\support\durable_run\declarations.rs:111:36:
home checkpoint: WorldSectionRequired
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-store-court-application::worth_store_court_application$process::application::query_readmission::native_artifacts::home_close::host_checkpoint_memory_failure_is_reported_while_original_retired_owners_remain_physically_closed`

CLEANUP-C2-close-memory-boundary; owner: store owning court / recorded cleanup.

Fixed1.5MiB capture now succeeds; declared-failure assertion remains red without grant/claim weakening.

Log: `target/lane-evidence/pc-merge-store-tier2/run-selected-native.log`.

```text

thread 'process::application::query_readmission::native_artifacts::home_close::host_checkpoint_memory_failure_is_reported_while_original_retired_owners_remain_physically_closed' (42100) panicked at crates\worth-store-court-application\tests\worth_store_court_application\process\application\query_readmission\native_artifacts\home_close.rs:178:14:
this declared memory ceiling must exercise the real capture failure
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-store-court-application::worth_store_court_application$process::relational::relational_home::publication_refusals::a_commit_the_medium_refuses_before_any_effect_is_denied_and_not_installed`

CLEANUP-C1-after-lifecycle-termination; owner: store owning court / recorded cleanup.

SourceUnavailable artifact snapshot occurs after explicit lifecycle termination injection atpublication_refusals/mod.rs94-118, not before fault.

Log: `target/lane-evidence/pc-merge-store-tier2/run-selected-native.log`.

```text

thread 'process::relational::relational_home::publication_refusals::a_commit_the_medium_refuses_before_any_effect_is_denied_and_not_installed' (39760) panicked at crates\worth-store-court-application\tests\worth_store_court_application\support\workload\declarations.rs:280:18:
retained commits encode: StorageUnavailable(RelationalStorageUnavailable { subject: Envelope(PatchStreamPosition(1)), cause: SourceUnavailable })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-store-court-application::worth_store_court_application$process::relational::relational_home::residency_agreement::cold_warm_and_full_home_reads_and_retained_observations_agree_with_memory`

CLEANUP-C2-cold-member-read-bound; owner: store owning court / recorded cleanup.

Ordinary semantic residency/reopen journey uses Native-only Home checkpoint, which correctly requires actual World section. Existing durable_run::world owner provides fresh/reopen/bootstrap/close; preserve semantic and later cost assertions.

Log: `target/lane-evidence/pc-merge-store-tier2/run-selected-native.log`.

```text

thread 'process::relational::relational_home::residency_agreement::cold_warm_and_full_home_reads_and_retained_observations_agree_with_memory' (59320) panicked at crates\worth-store-court-application\tests\worth_store_court_application\process\relational\relational_home\residency_agreement\mod.rs:106:5:
assertion `left == right` failed: canonical patch history needs no dictionary or lineage member
  left: 1
 right: 0
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-store-court-application::worth_store_court_application$process::relational::relational_home::residency_capture::a_one_partition_commit_publishes_constant_new_members_and_capture_reads_nothing`

CLEANUP-C2-worldless-capture-count-fixture; owner: store owning court / recorded cleanup.

Constant-new-member/load-free capture court stops at WorldSectionRequired setup; actual count assertions unreached and unchanged.

Log: `target/lane-evidence/pc-merge-store-tier2/run-selected-native.log`.

```text

thread 'process::relational::relational_home::residency_capture::a_one_partition_commit_publishes_constant_new_members_and_capture_reads_nothing' (41668) panicked at crates\worth-store-court-application\tests\worth_store_court_application\process\relational\relational_home\residency_capture\mod.rs:28:33:
called `Result::unwrap()` on an `Err` value: WorldSectionRequired
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-store-court-application::worth_store_court_application$process::relational::relational_home::residency_reopen::a_cold_schema_transition_replays_and_reframes_all_carriers_before_load_free_capture`

CLEANUP-C2-cold-member-read-bound; owner: store owning court / recorded cleanup.

Ordinary semantic residency/reopen journey uses Native-only Home checkpoint, which correctly requires actual World section. Existing durable_run::world owner provides fresh/reopen/bootstrap/close; preserve semantic and later cost assertions.

Log: `target/lane-evidence/pc-merge-store-tier2/run-selected-native.log`.

```text

thread 'process::relational::relational_home::residency_reopen::a_cold_schema_transition_replays_and_reframes_all_carriers_before_load_free_capture' (52752) panicked at crates\worth-store-court-application\tests\worth_store_court_application\process\relational\relational_home\residency_reopen\mod.rs:262:5:
assertion `left == right` failed: carrier reframing must use preparation's admitted contents
  left: RelationalStoreColdReadCounters { member_reads: 1, commit_reads: 0 }
 right: RelationalStoreColdReadCounters { member_reads: 0, commit_reads: 0 }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-store-court-application::worth_store_court_application$process::relational::relational_home::residency_digest::a_checksum_valid_substituted_member_refuses_its_digest_and_keeps_subject_damage_sticky`

CLEANUP-C1-worldless-damage-fixture; owner: store owning court / recorded cleanup.

Damage/refusal court stops at benign WorldSectionRequired setup before fault assertion. Cleanup must compose actual World and preserve chosen checkpoint/tail/abort; no fake bytes or graceful-close substitution.

Log: `target/lane-evidence/pc-merge-store-tier2/run-selected-native.log`.

```text

thread 'process::relational::relational_home::residency_digest::a_checksum_valid_substituted_member_refuses_its_digest_and_keeps_subject_damage_sticky' (16936) panicked at crates\worth-store-court-application\tests\worth_store_court_application\support\durable_run\declarations.rs:111:36:
home checkpoint: WorldSectionRequired
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-store-court-application::worth_store_court_application$process::relational::relational_home::residency_reopen::reopen_keeps_regions_cold_and_a_point_read_and_commit_demand_only_their_partition`

CLEANUP-C2-cold-member-read-bound; owner: store owning court / recorded cleanup.

Ordinary semantic residency/reopen journey uses Native-only Home checkpoint, which correctly requires actual World section. Existing durable_run::world owner provides fresh/reopen/bootstrap/close; preserve semantic and later cost assertions.

Log: `target/lane-evidence/pc-merge-store-tier2/run-selected-native.log`.

```text

thread 'process::relational::relational_home::residency_reopen::reopen_keeps_regions_cold_and_a_point_read_and_commit_demand_only_their_partition' (69564) panicked at crates\worth-store-court-application\tests\worth_store_court_application\process\relational\relational_home\residency_reopen\mod.rs:129:5:
assertion `left == right` failed: capture must walk leaves without demanding payloads
  left: RelationalStoreColdReadCounters { member_reads: 1, commit_reads: 0 }
 right: RelationalStoreColdReadCounters { member_reads: 0, commit_reads: 0 }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-store-court-recovery-runtime-replication-phase-eight-process::worth_store_court_recovery_runtime_replication_phase_eight_process$cost::recovery_runtime::phase_eight_process::production::size_independence::recovery_work_is_independent_of_unrelated_persisted_store_size`

CLEANUP-C2-recovery-growth; owner: Store physical recovery court.

Larger persisted-size twin cannot be built within the bounded residue fixture.

Log: `target/lane-evidence/pc-merge-store-tier2/run-selected-native.log`.

```text
C8 writer seeded 257 operations and two checkpoints in 22.5458236s
C8 writer ready: operations=257, elapsed=23.8796646s, last_phase=no-effect and final checkpoint: 0/1

thread 'cost::recovery_runtime::phase_eight_process::production::size_independence::recovery_work_is_independent_of_unrelated_persisted_store_size' (85968) panicked at crates\worth-store-court-recovery-runtime-replication-phase-eight-process\tests\worth_store_court_recovery_runtime_replication_phase_eight_process\support\phase_eight_process\production\size_independence\declarations.rs:136:9:
bounded residue fixture could not build the larger Store twin
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-store-court-recovery-runtime-replication-phase-eight-process::worth_store_court_recovery_runtime_replication_phase_eight_process$process::recovery_runtime::phase_eight_process::production::recovery_crash::cancelled_recovery_process_reports_a_blocked_post_effect_outcome`

CLEANUP-C1-C2-recovery-interruption-fixture; owner: Store physical recovery court.

Cancellation campaign fixture asserts its footprint exceeds admission before the interruption.

Log: `target/lane-evidence/pc-merge-store-tier2/run-selected-native.log`.

```text

thread 'process::recovery_runtime::phase_eight_process::production::recovery_crash::cancelled_recovery_process_reports_a_blocked_post_effect_outcome' (60840) panicked at crates\worth-store-court-recovery-runtime-replication-phase-eight-process\tests\worth_store_court_recovery_runtime_replication_phase_eight_process\support\phase_eight_process\history\writer_history\expected_history\declarations.rs:26:9:
C8 fixture must exceed its admitted recovery memory budget
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `worth-store-court-recovery-runtime-replication-phase-eight-process::worth_store_court_recovery_runtime_replication_phase_eight_process$process::recovery_runtime::phase_eight_process::production::recovery_crash::killed_recovery_process_reopens_after_each_named_recovery_seam`

CLEANUP-C1-C2-recovery-interruption-fixture; owner: Store physical recovery court.

Kill-at-recovery-seams fixture asserts its footprint exceeds admission before the interruption.

Log: `target/lane-evidence/pc-merge-store-tier2/run-selected-native.log`.

```text

thread 'process::recovery_runtime::phase_eight_process::production::recovery_crash::killed_recovery_process_reopens_after_each_named_recovery_seam' (58248) panicked at crates\worth-store-court-recovery-runtime-replication-phase-eight-process\tests\worth_store_court_recovery_runtime_replication_phase_eight_process\support\phase_eight_process\history\writer_history\expected_history\declarations.rs:26:9:
C8 fixture must exceed its admitted recovery memory budget
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

```

## `parameter_invariant::lease_tests::parameter_lease_one_unit_short_is_the_work_stop`

CLEANUP-C2-exact-work-boundary; owner: fixture owning court / recorded cleanup.

Explicit paid.work-1 or2*ENTRY_WORK-1 resource court; original boundary and exact outcome assertion stay unchanged.

Log: `target/lane-evidence/pc-merge-fixture-tier2/fixture-chunk-000.log`.

```text
thread 'parameter_invariant::lease_tests::parameter_lease_one_unit_short_is_the_work_stop' (24040) panicked at parameter_entry\src\parameter_invariant\lease_tests.rs:120:9:
ProductUnpublished(WorthQueryProductUnpublishedApplication { handle: ProductUnpublishedRecoveryHandle { identity: ProductUnpublishedOwnerEffectsIdentity { owner: RuntimeWorldOwnerIdentity { scope: Ephemeral(EphemeralWorldOwnerIdentity(4)), instance: EphemeralWorldOwnerIdentity(4), home_fence: None }, ordinal: 0 }, catalog_affinity: 2585738088160 }, .. })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    parameter_invariant::lease_tests::parameter_lease_one_unit_short_is_the_work_stop

test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s


```

## `application_invariant_acceptance::proof::application_program::discovered::retention::discovered_committed_retention_refuses_twice_then_promotes_original_carrier`

CLEANUP-C1-retention-or-publication-interruption; owner: fixture owning court / recorded cleanup.

Actual retention reservation refusal, abandoned performed result, cancellation or paused publication precedes failing recovery assertion.

Log: `target/lane-evidence/pc-merge-fixture-tier2/fixture-chunk-004.log`.

```text

running 150 tests
test application_invariant_acceptance::contribution_denials::producer_denials::tests::complete_handler_registration_leaves_only_the_intended_producer_omission ... ok
test application_invariant_acceptance::candidate_index::boxes::candidate_allocation::pruned_completed_source_allocates_no_dense_candidate_backing ... ok
test application_invariant_acceptance::managed_index::boxes::candidate_allocation::pruned_completed_source_allocates_no_dense_candidate_backing ... ok
test application_invariant_acceptance::candidate_index::boxes::lifecycle::cancelled_cold_box_build_releases_partial_bvh_and_rebuilds_with_fresh_request ... ok
test application_invariant_acceptance::managed_index::expectations::membership::managed_empty_result_expectation_preserves_future_entrant_dependencies ... ok
test application_invariant_acceptance::managed_index::expectations::membership::managed_row_expectation_preserves_nonmatches_and_recorded_retry ... ok
test application_invariant_acceptance::managed_index::boxes::affinity::registered_box_rejects_same_runtime_foreign_branch_and_retained_stale_basis ... ok
test application_invariant_acceptance::managed_index::boxes::summary::predecessor_lookup_on_a_genuine_cold_source_slot_refuses_before_a_lawful_build ... ok
test application_invariant_acceptance::candidate_index::boxes::summary::predecessor_lookup_on_a_genuine_cold_source_slot_refuses_before_a_lawful_build ... ok
test application_invariant_acceptance::managed_index::boxes::geometry::public_box_provider_preserves_closed_boundaries_source_slots_and_typed_bounds_refusals ... ok
test application_invariant_acceptance::candidate_index::boxes::geometry::public_box_provider_preserves_closed_boundaries_source_slots_and_typed_bounds_refusals ... ok
test application_invariant_acceptance::managed_index::lifecycle::affinity::registered_prefix_rejects_genuine_reclaimed_scope_generation_replacement ... ok
test application_invariant_acceptance::managed_index::lifecycle::complete_membership::complete_membership_visits_nonmatches_and_refuses_interrupted_partial_selection ... ok
test application_invariant_acceptance::managed_index::lifecycle::publication::public_cache_native_ahead_of_world_keeps_selected_predecessor_then_requires_rebuild ... ok
test application_invariant_acceptance::candidate_index::boxes::summary::checked_summary_refuses_corrupt_cold_bounds_then_rebuilds_a_lawful_twin ... ok
test application_invariant_acceptance::candidate_index::boxes::affinity::registered_box_rejects_same_runtime_foreign_branch_and_retained_stale_basis ... ok
test application_invariant_acceptance::managed_index::boxes::lifecycle::registered_box_backing_releases_after_results_and_last_runtime_close ... ok
test application_invariant_acceptance::managed_index::boxes::summary::checked_summary_refuses_corrupt_cold_bounds_then_rebuilds_a_lawful_twin ... ok
test application_invariant_acceptance::managed_index::journey::public_prefix_index_rebuild_preserves_source_slots_and_ordinary_query_results .
[Projection abbreviated here; full original output remains in the cited log.]
ction::fixed_constructor_rejects_missing_governed_locality_without_effects ... ok
test application_invariant_acceptance::managed_index::maintained::geometry::generated::court::late_cancellation::generated_source_late_cancellation_preserves_performed_and_published_recovery ... ok
test application_invariant_acceptance::managed_index::maintained::geometry::generated::court::pending::pending_generated_materialization_keeps_world_selected_images_and_recovers_both_directions ... ok
test application_invariant_acceptance::managed_index::maintained::source_authorization::court::reads::native_source_read_policy_revoke_keeps_structural_root_and_denies_before_callbacks ... ok
test application_invariant_acceptance::managed_index::maintained::source_authorization::court::expectations::separately_admitted_source_reads_keep_semantic_identity_for_retained_replay ... ok
test application_invariant_acceptance::proof::application_program::delivery_binding::common_operation_handoff_required_output_cannot_select_source_completion ... ok
test application_invariant_acceptance::proof::application_program::closed_delivery::phase_cleanup::unstarted_original_cleanup_refuses_cancellation_before_any_native_tree_exists ... ok
test application_invariant_acceptance::proof::application_program::closed_delivery::exact_observation::selected_action::closed_leaf_clean_currentness_uses_native_marks_before_projection ... ok
test application_invariant_acceptance::managed_index::maintained::nested::court::retirement::forbidden_leaf_reaches_exact_proposed_rule_and_refuses_publication_atomically ... ok
test application_invariant_acceptance::managed_index::maintained::protected::court::governed_predicate_watch_readmits_current_capability_and_read_before_metadata_or_ack ... ok
test application_invariant_acceptance::proof::application_program::closed_delivery::original_endpoint::unrelated_publication_between_original_outputs_cannot_become_the_tree_endpoint ... ok
test application_invariant_acceptance::proof::application_program::closed_delivery::exact_observation::no_projection_fixed_and_no_source_deliver_at_one_caller_observation ... ok
test application_invariant_acceptance::proof::application_program::delivery_binding::counterfeit_delivery_selector_refuses_and_installed_twin_constructs ... ok
test application_invariant_acceptance::managed_index::refusing::court::explicit_initialization_is_confined_to_one_unpublished_seed_attempt ... ok
test application_invariant_acceptance::proof::application_program::descriptor_result_set::owning_result_set_false_original_condition_refuses_before_candidate_effect ... ok
test application_invariant_acceptance::proof::application_program::closed_delivery::exact_observation::selected_action::closed_no_projection_compares_the_action_selected_at_the_caller_observation ... ok
test application_invariant_acceptance::proof::application_program::discovered_contracts::discovered_authoring_refuses_wrong_occurrence_and_missing_actual_root ... ok
test application_invariant_acceptance::proof::application_program::discovered_contracts::discovered_closed_projection_shapes_and_empty_batch_authoring ... ok
test application_invariant_acceptance::proof::application_program::closed_delivery::phase_cleanup::original_delivery_cleanup_uses_owning_phase_after_scope_loss_and_keeps_close_only_retries ... ok
test application_invariant_acceptance::proof::application_program::closed_delivery::fixed_and_no_source_close_before_fresh_complete_current_read ... ok
test application_invariant_acceptance::document_history::tests::the_executable_document_adapter_selects_both_futures_and_retains_real_output_custody ... ok
test application_invariant_acceptance::managed_index::maintained::nested::court::influence::descendant_changes_refresh_watch_and_refuse_populated_and_negative_expectations ... ok
test application_invariant_acceptance::managed_index::maintained::geometry::court::rebuild::public_rebuild_refills_canonical_source_and_cancelled_build_stays_cold ... ok
test application_invariant_acceptance::proof::application_program::discovered::currentness::retired::discovered_currentness_all_native_retired_keeps_original_discovery ... ok
test application_invariant_acceptance::managed_index::maintained::geometry::generated::court::interference::generated_source_suspension_loses_to_signal_successor ... ok
test application_invariant_acceptance::proof::application_program::closed_delivery::original_endpoint::memory::original_endpoint_index_refusal_keeps_its_precise_memory_cause_and_original_carrier ... ok
test application_invariant_acceptance::managed_index::maintained::geometry::court::cold::cold_scope_reconstructs_before_rules_and_preserves_unchanged_source_expectations ... ok
test application_invariant_acceptance::managed_index::maintained::geometry::court::watch::predicate_watch_boxes_preserve_each_notification_with_current_full_receipts ... ok
test application_invariant_acceptance::proof::application_program::discovered::currentness::deep_output::discovered_currentness_all_retained_dependent_settlements ... ok
test application_invariant_acceptance::proof::application_program::discovered_contracts::discovered_installed_concrete_delivery_and_occurrence_are_exact ... ok
test application_invariant_acceptance::proof::application_program::deployment_authorization::fixed_adoption_requires_an_explicit_source_revision_grant ... ok
test application_invariant_acceptance::proof::application_program::closed_delivery::selected_meaning::closed_delivery_uses_actual_selected_action_and_root_after_adoption ... ok
test application_invariant_acceptance::managed_index::maintained::geometry::court::indirect_vertex_motion_prepares_unopened_scope_and_keeps_atomic_geometry_roots ... ok
test application_invariant_acceptance::proof::application_program::discovered::currentness::discovered_currentness_foreign_runtime_preserves_owner ... ok
test application_invariant_acceptance::proof::application_program::closed_delivery::qualification::read_origin_rebuild_and_receiptless_reuse_qualify_the_original_complete_tree ... ok
test application_invariant_acceptance::managed_index::maintained::protected::court::realized_influence::governed_realized_expectation_preserves_irrelevant_edits_and_rejects_changed_influence ... ok
test application_invariant_acceptance::managed_index::maintained::geometry::court::growth::source_structure_warm_work_keeps_local_growth_on_real_small_chains ... ok
test application_invariant_acceptance::proof::application_program::closed_delivery::original_endpoint::no_publication_tree_keeps_its_source_endpoint_after_unrelated_head_movement ... ok
test application_invariant_acceptance::proof::application_program::discovered::retention::discovered_committed_retention_refuses_twice_then_promotes_original_carrier ... FAILED
test application_invariant_acceptance::proof::application_program::discovered::currentness::dependent::discovered_currentness_dependent_membership_empty_and_deep ... ok
test application_invariant_acceptance::managed_index::maintained::geometry::court::summary_locality::checked_summary_pruning_visits_local_paths_and_preserves_boundary_membership ... ok
test application_invariant_acceptance::managed_index::maintained::court::prefix_growth::real_prefix_width_small_batches_and_sparse_queries_preserve_both_roots ... ok
test application_invariant_acceptance::proof::application_program::discovered::currentness::discovered_currentness_original_membership_and_empty_negative_facts ... ok
test application_invariant_acceptance::managed_index::maintained::court::watch::predicate_watch_prefix_refreshes_full_parent_and_prunes_unrelated_probes ... ok
test application_invariant_acceptance::managed_index::maintained::court::prefix_growth::prefix_width_growth_keeps_local_edits_bounded_beside_unrelated_source_names has been running for over 60 seconds
test application_invariant_acceptance::managed_index::maintained::geometry::court::growth::fixed_geometry_edit_keeps_source_work_local_beside_independent_remote_populations has been running for over 60 seconds
test application_invariant_acceptance::managed_index::maintained::geometry::court::growth::geometry_chain_growth_uses_real_source_and_original_sdk_invoice has been running for over 60 seconds
test application_invariant_acceptance::managed_index::maintained::geometry::court::growth::spatial_rows::separated_alternating_rows_keep_checked_queries_sparse_through_cold_and_warm_edits has been running for over 60 seconds
test application_invariant_acceptance::managed_index::maintained::geometry::court::growth::spatial_rows::separated_alternating_rows_keep_checked_queries_sparse_through_cold_and_warm_edits ... ok
test application_invariant_acceptance::managed_index::maintained::geometry::court::growth::fixed_geometry_edit_keeps_source_work_local_beside_independent_remote_populations ... ok
test application_invariant_acceptance::managed_index::maintained::court::prefix_growth::prefix_width_growth_keeps_local_edits_bounded_beside_unrelated_source_names ... ok

```

## `application_invariant_acceptance::proof::application_program::discovered_owning::retention::discovered_owning_committed_retention_retry_preserves_fresh_result_and_source`

CLEANUP-C1-retention-or-publication-interruption; owner: fixture owning court / recorded cleanup.

Actual retention reservation refusal, abandoned performed result, cancellation or paused publication precedes failing recovery assertion.

Log: `target/lane-evidence/pc-merge-fixture-tier2/fixture-chunk-005.log`.

```text
thread 'application_invariant_acceptance::proof::application_program::discovered_owning::retention::discovered_owning_committed_retention_retry_preserves_fresh_result_and_source' (10384) panicked at consumer_root\src\application_invariant_acceptance\proof\application_program\discovered_owning\retention.rs:163:14:
fresh authorized retry promotes the same original source
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- application_invariant_acceptance::proof::application_program::document_history::navigation_outputs::publication::stopped_recovery::stopped_original_recovery_retries_without_reexecuting_the_producer stdout ----

thread 'application_invariant_acceptance::proof::application_program::document_history::navigation_outputs::publication::stopped_recovery::stopped_original_recovery_retries_without_reexecuting_the_producer' (16084) panicked at consumer_root\src\application_invariant_acceptance\proof\application_program\document_history\navigation_outputs\publication\stopped_recovery.rs:145:21:
fresh recovery request: Outputs(Preparation(Demand(History(Demand(WorthQueryOutputDemandDenial { kind: Superseded, evidence: None, cleanup: None, subject: "output changed before its settlement was accepted", domain: None, readmission_failure: None, recovery_posture: Terminal })))))

---- application_invariant_acceptance::proof::application_program::document_history::navigation_outputs::publication::unpublished_history_producer_resumes_original_effect_once_before_selecting_the_same_candidate stdout ----

thread 'application_invariant_acceptance::proof::application_program::document_history::navigation_outputs::publication::unpublished_history_producer_resumes_original_effect_once_before_selecting_the_same_candidate' (60040) panicked at consumer_root\src\application_invariant_acceptance\proof\application_program\document_history\navigation_outputs\publication.rs:282:17:
resume original World effect: Outputs(Preparation(Demand(History(Demand(WorthQueryOutputDemandDenial { kind: Superseded, evidence: None, cleanup: None, subject: "output changed before its settlement was accepted", domain: None, readmission_failure: None, recovery_posture: Terminal })))))


failures:
    application_invariant_acceptance::proof::application_program::discovered_owning::retention::discovered_owning_committed_retention_retry_preserves_fresh_result_and_source
    application_invariant_acceptance::proof::application_program::document_history::navigation_outputs::publication::stopped_recovery::stopped_original_recovery_retries_without_reexecuting_the_producer
    application_invariant_acceptance::proof::application_program::document_history::navigation_outputs::publication::unpublished_history_producer_resumes_original_effect_once_before_selecting_the_same_candidate

test result: FAILED. 119 passed; 3 failed; 0 ignored; 0 measured; 150 filtered out; finished in 50.30s


```

## `application_invariant_acceptance::proof::application_program::document_history::navigation_outputs::publication::stopped_recovery::stopped_original_recovery_retries_without_reexecuting_the_producer`

CLEANUP-C1-retention-or-publication-interruption; owner: fixture owning court / recorded cleanup.

Actual retention reservation refusal, abandoned performed result, cancellation or paused publication precedes failing recovery assertion.

Log: `target/lane-evidence/pc-merge-fixture-tier2/fixture-chunk-005.log`.

```text
thread 'application_invariant_acceptance::proof::application_program::document_history::navigation_outputs::publication::stopped_recovery::stopped_original_recovery_retries_without_reexecuting_the_producer' (16084) panicked at consumer_root\src\application_invariant_acceptance\proof\application_program\document_history\navigation_outputs\publication\stopped_recovery.rs:145:21:
fresh recovery request: Outputs(Preparation(Demand(History(Demand(WorthQueryOutputDemandDenial { kind: Superseded, evidence: None, cleanup: None, subject: "output changed before its settlement was accepted", domain: None, readmission_failure: None, recovery_posture: Terminal })))))

---- application_invariant_acceptance::proof::application_program::document_history::navigation_outputs::publication::unpublished_history_producer_resumes_original_effect_once_before_selecting_the_same_candidate stdout ----

thread 'application_invariant_acceptance::proof::application_program::document_history::navigation_outputs::publication::unpublished_history_producer_resumes_original_effect_once_before_selecting_the_same_candidate' (60040) panicked at consumer_root\src\application_invariant_acceptance\proof\application_program\document_history\navigation_outputs\publication.rs:282:17:
resume original World effect: Outputs(Preparation(Demand(History(Demand(WorthQueryOutputDemandDenial { kind: Superseded, evidence: None, cleanup: None, subject: "output changed before its settlement was accepted", domain: None, readmission_failure: None, recovery_posture: Terminal })))))


failures:
    application_invariant_acceptance::proof::application_program::discovered_owning::retention::discovered_owning_committed_retention_retry_preserves_fresh_result_and_source
    application_invariant_acceptance::proof::application_program::document_history::navigation_outputs::publication::stopped_recovery::stopped_original_recovery_retries_without_reexecuting_the_producer
    application_invariant_acceptance::proof::application_program::document_history::navigation_outputs::publication::unpublished_history_producer_resumes_original_effect_once_before_selecting_the_same_candidate

test result: FAILED. 119 passed; 3 failed; 0 ignored; 0 measured; 150 filtered out; finished in 50.30s


```

## `application_invariant_acceptance::proof::application_program::document_history::navigation_outputs::publication::unpublished_history_producer_resumes_original_effect_once_before_selecting_the_same_candidate`

CLEANUP-C1-retention-or-publication-interruption; owner: fixture owning court / recorded cleanup.

Actual retention reservation refusal, abandoned performed result, cancellation or paused publication precedes failing recovery assertion.

Log: `target/lane-evidence/pc-merge-fixture-tier2/fixture-chunk-005.log`.

```text
thread 'application_invariant_acceptance::proof::application_program::document_history::navigation_outputs::publication::unpublished_history_producer_resumes_original_effect_once_before_selecting_the_same_candidate' (60040) panicked at consumer_root\src\application_invariant_acceptance\proof\application_program\document_history\navigation_outputs\publication.rs:282:17:
resume original World effect: Outputs(Preparation(Demand(History(Demand(WorthQueryOutputDemandDenial { kind: Superseded, evidence: None, cleanup: None, subject: "output changed before its settlement was accepted", domain: None, readmission_failure: None, recovery_posture: Terminal })))))


failures:
    application_invariant_acceptance::proof::application_program::discovered_owning::retention::discovered_owning_committed_retention_retry_preserves_fresh_result_and_source
    application_invariant_acceptance::proof::application_program::document_history::navigation_outputs::publication::stopped_recovery::stopped_original_recovery_retries_without_reexecuting_the_producer
    application_invariant_acceptance::proof::application_program::document_history::navigation_outputs::publication::unpublished_history_producer_resumes_original_effect_once_before_selecting_the_same_candidate

test result: FAILED. 119 passed; 3 failed; 0 ignored; 0 measured; 150 filtered out; finished in 50.30s


```

## `checkpoint_recovery::component_islands::locality::serial::real_split_component_reads_are_unchanged_with_ten_and_ten_thousand_unrelated_islands`

CLEANUP-C2-population-growth; owner: fixture owning court / recorded cleanup.

10/10000-island setup reaches canonical artifact size268435459 >268435456 before locality assertion.

Log: `target/lane-evidence/pc-merge-fixture-tier2/fixture-chunk-023.log`.

```text
thread 'checkpoint_recovery::component_islands::locality::serial::real_split_component_reads_are_unchanged_with_ten_and_ten_thousand_unrelated_islands' (49924) panicked at topology_entry\src\checkpoint_recovery\component_islands\world.rs:68:14:
population setup leaves the World's request policy unchanged: ConditionalPublication(WorthQueryConditionalRuntimeInstallationDenial { kind: PrimaryGraphPublication, subject: "primary graph installation denied: RelationalCommitRejected (Relational rejected the application seed transaction: branch publication denied: CanonicalArtifactEncoding(SizeLimit { observed: 268435459, maximum: 268435456 }))" })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    checkpoint_recovery::component_islands::locality::serial::real_split_component_reads_are_unchanged_with_ten_and_ten_thousand_unrelated_islands

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 319 filtered out; finished in 425.82s


```

## `checkpoint_recovery::computation_partition::oracle::advancement_custody::serial::caller_refresh_retry_spends_what_the_first_attempt_left`

CLEANUP-C2-exact-work-boundary; owner: fixture owning court / recorded cleanup.

Explicit paid.work-1 or2*ENTRY_WORK-1 resource court; original boundary and exact outcome assertion stay unchanged.

Log: `target/lane-evidence/pc-merge-fixture-tier2/fixture-chunk-034.log`.

```text
thread 'checkpoint_recovery::computation_partition::oracle::advancement_custody::serial::caller_refresh_retry_spends_what_the_first_attempt_left' (68268) panicked at topology_entry\src\checkpoint_recovery\computation_partition\oracle\advancement_custody.rs:89:5:
assertion `left == right` failed: charged=1990611; attempts=[(Ok(()), 275981), (Ok(()), 275981), (Err(ExecutionRequest(Resource(WorkExhausted))), 395844)]; diagnostics=runs=1; first=Some(Ok((4607182418800017408, 1000697))); last=Some(Ok((4607182418800017408, 1000697)))
  left: ExecutionRequest(Resource(WorkExhausted))
 right: ExecutionRequest(NestedStopped)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    checkpoint_recovery::computation_partition::oracle::advancement_custody::serial::caller_refresh_retry_spends_what_the_first_attempt_left

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 319 filtered out; finished in 3.27s


```

## `checkpoint_recovery::computation_partition::oracle::courtroom::lifecycle::serial::seeded_edits_cross_restore_and_republication_with_model_counts`

CLEANUP-C4-generated-fault-or-race-oracle; owner: fixture owning court / recorded cleanup.

Generated sequences include Fault/Repair and/or own-write interleavings and charged-work/observation comparison; actual failure after restored prefix or capture is preserved.

Log: `target/lane-evidence/pc-merge-fixture-tier2/fixture-chunk-070.log`.

```text
thread 'checkpoint_recovery::computation_partition::oracle::courtroom::lifecycle::serial::seeded_edits_cross_restore_and_republication_with_model_counts' (55012) panicked at topology_entry\src\checkpoint_recovery\computation_partition\oracle\demand.rs:41:10:
the region output demand starts: Source(PrincipalResolution(WorthQueryPrincipalResolutionDenial { kind: Stopped(Ok(ExecutionResource { denial: WorkExhausted, partition_identity: None, policy_ancestor: None })), binding: "ConsumerPrincipalBinding", native_query_read_stop: None }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    checkpoint_recovery::computation_partition::oracle::courtroom::lifecycle::serial::seeded_edits_cross_restore_and_republication_with_model_counts

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 319 filtered out; finished in 111.55s


```

## `checkpoint_recovery::computation_partition::oracle::differential::serial::every_reused_run_equals_a_full_run_of_the_same_facts`

CLEANUP-C4-generated-fault-or-race-oracle; owner: fixture owning court / recorded cleanup.

Generated sequences include Fault/Repair and/or own-write interleavings and charged-work/observation comparison; actual failure after restored prefix or capture is preserved.

Log: `target/lane-evidence/pc-merge-fixture-tier2/fixture-chunk-077.log`.

```text
thread 'checkpoint_recovery::computation_partition::oracle::differential::serial::every_reused_run_equals_a_full_run_of_the_same_facts' (82704) panicked at topology_entry\src\checkpoint_recovery\computation_partition\oracle\demand.rs:41:10:
the region output demand starts: Source(PrincipalResolution(WorthQueryPrincipalResolutionDenial { kind: Stopped(Ok(ExecutionResource { denial: WorkExhausted, partition_identity: None, policy_ancestor: None })), binding: "ConsumerPrincipalBinding", native_query_read_stop: None }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    checkpoint_recovery::computation_partition::oracle::differential::serial::every_reused_run_equals_a_full_run_of_the_same_facts

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 319 filtered out; finished in 451.07s


```

## `checkpoint_recovery::computation_partition::oracle::restoration::serial::restoration_after_seeded_edits_rebuilds_fresh_state`

CLEANUP-C4-generated-fault-or-race-oracle; owner: fixture owning court / recorded cleanup.

Generated sequences include Fault/Repair and/or own-write interleavings and charged-work/observation comparison; actual failure after restored prefix or capture is preserved.

Log: `target/lane-evidence/pc-merge-fixture-tier2/fixture-chunk-096.log`.

```text
thread 'checkpoint_recovery::computation_partition::oracle::restoration::serial::restoration_after_seeded_edits_rebuilds_fresh_state' (12156) panicked at topology_entry\src\checkpoint_recovery\computation_partition\oracle\restoration.rs:17:218:
called `Result::unwrap()` on an `Err` value: Execution(WorthQueryPrimaryGraphInstallationDenial { kind: ExecutionDenied { kind: ExecutionResource { denial: WorkExhausted, partition_identity: None, policy_ancestor: None } }, subject: "checkpoint history consolidation stopped", cause: None })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    checkpoint_recovery::computation_partition::oracle::restoration::serial::restoration_after_seeded_edits_rebuilds_fresh_state

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 319 filtered out; finished in 5.09s


```

## `checkpoint_recovery::required_chain::exact_invalidation::randomized::serial::randomized_marking_agrees_with_full_verification`

CLEANUP-C4-generated-fault-or-race-oracle; owner: fixture owning court / recorded cleanup.

Generated sequences include Fault/Repair and/or own-write interleavings and charged-work/observation comparison; actual failure after restored prefix or capture is preserved.

Log: `target/lane-evidence/pc-merge-fixture-tier2/fixture-chunk-250.log`.

```text
thread 'checkpoint_recovery::required_chain::exact_invalidation::randomized::serial::randomized_marking_agrees_with_full_verification' (47564) panicked at topology_entry\src\checkpoint_recovery\required_chain\exact_invalidation\randomized.rs:97:21:
seed 0x91763c03, restored before step 37: the root demand of ring-0-a starts: Source(PrincipalResolution(WorthQueryPrincipalResolutionDenial { kind: Stopped(Ok(ExecutionResource { denial: WorkExhausted, partition_identity: None, policy_ancestor: None })), binding: "ConsumerPrincipalBinding", native_query_read_stop: None }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
the randomized courtroom failed after these steps:
  seed 0x91763c03, opening before step 0
  seed 0x91763c03, step 0: RootOutputOverwritten on ring 1, then Nothing (held order ['a', 'b', 'c'])
  seed 0x91763c03, step 1: FetchedOnly on ring 0, then Ring (held order ['b', 'c', 'a'])
  seed 0x91763c03, step 2: Nothing on ring 0, then Everything (held order ['a', 'b', 'c'])
  seed 0x91763c03, restored before step 3
  seed 0x91763c03, step 3: FetchedFar on ring 0, then Held (held order ['c', 'a', 'b'])
  seed 0x91763c03, step 4: EqualWrite on ring 2, then Ring (held order ['a', 'c', 'b'])
  seed 0x91763c03, step 5: EqualWrite on ring 1, then Everything (held order ['c', 'b', 'a'])
  seed 0x91763c03, step 6: RootOutputOverwritten on ring 0, then Everything (held order ['a', 'b', 'c'])
  seed 0x91763c03, step 7: FetchedFar on ring 1, then Nothing (held order ['c', 'a', 'b'])
  seed 0x91763c03, step 8: FetchedFar on ring 0, then Everything (held order ['a', 'b', 'c'])
  seed 0x91763c03, step 9: FetchedFar on ring 0, then Nothing (held order ['b', 'c', 'a'])
  seed 0x91763c03, restored before step 10
  seed 0x91763c03, step 10: ChainOutputOverwritten on ring 1, then Held (held order ['c', 'a', 'b'])
  seed 0x91763c03, step 11: FetchedFar on ring 1, then Ring (held order ['a', 'c', 'b'])
  seed 0x91763c03, step 12: ChainOutputOverwritten on ring 0, then Ring (held order ['a', 'c', 'b'])
  seed 0x91763c03, step 13: Nothing on ring 1, then Nothing (held order ['c', 'b', 'a'])
  seed 0x91763c03, restored before step 14
  seed 0x91763c03, step 14: SuccessorReplaced on ring 2, then Ring (held order ['c', 'a', 'b'])
  seed 0x91763c03, step 15: Nothing on ring 1, then Nothing (held order ['a', 'b', 'c'])
  seed 0x91763c03, step 16: Nothing on ring 0, then Ring (held order ['c', 'a', 'b'])
  seed 0x91763c03, step 17: FetchedOnly on ring 1, then Held (held order ['b', 'c', 'a'])
  seed 0x91763c03, step 18: FetchedFar on ring 2, then Everything (held order ['b', 'a', 'c'])
  seed 0x91763c03, restored before step 19
  seed 0x91763c03, step 19: RootInput on ring 2, then Held (held order ['b', 'c', 'a'])
  seed 0x91763c03, step 20: SuccessorReplaced on ring 1, then Ring (held order ['a', 'b', 'c'])
  seed 0x91763c03, step 21: RacingRootInput on ring 1, then Held (held order ['b', 'a', 'c'])
  seed 0x91763c03, step 22: ChainOutputOverwritten on ring 0, then Held (held order ['a', 'b', 'c'])
  seed 0x91763c03, step 23: RacingRootInput on ring 1, then Everything (held order ['c', 'a', 'b'])
  seed 0x91763c03, step 24: RootOutputOverwritten on ring 1, then Held (held order ['c', 'a', 'b'])
  seed 0x91763c03, step 25: Nothing on ring 1, then Nothing (held order ['c', 'b', 'a'])
  seed 0x91763c03, step 26: ChainOutputOverwritten on ring 2, then Ring (held order ['b', 'a', 'c'])
  seed 0x91763c03, step 27: EqualWrite on ring 2, then Everything (held order ['b', 'a', 'c'])
  seed 0x91763c03, step 28: SuccessorReplaced on ring 1, then Everything (held order ['c', 'a', 'b'])
  seed 0x91763c03, step 29: RacingRootInput on ring 1, then Nothing (held order ['b', 'a', 'c'])
  seed 0x91763c03, step 30: RacingRootInput on ring 0, then Held (held order ['c', 'b', 'a'])
  seed 0x91763c03, step 31: EqualWrite on ring 1, then Ring (held order ['b', 'c', 'a'])
  seed 0x91763c03, step 32: SuccessorReplaced on ring 1, then Everything (held order ['a', 'b', 'c'])
  seed 0x91763c03, step 33: RootOutputOverwritten on ring 2, then Held (held order ['b', 'c', 'a'])
  seed 0x91763c03, step 34: FetchedFar on ring 1, then Held (held order ['b', 'a', 'c'])
  seed 0x91763c03, step 35: SuccessorReplaced on ring 1, then Everything (held order ['a', 'b', 'c'])
  seed 0x91763c03, step 36: ChainOutputOverwritten on ring 1, then Ring (held order ['a', 'c', 'b'])
  seed 0x91763c03, restored before step 37


failures:
    checkpoint_recovery::required_chain::exact_invalidation::randomized::serial::randomized_marking_agrees_with_full_verification

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 319 filtered out; finished in 207.25s


```

## `checkpoint_recovery::required_chain::exact_invalidation::randomized::serial::randomized_marking_makes_room_on_a_branch_with_few_observations`

CLEANUP-C4-generated-fault-or-race-oracle; owner: fixture owning court / recorded cleanup.

Generated sequences include Fault/Repair and/or own-write interleavings and charged-work/observation comparison; actual failure after restored prefix or capture is preserved.

Log: `target/lane-evidence/pc-merge-fixture-tier2/fixture-chunk-251.log`.

```text
thread 'checkpoint_recovery::required_chain::exact_invalidation::randomized::serial::randomized_marking_makes_room_on_a_branch_with_few_observations' (24432) panicked at topology_entry\src\checkpoint_recovery\required_chain\exact_invalidation\randomized.rs:97:21:
seed 0x91763c03, restored before step 37: the root demand of ring-0-a starts: Source(PrincipalResolution(WorthQueryPrincipalResolutionDenial { kind: Stopped(Ok(ExecutionResource { denial: WorkExhausted, partition_identity: None, policy_ancestor: None })), binding: "ConsumerPrincipalBinding", native_query_read_stop: None }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
the randomized courtroom failed after these steps:
  seed 0x91763c03, opening before step 0
  seed 0x91763c03, step 0: RootOutputOverwritten on ring 1, then Nothing (held order ['a', 'b', 'c'])
  seed 0x91763c03, step 1: FetchedOnly on ring 0, then Ring (held order ['b', 'c', 'a'])
  seed 0x91763c03, step 2: Nothing on ring 0, then Everything (held order ['a', 'b', 'c'])
  seed 0x91763c03, restored before step 3
  seed 0x91763c03, step 3: FetchedFar on ring 0, then Held (held order ['c', 'a', 'b'])
  seed 0x91763c03, step 4: EqualWrite on ring 2, then Ring (held order ['a', 'c', 'b'])
  seed 0x91763c03, step 5: EqualWrite on ring 1, then Everything (held order ['c', 'b', 'a'])
  seed 0x91763c03, step 6: RootOutputOverwritten on ring 0, then Everything (held order ['a', 'b', 'c'])
  seed 0x91763c03, step 7: FetchedFar on ring 1, then Nothing (held order ['c', 'a', 'b'])
  seed 0x91763c03, step 8: FetchedFar on ring 0, then Everything (held order ['a', 'b', 'c'])
  seed 0x91763c03, step 9: FetchedFar on ring 0, then Nothing (held order ['b', 'c', 'a'])
  seed 0x91763c03, restored before step 10
  seed 0x91763c03, step 10: ChainOutputOverwritten on ring 1, then Held (held order ['c', 'a', 'b'])
  seed 0x91763c03, step 11: FetchedFar on ring 1, then Ring (held order ['a', 'c', 'b'])
  seed 0x91763c03, step 12: ChainOutputOverwritten on ring 0, then Ring (held order ['a', 'c', 'b'])
  seed 0x91763c03, step 13: Nothing on ring 1, then Nothing (held order ['c', 'b', 'a'])
  seed 0x91763c03, restored before step 14
  seed 0x91763c03, step 14: SuccessorReplaced on ring 2, then Ring (held order ['c', 'a', 'b'])
  seed 0x91763c03, step 15: Nothing on ring 1, then Nothing (held order ['a', 'b', 'c'])
  seed 0x91763c03, step 16: Nothing on ring 0, then Ring (held order ['c', 'a', 'b'])
  seed 0x91763c03, step 17: FetchedOnly on ring 1, then Held (held order ['b', 'c', 'a'])
  seed 0x91763c03, step 18: FetchedFar on ring 2, then Everything (held order ['b', 'a', 'c'])
  seed 0x91763c03, restored before step 19
  seed 0x91763c03, step 19: RootInput on ring 2, then Held (held order ['b', 'c', 'a'])
  seed 0x91763c03, step 20: SuccessorReplaced on ring 1, then Ring (held order ['a', 'b', 'c'])
  seed 0x91763c03, step 21: RacingRootInput on ring 1, then Held (held order ['b', 'a', 'c'])
  seed 0x91763c03, step 22: ChainOutputOverwritten on ring 0, then Held (held order ['a', 'b', 'c'])
  seed 0x91763c03, step 23: RacingRootInput on ring 1, then Everything (held order ['c', 'a', 'b'])
  seed 0x91763c03, step 24: RootOutputOverwritten on ring 1, then Held (held order ['c', 'a', 'b'])
  seed 0x91763c03, step 25: Nothing on ring 1, then Nothing (held order ['c', 'b', 'a'])
  seed 0x91763c03, step 26: ChainOutputOverwritten on ring 2, then Ring (held order ['b', 'a', 'c'])
  seed 0x91763c03, step 27: EqualWrite on ring 2, then Everything (held order ['b', 'a', 'c'])
  seed 0x91763c03, step 28: SuccessorReplaced on ring 1, then Everything (held order ['c', 'a', 'b'])
  seed 0x91763c03, step 29: RacingRootInput on ring 1, then Nothing (held order ['b', 'a', 'c'])
  seed 0x91763c03, step 30: RacingRootInput on ring 0, then Held (held order ['c', 'b', 'a'])
  seed 0x91763c03, step 31: EqualWrite on ring 1, then Ring (held order ['b', 'c', 'a'])
  seed 0x91763c03, step 32: SuccessorReplaced on ring 1, then Everything (held order ['a', 'b', 'c'])
  seed 0x91763c03, step 33: RootOutputOverwritten on ring 2, then Held (held order ['b', 'c', 'a'])
  seed 0x91763c03, step 34: FetchedFar on ring 1, then Held (held order ['b', 'a', 'c'])
  seed 0x91763c03, step 35: SuccessorReplaced on ring 1, then Everything (held order ['a', 'b', 'c'])
  seed 0x91763c03, step 36: ChainOutputOverwritten on ring 1, then Ring (held order ['a', 'c', 'b'])
  seed 0x91763c03, restored before step 37


failures:
    checkpoint_recovery::required_chain::exact_invalidation::randomized::serial::randomized_marking_makes_room_on_a_branch_with_few_observations

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 319 filtered out; finished in 208.63s


```

## `application_invariant_acceptance::managed_index::maintained::geometry::court::growth::geometry_chain_growth_uses_real_source_and_original_sdk_invoice`

CLEANUP-C2-incomplete-growth-campaign; owner: fixture owning court / recorded cleanup.

Busy finite growth process stopped after>10min; exact PID/artifact/parent/sole remaining case verified. Completed prefix retained; no assertion or timeout claim.

Log: `target/lane-evidence/pc-merge-fixture-tier2/fixture-chunk-004.log`.

```text
Audited incomplete campaign; see interruption_audit in the JSON record.
```
