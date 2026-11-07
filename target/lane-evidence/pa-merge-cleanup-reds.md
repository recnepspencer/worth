# P-A merge cleanup reds

Latest source `79de5c7ad597cb207fa17a2d38ec1b80da748bba`. Each court is retained. Exact original and focused failures remain in the linked JSON and originating logs.

## worth-court-relational-certification::worth_court_relational_certification$content::durable_publication::content_durable_artifact_memory_refuses_before_the_medium_and_selected_commit

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'content::durable_publication::content_durable_artifact_memory_refuses_before_the_medium_and_selected_commit' (43100) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\content\durable_publication.rs:182:5:
assertion `left == right` failed
  left: 1559
 right: 0
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-court-relational-certification::worth_court_relational_certification$content::durable_publication::content_durable_artifact_cancellation_after_original_allocation_never_calls_medium

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'content::durable_publication::content_durable_artifact_cancellation_after_original_allocation_never_calls_medium' (35556) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\content\durable_publication.rs:255:5:
assertion `left == right` failed
  left: 1559
 right: 0
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-court-relational-certification::worth_court_relational_certification$content::durable_publication::content_durable_artifact_original_work_exhaustion_is_exact_and_pre_effect

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'content::durable_publication::content_durable_artifact_original_work_exhaustion_is_exact_and_pre_effect' (20472) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\content\durable_publication.rs:316:5:
assertion `left == right` failed
  left: Some(67107305)
 right: Some(67108864)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-court-relational-certification::worth_court_relational_certification$content::historical_kind::custody::original_kind_custody_survives_successor_basis_checkpoint_tail_and_reopen

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'content::historical_kind::custody::original_kind_custody_survives_successor_basis_checkpoint_tail_and_reopen' (16000) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\content\historical_kind\custody.rs:74:5:
assertion `left == right` failed
  left: 34066
 right: 8576
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-court-relational-certification::worth_court_relational_certification$content::historical_kind::funding::historical_kind_reconstruction_preserves_literal_original_stops_and_releases_scratch

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'content::historical_kind::funding::historical_kind_reconstruction_preserves_literal_original_stops_and_releases_scratch' (75064) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\content\historical_kind\funding.rs:70:36:
called `Result::unwrap()` on an `Err` value: Execution { denial: CommitExecutionDenial { kind: Cause(WorkExhausted), partition_identity: None }, context: ErrorContext { subsystem: Transaction, operation: Commit, affected_records: [], version_context: None, suggested_fix: None }, commit_log: CommitLog { events: [PhaseStarted(DraftPreparation), StructuralSummary, PhaseCompleted(DraftPreparation), PhaseStarted(InvariantPreCheck), InvariantEvaluated { execution_point: CommitBoundary, observation_kind: Committed, disposition: Executed, preparation_selected_mode: None, preparation_parallel_legality: None, preparation_parallel_profitability: None, preparation_serial_selection_reason: None, consumed_groups: InvariantGroupSet { mask: 377 }, applicable_groups: InvariantGroupSet { mask: 337 }, result_count: 0, advisory_count: 0, violation_count: 0, blocking_violation: false, publication_violation: false }, PhaseCompleted(InvariantPreCheck), PhaseStarted(AuthoritativeMutation), InvariantEvaluated { execution_point: MutationSensitive, observation_kind: Speculative, disposition: Executed, preparation_selected_mode: Some(Serial), preparation_parallel_legality: Some(RequiresSerial), preparation_parallel_profitability: Some(NotProfitable), preparation_serial_selection_reason: Some(NoLease), consumed_groups: InvariantGroupSet { mask: 317 }, applicable_groups: InvariantGroupSet { mask: 277 }, result_count: 2, advisory_count: 0, violation_count: 0, blocking_violation: false, publication_violation: false }, PhaseCompleted(AuthoritativeMutation), PhaseStarted(HistoryResolution), HistoryResolved, PhaseCompleted(HistoryResolution), PhaseStarted(InvariantPostCheck), InvariantEvaluated { execution_point: SnapshotPublication, observation_kind: Speculative, disposition: Executed, preparation_selected_mode: None, preparation_parallel_legality: None, preparation_parallel_profitability: None, preparation_serial_selection_reason: None, consumed_groups: InvariantGroupSet { mask: 66 }, applicable_groups: InvariantGroupSet { mask: 66 }, result_count: 0, advisory_count: 0, violation_count: 0, blocking_violation: false, publication_violation: false }, PhaseCompleted(InvariantPostCheck), PhaseStarted(ArtifactAssembly), PatchBudgetEvaluated, CommitRejected { phase: ArtifactAssembly, diagnostic_code: None, publication_stage: None, detail: "commit preparation execution stopped: CommitExecutionDenial { kind: Cause(WorkExhausted), partition_identity: None }" }], running_summary: CommitSummary { phase_count: 6, structural_summary: Some(CommitStructuralSummary { invariant_groups: InvariantGroupSet { mask: 343 }, commit_topology: GraphMutation, touched_partitions: [PartitionId(7)], bulk_entity_slots_reserved: 0, bulk_relation_slots_reserved: 0, descriptor_semantics_version: DescriptorSemanticsVersion(1) }), history_summary: Some(CommitHistorySummary { target_branch: "main", requested_merge_parent_count: 0, effective_merge_parent_count: 0, parent_count: 1, merge_base_count: 0, had_previous_branch_head: true }), patch_budget_summary: Some(CommitPatchBudgetSummary { patch_record_count: 1, max_patch_records_per_commit: None }), change_summary: None, aspect_summary: None, publication_summary: None, invariant_result_count: 2, invariant_advisory_count: 0, invariant_violation_count: 0 } } }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-court-relational-certification::worth_court_relational_certification$relational::validated_proposal_memory::unfunded_validated_proposal_backing_refuses_and_releases_original_custody

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'relational::validated_proposal_memory::unfunded_validated_proposal_backing_refuses_and_releases_original_custody' (992) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\relational\validated_proposal_memory.rs:61:5:
CommitExecutionDenial { kind: Cause(PolicyMemoryExhausted { requested: 4000, admitted: 1280, ancestor: 0 }), partition_identity: None }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-relational::worth_relational$authority::commit::pipeline::request_checkpoints::work_account::one_work_account_bounds_each_entry_with_its_original_canonical_evidence

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'authority::commit::pipeline::request_checkpoints::work_account::one_work_account_bounds_each_entry_with_its_original_canonical_evidence' (19748) panicked at crates\worth-relational\src\authority\commit\pipeline\request_checkpoints\work_account.rs:148:9:
assertion `left == right` failed: Ordinary: every original SDK charge settles into the same account
  left: 8059
 right: 8595
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-relational::worth_relational$authority::commit::pipeline::request_checkpoints::work_account::one_work_account_bounds_the_same_proposal_on_every_entry

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
native commit work invoice: actual=8595 settled=8059

thread 'authority::commit::pipeline::request_checkpoints::work_account::one_work_account_bounds_the_same_proposal_on_every_entry' (33636) panicked at crates\worth-relational\src\authority\commit\pipeline\request_checkpoints\work_account.rs:175:5:
assertion `left == right` failed: the authoritative account settles all SDK work
  left: 8059
 right: 8595
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-relational::worth_relational$durability::log::native_checkpoint_codec::kind_authentication_tests::forged_cross_region_kind_commitment_is_refused_before_member_reads

CLEANUP-malformed-input-boundary: The failing assertion compares ordering or typed malformed-input diagnostics in a fault court.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'durability::log::native_checkpoint_codec::kind_authentication_tests::forged_cross_region_kind_commitment_is_refused_before_member_reads' (10180) panicked at crates\worth-relational\src\durability\log\native_checkpoint_codec\kind_authentication_tests.rs:155:5:
assertion `left == right` failed
  left: 1
 right: 0
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-relational::worth_relational$history::commit::envelope_residency::memory_publication_tests::memory_frame_quota_and_cancel_refuse_before_position_or_head

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'history::commit::envelope_residency::memory_publication_tests::memory_frame_quota_and_cancel_refuse_before_position_or_head' (21720) panicked at crates\worth-relational\src\history\commit\envelope_residency\memory_publication_tests.rs:154:9:
the actual frame reservation must refuse
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-relational::worth_relational$storage::overlay::partition_content::digest_index::carrier_tests::inline_digest_headers_measure_empty_singleton_dense_and_retained_owners

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
DIGEST_HEADER_MEASUREMENT {"axis":"record_history_u64","scenario":"empty","retained_roots":1,"old_map_nodes":0,"new_map_nodes":0,"old_map_node_bytes":null,"new_map_node_bytes":null,"old_unique_header_arcs":0,"header_arc_bytes":32,"shared_child_owners":0,"shared_child_bytes":0,"old_total_bytes":0,"new_total_bytes":0,"new_minus_old_bytes":0}

thread 'storage::overlay::partition_content::digest_index::carrier_tests::inline_digest_headers_measure_empty_singleton_dense_and_retained_owners' (34668) panicked at crates\worth-relational\src\storage\overlay\partition_content\digest_index\carrier_tests.rs:87:9:
assertion `left == right` failed
  left: 80
 right: 32
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-relational::worth_relational$storage::substrate::record_arena::values::metadata::tests::metadata_controls_follow_actual_history_snapshots_and_refund

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'storage::substrate::record_arena::values::metadata::tests::metadata_controls_follow_actual_history_snapshots_and_refund' (67200) panicked at crates\worth-relational\src\storage\substrate\record_arena\values\metadata\tests.rs:123:5:
assertion `left == right` failed
  left: 504
 right: 360
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-relational::worth_relational$storage::substrate::record_arena::values::metadata::tests::metadata_controls_refuse_before_history_edits_with_typed_cause

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'storage::substrate::record_arena::values::metadata::tests::metadata_controls_refuse_before_history_edits_with_typed_cause' (64648) panicked at crates\worth-relational\src\storage\substrate\record_arena\values\metadata\tests.rs:64:5:
assertion failed: matches!(denied, Err(Cause::PolicyMemoryExhausted { requested, .. }) if
    requested == bill)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-relational::worth_relational$storage::substrate::record_arena::values::metadata::tests::metadata_footer_preserves_legacy_named_and_compact_wire

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'storage::substrate::record_arena::values::metadata::tests::metadata_footer_preserves_legacy_named_and_compact_wire' (51832) panicked at crates\worth-relational\src\storage\substrate\record_arena\values\metadata\tests.rs:225:5:
assertion `left == right` failed
  left: 352
 right: 256
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-relational::worth_relational$storage::substrate::record_arena::values::metadata::tests::moved_metadata_retains_only_its_fixed_bill_and_ordinary_clone_is_independent

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'storage::substrate::record_arena::values::metadata::tests::moved_metadata_retains_only_its_fixed_bill_and_ordinary_clone_is_independent' (73992) panicked at crates\worth-relational\src\storage\substrate\record_arena\values\metadata\tests.rs:155:5:
assertion `left == right` failed: only the fixed metadata bill survives the unique move
  left: 168
 right: 120
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-relational::worth_relational$storage::substrate::record_arena::values::tests::escaped_extra_carrier_keeps_only_its_own_bill_after_node_drop

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'storage::substrate::record_arena::values::tests::escaped_extra_carrier_keeps_only_its_own_bill_after_node_drop' (50652) panicked at crates\worth-relational\src\storage\substrate\record_arena\values\tests.rs:318:5:
assertion `left == right` failed
  left: 385
 right: 337
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-relational::worth_relational$storage::substrate::record_arena::values::tests::funded_extra_refuses_copy_and_replacement_before_growth

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'storage::substrate::record_arena::values::tests::funded_extra_refuses_copy_and_replacement_before_growth' (34000) panicked at crates\worth-relational\src\storage\substrate\record_arena\values\tests.rs:169:5:
assertion `left == right` failed
  left: 10600
 right: 10552
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-relational::worth_relational$storage::substrate::record_arena::values::tests::funded_extra_shares_actual_row_history_and_private_holds_until_last_drop

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'storage::substrate::record_arena::values::tests::funded_extra_shares_actual_row_history_and_private_holds_until_last_drop' (40852) panicked at crates\worth-relational\src\storage\substrate\record_arena\values\tests.rs:116:5:
assertion `left == right` failed
  left: 368
 right: 320
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-relational::worth_relational$storage::substrate::record_arena::values::tests::unique_extra_move_and_borrowed_export_preserve_original_funding

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'storage::substrate::record_arena::values::tests::unique_extra_move_and_borrowed_export_preserve_original_funding' (76140) panicked at crates\worth-relational\src\storage\substrate\record_arena\values\tests.rs:204:5:
assertion `left == right` failed: the whole unique extra moved, including its footer
  left: 375
 right: 327
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-relational::worth_relational$storage::substrate::shared_map::custody_tests::map_cow_owns_only_actual_nodes_and_unique_replacement_refunds_nothing_early

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'storage::substrate::shared_map::custody_tests::map_cow_owns_only_actual_nodes_and_unique_replacement_refunds_nothing_early' (40948) panicked at crates\worth-relational\src\storage\substrate\shared_map\custody_tests.rs:19:5:
assertion `left == right` failed
  left: 20480
 right: 14336
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-relational::worth_relational$storage::substrate::shared_map::tests::inline_header_footprints_remove_only_actual_header_arcs

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'storage::substrate::shared_map::tests::inline_header_footprints_remove_only_actual_header_arcs' (40032) panicked at crates\worth-relational\src\storage\substrate\shared_map\tests.rs:157:5:
assertion `left == right` failed
  left: 24
 right: 72
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-relational::worth_relational$tests::durability::contracts::branch_root_shared_target_binding::checkpoint_rejects_later_shared_root_cell_with_relabeled_schema_root

CLEANUP-malformed-input-boundary: The failing assertion compares ordering or typed malformed-input diagnostics in a fault court.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'tests::durability::contracts::branch_root_shared_target_binding::checkpoint_rejects_later_shared_root_cell_with_relabeled_schema_root' (39288) panicked at crates\worth-relational\src\tests\durability\contracts\branch_root_content_binding.rs:320:5:
unexpected denial: DurabilityError { class: CorruptCheckpoint, detail: "branch cell `shared-root-sibling` target does not match immutable commit artifact `1`: target version/parents/roots = 1/[]/RelationalBranchRootDescriptor { truth_root: CanonicalDigestId([240, 8, 49, 175, 98, 98, 7, 71, 71, 108, 109, 33, 35, 88, 194, 3, 159, 176, 242, 209, 96, 10, 218, 83, 5, 164, 202, 92, 168, 48, 140, 166]), schema_root: CanonicalDigestId([197, 97, 197, 233, 123, 140, 54, 30, 146, 28, 253, 137, 138, 5, 65, 1, 73, 142, 153, 85, 71, 242, 107, 77, 240, 100, 71, 241, 106, 117, 106, 86]) }, artifact = 1/[]/RelationalBranchRootDescriptor { truth_root: CanonicalDigestId([240, 8, 49, 175, 98, 98, 7, 71, 71, 108, 109, 33, 35, 88, 194, 3, 159, 176, 242, 209, 96, 10, 218, 83, 5, 164, 202, 92, 168, 48, 140, 166]), schema_root: CanonicalDigestId([133, 97, 197, 233, 123, 140, 54, 30, 146, 28, 253, 137, 138, 5, 65, 1, 73, 142, 153, 85, 71, 242, 107, 77, 240, 100, 71, 241, 106, 117, 106, 86]) }", history_drift_class: None, authority_continuity_mismatch: None, storage_unavailable: None, context: ErrorContext { subsystem: Durability, operation: Recover, affected_records: [], version_context: None, suggested_fix: Some(RepairDurableStore) } }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-relational::worth_relational$tests::durability::contracts::branch_root_shared_target_binding::checkpoint_rejects_later_shared_root_cell_with_relabeled_truth_root

CLEANUP-malformed-input-boundary: The failing assertion compares ordering or typed malformed-input diagnostics in a fault court.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'tests::durability::contracts::branch_root_shared_target_binding::checkpoint_rejects_later_shared_root_cell_with_relabeled_truth_root' (9488) panicked at crates\worth-relational\src\tests\durability\contracts\branch_root_content_binding.rs:320:5:
unexpected denial: DurabilityError { class: CorruptCheckpoint, detail: "branch cell `shared-root-sibling` target does not match immutable commit artifact `1`: target version/parents/roots = 1/[]/RelationalBranchRootDescriptor { truth_root: CanonicalDigestId([38, 15, 14, 121, 126, 225, 71, 108, 186, 49, 130, 35, 63, 67, 175, 227, 107, 1, 134, 220, 80, 161, 183, 155, 119, 64, 147, 173, 64, 213, 172, 200]), schema_root: CanonicalDigestId([133, 97, 197, 233, 123, 140, 54, 30, 146, 28, 253, 137, 138, 5, 65, 1, 73, 142, 153, 85, 71, 242, 107, 77, 240, 100, 71, 241, 106, 117, 106, 86]) }, artifact = 1/[]/RelationalBranchRootDescriptor { truth_root: CanonicalDigestId([166, 15, 14, 121, 126, 225, 71, 108, 186, 49, 130, 35, 63, 67, 175, 227, 107, 1, 134, 220, 80, 161, 183, 155, 119, 64, 147, 173, 64, 213, 172, 200]), schema_root: CanonicalDigestId([133, 97, 197, 233, 123, 140, 54, 30, 146, 28, 253, 137, 138, 5, 65, 1, 73, 142, 153, 85, 71, 242, 107, 77, 240, 100, 71, 241, 106, 117, 106, 86]) }", history_drift_class: None, authority_continuity_mismatch: None, storage_unavailable: None, context: ErrorContext { subsystem: Durability, operation: Recover, affected_records: [], version_context: None, suggested_fix: Some(RepairDurableStore) } }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-relational::worth_relational$visibility::materialization::read_records::projection::field_revisions::restore_tests::checkpoint_restore_rejects_incomplete_field_revision_column

CLEANUP-malformed-input-boundary: The failing assertion compares ordering or typed malformed-input diagnostics in a fault court.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'visibility::materialization::read_records::projection::field_revisions::restore_tests::checkpoint_restore_rejects_incomplete_field_revision_column' (52248) panicked at crates\worth-relational\src\visibility\materialization\read_records\projection\field_revisions\restore_tests.rs:324:5:
DurabilityError { class: CorruptCheckpoint, detail: "record arena SoA column lengths differ", history_drift_class: None, authority_continuity_mismatch: None, storage_unavailable: None, context: ErrorContext { subsystem: Durability, operation: Recover, affected_records: [], version_context: None, suggested_fix: Some(RepairDurableStore) } }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-court-relational-certification::worth_court_relational_certification$cost::relational::checkpoint_stage::checkpoint_stage_charges_the_documented_rate_for_each_encoded_byte

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'cost::relational::checkpoint_stage::checkpoint_stage_charges_the_documented_rate_for_each_encoded_byte' (52248) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\cost\relational\checkpoint_stage.rs:91:9:
assertion `left == right` failed: rows=0
  left: 2628
 right: 1108
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-court-relational-certification::worth_court_relational_certification$cost::relational::checkpoint_stage::empty_checkpoint_memory_boundary_is_its_bytes_plus_one_stage

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'cost::relational::checkpoint_stage::empty_checkpoint_memory_boundary_is_its_bytes_plus_one_stage' (81816) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\cost\relational\checkpoint_stage.rs:215:9:
leased=false: Err(Request(PolicyMemoryExhausted { requested: 2628, admitted: 724, ancestor: 0 }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-court-relational-certification::worth_court_relational_certification$cost::relational::relational_certification::retained_entity_selection::equal_payload_jumps_charge_the_same_receiving_owner_work_at_near_and_distant_history_nodes

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'cost::relational::relational_certification::retained_entity_selection::equal_payload_jumps_charge_the_same_receiving_owner_work_at_near_and_distant_history_nodes' (48116) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\cost\relational\relational_certification\retained_entity_selection.rs:52:5:
assertion `left == right` failed: actual request charged work
  left: 7095
 right: 7094
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-court-relational-certification::worth_court_relational_certification$cost::relational::checkpoint_stage::checkpoint_capture_sets_up_exactly_one_stage_whatever_the_row_count

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'cost::relational::checkpoint_stage::checkpoint_capture_sets_up_exactly_one_stage_whatever_the_row_count' (78460) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\cost\relational\checkpoint_stage.rs:135:9:
assertion `left == right` failed: rows=0
  left: ExecutionScopeSetupCountsForTest { stage_setups: 14, limit_derivations: 14, admissions: 14 }
 right: ExecutionScopeSetupCountsForTest { stage_setups: 1, limit_derivations: 1, admissions: 1 }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-court-relational-certification::worth_court_relational_certification$cost::relational::checkpoint_stage::empty_checkpoint_work_ceiling_bounds_the_whole_capture

CLEANUP-C2-resource-accounting: The failing assertion is the explicit work, quota, refund or footprint bound, not benign setup (independent review of all69 records).

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'cost::relational::checkpoint_stage::empty_checkpoint_work_ceiling_bounds_the_whole_capture' (9504) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\cost\relational\checkpoint_stage.rs:169:9:
assertion `left == right` failed: ceiling=2376
  left: 550
 right: 585
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-court-relational-certification::worth_court_relational_certification$process::serial::relational::leased_preparation::preimage::preimage_endpoint_slots

CLEANUP-C2-resource-accounting: Exact child exit101: actual1024-endpoint commit exceeds remaining64MiB fixture grant by1118 bytes before COW growth assertions; no overflow. Evidence: pa-merge-preimage-child-exit.log; milestone C2 backing/work-budget and growth courts.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'process::serial::relational::leased_preparation::preimage::preimage_endpoint_slots' (21928) panicked at crates\worth-testkit-relational\src\test_process\declarations.rs:16:5:
isolated court process::serial::relational::leased_preparation::preimage::preimage_endpoint_slots failed:

running 1 test
```

## worth-query::worth_query$domain_installation::operation_execution::workflow_parallel_progression::frontier::tests::dispatch::reversed_completion_still_applies_canonical_effects_and_receipts

CLEANUP-C2-cost-or-growth: Complete original SDK invoice45717 versus1992.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'domain_installation::operation_execution::workflow_parallel_progression::frontier::tests::dispatch::reversed_completion_still_applies_canonical_effects_and_receipts' (40836) panicked at crates\worth-query\src\domain_installation\operation_execution\workflow_execution\progression\frontier\tests\dispatch.rs:277:5:
assertion `left == right` failed: complete original SDK invoice
  left: 45717
 right: 1992
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-query::worth_query$domain_installation::operation_execution::workflow_parallel_progression::frontier::tests::dispatch::stops::admitted_opening_refuses_the_owned_maps_declared_reservation

CLEANUP-C2-cost-or-growth: Declared owned-map reservation3184 versus1002.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'domain_installation::operation_execution::workflow_parallel_progression::frontier::tests::dispatch::stops::admitted_opening_refuses_the_owned_maps_declared_reservation' (66404) panicked at C:\programming\worth worktees\worktrees\m3-host-home\crates\worth-proof\src\transition\outcomes.rs:99:36:
called TransitionOutcome::unwrap on Denied(WorthQueryWorkflowStartDenial { kind: ManagedRun("original request refused immutable lowering storage: Admission(MemoryExhausted(Capacity { requested: 3184, admitted: 1002, level: Policy { ancestor: 0 } }))"), counters: WorthQueryWorkflowRunCounters { runtime_authority_checks: 0, stage_index_lookups: 0, stage_admission_checks: 0, predecessor_checks: 0, predecessor_receipt_lookups: 0, required_capability_checks: 0, required_domain_checks: 0, graph_read_contacts: 0, touch_effect_contacts: 0, effect_receipt_checks: 0, commit_admission_contacts: 0, invariant_checks: 0, parallel_admission_checks: 0, stage_executor_contacts: 0, computation_charged_work: 0, output_contract_checks: 0, terminal_contract_checks: 0, consumption_contacts: 0, unrelated_run_scans: 0, conditional_request_admission_checks: 0, conditional_contract_lookups: 0, conditional_dependency_observation_reads: 0, conditional_dependency_checks: 0, conditional_semantic_reads: 0, conditional_condition_checks: 0, conditional_condition_deferrals: 0, conditional_temporal_deferrals: 0, conditional_on_demand_deferrals: 0, conditional_comparator_checks: 0, conditional_compute_contacts: 0, conditional_output_version_reads: 0, conditional_runtime_dependency_edges_captured: 0, conditional_application_contacts: 0, conditional_semantic_classifications: 0, conditional_reverted_clean_outcomes: 0, conditional_semantic_changes: 0, conditional_reuse_checks: 0, conditional_decisions_delivered: 0 } })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-query-court-decl-execution-package-archive::worth_query_court_decl_execution_package_archive$domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::retained_recovery::checkpoint_inspection_compares_query_header_count_and_output_metadata_bytes

CLEANUP-C1-recovery-or-fault: Authenticated metadata fault uses stale v13 offsets; attacks Native bytes and hits work denial before comparison.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::retained_recovery::checkpoint_inspection_compares_query_header_count_and_output_metadata_bytes' (9712) panicked at crates\worth-query-court-decl-execution-package-archive\tests\worth_query_court_decl_execution_package_archive\domain_computation\primary_graph\output_lineage\ownership_oracle\optional_aspects\retained_recovery\mod.rs:251:22:
called `Result::unwrap()` on an `Err` value: Native(Request(WorkExhausted))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-query-court-decl-host::worth_query_court_decl_host$certification::document_history::group::recovery::an_unpublished_first_member_resumes_once_then_completes_the_one_group_step

CLEANUP-C1-recovery-or-fault: Cancellation injected between owner effect and product movement; unpublished group recovery assertion fails.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'certification::document_history::group::recovery::an_unpublished_first_member_resumes_once_then_completes_the_one_group_step' (67536) panicked at crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\group\recovery.rs:155:9:
assertion failed: matches!(execution.completed_source(0).unwrap().evidence(),
    WorthQuerySourceCompletion::Recovered { publication: Some(_), .. })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-query-court-decl-host::worth_query_court_decl_host$certification::document_history::recording::recovery::unpublished_candidate_keeps_its_original_recovery_until_one_recorded_selection

CLEANUP-C1-recovery-or-fault: Cancellation injected between owner effect and product movement; original retained recovery assertion fails.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'certification::document_history::recording::recovery::unpublished_candidate_keeps_its_original_recovery_until_one_recorded_selection' (33636) panicked at crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\certification\document_history\recording\recovery.rs:119:9:
keep the original owner-issued recovered publication, without replaying the handler
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-query-court-decl-host::worth_query_court_decl_host$producer_succession::history_courts::checkpoint_keeps_exact_per_route_retirement_and_recovers_the_surviving_generator

CLEANUP-C2-cost-or-growth: Coordinator explicitly holds exact per-route retirement shrink asC2; retained roots0 versus4.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'producer_succession::history_courts::checkpoint_keeps_exact_per_route_retirement_and_recovers_the_surviving_generator' (37228) panicked at crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\producer_succession\checkpoint.rs:143:9:
assertion `left == right` failed: all retained roots: before={CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 0, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 1, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 2, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 3, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 4, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 5, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 6, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 7, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 8, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 9, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 10, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 11, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 12, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 13, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 14, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 15, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 16, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 17, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 18, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 0, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 1, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 2, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 3, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 4, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 5, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 6, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 7, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 8, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 9, generation: 1 }}, after={CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 0, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 1, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 2, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 3, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 4, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 5, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 6, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 7, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 8, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 9, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 10, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 11, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 12, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 13, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 14, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 15, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 16, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 17, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 0, slot: 18, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 6, slot: 0, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 7, slot: 0, generation: 1 }, CheckpointRecordIdentity { class: Entity, partition_id: 8, slot: 0, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 0, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 1, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 2, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 3, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 4, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 5, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 6, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 7, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 8, generation: 1 }, CheckpointRecordIdentity { class: Relation, partition_id: 0, slot: 9, generation: 1 }}
  left: 0
 right: 4
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-query-court-decl-host-package-archive-replay-application-graph::worth_query_court_decl_host_package_archive_replay_application_graph$certification::application_graph::restored_primary_backend::the_first_production_write_after_restore_answers_as_before_the_capture

CLEANUP-C1-recovery-or-fault: CommitThenDecline adapter injects post-Native-effect refusal twice before capture; WrongImmutableTarget before cold first-write proof.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'certification::application_graph::restored_primary_backend::the_first_production_write_after_restore_answers_as_before_the_capture' (60696) panicked at crates\worth-query-court-decl-host-package-archive-replay-application-graph\tests\worth_query_court_decl_host_package_archive_replay_application_graph\certification\application_graph\restored_primary_backend\mod.rs:22:18:
the host captures: WorldOriginals(Component(Basis(WrongImmutableTarget)))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-query-court-execution-primary-graph::worth_query_court_execution_primary_graph$domain_computation::primary_graph::tests::application_attempt::unwind_custody::world_unwind_retains_and_releases_the_exact_query_idempotency_slot

CLEANUP-C1-recovery-or-fault: Injected unwind custody cleanup leaves Query idempotency slot1, expected0.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'domain_computation::primary_graph::tests::application_attempt::unwind_custody::world_unwind_retains_and_releases_the_exact_query_idempotency_slot' (36112) panicked at C:\programming\worth worktees\worktrees\m3-host-home\crates\worth-runtime-world\src\lifecycle\operation_control.rs:164:17:
injected unwind before product comparison
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'domain_computation::primary_graph::tests::application_attempt::unwind_custody::world_unwind_retains_and_releases_the_exact_query_idempotency_slot' (36112) panicked at crates\worth-query-court-execution-primary-graph\tests\worth_query_court_execution_primary_graph\domain_computation\primary_graph\tests\application_attempt\unwind_custody\mod.rs:139:5:
assertion `left == right` failed: cleanup releases Query's exact recovery-indexed slot
  left: 1
 right: 0
```

## worth-query-court-execution-primary-graph::worth_query_court_execution_primary_graph$domain_computation::primary_graph::tests::checkpoint_allocation::leased_checkpoint_retains_one_frame_charge_through_native_handoff_and_reopen

CLEANUP-C2-cost-or-growth: Leased checkpoint allocation child first fails zero-memory denial category; actual variant not printed, later stale offsets unreached; no overflow evidence.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'domain_computation::primary_graph::tests::checkpoint_allocation::leased_checkpoint_retains_one_frame_charge_through_native_handoff_and_reopen' (80612) panicked at crates\worth-query-testkit-execution-primary-graph-application-attempt-2\src\domain_computation\primary_graph\bootstrap_publication\execution_refusals\mod.rs:59:5:

running 1 test
```

## worth-query-court-query-decl-host::worth_query_court_query_decl_host$certification::application_graph::canonical_transition::related::companion_transition::capture::companion_post_ack_capture_refusal_repairs_same_capsule_with_historical_roster

CLEANUP-C1-post-ACK-deliberate-refusal: Benign cold-history setup repaired; now reaches unchanged deliberate zero-memory post-ACK repair assertion. Actual refusal variant is not printed; do not invent it.

Evidence: `target/lane-evidence/pa-merge-repair/query/run.log`. Original assertion/stage:

```text
thread 'certification::application_graph::canonical_transition::related::companion_transition::capture::companion_post_ack_capture_refusal_repairs_same_capsule_with_historical_roster' (18716) panicked at crates\worth-query-testkit-host\src\host\support\checkpoint_capture.rs:34:23:
capture-only limit must retain acknowledged custody: Graph(WorthQueryPrimaryGraphInstallationDenial { kind: CheckpointRecoveryRejected, subject: "native codec transition admission refused: Relational schema transition admission denied: CodecClosureRejected (relation exact canonical creation absent)", cause: None })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-query-court-query-decl-host::worth_query_court_query_decl_host$certification::application_graph::canonical_transition::related::settlement::related_acknowledged_successor_repairs_capture_without_reauthoring_or_old_decode

CLEANUP-C1-post-ACK-deliberate-refusal: Benign cold-history setup repaired; now reaches unchanged deliberate zero-memory post-ACK repair assertion. Actual refusal variant is not printed; do not invent it.

Evidence: `target/lane-evidence/pa-merge-repair/query/run.log`. Original assertion/stage:

```text
thread 'certification::application_graph::canonical_transition::related::settlement::related_acknowledged_successor_repairs_capture_without_reauthoring_or_old_decode' (56868) panicked at crates\worth-query-testkit-host\src\host\support\checkpoint_capture.rs:34:23:
capture-only limit must retain acknowledged custody: Graph(WorthQueryPrimaryGraphInstallationDenial { kind: CheckpointRecoveryRejected, subject: "native codec transition admission refused: Relational schema transition admission denied: CodecClosureRejected (relation exact canonical creation absent)", cause: None })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-query-court-query-decl-host::worth_query_court_query_decl_host$host::application_home::recovered_custody::recovered_original_custody_reaches_service_and_continues_without_reexecution

CLEANUP-C1-recovery-or-fault: Injected unwind recovery reaches cross-incarnation attempt identity inequality before continuing; held ProductBasisStale not reached.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'host::application_home::recovered_custody::recovered_original_custody_reaches_service_and_continues_without_reexecution' (8636) panicked at C:\programming\worth worktees\worktrees\m3-host-home\crates\worth-runtime-world\src\lifecycle\operation_control.rs:164:17:
injected unwind before product comparison
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'host::application_home::recovered_custody::recovered_original_custody_reaches_service_and_continues_without_reexecution' (8636) panicked at crates\worth-query-court-query-decl-host\tests\worth_query_court_query_decl_host\host\application_home\recovered_custody\disposition.rs:27:5:
assertion `left == right` failed
  left: CompositePublicationAttemptIdentity { owner: RuntimeWorldOwnerIdentity { scope: Home(DurableWorldOwnerIdentity { home: HomeIdentity([36, 152, 222, 209, 143, 182, 228, 44, 171, 108, 195, 171, 214, 117, 187, 244]), slot: WorldOwnerSlotIdentity(0) }), instance: EphemeralWorldOwnerIdentity(1), home_fence: Some(HomeIncarnationFence { incarnation: 2 }) }, ordinal: 1 }
 right: CompositePublicationAttemptIdentity { owner: RuntimeWorldOwnerIdentity { scope: Home(DurableWorldOwnerIdentity { home: HomeIdentity([36, 152, 222, 209, 143, 182, 228, 44, 171, 108, 195, 171, 214, 117, 187, 244]), slot: WorldOwnerSlotIdentity(0) }), instance: EphemeralWorldOwnerIdentity(0), home_fence: Some(HomeIncarnationFence { incarnation: 1 }) }, ordinal: 1 }
```

## worth-query-court-query-decl-host::worth_query_court_query_decl_host$host::application_home::recovered_custody::recovered_original_custody_reaches_service_and_release_stays_released

CLEANUP-C1-recovery-or-fault: Injected unwind recovery reaches cross-incarnation attempt identity inequality before releasing; held ProductBasisStale not reached.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'host::application_home::recovered_custody::recovered_original_custody_reaches_service_and_release_stays_released' (9348) panicked at C:\programming\worth worktees\worktrees\m3-host-home\crates\worth-runtime-world\src\lifecycle\operation_control.rs:164:17:
injected unwind before product comparison
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'host::application_home::recovered_custody::recovered_original_custody_reaches_service_and_release_stays_released' (9348) panicked at crates\worth-query-court-query-decl-host\tests\worth_query_court_query_decl_host\host\application_home\recovered_custody\disposition.rs:27:5:
assertion `left == right` failed
  left: CompositePublicationAttemptIdentity { owner: RuntimeWorldOwnerIdentity { scope: Home(DurableWorldOwnerIdentity { home: HomeIdentity([166, 102, 221, 231, 151, 171, 84, 116, 47, 133, 239, 169, 2, 204, 137, 64]), slot: WorldOwnerSlotIdentity(0) }), instance: EphemeralWorldOwnerIdentity(1), home_fence: Some(HomeIncarnationFence { incarnation: 2 }) }, ordinal: 1 }
 right: CompositePublicationAttemptIdentity { owner: RuntimeWorldOwnerIdentity { scope: Home(DurableWorldOwnerIdentity { home: HomeIdentity([166, 102, 221, 231, 151, 171, 84, 116, 47, 133, 239, 169, 2, 204, 137, 64]), slot: WorldOwnerSlotIdentity(0) }), instance: EphemeralWorldOwnerIdentity(0), home_fence: Some(HomeIncarnationFence { incarnation: 1 }) }, ordinal: 1 }
```

## worth-query-court-query-decl-host::worth_query_court_query_decl_host$host::application_home::recovered_custody::continued_recovered_custody_closes_cooperatively_and_resumes_after_reopen

CLEANUP-C1-recovery-or-fault: Injected unwind recovery reaches cross-incarnation attempt identity inequality before cooperative close; held ProductBasisStale not reached.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'host::application_home::recovered_custody::continued_recovered_custody_closes_cooperatively_and_resumes_after_reopen' (72432) panicked at C:\programming\worth worktees\worktrees\m3-host-home\crates\worth-runtime-world\src\lifecycle\operation_control.rs:164:17:
injected unwind before product comparison
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'host::application_home::recovered_custody::continued_recovered_custody_closes_cooperatively_and_resumes_after_reopen' (72432) panicked at crates\worth-query-court-query-decl-host\tests\worth_query_court_query_decl_host\host\application_home\recovered_custody\disposition.rs:27:5:
assertion `left == right` failed
  left: CompositePublicationAttemptIdentity { owner: RuntimeWorldOwnerIdentity { scope: Home(DurableWorldOwnerIdentity { home: HomeIdentity([113, 28, 195, 68, 198, 164, 59, 66, 126, 45, 225, 159, 72, 242, 108, 39]), slot: WorldOwnerSlotIdentity(0) }), instance: EphemeralWorldOwnerIdentity(1), home_fence: Some(HomeIncarnationFence { incarnation: 2 }) }, ordinal: 1 }
 right: CompositePublicationAttemptIdentity { owner: RuntimeWorldOwnerIdentity { scope: Home(DurableWorldOwnerIdentity { home: HomeIdentity([113, 28, 195, 68, 198, 164, 59, 66, 126, 45, 225, 159, 72, 242, 108, 39]), slot: WorldOwnerSlotIdentity(0) }), instance: EphemeralWorldOwnerIdentity(0), home_fence: Some(HomeIncarnationFence { incarnation: 1 }) }, ordinal: 1 }
```

## worth-query-court-execution-primary-graph-application-attempt::worth_query_court_execution_primary_graph_application_attempt$domain_computation::primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::execution::authority_cause::tests::authority_evidence_exposes_request_resources_by_value_through_the_commit_door

CLEANUP-C2-incomplete-charge-prefix-sweep: All unique charged prefixes are replayed at required-1; stopped exact own process after prolonged busy execution. Incomplete, not passed or an assertion result; abort-v2 evidence retained.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

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
authority probe=5500, req
```

## worth-query-court-decl-host-package-archive-replay-application-graph::worth_query_court_decl_host_package_archive_replay_application_graph$certification::application_graph::relation_laws::logical::locality::logical_local_rule_business_acquisitions_ignore_ten_and_ten_thousand_unrelated_cohorts

CLEANUP-C2-cost-or-growth: Ten-versus-ten-thousand cohort growth journey expires30000ms candidate during setup.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
test certification::application_graph::relation_laws::logical::locality::logical_local_rule_business_acquisitions_ignore_ten_and_ten_thousand_unrelated_cohorts has been running for over 60 seconds

thread 'certification::application_graph::relation_laws::logical::locality::logical_local_rule_business_acquisitions_ignore_ten_and_ten_thousand_unrelated_cohorts' (81268) panicked at crates\worth-query-testkit-certification-application-graph\src\certification\application_graph\relation_laws\installation.rs:37:59:
called `Result::unwrap()` on an `Err` value: Publication(WorthQueryPrimaryGraphInstallationDenial { kind: RelationalCommitRejected, subject: "Relational rejected the application seed transaction: branch publication deferred: CandidateLifetimeExpired { maximum_lifetime_millis: 30000 }", cause: Some(NativeValidation(WorthQueryInvariantExecutionFailure { kind: ProviderRejected, posture: Denied, detail: "Relational rejected the installed proposed-state invariant", custom_invariant: None, native: None })) })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-query-court-query-decl-host::worth_query_court_query_decl_host$process::certification::program_example_authority::execution_memory::execution_memory_is_shared_by_installed_worlds_and_reselected_on_checkpoint_reopen

CLEANUP-C2-memory-ledger: Exact existing child probe fails expected process-capacity denial after checkpoint reopen; actual variant not printed, no overflow evidence.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'process::certification::program_example_authority::execution_memory::execution_memory_is_shared_by_installed_worlds_and_reselected_on_checkpoint_reopen' (53608) panicked at crates\worth-query-court-query-decl-host\tests\worth_query_court_query_decl_host\process\certification\program_example_authority\execution_memory\mod.rs:18:9:

running 1 test
```

## worth-query-court-query-decl-host::worth_query_court_query_decl_host$process::host::durable_retry::request_stops::original_directory_receipt_preserves_cancellation_at_every_consult

CLEANUP-C1-incomplete-consult-cancellation-sweep: Cancellation at every consult; stopped exact own busy child after>10min. Incomplete, not passed; abort evidence retained.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
test process::host::durable_retry::request_stops::original_directory_receipt_preserves_cancellation_at_every_consult has been running for over 60 seconds
```

## worth-query-bank-court-external-rail-server::worth_query_bank_court_external_rail_server$server::ordinary_reads::pending_payments::empty_guarded_frontier_work_grows_at_most_linearly

CLEANUP-C2-cost-or-growth: Explicit fanout/linear growth court is denied by original seed-memory quota during large-population setup, before the work-bound assertion. Grants and assertions unchanged.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'server::ordinary_reads::pending_payments::empty_guarded_frontier_work_grows_at_most_linearly' (2428) panicked at crates\worth-query-bank-testkit-server\src\support\mod.rs:79:34:
bank test runtime should install: ApplicationInstallation(ConditionalPublication(WorthQueryConditionalRuntimeInstallationDenial { kind: PrimaryGraphPublication, subject: "primary graph installation denied: ExecutionDenied { kind: ExecutionResource { denial: MemoryLimit { requested: 61963880, admitted: 61961560, level: Policy }, partition_identity: None, policy_ancestor: Some(0) } } (Relational rejected the application seed transaction: commit preparation execution stopped: CommitExecutionDenial { kind: Cause(PolicyMemoryExhausted { requested: 61963880, admitted: 61961560, ancestor: 0 }), partition_identity: None })" }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-query-bank-court-external-rail-server::worth_query_bank_court_external_rail_server$cost::server::ordinary_reads::canonical_work_scale::policy_fact_fanout_does_not_leak_into_closed_disclosure_evidence

CLEANUP-C2-cost-or-growth: Explicit fanout/linear growth court is denied by original seed-memory quota during large-population setup, before the work-bound assertion. Grants and assertions unchanged.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'cost::server::ordinary_reads::canonical_work_scale::policy_fact_fanout_does_not_leak_into_closed_disclosure_evidence' (31852) panicked at crates\worth-query-bank-testkit-server\src\support\mod.rs:79:34:
bank test runtime should install: ApplicationInstallation(ConditionalPublication(WorthQueryConditionalRuntimeInstallationDenial { kind: PrimaryGraphPublication, subject: "primary graph installation denied: ExecutionDenied { kind: ExecutionResource { denial: MemoryLimit { requested: 48762312, admitted: 48762199, level: Policy }, partition_identity: None, policy_ancestor: Some(0) } } (Relational rejected the application seed transaction: commit preparation execution stopped: CommitExecutionDenial { kind: Cause(PolicyMemoryExhausted { requested: 48762312, admitted: 48762199, ancestor: 0 }), partition_identity: None })" }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-query-bank-court-external-rail-server::worth_query_bank_court_external_rail_server$cost::server::ordinary_reads::canonical_work_scale::result_and_graph_fanout_is_visible_only_as_closed_public_work

CLEANUP-C2-cost-or-growth: Explicit fanout/linear growth court is denied by original seed-memory quota during large-population setup, before the work-bound assertion. Grants and assertions unchanged.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'cost::server::ordinary_reads::canonical_work_scale::result_and_graph_fanout_is_visible_only_as_closed_public_work' (40708) panicked at crates\worth-query-bank-testkit-server\src\support\mod.rs:79:34:
bank test runtime should install: ApplicationInstallation(ConditionalPublication(WorthQueryConditionalRuntimeInstallationDenial { kind: PrimaryGraphPublication, subject: "primary graph installation denied: ExecutionDenied { kind: ExecutionResource { denial: MemoryLimit { requested: 48762312, admitted: 48762199, level: Policy }, partition_identity: None, policy_ancestor: Some(0) } } (Relational rejected the application seed transaction: commit preparation execution stopped: CommitExecutionDenial { kind: Cause(PolicyMemoryExhausted { requested: 48762312, admitted: 48762199, ancestor: 0 }), partition_identity: None })" }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-query-bank-court-external-rail-server::worth_query_bank_court_external_rail_server$cost::server::ordinary_reads::locality::account_discovery_root_union_work_grows_at_most_linearly

CLEANUP-C2-cost-or-growth: Explicit fanout/linear growth court is denied by original seed-memory quota during large-population setup, before the work-bound assertion. Grants and assertions unchanged.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'cost::server::ordinary_reads::locality::account_discovery_root_union_work_grows_at_most_linearly' (56064) panicked at crates\worth-query-bank-testkit-server\src\support\mod.rs:79:34:
bank test runtime should install: ApplicationInstallation(ConditionalPublication(WorthQueryConditionalRuntimeInstallationDenial { kind: PrimaryGraphPublication, subject: "primary graph installation denied: ExecutionDenied { kind: ExecutionResource { denial: MemoryLimit { requested: 8440, admitted: 594, level: Policy }, partition_identity: None, policy_ancestor: Some(0) } } (Relational rejected the application seed transaction: commit preparation execution stopped: CommitExecutionDenial { kind: Cause(PolicyMemoryExhausted { requested: 8440, admitted: 594, ancestor: 0 }), partition_identity: None })" }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## parameter_invariant::lease_tests::parameter_lease_one_unit_short_is_the_work_stop

CLEANUP-C2-exact-work-boundary: Explicit paid.work-1 or2*ENTRY_WORK-1 resource court; original boundary and exact outcome assertion stay unchanged.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'parameter_invariant::lease_tests::parameter_lease_one_unit_short_is_the_work_stop' (51228) panicked at parameter_entry\src\parameter_invariant\lease_tests.rs:120:9:
ProductUnpublished(WorthQueryProductUnpublishedApplication { handle: ProductUnpublishedRecoveryHandle { identity: ProductUnpublishedOwnerEffectsIdentity { owner: RuntimeWorldOwnerIdentity { scope: Ephemeral(EphemeralWorldOwnerIdentity(4)), instance: EphemeralWorldOwnerIdentity(4), home_fence: None }, ordinal: 0 }, catalog_affinity: 1436974096688 }, .. })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## application_invariant_acceptance::proof::application_program::discovered_owning::retention::discovered_owning_committed_retention_retry_preserves_fresh_result_and_source

CLEANUP-C1-retention-or-publication-interruption: Actual retention reservation refusal, abandoned performed result, cancellation or paused publication precedes failing recovery assertion.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'application_invariant_acceptance::proof::application_program::discovered_owning::retention::discovered_owning_committed_retention_retry_preserves_fresh_result_and_source' (40044) panicked at consumer_root\src\application_invariant_acceptance\proof\application_program\discovered_owning\retention.rs:163:14:
fresh authorized retry promotes the same original source
```

## application_invariant_acceptance::proof::application_program::document_history::navigation_outputs::publication::stopped_recovery::stopped_original_recovery_retries_without_reexecuting_the_producer

CLEANUP-C1-retention-or-publication-interruption: Actual retention reservation refusal, abandoned performed result, cancellation or paused publication precedes failing recovery assertion.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'application_invariant_acceptance::proof::application_program::document_history::navigation_outputs::publication::stopped_recovery::stopped_original_recovery_retries_without_reexecuting_the_producer' (22808) panicked at consumer_root\src\application_invariant_acceptance\proof\application_program\document_history\navigation_outputs\publication\stopped_recovery.rs:145:21:
fresh recovery request: Outputs(Preparation(Demand(History(Demand(WorthQueryOutputDemandDenial { kind: Superseded, evidence: None, cleanup: None, subject: "output changed before its settlement was accepted", domain: None, readmission_failure: None, recovery_posture: Terminal })))))
```

## application_invariant_acceptance::proof::application_program::document_history::navigation_outputs::publication::unpublished_history_producer_resumes_original_effect_once_before_selecting_the_same_candidate

CLEANUP-C1-retention-or-publication-interruption: Actual retention reservation refusal, abandoned performed result, cancellation or paused publication precedes failing recovery assertion.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'application_invariant_acceptance::proof::application_program::document_history::navigation_outputs::publication::unpublished_history_producer_resumes_original_effect_once_before_selecting_the_same_candidate' (81048) panicked at consumer_root\src\application_invariant_acceptance\proof\application_program\document_history\navigation_outputs\publication.rs:282:17:
resume original World effect: Outputs(Preparation(Demand(History(Demand(WorthQueryOutputDemandDenial { kind: Superseded, evidence: None, cleanup: None, subject: "output changed before its settlement was accepted", domain: None, readmission_failure: None, recovery_posture: Terminal })))))
```

## checkpoint_recovery::component_islands::locality::serial::real_split_component_reads_are_unchanged_with_ten_and_ten_thousand_unrelated_islands

CLEANUP-C2-population-growth: 10/10000-island setup reaches canonical artifact size268435459 >268435456 before locality assertion.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'checkpoint_recovery::component_islands::locality::serial::real_split_component_reads_are_unchanged_with_ten_and_ten_thousand_unrelated_islands' (55244) panicked at topology_entry\src\checkpoint_recovery\component_islands\world.rs:68:14:
population setup leaves the World's request policy unchanged: ConditionalPublication(WorthQueryConditionalRuntimeInstallationDenial { kind: PrimaryGraphPublication, subject: "primary graph installation denied: RelationalCommitRejected (Relational rejected the application seed transaction: branch publication denied: CanonicalArtifactEncoding(SizeLimit { observed: 268435459, maximum: 268435456 }))" })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## checkpoint_recovery::computation_partition::oracle::advancement_custody::serial::caller_refresh_retry_spends_what_the_first_attempt_left

CLEANUP-C2-exact-work-boundary: Explicit paid.work-1 or2*ENTRY_WORK-1 resource court; original boundary and exact outcome assertion stay unchanged.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
finite retry: ceiling=1999999; charged=1990611; passes=[(Ok(()), 275981), (Ok(()), 275981), (Err(ExecutionRequest(Resource(WorkExhausted))), 395844)]; openings=1; producer_runs=1

thread 'checkpoint_recovery::computation_partition::oracle::advancement_custody::serial::caller_refresh_retry_spends_what_the_first_attempt_left' (1780) panicked at topology_entry\src\checkpoint_recovery\computation_partition\oracle\advancement_custody.rs:89:5:
assertion `left == right` failed: charged=1990611; attempts=[(Ok(()), 275981), (Ok(()), 275981), (Err(ExecutionRequest(Resource(WorkExhausted))), 395844)]; diagnostics=runs=1; first=Some(Ok((4607182418800017408, 1000697))); last=Some(Ok((4607182418800017408, 1000697)))
  left: ExecutionRequest(Resource(WorkExhausted))
 right: ExecutionRequest(NestedStopped)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## checkpoint_recovery::computation_partition::oracle::courtroom::lifecycle::serial::seeded_edits_cross_restore_and_republication_with_model_counts

CLEANUP-C4-generated-fault-or-race-oracle: Generated sequences include Fault/Repair and/or own-write interleavings and charged-work/observation comparison; actual failure after restored prefix or capture is preserved.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'checkpoint_recovery::computation_partition::oracle::courtroom::lifecycle::serial::seeded_edits_cross_restore_and_republication_with_model_counts' (50124) panicked at topology_entry\src\checkpoint_recovery\computation_partition\oracle\demand.rs:41:10:
the region output demand starts: Source(PrincipalResolution(WorthQueryPrincipalResolutionDenial { kind: Stopped(Ok(ExecutionResource { denial: WorkExhausted, partition_identity: None, policy_ancestor: None })), binding: "ConsumerPrincipalBinding", native_query_read_stop: None }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## checkpoint_recovery::computation_partition::oracle::differential::serial::every_reused_run_equals_a_full_run_of_the_same_facts

CLEANUP-C4-generated-fault-or-race-oracle: Generated sequences include Fault/Repair and/or own-write interleavings and charged-work/observation comparison; actual failure after restored prefix or capture is preserved.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'checkpoint_recovery::computation_partition::oracle::differential::serial::every_reused_run_equals_a_full_run_of_the_same_facts' (13844) panicked at topology_entry\src\checkpoint_recovery\computation_partition\oracle\demand.rs:41:10:
the region output demand starts: Source(PrincipalResolution(WorthQueryPrincipalResolutionDenial { kind: Stopped(Ok(ExecutionResource { denial: WorkExhausted, partition_identity: None, policy_ancestor: None })), binding: "ConsumerPrincipalBinding", native_query_read_stop: None }))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## checkpoint_recovery::computation_partition::oracle::restoration::serial::restoration_after_seeded_edits_rebuilds_fresh_state

CLEANUP-C4-generated-fault-or-race-oracle: Generated sequences include Fault/Repair and/or own-write interleavings and charged-work/observation comparison; actual failure after restored prefix or capture is preserved.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'checkpoint_recovery::computation_partition::oracle::restoration::serial::restoration_after_seeded_edits_rebuilds_fresh_state' (84544) panicked at topology_entry\src\checkpoint_recovery\computation_partition\oracle\restoration.rs:17:218:
called `Result::unwrap()` on an `Err` value: Execution(WorthQueryPrimaryGraphInstallationDenial { kind: ExecutionDenied { kind: ExecutionResource { denial: WorkExhausted, partition_identity: None, policy_ancestor: None } }, subject: "checkpoint history consolidation stopped", cause: None })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## checkpoint_recovery::required_chain::exact_invalidation::randomized::serial::randomized_marking_agrees_with_full_verification

CLEANUP-C4-generated-fault-or-race-oracle: Generated sequences include Fault/Repair and/or own-write interleavings and charged-work/observation comparison; actual failure after restored prefix or capture is preserved.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'checkpoint_recovery::required_chain::exact_invalidation::randomized::serial::randomized_marking_agrees_with_full_verification' (57364) panicked at topology_entry\src\checkpoint_recovery\required_chain\exact_invalidation\randomized.rs:97:21:
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
```

## checkpoint_recovery::required_chain::exact_invalidation::randomized::serial::randomized_marking_makes_room_on_a_branch_with_few_observations

CLEANUP-C4-generated-fault-or-race-oracle: Generated sequences include Fault/Repair and/or own-write interleavings and charged-work/observation comparison; actual failure after restored prefix or capture is preserved.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'checkpoint_recovery::required_chain::exact_invalidation::randomized::serial::randomized_marking_makes_room_on_a_branch_with_few_observations' (73300) panicked at topology_entry\src\checkpoint_recovery\required_chain\exact_invalidation\randomized.rs:97:21:
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
```

## application_invariant_acceptance::proof::application_program::discovered::retention::discovered_committed_retention_refuses_twice_then_promotes_original_carrier

CLEANUP-C1-retention-or-publication-interruption: Actual retention reservation refusal, abandoned performed result, cancellation or paused publication precedes failing recovery assertion.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
COMMAND: ["python", "target/lane-evidence/pa_merge_fixture_retention_owned_probe.py"]
HEAD: 589f6fce9c002fa7ef44082808137bac7b03516f
CLEAN: false
START: 2026-11-07T16:29:09.275316+00:00
ENV: removed stale external linker override; RUST_MIN_STACK unset in parent
OWNED POLICY: query_gate_environment.environment, unchanged RUST_MIN_STACK= 16777216
This reproduces the original fixture lane only; it is NOT default-stack success evidence.
EXACT COMMAND: ["C:\\programming\\worth worktees\\worktrees\\m3-host-home\\workspaces\\worth-query\\crates\\worth-query-certification\\fixtures\\consumer_entry\\target\\debug\\deps\\worth_query_consumer_root-af38a5b0f068f96f.exe", "--exact", "application_invariant_acceptance::proof::application_program::discovered::retention::discovered_committed_retention_refuses_twice_then_promotes_original_carrier"]
BINARY_SHA256: b908ba3215e96676dc4e396cff76709a5553b13c2fc8e14699413ff8b243e204
{"chunk": 0, "binary": "worth-query-consumer-root:worth-query-consumer-root", "tests": 1, "exit": 101, "failed": ["application_invariant_acceptance::proof::application_program::discovered::retention::discovered_committed_retention_refuses_twice_then_promotes_original_carrier"], "passed": 0, "seconds": 1, "log": "fixture-chunk-000.log", "names": ["application_invariant_acceptance::proof::application_program::discovered::retention::discovered_committed_retention_refuses_twice_then_promotes_original_carrier"]}

running 1 test
test application_invariant_acceptance::proof::application_program::discovered::retention::discovered_committed_retention_refuses_twice_then_promotes_original_carrier ... FAILED

failures:

---- application_invariant_acceptance::proof::application_program::discovered::retention::discovered_committed_retention_refuses_twice_then_promotes_original_carrier stdout ----

thread 'application_invariant_acceptance::proof::application_program::discovered::retention::discovered_committed_retention_refuses_twice_then_promotes_original_carrier' (68136) panicked at consumer_root\src\application_invariant_acceptance\proof\application_program\discovered\retention.rs:225:5:
assertion failed: matches!(denial,
    worth_query_host::facade::application_entry::WorthQueryApplicationRecoveryRequestDenial::Recovery(worth_query_host::facade::primary_graph::WorthQueryManagedApplicationRecoveryDenial::Demand(ref
    denial)) if
    denial.kind()==WorthQueryOutputDemandDenialKind::PublicationCapacityExceeded)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    application_invariant_acceptance::proof::application_program::discovered::retention::discovered_committed_retention_refuses_twice_then_promotes_original_carrier

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 271 filtered out; finished in 0.81s



END: 2026-11-07T16:29:11.924250+00:00
EXIT: 1
```

## application_invariant_acceptance::managed_index::maintained::geometry::court::growth::geometry_chain_growth_uses_real_source_and_original_sdk_invoice

CLEANUP-C2-incomplete-growth-campaign: Busy finite growth process stopped after>10min; exact PID/artifact/parent/sole remaining case verified. Completed prefix retained; no assertion or timeout claim.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
{
  "pid": 55288,
  "exe": "C:\\programming\\worth worktees\\worktrees\\m3-host-home\\workspaces\\worth-query\\crates\\worth-query-certification\\fixtures\\consumer_entry\\target\\debug\\deps\\worth_query_consumer_root-af38a5b0f068f96f.exe",
  "command": "\"C:\\programming\\worth worktees\\worktrees\\m3-host-home\\workspaces\\worth-query\\crates\\worth-query-certification\\fixtures\\consumer_entry\\target\\debug\\deps\\worth_query_consumer_root-af38a5b0f068f96f.exe\" --exact application_invariant_acceptance::candidate_index::boxes::affinity::registered_box_rejects_same_runtime_foreign_branch_and_retained_stale_basis application_invariant_acceptance::candidate_index::boxes::authority::governed_box_provider_rechecks_real_scope_and_revoked_grant_before_callbacks application_invariant_acceptance::candidate_index::boxes::candidate_allocation::pruned_completed_source_allocates_no_dense_candidate_backing application_invariant_acceptance::candidate_index::boxes::geometry::public_box_provider_preserves_closed_boundaries_source_slots_and_typed_bounds_refusals application_invariant_acceptance::candidate_index::boxes::lifecycle::cancelled_cold_box_build_releases_partial_bvh_and_rebuilds_with_fresh_request application_invariant_acceptance::candidate_index::boxes::lifecycle::registered_box_backing_releases_after_results_and_last_runtime_close application_invariant_acceptance::candidate_index::boxes::summary::checked_summary_refuses_corrupt_cold_bounds_then_rebuilds_a_lawful_twin application_invariant_acceptance::candidate_index::boxes::summary::predecessor_lookup_on_a_genuine_cold_source_slot_refuses_before_a_lawful_build application_invariant_acceptance::candidate_index::court::actual_candidate_slots_are_output_ordered_duplicate_idempotent_and_foreign_refused application_invariant_acceptance::contribution_denials::producer_denials::tests::complete_handler_registration_leaves_only_the_intended_producer_omission application_invariant_acceptance::document_history::failures::authority::a_listed_node_requires_current_navigation_authority_and_restoration_needs_a_fresh_gesture application_invariant_acceptance::document_history::failures::delivery::lost_delivery_replays_the_original_completion_without_resetting_a_later_document_selection application_invariant_acceptance::document_history::failures::group::a_second_group_scope_refusal_preserves_actual_source_prefix_and_aborts_the_original_owner application_invariant_acceptance::document_history::failures::selection::stale_click_and_prepared_loser_preserve_the_actual_adoption_winner_and_original_cleanup application_invariant_acceptance::document_history::tests::the_executable_document_adapter_selects_both_futures_and_retains_real_output_custody application_invariant_acceptance::managed_index::boxes::affinity::registered_box_rejects_same_runtime_foreign_branch_and_retained_stale_basis application_invariant_acceptance::managed_index::boxes::authority::governed_box_provider_rechecks_real_scope_and_revoked_grant_before_callbacks application_invariant_acceptance::managed_index::boxes::candidate_allocation::pruned_completed_source_allocates_no_dense_candidate_backing application_invariant_acceptance::managed_index::boxes::geometry::public_box_provider_preserves_closed_boundaries_source_slots_and_typed_bounds_refusals application_invariant_acceptance::managed_index::boxes::lifecycle::cancelled_cold_box_build_releases_partial_bvh_and_rebuilds_with_fresh_request application_invariant_acceptance::managed_index::boxes::lifecycle::registered_box_backing_releases_after_results_and_last_runtime_close application_invariant_acceptance::managed_index::boxes::summary::checked_summary_refuses_corrupt_cold_bounds_then_rebuilds_a_lawful_twin application_invariant_acceptance::managed_index::boxes::summary::predecessor_lookup_on_a_genuine_cold_source_slot_refuses_before_a_lawful_build application_invariant_acceptance::managed_index::expectations::affinity::managed_expectation_checks_actual_runtime_and_parameters_before_mutation application_invariant_acceptance::managed_index::expectations::membership::managed_empty_result_expectation_preserves_future_entrant_dependencies application_invariant_acceptance::managed_index::expectations::membership::managed_row_expectation_preserves_nonmatches_and_recorded_retry application_invariant_acceptance::managed_index::journey::public_prefix_index_rebuild_preserves_source_slots_and_ordinary_query_results application_invariant_acceptance::managed_index::lifecycle::affinity::registered_prefix_rejects_genuine_reclaimed_scope_generation_replacement application_invariant_acceptance::managed_index::lifecycle::affinity::registered_prefix_rejects_same_runtime_foreign_branch_and_retained_stale_basis application_invariant_acceptance::managed_index::lifecycle::cancelled_cold_prefix_build_releases_partial_backing_and_rebuilds_with_fresh_request application_invariant_acceptance::managed_index::lifecycle::complete_membership::complete_membership_visits_nonmatches_and_refuses_interrupted_partial_selection application_invariant_acceptance::managed_index::lifecycle::installation::registered_prefix_rejects_foreign_runtime_and_incompatible_executable_before_callbacks application_invariant_acceptance::managed_index::lifecycle::publication::public_cache_native_ahead_of_world_keeps_selected_predecessor_then_requires_rebuild application_invariant_acceptance::managed_index::lifecycle::registered_prefix_backing_releases_after_results_siblings_and_runtime_owners_close application_invariant_acceptance::managed_index::lifecycle::registration::distinct_parent_bindings_to_one_query_keep_independent_registrations application_invariant_acceptance::managed_index::lifecycle::registration::retained_registrations_have_indexed_query_and_repeated_open_work application_invariant_acceptance::managed_index::lifecycle::source_structure::source_tree_refuses_omitted_duplicate_and_cyclic_roots_before_ready application_invariant_acceptance::managed_index::maintained::court::correction::geometry::box_correction_accepts_genuine_recorded_motion_cancellation_for_populated_and_empty_inputs application_invariant_acceptance::managed_index::maintained::court::correction::geometry::box_correction_excludes_uncancelled_nonmatching_indirect_motion application_invariant_acceptance::managed_index::maintained::court::correction::geometry::box_correction_keeps_matching_authored_noops_rejected_by_exact_contact application_invariant_acceptance::managed_index::maintained::court::correction::geometry::box_correction_keeps_transient_entrants_and_raw_restoration_without_performed_pairs application_invariant_acceptance::managed_index::maintained::court::correction::managed_correction_excludes_nonmatching_authored_noops_from_empty_predicate_history application_invariant_acceptance::managed_index::maintained::court::correction::managed_correction_excludes_uncancelled_irrelevant_edits_using_actual_historical_rows application_invariant_acceptance::managed_index::maintained::court::correction::manag
```

## checkpoint_recovery::computation_partition::oracle::collection::lifecycle::locality::serial::correction_and_family_work_excludes_ten_and_one_thousand_unrelated_histories

CLEANUP-C2-incomplete-growth-campaign: Busy finite growth process stopped after>10min; exact PID/artifact/parent/sole remaining case verified. Completed prefix retained; no assertion or timeout claim.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
{
  "pid": 10928,
  "exe": "C:\\programming\\worth worktees\\worktrees\\m3-host-home\\workspaces\\worth-query\\crates\\worth-query-certification\\fixtures\\consumer_entry\\target\\debug\\deps\\worth_query_topology_entry-b489d494f473e51b.exe",
  "command": "\"C:\\programming\\worth worktees\\worktrees\\m3-host-home\\workspaces\\worth-query\\crates\\worth-query-certification\\fixtures\\consumer_entry\\target\\debug\\deps\\worth_query_topology_entry-b489d494f473e51b.exe\" --exact checkpoint_recovery::computation_partition::oracle::collection::lifecycle::locality::serial::correction_and_family_work_excludes_ten_and_one_thousand_unrelated_histories",
  "parent": "C:\\Users\\shepworth\\AppData\\Local\\Programs\\Python\\Python314\\python.exe \"C:\\programming\\worth worktees\\worktrees\\m3-host-home\\target\\lane-evidence\\pa_merge_permitted_gate.py\" fixture --census --out \"C:\\programming\\worth worktees\\worktrees\\m3-host-home\\target\\lane-evidence\\pa-merge-fixture-tier2\"",
  "start": "2026-11-07T16:11:27.0675041Z",
  "observed": "2026-11-07T16:23:55.1216161Z",
  "elapsed_seconds": 748.0541119999999,
  "cpu_seconds": 727.9375,
  "completed_cases": 0,
  "remaining": [
    "checkpoint_recovery::computation_partition::oracle::collection::lifecycle::locality::serial::correction_and_family_work_excludes_ten_and_one_thousand_unrelated_histories"
  ],
  "log": "C:\\programming\\worth worktees\\worktrees\\m3-host-home\\target\\lane-evidence\\pa-merge-fixture-tier2\\fixture-chunk-048.log",
  "log_sha256": "1068E3CC3A5F67BD9292D365943B29E670DD14BA9FB405297C532086C5CE947E",
  "classification": "C2 finite growth campaign; incomplete",
  "action": "Terminate exact own process only; no asserted failure/timeout or pass claim; every completed prefix remains logged"
}
```

## worth-store-court-application::worth_store_court_application$process::application::query_readmission::bootstrap_publication::retired_format::retired_bootstrap_completion_format_refuses_existing_home_without_replaying_seeds

CLEANUP-C1-retired-format-refusal-shape: Deliberately altered completion record is correctly refused by D8 quarantined primary-graph denial; court expects old ExistingQueryAttachment shape. No service conversion restored.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'process::application::query_readmission::bootstrap_publication::retired_format::retired_bootstrap_completion_format_refuses_existing_home_without_replaying_seeds' (77108) panicked at crates\worth-store-court-application\tests\worth_store_court_application\process\application\query_readmission\bootstrap_publication\retired_format.rs:103:9:
actual selected Query completion must refuse: WorthQueryApplicationHomeOpenRefused("primary graph installation denied: CheckpointRecoveryRejected (Home bootstrap completion format is unsupported)")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-store-court-application::worth_store_court_application$process::application::query_readmission::native_artifacts::selected::selected_completion_and_activation_failures_keep_owned_home_without_initializing

CLEANUP-C1-retired-format-refusal-shape: Deliberately altered completion record is correctly refused by D8 quarantined primary-graph denial; court expects old ExistingQueryAttachment shape. No service conversion restored.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'process::application::query_readmission::native_artifacts::selected::selected_completion_and_activation_failures_keep_owned_home_without_initializing' (82828) panicked at crates\worth-store-court-application\tests\worth_store_court_application\process\application\query_readmission\native_artifacts\selected.rs:111:22:
unsupported-format must reach selected Query record admission: WorthQueryApplicationHomeOpenRefused("primary graph installation denied: CheckpointRecoveryRejected (Home bootstrap completion format is unsupported)")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-store-court-application::worth_store_court_application$process::application::query_readmission::native_artifacts::stops::original_cancellation_and_memory_exhaustion_retain_the_same_pre_service_home

CLEANUP-C1-C2-pre-service-request-stops: Cancellation/exhausted original memory injected before finalization; typed schema PolicyMemoryExhausted wrapped as CheckpointRecoveryRejected, old expected shape differs.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'process::application::query_readmission::native_artifacts::stops::original_cancellation_and_memory_exhaustion_retain_the_same_pre_service_home' (81160) panicked at crates\worth-store-court-application\tests\worth_store_court_application\process\application\query_readmission\native_artifacts\stops.rs:59:17:
WorthQueryPrimaryGraphInstallationDenial { kind: CheckpointRecoveryRejected, subject: "Home recovered schema authority denied: initial Relational schema installation denied: Request(PolicyMemoryExhausted { requested: 376, admitted: 0, ancestor: 0 }) (initial authority digest request refused)", cause: None }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-store-court-application::worth_store_court_application$process::relational::relational_home::incarnation::anchor_replay::wal_replays_the_thirty_second_anchor_advance_after_process_death

CLEANUP-C1-C2-anchor-growth-kill: 32-anchor/high-generation process-death control hits AnchorTableFull before WAL pause; no successful kill/replay proof.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'process::relational::relational_home::incarnation::anchor_replay::wal_replays_the_thirty_second_anchor_advance_after_process_death' (84060) panicked at crates\worth-store-court-application\tests\worth_store_court_application\support\killed_child\declarations.rs:62:13:
child process::relational::relational_home::incarnation::anchor_replay::high_anchor_child ended without parking: 
running 1 test
test process::relational::relational_home::incarnation::anchor_replay::high_anchor_child ... FAILED

failures:

failures:
    process::relational::relational_home::incarnation::anchor_replay::high_anchor_child

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 179 filtered out; finished in 3.32s



thread 'process::relational::relational_home::incarnation::anchor_replay::high_anchor_child' (57044) panicked at crates\worth-store-court-application\tests\worth_store_court_application\support\incarnation\anchor_replay\declarations.rs:34:10:
called `Result::unwrap()` on an `Err` value: AnchorTableFull
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-store-court-application::worth_store_court_application$process::relational::relational_home::open_refusals::a_checkpoint_record_store_accepts_but_relational_cannot_readmit_refuses

CLEANUP-C1-worldless-damage-fixture: Damage/refusal court stops at benign WorldSectionRequired setup before fault assertion. Cleanup must compose actual World and preserve chosen checkpoint/tail/abort; no fake bytes or graceful-close substitution.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'process::relational::relational_home::open_refusals::a_checkpoint_record_store_accepts_but_relational_cannot_readmit_refuses' (7052) panicked at crates\worth-store-court-application\tests\worth_store_court_application\support\durable_run\declarations.rs:111:36:
home checkpoint: WorldSectionRequired
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-store-court-application::worth_store_court_application$process::relational::relational_home::open_refusals::a_damaged_checkpoint_in_the_store_files_refuses_before_service

CLEANUP-C1-worldless-damage-fixture: Damage/refusal court stops at benign WorldSectionRequired setup before fault assertion. Cleanup must compose actual World and preserve chosen checkpoint/tail/abort; no fake bytes or graceful-close substitution.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'process::relational::relational_home::open_refusals::a_damaged_checkpoint_in_the_store_files_refuses_before_service' (20660) panicked at crates\worth-store-court-application\tests\worth_store_court_application\support\durable_run\declarations.rs:111:36:
home checkpoint: WorldSectionRequired
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-store-court-application::worth_store_court_application$process::relational::relational_home::open_refusals::a_damaged_journal_entry_in_the_store_files_refuses_before_service

CLEANUP-C1-worldless-damage-fixture: Damage/refusal court stops at benign WorldSectionRequired setup before fault assertion. Cleanup must compose actual World and preserve chosen checkpoint/tail/abort; no fake bytes or graceful-close substitution.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'process::relational::relational_home::open_refusals::a_damaged_journal_entry_in_the_store_files_refuses_before_service' (79388) panicked at crates\worth-store-court-application\tests\worth_store_court_application\support\durable_run\declarations.rs:111:36:
home checkpoint: WorldSectionRequired
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-store-court-application::worth_store_court_application$process::relational::relational_home::open_refusals::a_journal_entry_store_accepts_but_relational_cannot_readmit_refuses

CLEANUP-C1-worldless-damage-fixture: Damage/refusal court stops at benign WorldSectionRequired setup before fault assertion. Cleanup must compose actual World and preserve chosen checkpoint/tail/abort; no fake bytes or graceful-close substitution.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'process::relational::relational_home::open_refusals::a_journal_entry_store_accepts_but_relational_cannot_readmit_refuses' (45712) panicked at crates\worth-store-court-application\tests\worth_store_court_application\support\durable_run\declarations.rs:111:36:
home checkpoint: WorldSectionRequired
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-store-court-application::worth_store_court_application$process::relational::relational_home::publication_refusals::a_commit_the_medium_refuses_before_any_effect_is_denied_and_not_installed

CLEANUP-C1-after-lifecycle-termination: SourceUnavailable artifact snapshot occurs after explicit lifecycle termination injection atpublication_refusals/mod.rs94-118, not before fault.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'process::relational::relational_home::publication_refusals::a_commit_the_medium_refuses_before_any_effect_is_denied_and_not_installed' (51616) panicked at crates\worth-store-court-application\tests\worth_store_court_application\support\workload\declarations.rs:280:18:
retained commits encode: StorageUnavailable(RelationalStorageUnavailable { subject: Envelope(PatchStreamPosition(1)), cause: SourceUnavailable })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-store-court-application::worth_store_court_application$process::relational::relational_home::residency_agreement::cold_warm_and_full_home_reads_and_retained_observations_agree_with_memory

CLEANUP-C2-cold-member-read-bound: Real World composition and all 217 caller-owned snapshot releases restore graceful close. Journey now reaches exact member-read bound 1 versus 0. Later suffix not reached.

Evidence: `target/lane-evidence/pa-merge-round6/store/run.log`. Original assertion/stage:

```text
thread 'process::relational::relational_home::residency_agreement::cold_warm_and_full_home_reads_and_retained_observations_agree_with_memory' (42080) panicked at crates\worth-store-court-application\tests\worth_store_court_application\process\relational\relational_home\residency_agreement\mod.rs:58:33:
called `Result::unwrap()` on an `Err` value: WorldSectionRequired
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-store-court-application::worth_store_court_application$process::application::query_readmission::native_artifacts::home_close::host_checkpoint_memory_failure_is_reported_while_original_retired_owners_remain_physically_closed

CLEANUP-C2-close-memory-boundary: Fixed1.5MiB capture now succeeds; declared-failure assertion remains red without grant/claim weakening.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'process::application::query_readmission::native_artifacts::home_close::host_checkpoint_memory_failure_is_reported_while_original_retired_owners_remain_physically_closed' (55148) panicked at crates\worth-store-court-application\tests\worth_store_court_application\process\application\query_readmission\native_artifacts\home_close.rs:178:14:
this declared memory ceiling must exercise the real capture failure
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-store-court-application::worth_store_court_application$process::relational::relational_home::residency_reopen::a_cold_schema_transition_replays_and_reframes_all_carriers_before_load_free_capture

CLEANUP-C2-cold-member-read-bound: Real World composition and all 217 caller-owned snapshot releases restore graceful close. Journey now reaches exact member-read bound 1 versus 0. Later suffix not reached.

Evidence: `target/lane-evidence/pa-merge-round6/store/run.log`. Original assertion/stage:

```text
thread 'process::relational::relational_home::residency_reopen::a_cold_schema_transition_replays_and_reframes_all_carriers_before_load_free_capture' (41288) panicked at crates\worth-store-court-application\tests\worth_store_court_application\process\relational\relational_home\residency_reopen\mod.rs:172:33:
called `Result::unwrap()` on an `Err` value: WorldSectionRequired
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-store-court-application::worth_store_court_application$process::relational::relational_home::residency_capture::a_one_partition_commit_publishes_constant_new_members_and_capture_reads_nothing

CLEANUP-C2-worldless-capture-count-fixture: Constant-new-member/load-free capture court stops at WorldSectionRequired setup; actual count assertions unreached and unchanged.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'process::relational::relational_home::residency_capture::a_one_partition_commit_publishes_constant_new_members_and_capture_reads_nothing' (21824) panicked at crates\worth-store-court-application\tests\worth_store_court_application\process\relational\relational_home\residency_capture\mod.rs:28:33:
called `Result::unwrap()` on an `Err` value: WorldSectionRequired
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-store-court-application::worth_store_court_application$process::relational::relational_home::residency_digest::a_checksum_valid_substituted_member_refuses_its_digest_and_keeps_subject_damage_sticky

CLEANUP-C1-worldless-damage-fixture: Damage/refusal court stops at benign WorldSectionRequired setup before fault assertion. Cleanup must compose actual World and preserve chosen checkpoint/tail/abort; no fake bytes or graceful-close substitution.

Evidence: `pa-merge-classified-failures.json`. Original assertion/stage:

```text
thread 'process::relational::relational_home::residency_digest::a_checksum_valid_substituted_member_refuses_its_digest_and_keeps_subject_damage_sticky' (43580) panicked at crates\worth-store-court-application\tests\worth_store_court_application\support\durable_run\declarations.rs:111:36:
home checkpoint: WorldSectionRequired
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## worth-store-court-application::worth_store_court_application$process::relational::relational_home::residency_reopen::reopen_keeps_regions_cold_and_a_point_read_and_commit_demand_only_their_partition

CLEANUP-C2-cold-member-read-bound: Real World composition and all 217 caller-owned snapshot releases restore graceful close. Journey now reaches exact member-read bound 1 versus 0. Later suffix not reached.

Evidence: `target/lane-evidence/pa-merge-round6/store/run.log`. Original assertion/stage:

```text
thread 'process::relational::relational_home::residency_reopen::reopen_keeps_regions_cold_and_a_point_read_and_commit_demand_only_their_partition' (43464) panicked at crates\worth-store-court-application\tests\worth_store_court_application\process\relational\relational_home\residency_reopen\mod.rs:37:33:
called `Result::unwrap()` on an `Err` value: WorldSectionRequired
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

