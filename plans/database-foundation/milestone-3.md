# Milestone 3: Durable Application Runtime

Governing roadmap: [Database Foundation Roadmap](roadmap.md#milestone-3-durable-application-runtime).
Builds on: [Milestone 1](milestone-1.md) and [Milestone 2](milestone-2.md).

An application opens one file-backed home through one Query host call. Every
supported runtime owner publishes its own durable artifacts through Store,
recovers through its own readmission, and continues working after process
death. That covers graph state, application occurrences and their original
results, Signal and Bridge definitions, programs, workflows and their
obligations, external custody and corrections. A retry after a lost response
returns the original occurrence. Nothing an application was told is committed
depends on process memory, orderly close or application-owned byte transport.

This milestone is delivered in ten numbered phases. Each phase states its own
requirements, design, required tests, oracle extension, documentation and exit
criteria. The cross-cutting decisions below bind every phase. D17 decides how
each invariant is enforced and where each new type lives.

## Moved courts

### C4 - Injected faults

Additional orphan control closure for the moved transition/publication campaigns:
restore `86e84dc5cd:crates/worth-relational/src/branch/transition/publication_pause_for_test.rs`
with its fields, initialization, arming door and pause call in same-SHA
`branch/transition/durable_step.rs`. Restore the complete local scripted-answer
fixture from same-SHA `tests/support/durable_medium.rs` and
`tests/support/durable_medium/{scripted_publication,journal,transition_journal,transition_contents}.rs`.
These controls lost every arming caller with the moved C4 roots. Their complete
original behavior remains required on restoration through genuine owned media;
the ordinary next-position/sequence and identical-retry rules stay live unchanged.


Remaining pure cleanup roots (per-claim Landing 2a disposition):

- `durability_contract_failure_missing_authoritative_parent_closure_is_explicit`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/durability/contracts/rejected_and_parent_closure.rs`.
- `durable_fork_and_source_commit_complete_without_deferral` (also retains its original C2 cost/custody guards): `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/durability/medium/branch_transition_races.rs`.
- `durable_fork_and_source_archive_complete_in_lock_order` (also retains its original C2 cost/custody guards): `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/durability/medium/branch_transition_races.rs`.
- `durable_fork_waits_for_another_branch_commit` (also retains its original C2 cost/custody guards): `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/durability/medium/branch_transition_races.rs`.
- `memory_commit_waits_behind_a_transition_without_consuming_its_candidate` (also retains its original C2 cost/custody guards): `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/durability/medium/branch_transition_races.rs`.
- `divergent_mirror_partition_rebuilds_cross_partition_adjacency_after_reuse` (also retains its original C2 cost/custody guards): `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs`.
- `equal_image_cannot_bypass_global_contract_readmission`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs`.
- `forged_cross_region_kind_commitment_is_refused_before_member_reads`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/durability/log/native_checkpoint_codec/kind_authentication_tests.rs`.
- `an_indexed_envelope_storage_failure_is_typed_and_sticky`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/envelope_residency_tests.rs`.
- `replay_commit_reports_strategy_executor_unavailable_when_recovered_runtime_lacks_executor`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/commit_strategies/facade_tests/replay_recovery_failures.rs`.
- `replay_commit_reports_strategy_execution_failure_when_recovered_executor_rejects`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/commit_strategies/facade_tests/replay_recovery_failures.rs`.
- `restore_refuses_unknown_fields_before_publishing_any_model_state`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/durability/log/native_checkpoint_codec/inspection/tests.rs`.
- `rehashed_cross_region_kind_forgery_is_refused_by_the_canonical_root`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/storage/residency/cross_region_forgery_tests.rs`.
- `refused_durable_publication_leaves_no_replacement_lineage_behind`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/lineage/resolution.rs`.
- `managed_fork_refusal_and_success_drop_lookup_outside_native_coordination` (also retains its original C2 cost/custody guards): `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/branch/coordination/managed_fork_cleanup.rs`.
- `fintech_recovery_refuses_a_damaged_checkpoint_instead_of_losing_truth`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/domains/fintech/workflows/persistence/checkpoint_fallback.rs`.
  Restore each root's complete same-SHA parent/descendant fixture and control
  closure, including its original local medium script and owner units; migrate
  the source boundary through genuine owned media without reviving the retired
  runtime-discovery adapter. All original guards remain required.

  The inert lock-order control is deleted completely. Restore its full arming,
  observations, fields and call sites from `f9f15bf8c8:crates/worth-relational/src/runtime/state/subsystems/canonical_publication_routes/lifecycle_order_control.rs`,
  with the same-SHA `canonical_publication_routes.rs` owner and its
  `canonical_publication_routes/{branch_transition_stream,checkpoint_admission}.rs`
  call sites when restoring the original lock-order campaigns.


- Associated-original missing-route and dropped-unrelated-body campaign parts:
  `associated_effects_refuse_memory_and_absent_settlement_evidence` and
  `associated_effects_discover_cold_subject_without_reading_unrelated_history`:
  `restore from 0615dd733c:crates/worth-relational/src/tests/durability/contracts/associated_effects.rs`.
  Restore the exact route removal / unrelated-artifact deletion and their
  original refusals or successful selected-body guards with `perform` fixture.
- `native_custody_correction_enumerates_known_orphan_and_unrelated_association_metadata`,
  actual three-body deletion followed by successful metadata enumeration:
  `restore from 0615dd733c:crates/worth-relational/src/tests/durability/contracts/associated_effects_discovery.rs`.
- `caller_association_survives_embedded_checkpoint_and_refuses_old_semantics`,
  both original `CHK13`/`CHK14` mutations and exact `UnsupportedLegacySemantics`:
  `restore from 0615dd733c:crates/worth-relational/src/tests/durability/contracts/caller_association.rs`.
  The positive actual embedded original association/re-encoding assertion stays
  as a genuine Memory owner codec unit.
- `recorded_component_checkpoint_readmits_cold_canonical_predecessor_and_successor`,
  `recorded_component_checkpoint_readmits_original_empty_predecessor`, and
  `recorded_original_retirement_preserves_both_leases_across_busy_and_stop`,
  injected unrelated-artifact deletion parts:
  `restore from 0615dd733c:crates/worth-relational/src/tests/durability/contracts/recorded_component_checkpoint.rs`.
  Restore complete same-SHA `original_custody`/original capture/recovery fixtures
  and successful selected readmission guards, coordinated with C2's physical
  custody/release portions. Public ordinary readmission is separately retained.


- `refused_durable_transition_returns_its_authority_and_moves_nothing` and
  `unresolved_durable_transition_installs_nothing_and_names_its_fate`:
  `restore from e4d3b01c8d:crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs`.
  C4 retains the actual ProvenNoEffect/Indeterminate scripts, exact original
  offered fate, authority return/retry and no-effect guards with the same-SHA
  `recovered_world`, `candidate`, `main_basis` fixtures. Restore without the
  deleted local provider/discovery adapters. Ordinary transition ownership
  remains covered through `checkpoint_successor_authority.rs` public courts.


- `checkpoint_recovery_rejects_foreign_branch_target_substitution`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint/foreign_targets.rs`.
- `tail_recovery_rejects_foreign_branch_target_substitution`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint/foreign_targets.rs`.
- `checkpoint_recovery_rejects_missing_exact_branch_root_artifact`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs`.
- `checkpoint_recovery_rejects_root_artifact_without_commit_envelope`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs`.
- `checkpoint_recovery_rejects_duplicate_commit_envelopes_before_root_readmission`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs`.
- `checkpoint_recovery_rejects_duplicate_global_partition_images`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs`.
- `checkpoint_recovery_rejects_duplicate_branch_root_partition_images`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs`.
- `checkpoint_rejects_previous_branch_root_format_before_readmission`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs`.
- `checkpoint_rejects_altered_root_content_with_recomputed_image_digest`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs`.
- `checkpoint_rejects_omitted_root_partition_with_recomputed_image_digest`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs`.
- `checkpoint_rejects_same_commit_truth_root_relabeling`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs`.
- `checkpoint_rejects_same_commit_schema_root_relabeling`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs`.
- `tail_rejects_same_commit_root_descriptor_substitution`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs`.
- `checkpoint_owner_root_is_stable_across_fingerprint_symbol_ids`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs`.
- `recovery_rejects_a_missing_branch_root_schema_carrier`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_schema_binding.rs`.
- `recovery_rejects_a_tampered_branch_root_schema_carrier`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_schema_binding.rs`.
- `recovery_rejects_a_schema_carrier_swapped_between_exact_roots`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_schema_binding.rs`.
- `checkpoint_rejects_later_shared_root_cell_with_relabeled_schema_root`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_shared_target_binding.rs`.
- `checkpoint_rejects_later_shared_root_cell_with_relabeled_truth_root`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_shared_target_binding.rs`.
- `a_deleting_checkpoint_cell_requires_its_retired_name`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_transition/checkpoint_state.rs`.
- `every_transition_fate_preserves_the_durable_before_live_boundary`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_transition/fates.rs`.
- `an_unresolved_commit_also_blocks_branch_transitions_with_its_original_fate`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_transition/fates.rs`.
- `a_damaged_transition_frame_is_refused_at_the_medium_read`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_transition/tampering.rs`.
- `a_sequence_gap_and_invalid_follows_positions_are_refused`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_transition/tampering.rs`.
- `a_commit_carrying_an_unknown_branch_cell_cannot_replace_its_missing_fork`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_transition/tampering.rs`.
- `durability_contract_damaged_checkpoint_refuses_rather_than_shortening_history`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery/checkpoint_fallback.rs`.
- `durability_contract_recovery_fails_closed_without_exact_branch_cells`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs`.
- `durability_contract_recovery_rejects_branch_cell_target_without_catalog_artifact`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs`.
- `durability_contract_recovery_rejects_tail_checkpoint_drift_before_mutation`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs`.
- `legacy_index_affinity_is_checked_before_replacing_a_live_recovery_target`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/artifact_affinity.rs`.
- `envelope_index_affinity_requires_admission_before_installation`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/artifact_affinity.rs`.
- `digest_valid_unissued_scoped_branch_denies_without_replacing_target_state`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/scoped_affinity.rs`.
- `checkpoint_index_artifact_missing_payload_denies_before_recovery`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs`.
- `checkpoint_recovery_denies_a_digest_valid_foreign_generation_affinity`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs`.
- `contradictory_descriptor_version_refuses_before_recovery_or_owner_effects`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/descriptor_continuity.rs`.
- `durability_contract_failure_descriptor_canonical_basis_version_mismatch_is_explicit`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/descriptor_continuity.rs`.
- `recovery_rejects_one_sided_fork_provenance_in_checkpoint`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/fork_provenance_checkpoint.rs`.
- `recovery_rejects_mismatched_fork_provenance_identity_in_checkpoint`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/fork_provenance_checkpoint.rs`.
- `recovery_rejects_one_sided_fork_provenance_in_tail`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/fork_provenance_checkpoint.rs`.
- `tail_created_lineage_allocator_exhaustion_denies_before_replay`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_allocator_denials.rs`.
- `live_lineage_allocator_exhaustion_denies_before_public_effects`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_allocator_denials.rs`.
- `tail_created_lineage_regression_below_checkpoint_high_water_denies`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery/lineage_regression.rs`.
- `tail_lineage_event_regression_below_checkpoint_high_water_denies`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery/lineage_regression.rs`.
- `checkpoint_duplicate_lineage_event_ids_deny`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs`.
- `tail_duplicate_lineage_event_ids_deny`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs`.
- `checkpoint_and_tail_lineage_event_id_collision_denies`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs`.
- `tail_lineage_commit_and_branch_cross_splices_deny`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs`.
- `checkpoint_lineage_commit_cross_splice_denies`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs`.
- `replay_and_durable_lineage_payload_conflict_denies`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs`.
- `checkpoint_lineage_node_deletion_denies_before_installation`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs`.
- `checkpoint_lineage_node_remap_denies_before_installation`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs`.
- `checkpoint_lineage_node_swap_denies_before_installation`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs`.
- `checkpoint_duplicate_lineage_entity_mapping_denies_before_installation`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs`.
- `tail_lineage_event_allocator_exhaustion_denies_before_replay`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_event_allocator_recovery.rs`.
- `unresolved_durable_publication_blocks_descendants_and_recovers_last_checkpoint`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_event_allocator_recovery.rs`.
- `multi_event_reservation_exhaustion_denies_before_public_effects`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/lineage_event_allocator_recovery.rs`.
- `native_field_text_missing_tail_dictionary_refuses_without_target_or_source_effect`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/managed_field_text/missing_dictionary.rs`.
- `native_field_text_prior_checkpoint_format_refuses_before_managed_text`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/managed_field_text.rs`.
- `durability_contract_rejects_parent_order_tampering_at_source_admission`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/merge_replay_continuity.rs`.
- `durability_contract_does_not_reconstruct_missing_merge_authority_from_diagnostics`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/merge_replay_continuity.rs`.
- `corrupt_native_checkpoint_is_denied_without_publishing_recovered_state`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/native_checkpoint.rs`.
- `tail_replay_rejects_canonical_allocation_targeting_a_live_slot`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs`.
- `tail_replay_rejects_missing_canonical_allocation_evidence`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs`.
- `tail_replay_gap_admission_cannot_replace_a_different_canonical_effect`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs`.
- `metadata_only_merge_rejects_unconsumable_allocation_evidence`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs`.
- `recorded_component_checkpoint_refuses_chk15_before_source_admission`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/recorded_component_checkpoint.rs`.
- `semantic_result_bulk_malformed_correspondence_refuses_after_frame_reseal`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/semantic_result/bulk.rs`.
- `durability_contract_recovery_rejects_tail_target_without_immutable_artifact`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/tail_checkpoint_validation.rs`.
- `a_refused_publication_was_offered_to_the_medium_and_moved_nothing_live`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/durable_before_live.rs`.
- `a_refused_position_is_not_consumed`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/durable_before_live.rs`.
- `fork_against_checkpoint_capture` (also holds a C2 cost/residency assertion): `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs`.
- `fork_against_runtime_fork` (also holds a C2 cost/residency assertion): `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs`.
- `fork_against_reclamation` (also holds a C2 cost/residency assertion): `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs`.
- `archive_against_checkpoint_capture` (also holds a C2 cost/residency assertion): `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs`.
- `archive_against_runtime_fork` (also holds a C2 cost/residency assertion): `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs`.
- `archive_against_reclamation` (also holds a C2 cost/residency assertion): `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs`.
- `checkpoint_registry_is_readable_while_a_fork_is_durable_not_yet_live`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs`.
- `an_artifact_whose_frame_is_damaged_is_refused_at_the_read`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs`.
- `an_artifact_that_does_not_readmit_refuses_as_corrupt`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs`.
- `a_gap_in_the_journal_refuses_as_non_contiguous`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs`.
- `a_repeated_entry_refuses_as_non_contiguous`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs`.
- `a_tail_that_does_not_continue_from_the_checkpoint_refuses`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs`.
- `a_checkpoint_filed_under_the_wrong_position_refuses_as_corrupt`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs`.
- `a_damaged_checkpoint_refuses_as_corrupt`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs`.
- `a_durable_publisher_waits_for_the_reservation_instead_of_deferring` (also holds a C2 cost/residency assertion): `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/reservation_wait.rs`.
- `a_cancelled_waiter_stops_before_any_effect` (also holds a C2 cost/residency assertion): `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/reservation_wait.rs`.
- `an_unresolved_publication_installs_nothing_and_refuses_every_later_commit_alike`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/unresolved_fate.rs`.
- `reopening_resolves_an_absent_unresolved_commit_as_never_committed`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/unresolved_fate.rs`.
- `reopening_resolves_a_present_unresolved_commit_as_committed_exactly_once`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/unresolved_fate.rs`.
  C4 owns each of these durability contract/medium roots under Landing 2a's
  per-claim disposition. Restore each root's same-SHA fixture closure (parent
  module helpers, `tests/support/durability.rs` and the original local medium
  controls) as recorded per root in
  `target/lane-evidence/landing2a-durability-contract-claim-disposition.md`.
  This is relocation with the original assertions, scripted fates and no-effect
  oracles, not retirement or weakened evidence. Restore over genuine owned
  media; do not revive the removed runtime-discovery adapter.
  The `cfg(test)` owner control hooks that only these moved roots called were
  removed with them and restore from the same SHA (paths under
  `crates/worth-relational/src/`): `durability/data/branch_root_schema_images.rs::corrupt_schema_root_for_test`, `durability/migration/readmitted_canonical_commit.rs::envelope_mut_for_test`, `history/data/canonical_commit_envelope.rs::rebuild_lineage_basis_for_test`, `history/data/canonical_record_allocation.rs::new`, `history/retention/owner.rs::retired_branch_binding_count`, `indexes/access/pinning/definition_inventory.rs::definitions_snapshot`, `lineage/data/artifacts/checkpoint_artifact.rs::nodes_mut`, `runtime/state/subsystems/canonical_publication_routes/lifecycle_order_control.rs::LifecycleSnapshotPause`, `runtime/state/subsystems/canonical_publication_routes/lifecycle_order_control.rs::await_admission`, `runtime/state/subsystems/canonical_publication_routes/lifecycle_order_control.rs::await_transition_lifecycle_contact_for_test`, `runtime/state/subsystems/canonical_publication_routes/lifecycle_order_control.rs::pause_exclusive_snapshot_for_test`, `runtime/state/subsystems/canonical_publication_routes/lifecycle_order_control.rs::release`, `runtime/state/subsystems/canonical_publication_routes/lifecycle_order_control.rs::transition_lifecycle_contacts_for_test`, `runtime/state/subsystems/history_retention.rs::retired_branch_binding_count`, `runtime/state/subsystems/lineage.rs::event_count`, `runtime/state/subsystems/lineage.rs::node_count`.

- `compile_execution_ready_merge_plan_rejects_missing_source_record`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7c_phase_a/compilation_denials.rs`.
- `verify_prepared_merge_execution_rejects_corrupted_compiled_plan`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7c_phase_a/compilation_denials.rs`.
- `verify_prepared_merge_execution_rejects_schema_semantic_drift`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7c_phase_a/prepared_merge_verification.rs`.
- `verify_prepared_merge_execution_rejects_merge_base_drift`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7c_phase_a/prepared_merge_verification.rs`.
- `derive_merge_commit_mutation_plan_does_not_rely_on_raw_lowered_record_arrays`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7c_phase_d.rs`.
- `authoritative_merge_execution_certification_rejects_metadata_reference_movement`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7c_phase_g/stale_prepared_merge_denials.rs`.
- `authoritative_merge_execution_certification_rejects_schema_semantic_drift_after_planning`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7c_phase_g/stale_prepared_merge_denials.rs`.
- `compile_rejects_corrupted_non_executable_resolution_class`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7d_phase_b/executable_admission.rs`.
- `promoted_deleted_on_both_sides_compiles_to_an_explicit_executable_record_variant`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7d_phase_c.rs`.
- `promoted_deleted_on_both_sides_derives_zero_mutation_intent_execution_plan`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7d_phase_c.rs`.
- `promoted_deleted_on_both_sides_executes_through_authoritative_merge_publication`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7d_phase_c.rs`.
- `deleted_on_both_sides_prepared_merge_rejects_schema_semantic_drift`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7d_phase_d/prepared_merge_denials.rs`.
  C4 owns the original claim under Landing2a's per-claim disposition.
  Restore each named root's same-SHA owning fixture/module imports and private
  control references recorded in `target/lane-evidence/landing2a-history-compiler-claim-disposition.md`.
  This is relocation with its original guard assertions, not retirement or
  weakened evidence. Restore with genuine owned inputs, without reviving the
  removed local provider/discovery adapters.
  The `cfg(test)` plan-mutation hooks only these merge roots called were removed
  with them. Restore `execution_ready_plan_mut_for_test`,
  `authority_binding_mut_for_test`, `bound_executable_plan_mut_for_test` and
  `replace_bound_plan_and_mutation_plan_for_test` from
  `f9f15bf8c8:crates/worth-relational/src/merge/data/execution/test_support.rs`,
  and `compile_execution_ready_merge_plan_for_test` and
  `replace_bound_merge_plan_for_test` from
  `f9f15bf8c8:crates/worth-relational/src/merge/execution/mod.rs`.


- `retained_merge_strategy_witness_live_authority_denies_internal_forged_strategy_truth`: `restore from 26cc5a58f2:crates/worth-relational/src/tests/history/milestone_7e_phase_i/mod.rs`.
  C4 owns this injected typed-summary/witness or stored-original fault; restore
  the same-SHA `fixtures.rs` beside the root, preserving exact denials.
- `merge_support_inspection_witness_denies_inconsistent_retained_proof_authority`: `restore from 26cc5a58f2:crates/worth-relational/src/tests/history/milestone_7e_phase_l/mod.rs`.
  C4 owns this injected typed-summary/witness or stored-original fault; restore
  the same-SHA `fixtures.rs` beside the root, preserving exact denials.
- `merge_collaboration_truth_denies_cross_family_witness_mismatch`: `restore from 26cc5a58f2:crates/worth-relational/src/tests/history/milestone_7e_phase_m/mod.rs`.
  C4 owns this injected typed-summary/witness or stored-original fault; restore
  the same-SHA `fixtures.rs` and `drift_support.rs` beside the root, preserving exact denials.
- `merge_collaboration_truth_denies_summary_only_and_planner_shortcuts`: `restore from 26cc5a58f2:crates/worth-relational/src/tests/history/milestone_7e_phase_m/mod.rs`.
  C4 owns this injected typed-summary/witness or stored-original fault; restore
  the same-SHA `fixtures.rs` and `drift_support.rs` beside the root, preserving exact denials.

- Disconnected-graph half of `merge_branch_basis_denial_happens_before_planning_for_missing_or_disconnected_branches`:
  `restore from 45b6e87a0a:crates/worth-relational/src/tests/history/milestone_7e_phase_a.rs`.
  Restore `graft_disconnected_branch_head` from that file. C4 owns its forged
  receipt/cell/graph installation and exact missing-base/binding denials. The
  ordinary missing-source half now observes shipping owner binding publicly.
- `replay_rebuild_denies_summary_and_packet_drift`:
  `restore from 45b6e87a0a:crates/worth-relational/src/tests/history/milestone_7e_phase_e/replay_drift.rs`.
  Restore its parent fixture from the same SHA. This injects summary/packet drift
  and checks the private consistency predicate; it never actually ran a rebuild.
  C4 must retain the exact fault assertions without reviving runtime attachment.

- `residency_failure_matrix_nonhead_merge_history_covers_every_door_and_class`:
  `restore from 9addca7179:crates/worth-relational/src/tests/durability/medium/cold_merge_history_matrix/exhaustive.rs`.
  Moved under the coordinator's 2026-11-09 merge-parent ruling. C4 owns the
  cold-read fault matrix for merge history (every door and cause). Its original
  fixture, door dispatch and cold provider are retained at the same restore SHA
  in the parent `cold_merge_history_matrix.rs`, `cold_merge_history_fixture.rs`,
  `cold_recovery_fixture.rs`, `cold_read_matrix.rs`, `cold_source.rs` and
  `cold_source/protected_reads.rs`. This is relocation, not retirement or
  weakened fault assertions.

### C2 - Cost and bounds: backing and work budgets

- Exact old-layout portion of `memory_frame_preserves_original_payload_and_scalar_tuple_observations`:
  `restore from 86e84dc5cd:crates/worth-relational/src/history/commit/envelope_residency/memory_publication_tests.rs`.
  Restore the original complete OldBacking (both variants and order), OldState,
  OldCell, original Memory construction/drop diagnostic and both exact size
  assertions with the same-SHA parent type imports. The functional original
  Memory payload length, digest, real backing and full-frame/payload distinction
  remain in the mounted genuine owner unit. Never delete the Journal variant
  to fit a smaller layout oracle; this physical comparison belongs to C2.


Remaining pure cleanup roots (per-claim Landing 2a disposition):

- `positioned_agreement_uses_original_work_budget_and_preserves_authoritative_digest`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/durability/contracts/canonical_commit_artifact.rs`.
- `native_custody_correction_last_actual_capture_abandonment_releases_shared_root_custody`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/durability/data/native_checkpoint/component_custody/tests.rs`.
- `equal_exact_root_reuses_the_readmitted_partition_substrate`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs`.
- `native_custody_correction_cross_lane_acquire_release_has_one_actual_capture_cut`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/retention/component_cut_tests.rs`.
- `native_custody_correction_current_lane_inventory_is_collected_inside_actual_cut`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/retention/component_cut_tests.rs`.
- `an_indexed_envelope_remains_cold_until_explicit_readmission_and_loads_once`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/envelope_residency_tests.rs`.
- `relinking_other_envelopes_preserves_an_index_even_while_its_body_is_in_flight`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/envelope_residency_tests.rs`.
- `settlement_releases_a_journal_flight_but_preserves_memory_history_and_position`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/envelope_residency_tests.rs`.
- `positioned_demand_owns_its_result_across_journal_settlement`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/envelope_residency_tests.rs`.
- `journal_publication_keeps_journal_placement_without_memory_frame`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/envelope_residency/memory_publication_tests.rs`.
- `canonical_artifact_accounting_excludes_nested_diagnostic_perturbation`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/artifact_tests.rs`.
- `populated_derived_index_artifacts_only_change_the_optional_cache_lane`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/artifact_tests.rs`.
- `strategy_input_clone_retains_only_its_original_decoded_bytes_grant`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/artifact_tests/strategy_readmission.rs`.
- `strategy_descriptor_clone_retains_only_its_original_decoded_locator_grant`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/artifact_tests/strategy_readmission.rs`.
- `shared_readmitted_envelope_clones_keep_the_original_allocation_and_grant_until_last_drop`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/artifact_tests/shared_readmission.rs`.
- `actual_weak_position_retains_only_the_fixed_original_control_bill_until_last_drop`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/artifact_tests/shared_readmission.rs`.
- `embedded_checkpoint_preserves_ignored_unknown_nested_fields_without_retaining_their_heaps`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/history/commit/artifact_tests/shared_readmission.rs`.
- `result_snapshot_cleanup_registry_contention_preserves_exact_cell_and_competing_task`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/runtime/state/subsystems/visibility/snapshot_handles/published_registry/original_cleanup/tests.rs`.
- `checkpoint_recovery_preserves_abandoned_symbol_identity_frontiers`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs`.
- `seeking_past_an_acknowledged_index_prefix_never_demands_its_journal_bodies`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/durability/canonical_commit_artifact/retained_export/skip_tests.rs`.
- `structural_identity_recovery_preserves_current_evidence_and_queries`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/inspection/structural_identity.rs`.
- `complexity_budget_schema_transition_classification_is_changed_atom_bounded`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/complexity/contracts/commit_budgets/schema_continuity_boundaries.rs`.
- `complexity_budget_subscriber_resume_continuity_is_boundary_local`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/complexity/contracts/commit_budgets/schema_continuity_boundaries.rs`.
- `complexity_budget_milestone5_closeout_keeps_schema_cdc_and_recovery_boundary_local`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/complexity/contracts/commit_budgets/schema_continuity_boundaries.rs`.
- `complexity_budget_merge_planning_reports_request_shaped_work`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/complexity/contracts/merge_budgets.rs`.
- `complexity_budget_merge_execution_reports_admitted_records_and_emitted_mutations`: `restore from 86e84dc5cd738882bf50fced389fc7deaa31567f:crates/worth-relational/src/tests/complexity/contracts/merge_budgets.rs`.
  Restore each root's complete same-SHA parent/descendant fixture and control
  closure, including its original local medium script and owner units; migrate
  the source boundary through genuine owned media without reviving the retired
  runtime-discovery adapter. All original guards remain required.

  The merge execution root's four public semantic assertions remain in
  `merge_source_copy_identity.rs::source_only_merge_emits_one_new_entity_with_an_independent_identity`.
  C2 restores its full original cost/contract campaign and fixture closure.


B2 schema/history consumer split (coordinator per-claim Landing 2a ruling):

- `durability_contract_recovery_rebuilds_branch_head_root_obligations` — restore from `df2420390ecdbf870bbc47384ba45bddee5c5359:crates/worth-relational/src/tests/durability/contracts/retention_inspection_store.rs`: exact active-snapshot and branch pin census.
- `durability_contract_recovery_preserves_inspection_truth_bundle` — restore from `df2420390ecdbf870bbc47384ba45bddee5c5359:crates/worth-relational/src/tests/durability/contracts/retention_inspection_store.rs`: retention_summary and record_retention fields of the complete original bundle; keep all original producer and inspection helpers.
- `tail_replay_consumes_canonical_reused_record_allocation` — restore from `df2420390ecdbf870bbc47384ba45bddee5c5359:crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs`: exact reclaim limit, one reusable slot, same local slot and incremented generation; public current/historical truth is ported.
- `durability_certification_recovery_authority_continuity_is_explained_and_counted` — restore from `df2420390ecdbf870bbc47384ba45bddee5c5359:crates/worth-relational/src/tests/durability/contracts/descriptor_continuity.rs`: exact digest-parity and mismatch counters, alongside original diagnostic producer.
- `durable_recovery_and_schema_mismatch_test` — restore from `df2420390ecdbf870bbc47384ba45bddee5c5359:crates/worth-relational/src/tests/durability/contracts/schema_transition_continuity.rs`: actual recovered digest-parity work counter; all semantic descriptor and refusal assertions stay public.
- `native_field_text_checkpoint_then_tail_recovers_exact_json_after_original_owners_drop` — restore from `df2420390ecdbf870bbc47384ba45bddee5c5359:crates/worth-relational/src/tests/durability/contracts/managed_field_text.rs`: exact Symbol representation, charged managed payload and System client-key custody; retain read_exact and original request policy closure.

- Fixture closure for `recorded_original_retirement_preserves_both_leases_across_busy_and_stop`:
  `restore from 0615dd733c:crates/worth-relational/src/history/retention/lane/retained_release/tests.rs`
  (`assert_recovered_cut_and_lane_busy`), with its same-SHA exact re-export lines
  in `history/retention/mod.rs`, `history/retention/lane.rs`, and
  `history/retention/lane/retained_release.rs`. Its final caller moved to C2;
  the other external-cut fixture remains live. The original source tests above
  also reference the same-SHA local-medium `head`, `transition_head`,
  `drop_artifact` and journal `remove` fixtures. Restore those physical/fault
  observations through the actual owner/provider, without reviving adapters.

- Associated-original checkpoint metadata/body nonresidency portions:
  `associated_effects_discover_cold_subject_without_reading_unrelated_history`
  (`restore from 0615dd733c:crates/worth-relational/src/tests/durability/contracts/associated_effects.rs`)
  and `native_custody_correction_enumerates_known_orphan_and_unrelated_association_metadata`
  (`restore from 0615dd733c:crates/worth-relational/src/tests/durability/contracts/associated_effects_discovery.rs`).
  C2 preserves the exact per-cell before/after resident census, with original
  same-SHA fixture closure. Actual metadata/semantic-result behavior stays in
  public `association_checkpoint_journeys.rs`.
- `caller_association_bounds_and_rebinding_refuse_without_retaining_publication`
  (pending allocation/route census) and
  `caller_association_survives_cold_checkpoint_history_and_journal_tail`
  (selected directory, journal absence and cold-body posture):
  `restore from 0615dd733c:crates/worth-relational/src/tests/durability/contracts/caller_association.rs`.
  Restore original `propose`/`publish` fixture closure for those physical claims.
  Wire/bounds and embedded original codec remain Native value units; all public
  binding and cold original-frame equality remains in `caller_association_journeys.rs`.
- `recorded_component_checkpoint_readmits_cold_canonical_predecessor_and_successor`
  and `recorded_component_checkpoint_readmits_original_empty_predecessor`,
  private resident/cold-root and full physical-root demand portions;
  `recorded_component_checkpoint_omits_released_components_and_plain_observations`
  (whole retained-root census) and
  `recorded_original_retirement_preserves_both_leases_across_busy_and_stop`
  (whole private pin/release/stop accounting campaign):
  `restore from 0615dd733c:crates/worth-relational/src/tests/durability/contracts/recorded_component_checkpoint.rs`.
  Restore `original_custody`, `Original`, capture/recovery and retirement fixtures
  at the same SHA. C2 must supply honest public pin/allocation observations;
  never revive the local provider adapters. The Canonical/Empty original
  descriptors, readable snapshots after maintenance, semantic result,
  historical-currentness refusal and no-republication guards stay public in
  `recorded_original_checkpoint_journeys.rs`. The actual unrelated-body deletion
  remains a C4 component of this original campaign, as listed below.
- `a_recovered_runtime_publishes_its_next_commit_at_the_next_position`,
  original/reopened cold-source allocation pointer identity only:
  `restore from 0615dd733c:crates/worth-relational/src/tests/durability/medium/recovered_publication.rs`.
  Public source bytes/limits and actual recovered originals, receipt/coverage,
  no recovery publication, next ID/parent/position and second reopen remain in
  `medium_recovered_continuation.rs`.
- `checkpoint_size_tracks_branch_state_not_covered_transitions`, manifest
  growth bound and covered resident-log release portions:
  `restore from 0615dd733c:crates/worth-relational/src/tests/durability/contracts/branch_transition/checkpoint_state.rs`.
  Restore both exact 32-branch lifecycle scripts, 64/128 counts, captured
  sequences, empty covered logs and `through_archive_and_pending_deletion <= direct + 32`.
  The same lifecycle inputs and exact recovered sequence continuation stay
  public in `checkpoint_transition_continuation.rs`.


- `checkpoint_recovery_readmits_shared_root_once_for_sibling_heads`, physical
  single image and exact readmitted Root pointer sharing:
  `restore from e4d3b01c8d:crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs`.
  C2 restores the original fixture closure; the two readable branch contents
  and equal admitted targets remain public in `checkpoint_branch_targets.rs`.
- `durability_contract_checkpoint_tail_recovery_preserves_post_checkpoint_commits`,
  exact retained envelope census and original partition-image backing pointer:
  `restore from e4d3b01c8d:crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs`.
  C2 restores these original allocation assertions; exact checkpoint/tail
  coverage, receipts and old envelope readability remain in public
  `checkpoint_tail_and_indexes.rs`.
- `acknowledged_transition_admits_only_its_exact_successor_and_releases_snapshot`,
  `foreign_recovery_and_foreign_candidate_are_refused_before_effects`, and
  `moved_source_and_stale_candidate_cannot_extend_recovery_lineage`, private
  published/preparation snapshot-handle census portions only:
  `restore from e4d3b01c8d:crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs`.
  Restore the original same-SHA fixture closure for C2 allocation/custody
  observation. All exact public authority admission/refusal/retry and unchanged
  descriptors/counts remain in `checkpoint_successor_authority.rs`.


- `controlled_resident_cell_keeps_exact_charge_until_last_real_weak` and
  `controlled_resident_cell_refuses_exact_control_before_any_new_arc`:
  `restore from f9f15bf8c8:crates/worth-relational/src/storage/residency/cell/resident_full/tests.rs`.
- `controlled_full_region_matches_prior_projection_preimages_and_legacy_truth` and
  `controlled_full_commitment_clone_retains_only_its_actual_node_backing`:
  `restore from f9f15bf8c8:crates/worth-relational/src/storage/residency/region_publication/full/tests.rs`.
  These four assert the exact request-funded bills of `resident_for_request` and
  `PreparedResidentRegion::initial_for_request`. The ordinary kind-commitment
  preimage oracle of the third root is not moved: it stays in the tree on the
  live producer as `full_region_kind_commitment_matches_independent_projection_preimages`
  (`storage/residency/region_publication/kind_commitment_preimage_tests.rs`). Both constructors had no
  production caller and were deleted with the 32 Native dead-code diagnostics
  (`target/lane-evidence/landing2a-dead-code-deletion-notes.md`); the live Full
  producer `PreparedResidentRegion::initial` takes no request, so the bills
  cannot be observed through it. C2 restores the constructors with these roots
  from the same SHA only if request-funded Full construction gains a production
  caller; otherwise C2 states the equivalent bound on the live producer.

- Instrumentation half of `execute_prepared_merge_reports_execution_counters_and_structural_summary`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7c_phase_e/execution_counters_diagnostics.rs`.
- Instrumentation half of `execute_prepared_merge_records_attempt_without_success_on_failure`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7c_phase_e/failure_attempt_diagnostics.rs`.
- Instrumentation half of `native_merge_touches_only_changed_revision_and_both_index_membership_keys`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7d_phase_d/merge_descriptive_touches.rs`.
- Instrumentation half of `built_in_monotonic_counter_merge_is_auto_resolved_with_inline_value_and_recovery_parity`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7d_phase_d/monotonic_counter_resolution.rs`.
- Instrumentation half of `disjoint_rewire_neighborhoods_do_not_escalate_to_topology_region_conflict`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7d_phase_d/topology_local_rewire.rs`.
- Instrumentation half of `topology_region_conflict_detection_reports_bounded_neighborhood_counters`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7d_phase_d/topology_region.rs`.
  Each of these roots is split under Landing 2a's per-claim disposition. Its
  public semantic claim is ported to the certification court
  (`relational/durable_publication/merge_*`); C2 owns only the exact
  instrumentation-counter, complexity-delta or private resident-Root assertions
  named for the root in
  `target/lane-evidence/landing2a-history-compiler-claim-disposition.md`.
  Restore the whole original root and its same-SHA fixtures, with genuine owned
  inputs and without reviving the removed runtime-discovery adapter.

- `branch_roots_with_one_schema_deduplicate_the_durable_carrier`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_root_schema_binding.rs`.
- `replay_reads_only_the_gap_after_its_checkpoint`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/branch_transition/reconstruction.rs`.
- `durability_tail_replay_releases_each_commit_snapshot_beyond_live_capacity`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs`.
- `deleted_branch_reader_retains_only_its_exact_scoped_generation`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/deleted_branch_reader.rs`.
- `historical_reader_from_live_head_survives_head_retirement`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/historical_reader.rs`.
- `reclamation_and_checkpoint_keep_sibling_head_generation`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs`.
- `repeated_edits_checkpoint_only_the_live_index_payload`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs`.
- `reclamation_keeps_a_pinned_old_reader_until_release`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs`.
- `repeated_closed_branches_leave_no_latest_generation_residue`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs`.
- `repeated_global_same_basis_builds_reclaim_duplicates_but_keep_pinned_root`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/global_generation_reclamation.rs`.
- `native_field_text_checkpoint_readmission_denial_keeps_bytes_and_target_root_intact`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/managed_field_text.rs`.
- `repeated_native_captures_do_not_retain_checkpoint_images`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/native_checkpoint.rs`.
- `synthetic_native_restore_and_image_digest_benchmark`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/native_checkpoint_restore_benchmark.rs`.
- `checkpoint_restores_burned_append_frontier_beyond_all_retained_roots`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs`.
- `recovery_releases_orphaned_append_reservation_without_reusing_its_gap`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs`.
- `recovery_returns_orphaned_reclaimed_reservation_to_reuse`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs`.
- `durability_contract_branch_heads_do_not_mutate_legacy_record_pins`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/contracts/retention_inspection_store.rs`.
- `embedded_restore_allocation_grows_linearly_in_relation_bearing_regions`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/embedded_cost.rs`.
- `an_attachment_the_medium_moved_out_of_publishes_nothing`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/durability/medium/single_owner.rs`.
  C2 owns each of these durability contract/medium roots under Landing 2a's
  per-claim disposition. Restore each root's same-SHA fixture closure (parent
  module helpers, `tests/support/durability.rs` and the original local medium
  controls) as recorded per root in
  `target/lane-evidence/landing2a-durability-contract-claim-disposition.md`.
  This is relocation with the original assertions, scripted fates and no-effect
  oracles, not retirement or weakened evidence. Restore over genuine owned
  media; do not revive the removed runtime-discovery adapter.

- `verify_prepared_merge_execution_does_not_increment_planning_counters`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7c_phase_a/verification_counters.rs`.
- `verify_prepared_merge_execution_reports_verification_counters_without_planning_work`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7c_phase_a/verification_counters.rs`.
- `unrelated_relation_additions_do_not_inflate_topology_region_detection_counters`: `restore from f9f15bf8c8:crates/worth-relational/src/tests/history/milestone_7d_phase_d/topology_local_rewire/neighborhood_detection.rs`.
  C2 owns the original claim under Landing2a's per-claim disposition.
  Restore each named root's same-SHA owning fixture/module imports and private
  control references recorded in `target/lane-evidence/landing2a-history-compiler-claim-disposition.md`.
  This is relocation with its original guard assertions, not retirement or
  weakened evidence. Restore with genuine owned inputs, without reviving the
  removed local provider/discovery adapters.


The following adapter-dependent Native performance campaigns move to C2 under
Landing 2a's cost/growth disposition. Their assertions and durable posture are
preserved at the restore references. Restore each root's descendant modules
from the same SHA and its shared imports from `tests/performance_profiles.rs`;
the exact mount closures and metric/scale evidence are recorded in
`target/lane-evidence/landing2a-performance-claim-disposition.md`. The thirteen
campaigns outside the adapter closure remain unchanged in this tree.

Campaign-only fixtures removed with those roots also restore from `ee7aef9014`:
`tests/performance_profiles/measurement_support.rs`, `dense_patch_record_count`
in `tests/performance_profiles/invariant_support.rs`, `PerfMetricValue::is_null`
in `tests/performance_metrics.rs`, the ten performance adapters/exports in
`tests/domains/fintech/mod.rs`, and the two Chip variants/arms of
`PerfDiagnosticsPolicy` in `tests/support/runtime.rs` (paths relative to
`crates/worth-relational/src/`). Remaining shared fixtures and all ordinary
Fintech scenarios remain. C2 must restore these claims with actual owned durable
media through the public testkit, preserving the claims without reviving the
retired runtime-discovery adapters.

- `perf_artifact_recoverability_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/artifact_recoverability_matrix.rs`.
- `perf_chip_simulator_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/chip_simulator_matrix/mod.rs`.
- `perf_commit_delta_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/commit_delta_matrix.rs`.
- `perf_durability_append_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/durability_append_matrix.rs`.
- `perf_geometry_kernel_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/geometry_kernel_matrix/mod.rs`.
- `perf_hot_cold_path_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/hot_cold_path_matrix/mod.rs`.
- `perf_index_parity_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/index_parity_matrix.rs`.
- `perf_merge_lineage_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/merge_lineage_matrix/mod.rs`.
- `perf_recoverability_policy_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/recoverability_policy_matrix.rs`.
- `perf_replay_recovery_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/replay_recovery_matrix.rs`.
- `perf_sustained_load_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/sustained_load_matrix/mod.rs`.
- `perf_workflow_matrix`: `restore from ee7aef9014:crates/worth-relational/src/tests/performance_profiles/workflow_matrix/mod.rs`.

- `request_root_refusal_preserves_original_and_releases_partial_backing`:
  `restore from 2a51552fc5:crates/worth-relational/src/branch/root_tests/request.rs`.
  Moved under the coordinator's 2026-11-09 Landing 2a ruling, not retired or
  weakened. It proves the specific unchanged-root replacement allocation refuses
  at controller + `size_of(root)` - 1, releases partial backing, preserves the
  original, and honours cancellation. C2 must provide the public
  allocation-refusal observation before restoring this claim.

- `publication_cost_distinguishes_new_and_reused_schema_authority`: `restore from b736be6f47:crates/worth-relational/src/branch/root_tests.rs`.
- `replay_pin_of_a_cold_partition_is_typed_and_has_no_effect`: `restore from b736be6f47:crates/worth-relational/src/tests/durability/medium/cold_isolation.rs`.
- `resident_pin_custody_refuses_unread_mirror_frames_after_exact_source_admission`: `restore from b736be6f47:crates/worth-relational/src/tests/durability/medium/cold_pins.rs`.
- `selected_branch_views_preserve_cold_probe_and_demand`: `restore from b736be6f47:crates/worth-relational/src/tests/durability/medium/cold_invariants.rs`.
  These courts retain private physical pin actions/counts, resident-only probe
  state, or exact publication/materialization accounting. Restore the original
  claims in C2 through its honest allocation/residency observation; they are not
  retired. Public semantic read/failure courts remain in the happy tree.

- C2: the Graph and Divergence subcases of
  `residency_failure_matrix_lineage_event_doors_preserve_every_failure_class`:
  restore from `ea2ffee250:crates/worth-relational/src/tests/durability/medium/cold_lineage_event_matrix.rs`
  and `ea2ffee250:crates/worth-relational/src/tests/durability/medium/cold_lineage_event_matrix/exhaustive.rs`.
  These collectors exist only under cfg(test), with no shipping public door.
  Seven distinct shipping owner paths, plus the two original aspect-access
  forwarding aliases mapped to their identical public delegates, retain their full
  typed-refusal, repair/stickiness, no-publication and frontier assertions in
  the external cold-residency court.
- C2: private zero-resident-node/event preconditions from the lineage fixture
  helpers used by the migrated concurrent and integrity courts: restore from
  `ea2ffee250:crates/worth-relational/src/tests/durability/medium/cold_lineage_fixture.rs`
  and `ea2ffee250:crates/worth-relational/src/tests/durability/medium/cold_lineage_event_fixture.rs`.
  The external courts preserve zero physical reads at reopen and all original
  public result, byte-integrity, concurrency and source-custody assertions.

- C2: the removed `resident_replay_pins` resident-only fixture hook:
  `restore from ea2ffee250:crates/worth-relational/src/visibility/pins/pin_authority/replay_pins.rs`.
  Its only callers were the already-moved private pin courts; the shipping
  funded `demand_replay_pins` and its resident implementation remain unchanged.

- C2: the exact cached-Head/absent-per-runtime-dictionary preparation window in
  `metadata_pin_refuses_shared_head_until_its_runtime_has_the_aspect_meanings`:
  `restore from 427700edd0:crates/worth-relational/src/tests/durability/medium/cold_metadata_pins.rs`.
  Preserve private `demand_region_head_cached`, its no-read assertion and the
  original `SymbolsNotResident` refusal. Its public read/pin suffix is retained
  as `fork_metadata_pin_after_public_head_read_preserves_original_aspect_revision`.
- C2: the two direct Cold residency probes in
  `residency_retained_kind_demands_only_its_cold_head_and_preserves_damage`:
  `restore from 427700edd0:crates/worth-relational/src/tests/durability/medium/cold_materialization_doors.rs`.
  Preserve the before-kind Head-Cold and after-kind Full-Cold observations.
  Actual suspension, retained kind and typed storage refusal remain in
  `retained_kind_after_public_suspension_preserves_kind_and_storage_refusal`.
  Original fixture dependency: `restore from 427700edd0:crates/worth-relational/src/tests/durability/medium/cold_current_fixture.rs`.
  These private claims are moved, not retired; a public I/O count is not their
  physical residency/dictionary observation.


- C2: resident node/event cardinalities, exact per-runtime event-index population,
  and query-index nonmutation from the mixed lineage courts:
  `restore from 81d41f5575:crates/worth-relational/src/tests/durability/medium/cold_lineage_events.rs`
  and `restore from 81d41f5575:crates/worth-relational/src/tests/durability/medium/cold_lineage_nodes.rs`.
  Keep all zero/unchanged/admitted count, absent-node and shared-page warmup
  assertions; the external lineage semantics, point guards and capture-row
  courts retain original resolution/digest, member noninterference, sticky
  typed failures, public frontier/offer invariance and serialized row coverage.
  Original zero-node/zero-event preparation also belongs to C2:
  `restore from 81d41f5575:crates/worth-relational/src/tests/durability/medium/cold_lineage_event_fixture.rs`.
  The capture's exact no-load/shared-index preparation and no member-read suffix
  are resident-state/cost claims; C2 restores them, not a public semantic proxy.
- C2: Graph/Divergence subcases and the exact retry-resident-node predicate in
  `residency_failure_matrix_lineage_doors_preserve_every_failure_class`:
  `restore from 81d41f5575:crates/worth-relational/src/tests/durability/medium/cold_lineage_matrix/exhaustive.rs`
  with node-door setup in `cold_lineage_nodes.rs` above. Seven shipping doors
  keep all six failure classes and original successful/sticky/retry/publication
  consequences in the external node matrix. The fixture's resident-zero setup:
  `restore from 81d41f5575:crates/worth-relational/src/tests/durability/medium/cold_lineage_fixture.rs`.
- C2: exact zero-node observation after cold mirror refusal in
  `lineage_point_guard_preserves_a_cold_mirror_head_failure`:
  `restore from 81d41f5575:crates/worth-relational/src/tests/durability/medium/cold_observation_reads.rs`.
  Public historical inspection preserves the live mirror-lineage helper's exact
  typed/sticky refusal; the paired retained-observation court preserves all four
  original exact-root guards and no publication effects.
- C2: physical replay pin counts, direct historical-root Full/cold-mirror probes,
  replay-retention census, private cache diagnostics and exact cache-clear window:
  `restore from 81d41f5575:crates/worth-relational/src/tests/durability/medium/cold_pins.rs`.
  Public paired courts are `replay_pinning_preserves_sticky_mirror_refusal_without_publication`
  and `replay_retention_releases_original_references_without_cold_reads`.
  Keep C2's exact source-state/pinned-record setup and failed-installation census.
- C2: one-pin/zero-pin census in `concurrent_replay_references_share_one_physical_pin_set`:
  `restore from 81d41f5575:crates/worth-relational/src/tests/durability/medium/replay_pin_concurrency.rs`.
  Its public two eight-worker retain/release phases, all return values and final
  false release remain in `concurrent_replay_references_release_each_original_reference_once`.
- C2: exact discarded-round materialization and adjacency-lease counters, including
  resets and accepted-round equality, in
  `discarded_visibility_rounds_settle_only_accepted_materialization_and_leases`:
  `restore from 81d41f5575:crates/worth-relational/src/tests/durability/medium/cold_visibility_round_counters.rs`.
  Public cold/warm nonempty result equality and actual additional carrier reads
  remain in `partially_cold_visibility_reads_preserve_nonempty_warm_results`.
  All these private assertions move to C2; none is retired or weakened.

- C2: now-uncalled resident-node fixture probe used by the moved node matrix:
  `restore from 81d41f5575:crates/worth-relational/src/runtime/state/subsystems/lineage.rs`.
  Restore its cfg(test) `LineageSubsystem::node` only with the actual resident
  assertions above; the shipping counted semantic node read remains unchanged.


- C2: exact Weak canonical-body lifetimes, resident envelope/payload census and
  private reclamation setup in the three mixed envelope-lifetime courts:
  `restore from 1403ddb275:crates/worth-relational/src/tests/durability/medium/cold_envelope_lifetime.rs`.
  Originals: `settled_live_nonhead_envelopes_release_only_when_a_durable_copy_exists`,
  `released_live_history_remains_in_external_capture_and_retained_recovery`,
  `recovered_nonhead_tail_envelopes_release_their_actual_journal_backed_bodies`.
  Public counterparts retain original patch readability in memory/durable cases,
  all original commit identities after both recovery routes, the <32 source-read
  bound, zero initial tail-source reads and receipt equality. Public reclamation
  and the original <4 retired-root observation stay in the happy tree.
- C2: provisional packet/slot-scan/materialization counter snapshots and exact
  accepted-round equality in `invariant_storage_refusal_is_sticky_and_does_not_settle_provisional_counts`
  and `cold_and_warm_invariant_reads_settle_one_round_of_evidence`:
  `restore from 1403ddb275:crates/worth-relational/src/tests/durability/medium/cold_invariants.rs`.
  The paired public invariant courts keep every nonempty/result-equality,
  typed-refusal, first/warm/sticky I/O, selected-History noninterference and
  no-publication assertion, including the unchanged third Head/Full court.
- C2: exact private dictionary snapshot and provisional packet/scan counters in
  `candidate_invariant_discovery_refuses_stickily_without_publishing_round_effects`,
  Main-Full/remote-Cold probes and arena-slot floor in the candidate fixture,
  plus exact abandoned-allocation slot comparisons/headroom/settled counters in
  `a_region_retry_spends_the_applied_draft_identity_and_settles_one_round`:
  `restore from 1403ddb275:crates/worth-relational/src/tests/durability/medium/cold_candidates.rs`.
  Its paired failure court preserves Validation/Candidate/Commit refusals,
  sticky I/O and original diagnostic Vec equality through the public owner;
  its paired successful court preserves both changed-entity observations,
  actual created record values and <24 source reads. Public diagnostic contents
  are not moved to C2. Source recipe remains mounted for its remaining callers.
- C2: exact Cold/Resident Head probes and physical occupied-slot/generation
  enumeration in `head_probes_admit_each_forks_shared_region_without_reading_history`
  and `cold_forks_edit_existing_ring_records_in_both_orders_and_checkpoint`:
  `restore from 1403ddb275:crates/worth-relational/src/tests/durability/medium/cold_fork_edits.rs`.
  Public paired courts keep missing-History Head name reads in both fork orders,
  both edit orders, exact updated/deleted entity values, real relation creation,
  standalone checkpoint/recovery, partition4 relation count>=2 and per-ID
  effective relation equality. These private subclaims move, not retire.

- C2: private per-runtime interner absence in current-scan cells of
  `residency_failure_matrix_full_read_subjects_cover_every_door_and_class`:
  `restore from 14e282569e:crates/worth-relational/src/tests/durability/medium/cold_read_matrix/exhaustive.rs`.
  The public counterpart keeps all 47 doors, three subjects, six causes, exact
  refusal/retry behavior, target reads, current-scan history noninterference and
  publication invariance. The existing request-funded version read is now public.
- C2: exact receiver dictionary absence/admission/resolved text in
  `every_full_read_door_admits_its_runtime_dictionary_or_preserves_page_damage`:
  `restore from 14e282569e:crates/worth-relational/src/tests/durability/medium/cold_symbol_read_doors.rs`.
  All original public success/refusal, sticky I/O, Head identity and frontier
  observations remain in `full_read_doors_preserve_history_symbol_refusals_and_current_head`.
- C2: private Full/Head probes, metadata-history length, dictionary contents and
  <400 entry bound, and shared-Full/receiver-cold preparation in
  `head_symbols_never_cover_full_history_or_inherit_its_page_damage`:
  `restore from 14e282569e:crates/worth-relational/src/tests/durability/medium/cold_symbol_view_levels.rs`.
  Public paired court retains actual current field identity, Full typed refusal,
  sticky/no-extra-I/O retry, unchanged Head, historical values and frontier.
  Public history entries do not replace the private metadata-history claim.
- C2: private exact-basis representation assertion `view.is_exact_basis()` in
  `residency_failure_matrix_index_doors_preserve_every_failure_class`:
  `restore from 14e282569e:crates/worth-relational/src/tests/durability/medium/cold_index_matrix.rs`.
  Public issued-version equality, all 14 doors/six causes, no-load generation
  residency, exact generation/entry equality, successor bounds, reservation
  counters and publication/I/O assertions remain in the public matrix.
- C2: exact resident interner text and cold-envelope cache probes in
  `historical_index_reconstruction_uses_the_old_value_and_full_cold_history` and
  its `assert_cold` preparation:
  `restore from 14e282569e:crates/worth-relational/src/tests/durability/medium/cold_historical_index_matrix.rs`;
  matrix cold-envelope assertion:
  `restore from 14e282569e:crates/worth-relational/src/tests/durability/medium/cold_historical_index_matrix/exhaustive.rs`.
  Both public counterparts preserve genuine old source/rows, generation bounds,
  all five targets/six causes, exact unconsulted-envelope reads, reservation
  counters and no publication. The actual single carried MVCC root is checked
  against an issued main observation; an old receipt is not inferred to be a root.
- C2: whole private allocation/inventory court
  `residency_cold_index_frontier_prevents_reissue_before_any_image_demand`:
  `restore from 14e282569e:crates/worth-relational/src/tests/durability/medium/cold_index_frontier.rs`.
  Restore all four actual IndexSubsystem reserve/publish/frontier/inventory
  sequences with their zero/positive source reads. Public actual builds retain
  monotonic generations and the original <old+16 bound; they do not stand in for
  these private allocator actions. No claim is retired.

- C2 (2026-11-09 Landing2a move ruling, not retirement): exact mirror-Head
  residency preparation, capture layer/input bounds and >=2 consolidation
  count in `checkpoint_restart_cycles_preserve_full_reads_and_retention`:
  `restore from 3fa8da0cb4:crates/worth-relational/src/tests/durability/medium/checkpoint_restart_bound.rs`.
  Its public counterpart preserves three initial updates, 32 iterations, both
  genuine recoveries per iteration, selected-root and mirror Head exclusion,
  exact selected New/Carried membership (unselected bytes disappear), retention
  and exact record comparison. It does not claim to reproduce private Head-only
  preparation or prove repeated consolidation.
- C2: shared merge fixture's exact resident dictionary absence in
  `recovered_shape`, used by the two basic merge courts and member/index matrices:
  `restore from 3fa8da0cb4:crates/worth-relational/src/tests/durability/medium/cold_merges.rs`.
  Public counterparts retain all typed refusals/retries, source reuse, effects,
  adopted/changed counts, memory-reference records and exact index key/entity
  parity. Functional contributing-commit provenance remains HAPPY in the two
  separate merge-history courts; it is not moved with this dictionary claim.
- C2: exact receiver dictionary snapshot equality and provisional preparation
  packet/invariant slot-scan work accounting in
  `residency_failure_matrix_remote_invariant_heads_cover_every_preparation_door_and_class`:
  `restore from 3fa8da0cb4:crates/worth-relational/src/tests/durability/medium/cold_preparation_failure_matrix/exhaustive.rs`.
  Restore original Main Full/remote Cold/floor setup with it from
  `restore from 3fa8da0cb4:crates/worth-relational/src/tests/durability/medium/cold_candidates.rs`.
  Both public matrices retain Validation/Candidate/Commit, all six causes,
  diagnostics Vec equality, sticky/transient I/O, publication effects
  and exact History noninterference. Public outcomes do not replace private work.
- C2: private accepted-round adjacency lease/materialization counter equality in
  `projection_adjacency_discovery_settles_only_one_accepted_round`, plus exact
  resident dictionary text resolution in the incoming-door fixture:
  `restore from 3fa8da0cb4:crates/worth-relational/src/tests/durability/medium/cold_projection_adjacency_matrix.rs`.
  Public counterpart `cold_and_warm_projection_adjacency_reads_preserve_actual_edges`
  retains genuine cold/warm nine-door reads and adds warm no-I/O. The public
  nine-door/six-subject/six-cause matrix retains all current/historical, callbacks,
  exact failures/retries, unconsulted sources and publication assertions. The
  two genuine index DATA readers include audit-retained edges, never Live-only
  replacements. Exact private work accounting stays with this C2 original.
- C2: exact interner absence before/after anchor read in
  `the_remote_ring_label_stays_cold_until_its_incoming_edge_is_read`, and private
  dictionary resolution in its real paged-edge preparation:
  `restore from 3fa8da0cb4:crates/worth-relational/src/tests/durability/medium/cold_ring_dictionary.rs`.
  Restore the exact original physical shape from
  `restore from 3fa8da0cb4:crates/worth-relational/src/tests/durability/medium/cold_isolation.rs`.
  Public counterpart preserves incoming-edge typed refusal and proves the actual
  page is unconsulted before/after anchor read, then physically demanded.
- C2: exact resident dictionary snapshot equality in
  `cold_symbol_buckets_refuse_each_preparation_door_without_journal_effects_and_stay_damaged`,
  and arena spent-slot/floor/+16 headroom plus <population/2 resident-dictionary
  assertions in `a_cold_symbol_probe_spends_the_abandoned_record_identity_and_reads_only_touched_buckets`:
  `restore from 3fa8da0cb4:crates/worth-relational/src/tests/durability/medium/cold_symbols.rs`.
  Matrix dictionary equality:
  `restore from 3fa8da0cb4:crates/worth-relational/src/tests/durability/medium/cold_symbol_failure_matrix/exhaustive.rs`.
  Preserve original 600 client-key entities plus selected collision key. Public
  counterparts keep all five doors including Preview, all six causes, exact
  diagnostics/effects/sticky/retry behavior and successful actual relation with
  original <16 source reads. Public successful case is named
  `cold_symbol_publication_reads_only_touched_buckets`, not a spent-slot claim.
- C2: exact resident symbol absence and private direct Full-warming preparation in
  `cold_head_warm_and_full_warm_uniqueness_reuse_the_original_paged_field_identity`:
  `restore from 3fa8da0cb4:crates/worth-relational/src/tests/durability/medium/cold_unique_symbols.rs`.
  Public counterpart keeps all three warmth postures (actual public snapshot
  Full read), original producer-admitted symbol identity, exact public duplicate
  conflict payload equality and all frontier/offers/reservation assertions.
- C2: resident dictionary inventory/demand/resolution loop and exact private
  root warming in `a_reserved_candidate_published_after_another_branch_checkpoint_keeps_its_pages`:
  `restore from 3fa8da0cb4:crates/worth-relational/src/tests/durability/medium/cold_reserved_checkpoint.rs`.
  Public counterpart keeps exact reservation/publication order, both actual
  selected captures and ACKs, original member membership, reopen/record survival,
  exact counted lineage-node equality, public Full record demand and emitted
  nonempty lineage directory. No root, mutable constructor or counter is exposed.

## Phase map

| Phase | Name | Depends on | Lands |
|---|---|---|---|
| 1 | State survival audit | — | Landing 1 |
| 2 | Store capacity, single writer and incarnation | — | Landing 1 |
| 3 | Application home and host lifecycle | 2 | Landing 2 |
| 4 | Durable occurrences and original-result retry | 3 | Landing 2 |
| 5 | Signal and Bridge authored state; mixed publication | 4 | Landing 3 |
| 6 | Programs, adoption and build identity | 4 | Landing 3 |
| 7 | Workflows, conditional and long-running work | 5, 6 | Landing 3 |
| 8 | External dispatch and custody | 4, 7 | Landing 3 |
| 9 | Correction and original-owner advancement | 4, 6, 7 | Landing 3 |
| 10 | Public cutover and milestone certification | 1–9 | Landing 3; closes the milestone |

**Parallel work and landing order.**

- Phases whose dependencies have landed may be built in parallel, but only
  where their write scopes do not overlap.
- Phases 5 to 10 integrate on one milestone integration branch. Each phase
  merges into that branch when its own reviewer approves and its Tier-2 run
  on the branch is green. A phase built in parallel merges the current
  integration branch first.

**Landing rule.** The milestone reaches master in three landings:

| Landing | Contents | Why it is a separate landing |
|---|---|---|
| 1 | Phases 1–2 | Store foundations and the audit. Nothing is visible to applications. |
| 2 | Phases 3–4 | The first application-visible durability: a home through the host, with original-result retry. Phases 3 and 4 cannot land apart, because a home that recovered graph state but not World occurrences would silently lose application history. |
| 3 | Phases 5–10 | One broad cutover. Signal, programs, workflows, dispatch and correction become durable, and the interim paths are deleted in the same landing. |

- A landing reaches master only after its exit criteria hold and the
  coordinator certifies it at the final gate.
- Between landings 2 and 3, a directory home refuses every family that Phases
  5–9 support (D15), and the interim checkpoint door (D13) serves those
  families' callers. Master is never left half-recovered.
- Within the landing-3 integration branch, each phase lifts its own refusals
  as it merges.

**Format stability.** An artifact format defined in a phase is final for the
milestone. Later phases add new artifacts or new tagged entries in an
extensible list the earlier phase declared; they do not change an existing
format. A phase that must change a landed format states which development
homes stop opening. Under clean cutover they refuse with a typed reason and
are never migrated.

## Adversarial constraint

A home is killed at any durable I/O boundary of any supported operation. That
includes every owner's publication, product movement, program transitions,
checkpoints, close and recovery itself. The home is then reopened any number
of times, in a fresh process, under the same or a changed application build
identity.

Each reopen must:

- recover exactly the acknowledged prefix of every owner, plus at most the
  in-flight operation, decided from durable state;
- reach owner state equal to a memory-only reference host executing the same
  operation sequence at that prefix. This covers graph records, World
  occurrences and custody, product heads, idempotency outcomes, Signal and
  Bridge definitions, programs, workflow state, external custody and
  correction lineage;
- answer every retry of an admitted key with its original occurrence, its
  recovered custody posture, or its typed no-effect posture, and never with a
  second effect;
- never reissue a committed identity of any owner;
- refuse before service, with a typed reason, when it cannot do the above.

A crash between owner effects preserves which owners acted and whether the
product moved. Unresolved effects keep their recovery posture until their
owner establishes the fate. Loss of process memory, cancellation and caller
abandonment are never an implicit completion, rollback or no-effect.

## Decisions

**D1. One application home, Store-owned, host-opened.**

A new Store-workspace crate, `worth-store-application`, owns the application
home. It handles open, initialization, recovery sequencing, checkpoint,
resolve and close for every owner (Milestone 1 D1). It replaces
`RelationalStoreHome` as the only home type. `worth-store-relational` keeps the
Relational medium adapter and cold source, which become the Relational
section of the home.

The Query host is the composition root. `worth-query-host` opens the home
through one call. Applications never construct, open or synchronize a Store
runtime, and they import only `worth-query-decl` and `worth-query-host`.

This requires an explicit boundary ruling. It is recorded as
`dependency_denials` and allowlist entries in
`tools/boundary-check/config/road1.toml`; band rules alone do not cover crates
outside `worth-contracts`. The ruling has three parts:

- **`worth-query-host`** may depend on `worth-store-application`, and only its
  production composition module may name it. No other `worth-query*` crate may
  depend on any `worth-store*` crate.
- **`worth-store-application`** may depend on the owner crates whose ports it
  implements: Relational, World, Signal, Bridge, and the Query crates that
  define Query-owned artifacts. It must never depend on `worth-query-host`.
- **`worth-store-testkit-application-oracle`** (D14) is the fixture/reference
  library and may depend on `worth-query-host` and `worth-store-application`.
  It has no tests or integration target. `worth-store-court-application` is
  the canonical standard-harness leaf containing assertions and child-process
  reexecution. No production crate may depend on either package.

**Build cost.** From now on, the Store workspace builds the Query, World and
Signal stacks for its application crates, and the host builds the Store
stack. Phase 3 records the measured build-time impact on every lane in its
evidence. Store lane partitions keep the physical-only suites off the
application crates.

**D2. Owners define artifacts and ports; Store implements them.**

Each owner defines, in its own crate:

- its durable artifacts: frame, version, digest and readmission;
- a durable medium port with the Milestone 2 answers `Committed`,
  `ProvenNoEffect` and `Indeterminate`;
- a recovery source it readmits from.

Store never decodes an owner payload. `worth-store-application` implements
every port over named journals and anchored records. Owner ports are plain
traits with no filesystem or Store dependency, so Signal still compiles for
`wasm32`. An owner's readmission is its only door from bytes to authority.

**D3. Store capacity: root schema 12 and a journal registry.**

- **Anchors.** Root manifest schema 12 widens the inline anchor table to 32
  anchors. The payload is 3,760 bytes, within the smallest page.
- **Unchanged.** The WAL advance codec, recovery projection and successor-root
  carry-forward rules do not change.
- **Journals.** The home crate declares the ordered-journal registry as a
  closed, versioned set of names. Each journal carries its own unresolved
  state.
- **Clean cutover.** Schema 11 roots refuse at open with
  `UnsupportedRootSchema`, and nothing reads them. No production home exists to
  preserve.

**D4. Owners publish in order; product movement is its own durable step.**

No root decision moves more than one anchor. A composite operation publishes
each owner's effect as that owner's own durable step, in World's fixed owner
order, durable before live (Milestone 2 D2). Product movement is World's
publication of a `RuntimeWorldOccurrenceArtifact` to `world.occurrences`. The
live product compare-and-swap happens only after that publication answers
`Committed`.

- `ProvenNoEffect` on product movement leaves the owner effects as
  `ProductUnpublished` custody, with their existing next actions.
- `Indeterminate` returns `ProductUnpublished(ProductPublicationUnresolved)`.
  Store's unresolved state is per journal and `world.occurrences` is one
  journal, so this blocks product movement on every branch of the home until
  reopen. Owner effects that do not move a product remain admissible.
  Recovery decides the fate.

No competing transaction coordinator, idempotency table or recovery authority
is added. World stays the composite coordinator, and its existing catalogs gain
durable backing.

**D5. Owner effects carry their attempt association; custody dispositions are
durable.**

Before Native candidate preparation seals an envelope, World records an
`OpenAttempt` in its existing `world.custody` journal. It retains the original,
independently issued attempt, composite-commit and custody identities, exact
expected product head, branch/incarnation, provenance, program revision and
component intent. Only actual commitment of that open permits owner effects;
no-effect or indeterminate opening starts none. The existing preparing-operation
guard and reserved resources move through this progression once. Native consumes
the original validated proposal once, binding its opaque association before the
existing single preparation algorithm seals the candidate.

After the open commits, preparation refusal, cancellation or unwind retains or
terminalizes that same durable open; it cannot erase the record, claim total
World no-effect, reissue its identities or replay a handler. Completion verifies
the sealed candidate association against the original open and exact branch and
incarnation before effects. Recovery resumes issuer frontiers beyond every
durably recorded identity, including unfinished opens.

Every owner effect published inside a composite attempt records the attempt's
World-defined `CompositeAttemptAssociation`: the home-scoped attempt identity,
product branch identity and incarnation. It sits in the owner's own artifact:

- a Relational commit carries it in a Relational-owned, opaque, size-bounded
  caller-association field. The canonical envelope also records the original
  durable semantic result defined in Phase 4. Both are sealed during original
  preparation, under canonical artifact v3 and native checkpoint CHK16;
- a Signal advance carries it in its advance artifact (Phase 5);
- later owners follow the same pattern.

Relational's existing commit index carries required descriptive association
discovery metadata derived from that same envelope: `Unassociated` or
`Associated(original opaque association)`. An absent field is malformed
current-format data, never `Unassociated`. This selects subjects for actual
readmission; it grants no publication, settlement, root or recovery authority.

Recovery joins owner effects to occurrences by association. An owner effect
whose association has neither an occurrence nor a disposition is recovered as
durable `ProductUnpublished` custody with cause
`RecoveredWithoutProductMovement`.

Its identity bundle and predecessor come from the original open. A purported
World association with missing or inconsistent original-open evidence refuses
recovery; it never manufactures the missing identities. An open alone does not
prove that an owner effect happened.

Continuing or releasing custody publishes a `RuntimeWorldCustodyDisposition`
to `world.custody` before its live effect. Released custody therefore never
reappears on reopen. Open custody is carried in World's checkpoint section
(D12). Nothing is folded silently into a new root or re-executed.

For Phases 3–4, abandoning a live directory preparation retains its actual
Query/World custody and answers `ReopenRequired`; it does not grant live
no-effect release. Completed Native recovery and complete associated-effect
reconciliation establish the recovered fate. The exact authority, next-action
and production-court contract is in
[Shared durable owner contracts](milestone-3-durable-owner-contracts.md#live-abandonment-and-recovered-disposition).

**D6. Durable, home-scoped identities.**

Every identity that a durable artifact or durable row records becomes
home-scoped and runtime-neutral. For World these are owner, branch,
incarnation, reference generation, composite commit, publication attempt,
bootstrap attempt and `ProductUnpublishedOwnerEffects`. The equivalent Signal,
Bridge and Query identities follow in their phases.

- **Recording.** The home descriptor records a home identity at
  initialization.
- **Frontiers.** Issuers resume one past the largest durable value (Milestone 2
  D7) and issue deterministically in order.
- **Process-local identities.** `runtime_instance_id`, session ids and
  graph-instance ids remain only for values no durable artifact or row
  records. The Phase 1 audit lists every durable row that records one today.
  The phase that makes the row's family durable replaces that field with
  home-scoped identity.

**D7. Single writer and incarnation fencing.**

- **Writable opens.** Every open is writable; this milestone provides no
  read-only open.
- **Lease.** Store's OS mutation lease (`MutationOwnershipLease`) is held
  continuously from the start of recovery through serving. The handoff
  transfers the held lease; it never drops and reacquires it.
- **Incarnation.** Each open durably advances the home incarnation in the home
  descriptor before service. Values that carry authority are bound to that
  incarnation: capabilities, handles, sessions, leases and recovery claims. A
  value from an earlier incarnation refuses with `StaleIncarnation` before any
  effect.
- **Durable data is not fenced.** Receipts, occurrence identities, commit
  identities and history cursors are resolved by home identity and affinity,
  and remain valid across reopen.
- **New homes.** Initialization happens under the lease. A location is new
  only when the lease holder observes no Store at all. A Store left partly
  initialized is completed or refused (Milestone 2 D5), never treated as an
  empty home.

**D8. Recovery order is Store-sequenced.**

Open runs these steps in order:

1. Store physical recovery.
2. The home descriptor, then the incarnation advance.
3. Relational readmission.
4. Signal and Bridge readmission.
5. World readmission and reconciliation (D5).
6. Query readmission: idempotency, programs, workflows and custody.
7. Dispatch and correction custody.
8. Completion of a pending program transition (D16), or the transition the
   open request carries.
9. Service.

Each owner readmits only its own artifacts, after the owners it depends on.
Any refusal is typed and happens before service. Steps 1–7 are recovery: they
never republish, re-execute a handler, or issue a replacement commit or
occurrence. Step 8 is not recovery. It is the transition's own forward
progress, run against fully readmitted owners before service, under D16's
plan-digest check.

**D9. Idempotency and receipts resolve from durable state.**

Query's application idempotency binding is recorded in the same Relational
commit as the effect, as it is today. It now also records product branch
affinity (identity and incarnation) and the attempt association. A retry is
answered by reading the durable binding, the World occurrence or custody, and
the owner artifacts. The original receipt is rebuilt from those, field by
field, as Phase 4's receipt table defines.

- **Evidence store.** `WorthQueryCompletedCommitEvidenceStore` stops being an
  authority. It becomes a derived in-process index:
  - it is filled at commit and rebuilt on demand from durable state after
    reopen;
  - it is never the only source of an answer;
  - its session index stays process-local and incarnation-fenced;
  - its dispatch-correlation index is rebuilt from durable outbox rows.
- **Deleted.** `CommittedReceiptNotRetained` is deleted, and the
  retained-receipt-basis cap no longer bounds answers. `worth-server`'s
  in-memory retry caches are deleted; Server retries go through Query's
  contract.
- **Affinity check.** The `runtime_instance_id` equality in resolution is
  replaced by home identity plus affinity.
- **Concurrency.** Same-key concurrent retries are serialized by the existing
  branch commit lane under the single writer (D7).

**D10. Retention classes.**

Every durable artifact belongs to exactly one class:

- **Authoritative.** Never expires while its home exists.
- **Mandatory obligation.** Retained until its owner discharges it.
- **Optional retained material.** May expire under a declared policy. Expiry
  answers a typed `RetainedMaterialExpired` and never authorizes re-execution.
- **Derived.** Rebuildable, and never authoritative.

Idempotency bindings, occurrences and custody dispositions are authoritative.
Reclamation is Milestone 4; this milestone declares the classes and never
reclaims authoritative or obligation data.

**D11. Close.**

`close` stops admission, then settles or durably retains accepted work within
its request deadline. It answers `Closed` or `CloseRefused { posture }`.

- On refusal, the home stays open and serving, or it enters an explicit
  recovery posture the caller can inspect. It never silently seals a runtime
  the caller was told remains open.
- Acknowledged results never depend on close; durability is reached at
  publication.
- Close attempts a home checkpoint before releasing the lease (D12). A failed
  checkpoint never refuses close. It is reported in `Closed`, and the next open
  replays the longer tail.

**D12. One home checkpoint, taken automatically.**

A home checkpoint is one manifest under the `home.checkpoint` anchor,
published as one root decision.

- **Sections.** The manifest has one section per owner. Each section uses the
  owner's own checkpoint format, with content-addressed members as in
  Milestone 2 D4. The Relational manifest becomes the Relational section
  using its current canonical format and declared clean version cutover.
- **Coverage and crashes.** Each section covers a position of its owner's
  journals, and recovery replays only the tails. A crash at any point of a
  checkpoint leaves either the previous or the new checkpoint selected.
- **Exact Native component obligations.** Capture selects current branch cells
  and live owner-issued `ExternalComponentBasis` obligations at one coherent
  owner capture boundary. It retains their actual roots, schema authorities
  and source custody through checkpoint selection. It does not inherit the
  previous checkpoint's entire physical union or retain unrelated observations,
  transactions or prepared candidates merely because they share a root.
  Acquiring or releasing an external obligation concurrently with capture has
  an owner-linearized result. A selected obligation remains protected after
  its original lease is released. A later checkpoint may omit it when no
  current cell or live external obligation requires it.
- **Cross-owner capture cut.** Before Native section capture, World's existing
  checkpoint owner freezes its exact root/open-custody snapshot and the actual
  external Native component leases needed by that snapshot. Retain that same
  snapshot and leases through Home manifest selection; encode the World section
  from this cut. Later admitted World publications remain journal tails. No
  owner mutex spans I/O, and a Native header snapshot alone never certifies a
  coherent Home cut or permits World to add an uncaptured historical reference.
  In the after-effect/caller-loss window, the cut refuses while an attempt is
  executing or loaned without World's actual successor component retention.
  Settle the existing retained handoff and acquire that real obligation before
  capture; artifact bytes cannot substitute for missing root custody. Select
  the Home checkpoint while Native is still Serving, then release capture pins
  before Native `prepare_close`; never bypass its genuine read-custody refusal.
- **No partial Home manifest.** Until World's checkpoint section is implemented,
  an attached or unclaimed Home with actual committed World journal heads
  refuses checkpoint with `WorldSectionRequired` before Native capture or
  manifest publication. Absence of heads grants no World reconciliation proof.
  Close reports this failed checkpoint and follows D11, retaining journal tails.
- **Triggers.** The host checkpoints automatically:
  - when any journal's tail since the last checkpoint passes a threshold
    (count and bytes, with defaults in the host limits);
  - at close.

  Applications may also checkpoint explicitly. Reopen work is proportional to
  the manifest plus the bounded tails, not to history.

**D13. One durable path; the memory-only mode stays explicit.**

The host has one open call, with two locations:

- **A directory.** Durable.
- **Memory-only.** Explicitly non-durable, with unchanged semantics.
  Memory-only evidence never qualifies durability.

Application-owned checkpoint transport is retired by clean cutover in Phase
10. That covers:

- `capture_application_checkpoint*`;
- `WorthQueryApplicationCheckpoint` byte transport;
- the `*_from_checkpoint*` constructors;
- `repair_to_checkpoint*`;
- Relational's embedded-checkpoint placement.

Until then it is a named interim path for callers whose state families are
not yet durable.

- **One door.** Phase 3 collapses it into a single door,
  `WorthQueryInterimCheckpointRestore`, taking the new home limits. Nothing in
  the interim path keeps an `in_memory` name or `InMemory` type.
- **Frozen callers.** Its callers are frozen in
  `tools/boundary-check/config/interim-checkpoint-callers.txt`. A new caller
  fails boundary-check, and callers are only ever removed.
Checkpoint-transition and program-adoption contracts move onto the open home
(D16, Phase 6). The roadmap's former "explicit embedded-save" mode is retired
with this path.

**D14. Interruption controls and the host oracle.**

- **Store faults.** Store faults and pause gates use Store's
  `certification-test-authority` feature. `worth-store-application` forwards
  it, as `worth-store-relational` does, inside Store's one-build exemption.
  No feature is declared outside the Store workspace.
- **Owner boundaries.** Every owner effect, custody disposition, transition
  step and product movement is a Store publication, so Store's mutation seams
  interrupt every owner boundary.
- **Oracle location.** The host oracle lives in
  `worth-store-court-application`, using the fixture/reference library
  `worth-store-testkit-application-oracle`. Court children re-execute the test binary,
  as Milestone 2's relational_home children do, and run the real production
  host.
- **Derived-state door.** Discarding derived state uses a sealed certification
  door on the home (`discard_derived_for_test`). Its callers are recorded in
  `control-callers.txt`.
- **Owner unit courts.** Owners outside Store use `cfg(test)` in-crate media.
  Those are not a second durable path.
- **The oracle's identity rule.** The reference is the ordinary MemoryOnly
  host, issuing normal ephemeral identities without a durable binding,
  Home-scoped identity, frontier snapshot or reset. One typed run-wide
  `DurableIdentityCorrespondence` bijection relates Home durable identities
  to reference identities across every reopen. Its `admit(home, reference)`
  refuses either a Home identity mapped to a second reference identity or a
  reference identity reached from a second Home identity. Admit identities
  only when they first appear in compared durable graph state, occurrences,
  custody, product heads or idempotency outcomes. Process-local identities and
  identities consumed by attempts that never became durable stay out.
  Identity meaning is compared by correspondence, which needs no durable
  reference. The separate required test that identities issued after reopen
  never collide with recovered ones remains unchanged.
- **Seeds and lanes.** A failing seed reproduces deterministically. Focused
  lanes run a small seed range. A scheduled lane runs the broad campaign under
  a stated time and memory budget.

**D15. Typed unsupported families, for directory homes.**

A `DurableStateFamily` enumeration names every durable family from the Phase 1
audit. Each family maps to the operation kinds and declared capabilities that
can create it. The mapping includes capabilities that create the family only
on failure, such as managed resources, which can leave managed cleanup.

Until the phase that supports a family lands, on a directory home:

- an operation whose kind or declared capabilities can create the family
  refuses at operation admission with `UnsupportedDurableFamily { family }`;
- opening a directory home that holds the family refuses before service.

Memory-only homes are never refused. Each phase's "Interim refusals" lists
exactly which operation kinds refuse until it lands. It also states, for each
existing caller (bank world, Server, examples, fixtures), which
location that caller uses meanwhile. Phase 10 deletes every variant.

**D16. Program transitions are durable and resumable.**

Opening a home with a program or schema revision different from the one
recorded requires a transition. The open request carries it as an optional
part: predecessor, transition author, resources and dispositions. Opening
without a required transition refuses with `TransitionRequired`, naming the
recorded and requested revisions.

A transition proceeds in this order:

1. **Plan.** The transition author computes its complete step plan against
   the readmitted predecessor state.
2. **Intent.** Publish a `HomeProgramTransitionIntent` to `home.transitions`.
   It names the predecessor, the successor, the transition identity and the
   digest of the complete step plan.
3. **Steps.** Publish the migration steps: Relational schema transitions,
   commits and dispositions. Each step is positioned, and each carries the
   transition association.
4. **Binding.** Publish the descriptor's program binding.

At open step 8 (D8), an intent without a binding is pending. Open recomputes
the plan from the same predecessor state and checks it against the recorded
digest:

- **The plan matches.** The transition resumes from the first step that is not
  durable. Durable steps are never re-run.
- **The plan differs.** Open refuses with `TransitionPlanDrift`. A developer
  who edited the migration after a crash cannot mix two migrations.
- **The successor differs.** Opening with a different successor refuses with
  `TransitionPending`.

A pending transition can only be finished with its original plan, or
discarded with the home.

The supported development workflow is:

- **Edit and reopen.** After editing a program or schema, reopen with the
  transition.
- **Discard.** Discard the home through the host's `discard_home(location)`.
  - It does not open the Store. It takes the filesystem mutation lease, so an
    open home cannot be discarded.
  - It first writes a discard tombstone at the location, then removes the
    Store.
  - It is idempotent. It completes a partly discarded or damaged location.
  - Opening a tombstoned location refuses with `DiscardPending` until
    `discard_home` completes.

**Cleanup track: moved courts.** The following transition-source recovery
courts leave the happy-path tree under the M3 track split. Restore and migrate
them against real Store Home/World authority; do not ignore or gate them.

- `transition_source_recovers_the_exact_durable_prefix_and_original_planning_root`: Recovers the exact durable transition prefix and original planning root, keeps that historical root distinct from the current execution basis, and readmits successor schema version 2.
  `restore from 34edd18ff6:crates/worth-court-relational-certification/tests/worth_court_relational_certification/process/home_program_transition_source.rs`. Migration needs catalog-grant access before
  cutover and a replacement for the expected-zero prefix scenario, because
  zero-step plans are prohibited.
- `transition_source_reads_its_planning_predecessor_from_the_selected_admission_image`: Reads the original planning predecessor from the selected admission image, preserving one predecessor row and two current rows after the durable migration tail.
  `restore from 34edd18ff6:crates/worth-court-relational-certification/tests/worth_court_relational_certification/process/home_program_transition_source.rs`. Migration needs catalog-grant access before
  cutover and a replacement for the expected-zero prefix scenario, because
  zero-step plans are prohibited.
- `transition_source_refuses_a_missing_or_foreign_prefix_before_runtime_cutover`: Refuses a missing or foreign prefix with a typed refusal before runtime cutover; the receiver remains empty.
  `restore from 34edd18ff6:crates/worth-court-relational-certification/tests/worth_court_relational_certification/process/home_program_transition_source.rs`. Migration needs catalog-grant access before
  cutover and a replacement for the expected-zero prefix scenario, because
  zero-step plans are prohibited.

**D17. Type-enforced invariants, placed by the vocabulary rule.**

The decisions above are invariants. Each one is enforced by the compiler
wherever the arch laws require it (laws 3, 5, 9, 10, 13 and 16). A prose rule,
a review finding or a test is enough only where this decision says so.
Implementers are mostly AI agents. For them, a compile error at the edit site
is reliable. An invariant stated 1,400 lines away in a specification is not.

*Placement.* Before a new type is designed, it is placed by the AGENTS.md
rule:

- **`worth-proof`** holds anything that decides whether an operation is
  legal: witnesses, phase proofs and fenced authority.
- **`worth-foundational`** holds anything that means the same thing when it
  crosses a boundary into another runtime: identities, associations, fates,
  classes and families.
- **The owning runtime** holds anything that needs a clock, a counter, a live
  table or `Drop`: issuers, the lease, the incarnation advance and custody
  registries.

New vocabulary reuses what these crates already provide:

- `worth-proof`'s `AuthorityWitness`, `CapabilityWitness` and minting
  patterns;
- `linear::LinearResource`, for values that must end exactly once;
- `phase` markers;
- `brand`, only within a scope. Its documented limit applies: a long-lived
  carrier uses a runtime-owned identity value instead.
- `worth-foundational`'s `boundary_protocol` versioning and
  unsupported-version refusal, for every artifact frame (law 21).

The implementer verifies that the crate dependency exists before concluding
that vocabulary is missing. No parallel vocabulary is introduced.

*Invariants and their mechanisms.* The table names the decision each
invariant comes from.

| Invariant | Mechanism | Placement |
|---|---|---|
| Durable artifacts record only durable identity (D6, R17) | A sealed `DurableIdentity` marker trait. Artifact codecs accept only fields whose types implement it. Process-local identities (runtime instance, session, graph instance, participation pointer, installed-runtime ordinal) never implement it, so recording one does not compile. | Trait and home-scoped identity values: `worth-foundational`. Issuers: owning runtimes. |
| Durable before live (D2, D4) | The live cutover and the live product compare-and-swap each take a `DurablyCommitted` witness, move-only and consumed by the cutover. Only the owner's medium binding mints it, and only from a `Committed` answer. `ProvenNoEffect` and `Indeterminate` mint nothing. | Witness type: `worth-proof`. Minting: owner runtime. |
| Publication fates (D2) | One owner-neutral `DurablePublicationFate`: `Committed`, `ProvenNoEffect`, `Indeterminate`. Every owner port answers with it, and it never collapses to `Result` or a boolean. Owner-specific detail rides in typed payloads. M2's `RelationalDurablePublication` adopts it. | `worth-foundational`. |
| Recovery order (D8) | Typestate with plain, named proof structs, one per step: `StoreRecovered`, `HomeAdmitted`, `RelationalReadmitted`, `SignalReadmitted`, `WorldReconciled`, `QueryReadmitted`, `CustodyReadmitted`, `TransitionSettled`. Each step consumes its predecessor's proof. Service is constructible only from the last one. Adding an owner breaks compilation at every composition site until it is wired in (law 9). | Proof structs: `worth-proof` phase markers. Sequencing: `worth-store-application`. |
| Fencing (D7) | Authority values and durable data are distinct sealed types. Authority values are capabilities, handles, sessions, leases and recovery claims. They carry the home incarnation and are never `DurableIdentity`. Durable data is receipts, occurrence and commit identities, and cursors. It carries no incarnation and is never authority. The cross-process check stays a run-time comparison, because old in-memory values die with their process. | `HomeIncarnation` value: `worth-foundational`. Fenced-authority shape: `worth-proof`. Advance: Store home. |
| Receipts from durable state only (D9) | The original-receipt constructor takes only readmitted durable inputs: the binding, the occurrence and the owner artifacts. Building a receipt from the evidence index is uncallable. | Query. |
| Unsupported families (D15) | One declaration: an exhaustive `match` over `DurableStateFamily` returning `Supported` or `RefusedUntil(phase)`, with no wildcard arm. Operations declare their capabilities as types. Admission computes refusal from those declarations, never from hand-written checks at call sites. | `DurableStateFamily`: `worth-foundational`. Support table: `worth-query-host` composition. |
| Retention class (D10) | An associated constant on every artifact type, matched exhaustively wherever retention is decided. | `RetentionClass`: `worth-foundational`. |
| Custody that ends exactly once (D5, D11, R11) | Custody dispositions, transition intents, managed-cleanup obligations and pinned correction material are `LinearResource`s. Their terminal transition takes `self`. | `worth-proof` `linear`. Owners implement it. |
| Attempt association (D5) | `CompositeAttemptAssociation` is one `DurableIdentity` value carried by every owner effect. Owners treat it as opaque. | `worth-foundational`. |
| Continuous lease (D7, Phase 2) | The lease owner value moves from recovery into serving. No code path holds neither. | Store. |

*Agent-oriented type style* for all new durability code:

- **Plain named types.** Prefer concrete, named proof and witness structs
  over generic proof towers such as `Proof<Stage<Owner<..>>>`. Keep generics
  shallow, and add no new macros or blanket impls.
- **Exhaustive matching.** Every `match` over a durable family, fate, class,
  retained decision (R9) or recovery step is exhaustive, with no `_` arm, so
  that adding a variant lists every site to update.
- **One meaning per name** (law 10). Names are unique and greppable.
- **Enforcement or a traced test.** Every invariant in this specification is
  either compiler-enforced or covered by a test whose name states the
  behaviour, such as `live_cutover_requires_durable_commit` (iteration law 16:
  no milestone, decision or ticket ids in names). Each phase's requirement map
  traces requirement ids to those tests.
- **Explicit over clever.** Explicit, repetitive code is acceptable. An
  abstraction is acceptable only when it shares semantic authority, lifecycle,
  failure topology and cost class (law 12).

*Phase binding.*

- **Phase 2:** the continuous-lease row.
- **Phases 3 and 4:**
  - placement;
  - `DurableIdentity`;
  - `DurablyCommitted`;
  - `DurablePublicationFate`, including the adoption by Relational's port;
  - the recovery typestate, with the owners that exist by then;
  - fencing types;
  - the receipt constructor;
  - the `DurableStateFamily` declaration;
  - `RetentionClass`;
  - `CompositeAttemptAssociation`;
  - `LinearResource` custody for dispositions and transitions.
- **Phases 5 to 9:** each extends the typestate, the family declaration and
  linear custody for its owners. A phase is not complete while any of its
  invariants is enforced only by prose.

## Anchors and journals

These are the closed registry of D3. A name is final when the phase that
creates it lands.

| Name | Kind | Owner (format) | Class | Phase |
|---|---|---|---|---|
| `home` | Anchored record | Store home: identity, incarnation, program/schema/build binding | Authoritative | 2–3 |
| `home.checkpoint` | Anchored record | Store home manifest; owner sections | Derived | 3 |
| `home.transitions` | Journal | Store home: transition intents (D16) | Authoritative | 3 |
| `relational.canonical-commits` | Journal | Relational (with caller association) | Authoritative | M2, 4 |
| `relational.branch-transitions` | Journal | Relational | Authoritative | M2 |
| `world.occurrences` | Journal | World occurrence | Authoritative | 4 |
| `world.branches` | Journal | World product branch lifecycle | Authoritative | 4 |
| `world.custody` | Journal | World custody dispositions | Authoritative | 4 |
| `signal.definitions` | Journal | Signal definition transitions (with Bridge section) | Authoritative | 5 |
| `signal.advances` | Journal | Signal branch advances | Authoritative | 5 |
| `query.obligations` | Journal | Query: support lifecycle, adoption, required-output custody (R8), managed, conditional, owned-async custody, original-owner custody | Obligation | 6–7, 9 |
| `query.dispatch` | Journal | Query: dispatch attempts, inbound custody, recovery claims | Obligation | 8 |
| `query.corrections` | Journal | Query: retained correction material and custody | Per contract | 9 |

The M2 `relational.home` and `relational.checkpoint` anchors are replaced.
`relational.home` is replaced by `home`, and `relational.checkpoint` by the
Relational section of `home.checkpoint` (D12).

Query's idempotency binding is not a journal. It travels inside the
Relational commit (D9).

---

## Phase 1: State survival audit

**Goal.** Classify every production-owned state family before any of it is
made durable. This gives every later phase an exact scope and every refusal an
exact name.

**Requirements**

- **Inventory.** Cover every state family held by Relational, World, Signal,
  Bridge, Query (installation, execution, publication), Server and the bank
  world. For each one, record:
  - its owner and owning type, with path;
  - its survival class (D10), or that it is process-local;
  - its current persistence: none, a Relational row, or in-memory;
  - the process-local identities it records (D6);
  - the phase that makes it durable. If it stays process-local, record how it
    is reacquired through fresh authority.
- **Unsupported families.** Name the families that become
  `DurableStateFamily` variants. Map each to the operation kinds and
  capabilities that can create it (D15).
- **Durable rows with process-local identities.** List every durable row
  recording `runtime_instance_id` or any other process-local identity, with
  the phase that replaces it. This includes digests derived from such
  identities, such as transport causal digests, workflow receipt digests and
  runtime source commitments. The causality rows' `world-owner` is an
  example.
- **Known defects.** Record each with its target phase:
  - the temporal idempotency key is derived from per-process runtime
    identities;
  - `PayloadRejected` is classified as unresolved;
  - support lifecycle resets to Active at install;
  - only the main branch's program activation is verified on restore.
- **Interim-path inventory.** List every public entry path retired by D9 and
  D13, with its callers. For each caller, give the location it uses in each
  interim phase.

**Deliverable.**

- The index is `plans/database-foundation/milestone-3-survival-audit.md`. It
  holds:
  - the `DurableStateFamily` list and its capability mapping;
  - the table of durable rows that record process-local identities;
  - the defect table;
  - the interim-path table.
- Per-owner inventory parts live in
  `plans/database-foundation/milestone-3-survival-audit/`.

**Required tests.** None; this phase writes no code.

**Exit criteria**

- An independent reviewer samples at least twenty audit rows against the code
  and finds them accurate.
- Every family and every row with a process-local identity that a later phase
  names appears in the audit, and no family is unclassified.

### Audit resolutions

The Phase 1 audit raised eighteen gaps in the specification. Each one is
resolved below and tagged with the phases it binds. A phase's requirements
include every resolution tagged with it.

- **R1 (Phase 1, index).** Part A audits Relational as the carrier of rows.
  The rows themselves are written by callers (idempotency, outbox, workflow,
  causality and others), and Parts B and C audit those. The index joins
  carrier and caller into one family per row kind, named after the caller.
  There is no generic "caller rows" family.
- **R2 (Phase 5).** Signal's authored topology is the declared node
  definitions plus the static dependency edges those definitions declare.
  Everything else is derived and rediscovered by evaluation:
  - dependency snapshots;
  - subscriber and evaluation edges;
  - dynamically discovered dependencies;
  - cause sets;
  - diagnostics.

  The Phase 5 authored-image format states this split before it is final.
- **R3 (Phase 5).** The `worth-signal-wasm` facade's `RuntimeStore` and
  portable envelope are a separate product surface. Neither is opened through
  the Query host, and both are outside this milestone. JS callbacks are never
  serialized. D15 does not apply to the facade.
- **R4 (Phase 5).** Some records are process-local diagnostics, not durable
  state: Bridge's writeback execution, replay and comparison records, and
  Relational's diagnostic artifact vector.
  - Their accessors gain a typed answer that distinguishes three cases: never
    retained, expired under count limits, and not retained across reopen.
  - Any evidence that recovery needs must be derivable from durable commits.
    For writeback, Phase 5 proves this.
  - `SignalSnapshotV1` stops being a restore authority, because the Phase 5
    artifacts replace it. Its restore door is deleted, or stays memory-only
    if the audit shows a memory-only consumer. It is never a second durable
    path.
- **R5 (Phases 5 and 7).** Two Signal structures hold real request custody and
  are not derived: `ResourceRuntimeState.in_flight_by_request` and native
  managed admission.
  - Until Phase 7, operations that admit Signal resource requests refuse on
    directory homes under D15.
  - Phase 7 extracts that custody into a durable form before any remaining
    resource state is treated as derived.
- **R6 (Phases 1 and 8).** The outbox wording is corrected above. Phase 8
  removes the process-local identity from every row and digest that the audit
  lists for it.
- **R7 (Phases 7 and 9).** Some run kinds retain opaque process objects:
  provider checkpoints, memory arenas, artifact contexts and `Arc<dyn Any>`
  commit authority. Serializing such an object is never allowed. Each of
  these run kinds takes one of two routes:
  - an owner-defined, versioned data contract and readmission;
  - a terminal posture that its existing lifecycle contract already defines.

  The phase's first deliverable is a per-kind table naming the route each run
  kind takes.
- **R8 (Phase 6).** General required-output custody is a durable obligation in
  `query.obligations`. That covers `DemandRecord` performed obligations and
  prerequisites, and `SourceCustody`. Ready outputs remain derived.
- **R9 (Phase 7).** Every variant of the conditional retained decision is
  classified:
  - **Durable obligations**, because they carry an effect or an obligation:
    CleanupRequired, Retryable, CommitRetryable, ExecutionControlRetryable,
    ProductStale, ProductUnpublished, Indeterminate and
    InterruptedDuringReentry.
  - **Process-local**, because they can be re-derived from durable state and
    host time: Eligible, Deferred, Suppressed and Backpressured.
  - **Terminal**: ControlStopped, TerminalFailure, Failed and NoEffect. These
    are recorded durably exactly when a caller can still ask for them.

  Phase 7 publishes this table with its evidence.
- **R10 (Phase 7).** Workflow proposal coverage is a set of graph records: it
  is authored, and it recovers with Relational. Installed vocabulary coverage
  is derived and is recreated from the supplied program. An installed
  interpreter is never persisted as authored coverage.
- **R11 (Phase 9).** Each retained use of correction material declares its
  D10 class.
  - While a prepared correction or a recovery binding pins the material, it is
    a mandatory obligation.
  - Unpinned material is optional.

  Pinning, and transfers of a pin, are recorded together with the class.
- **R12 (Phases 4, 5 and 7).** D15's capability mapping splits branch cleanup
  by component. World native cleanup is supported from Phase 4, Signal cleanup
  from Phase 5, and conditional-assembly cleanup from Phase 7. On a directory
  home, a branch operation refuses if its cleanup includes a component that
  is not yet supported.
- **R13 (Phase 8).** Revocation and retirement of an inbound source are
  home-level security facts, and they are durable. Reopening never resets a
  revoked source to Active. Verifier installation is fresh host configuration
  applied on top of that durable posture.
- **R14 (Phase 8).** The bank external rail is the external party. Its own
  survival is outside WORTH, and Phase 8 does not make it durable.
  - Phase 8's rail courts kill the application, not the rail.
  - One court restarts the rail with its state lost. The application must not
    claim completion: the effect stays indeterminate and is reconciled under
    the rail's admitted contract.
- **R15 (Phases 3, 4 and 9).** `BankHttpMutationContinuation` is transport
  custody. It is re-created fresh at open.
  - The durable original owner is the per-operation record in its worker
    records and in `mutation_custody`. Phase 9 makes that record durable.
  - Until then, actions that retain original-owner custody refuse on
    directory homes under D15.
- **R16 (Phases 3 and 9).** Recovery of HTTP state:
  - HTTP recovery and continuation tokens carry authority, so they are fenced
    by incarnation (D7). After reopen they answer `StaleIncarnation`. A fresh
    authentication then reaches the recovered durable custody by its
    operation identity.
- **R17 (Phase 6).** Installation splits its canonical identities into two
  kinds:
  - **Semantic identity** is derived from content and may be durable.
  - **Authority identity** (installed-runtime ordinals and participation
    pointers) is process-local and fenced.

  Durable artifacts record only semantic identity. Equivalence checks compare
  semantic identity, plus the authority re-established at open. They are
  never dropped.
- **R18 (Phase 3).** Server upload staging is process-local. Interrupted
  upload sessions end at reopen with a typed `UploadSessionExpired`. Durable
  ingress is outside this roadmap.

---

## Phase 2: Store capacity, single writer and incarnation

**Goal.** Give Store what an application home needs, before any owner uses it.

**Requirements**

- **Schema 12.** Root schema 12 with 32 anchors (D3):
  - successor-root builders, maintenance and residue publications carry the
    table forward unchanged;
  - schema 11 is refused with `UnsupportedRootSchema`;
  - the offline integrity observer reads schema 12.
- **Journal registry.** The journal registry is declared by the home crate. A
  publication to an unregistered name refuses before any effect.
- **Lease handoff.** The lease is handed off continuously from recovery to
  serving (D7). The drop-and-reacquire window at `physical_store.rs` recovery
  is deleted.
- **Incarnation.** The home incarnation is recorded in the descriptor and
  advanced durably on every open, before service.
- **New-home detection.** Initialization happens under the lease (D7).
  `location_is_unused` stops deciding newness without the lease.
- **Milestone 2's home.** It runs on schema 12, with no other behavioral
  change.

**Required tests**

- **Anchor table:**
  - 32 anchors publish, recover and carry forward through maintenance;
  - a 33rd anchor refuses with `AnchorTableFull`;
  - the WAL recovery projection replays advances of high-index anchors.
- **Refusals before any effect:**
  - a schema 11 root refuses at open;
  - a publication to an unregistered journal refuses.
- **Two processes open the same location concurrently.** Exactly one serves;
  the other refuses with `MutationOwnerContended`. Covered for an empty
  location, an initialized home and a home that needs recovery.
- **Kill and reopen.** A process holding a home is killed. The next open
  recovers, and the incarnation increases by exactly one per open.
- **Lease held through recovery.** An observer process fails to acquire the
  lease at every recovery yieldpoint and at the handoff.
- **Offline integrity observer.** It reads a schema 12 root.

**Oracle extension**

- Milestone 2's relational_home oracle runs once on schema 12: media sweep,
  process kills and recovery interruption.
- New media and process cases cover the incarnation advance at open.

**Docs.** The Store README and the `worth-store-relational` guidance cover the
schema, the lease and the incarnation.

**Exit criteria**

- Landing 1's Tier-2 run is green on its head (see "Verification tiers"),
  with the Store crash oracle run exhaustively.
- Final-gate certification.

---

## Phase 3: Application home and host lifecycle

**Goal.** The production Query host opens, recovers, serves, checkpoints and
closes a file-backed home through one call, with every owner composed by the
home.

**Requirements**

*Home and boundary*

- Create `worth-store-application` and apply the D1 boundary ruling.
- Delete `RelationalStoreHome`. Its tests and the Milestone 2 relational_home
  oracle move to the application home's Relational section in
  `worth-store-court-application`, using
  `worth-store-testkit-application-oracle` for shared authored fixtures.
- Record the measured build-time impact on every lane in the phase evidence
  (D1).

*The open call*

- `worth-query-host` exposes one open call. Its home request carries:
  - the location, either a directory or memory-only;
  - the validated program and support roster;
  - the application build identity;
  - trusted runtime services (time source and World resources);
  - the execution authority;
  - limits, including the checkpoint thresholds (D12);
  - `initial_state`;
  - an optional program transition (D16).
- The `in_memory*` constructors are deleted. Their memory-only callers move to
  the open call, and "InMemory" disappears from the limits, profile and denial
  names.
- The `*_from_checkpoint*` constructors collapse into the single
  `WorthQueryInterimCheckpointRestore` door (D13). Its callers are frozen in
  boundary config.
- A new home is distinguished from an existing one only through Store (D7).
  `initial_state` runs only for a new home.
- `discard_home(location)` (D16).

*Program and build binding*

- **Descriptor.** The home descriptor records the program identity and
  revision, the rostered revisions, and the schema revision.
- **Program transitions** follow D16. An incompatible program refuses before
  service.
- **Changed build identity.** Opening under a changed build identity is
  admitted. Authored truth is unchanged. Recorded derived outputs whose
  producing implementation differs are stale under the existing lineage
  contract, and nothing regenerates eagerly. Every rebuild mints a new build
  identity, so this is the ordinary local-iteration path.

*Close, fencing and checkpoints*

- Close follows D11; incarnation fencing follows D7; automatic checkpoints
  follow D12.

**Interim refusals (Phases 3–4).** On a directory home, these refuse per D15:

- operations that touch Signal or Bridge state;
- conditional, temporal and managed operations;
- workflow runs;
- dispatch and inbound completion;
- corrections;
- program adoption.

**Unchanged initial components for the Relational-only lane.** A directory
Home may compose an owner-proven initial Empty Signal root and an owner-proven
Bridge baseline without enabling any Signal/Bridge mutation. Arbitrary authored
Signal graphs, even if an operation would leave them unchanged, remain
unsupported until Phase 5. Refuse an unsupported seed before the first durable
World bootstrap or owner effect; successful live bootstrap cannot create a
Home whose required component meaning is already unrecoverable.

Signal owns a versioned `RecordedSignalInitialEmptyRoot` description and its
admission. The sole `SignalOwnerPartition` to `SignalOwner` transfer can mint
the private initial-root witness only after inspecting actual state: one
original root, initial generation, no restore/snapshot/fork/retirement/merge
history, no allocated/authored nodes or operators, no authored values or
installed definition, and no accepted Signal movement. The witness remains
bound to that exact owner/cell/incarnation and actual initial semantics.
Existing owner admission and cell access guard later export/readmission;
closed, replaced or advanced cells refuse. Do not add an operation counter,
second registry or a caller-supplied emptiness flag. The folded `definition_basis`
alone does not prove any of these facts.

The recorded description has a required owner schema version and InitialEmpty
tag with its owner-defined stable schema/visibility/definition commitments.
Its meaning is the completely unmodified initial Empty root, not an old
process's admitted descriptor or pointer. On reopen, Signal verifies this
recorded contract against the actual newly composed initial owner and mints a
fresh runtime-local admitted basis with real retention. Neither World nor
Store rewrites an old graph/runtime ordinal or substitutes an arbitrary
current basis. Data is descriptive until this owner admission succeeds.

Bridge owns a versioned `RecordedRuntimeWorldBaseline` description and
readmission through its existing correspondence admission owner. Export
requires the actual admitted baseline with `dependency=None`, no installed
correspondence meaning, and the actual registered Native source. Record the
stable adapter semantic identity and baseline role/semantic commitments;
process runtime keys and admission identities are not durable authority.
Readmission requires the same private Store-composed Native owner, its actual
registered source/profile, and Signal's newly owner-admitted InitialEmpty
basis. Bridge compares stable meaning and issues fresh Bridge admission using
those actual owners. A missing source, changed adapter/role, installed
dependency, unsupported version or stale/foreign live owner refuses. There is
no public raw Native/Bridge exposure, graph-ID rewrite, generic restore trait
or caller-provided source/fate pairing.

World records both owner descriptions in its original Branch artifact, bound
to original product identity, program revision and exact Native predecessor.
Use World artifact v3 with mandatory seed fields; earlier v1/v2 development
frames refuse before effects, with no defaults or migration. Store privately
sequences Native recovery, Signal seed admission, Bridge baseline admission,
then World reconciliation. Original World identities, heads, attempts and
custody are unchanged. General nonempty Signal/Bridge authored recovery
remains Phase 5's owner artifact contract and does not reuse this Empty proof.

Place Signal meaning/export at existing `branch/recorded_initial_root.rs`
and owner witness/admission beneath `branch/owner_services/`; expose only its
branch facade. Place Bridge recorded baseline beneath existing
`correspondence/runtime_world_admission/`; expose its existing facade. World
seed encoding/readmission stays under its `durability/` owner; filesystem
composition stays in Store's existing `world_section/` stage. Create only
populated responsibilities, with no generic seed registry or placeholder tree.

The real kill/reopen Home journey uses an actually Empty Signal program. It
must recover the same World head and original unpublished Native custody with
new owner-admitted local component bases, release or continue, then reopen
again without handler execution or effect republication. Nonempty and
modified-then-empty Signal worlds, snapshots/forks, installed Bridge
dependencies, altered adapter/role, missing seed fields and foreign live
owners must fail before effects for the intended reason. Actual seed bases
must retain/release owners honestly under Home close. Owner tests and public
construction fences support, but do not replace, this filesystem proof.
Signal/Bridge READMEs, capability index and the Home build guide describe the
Empty-only supported lane and typed refusal of authored state until Phase 5.

Meanwhile, the bank world, Server and Signal-dependent examples
use memory-only homes or the interim checkpoint path, as the audit lists.

**Required tests**

- **Open and reopen:**
  - opening an empty directory initializes the home and serves;
  - reopening recovers;
  - a damaged descriptor, a foreign home, an unknown root schema and an
    incomplete recovery each refuse before service, with typed reasons.
- **Competing opens at host level.** Two processes open one directory through
  the host. One serves; the other gets a typed contention refusal.
- **Rebuild.** Same program and schema, new build UUID: the home opens with no
  refusal, and stale outputs are reported without being regenerated.
- **Program transitions:**
  - a schema edit without a transition refuses with `TransitionRequired`;
  - with a transition, it migrates and serves;
  - a process killed at each seam of the transition (intent, each step,
    binding) reopens and resumes from the first undurable step, without
    re-running a durable one;
  - a resume whose recomputed plan differs from the recorded digest refuses
    with `TransitionPlanDrift`;
  - opening with a different successor while a transition is pending refuses
    with `TransitionPending`;
  - an unrostered or incompatible program refuses.
- **Discard:**
  - `discard_home` on an open home refuses;
  - on a closed home it removes the home, and the next open initializes;
  - a process killed during discard leaves a tombstoned location;
  - opening that location refuses with `DiscardPending`;
  - running `discard_home` again completes, including on a damaged Store.
- **Frozen interim callers.** Adding a caller of the interim checkpoint door
  fails boundary-check.
- **Fencing and durable data.** A capability, handle and session from
  incarnation N refuse with `StaleIncarnation` at N+1. A receipt and an
  occurrence identity from N still resolve at N+1.
- **Close:**
  - with accepted work in flight, close either settles it or refuses with an
    inspectable posture;
  - a refused close leaves a serving home;
  - acknowledged commits survive a kill without close.
- **Checkpoints.** After more commits than the threshold, a checkpoint exists.
  The reopen journal tail stays at or below the threshold, measured through
  the home's read counters with headroom.
- **Memory-only.** Memory-only mode passes its existing semantic suites through
  the new call.

**Oracle extension.** `worth-store-court-application`, using the shared
`worth-store-testkit-application-oracle` library, covers home open,
initialization, reopen, program transition, automatic checkpoint and close.
That includes the media sweep and process kills at every Store seam of each,
and recovery interruption at every stage.

**Docs**

- The host README sections on open, close, checkpoints and transitions.
- `build-an-application.md` §4 "Install or reopen", including the development
  workflow (D16).

**Exit criteria.** This phase lands with Phase 4, under Phase 4's exit
criteria.

---

## Phase 4: Durable occurrences, interrupted publication and original-result retry

**Goal.** The decisive journey: a lost response after a durable commit, a
kill, a reopen and a retry return the original occurrence, without
re-executing anything.

**Requirements**

*World becomes durable*

- World defines and readmits these artifacts:
  - `RuntimeWorldOccurrenceArtifact`. It records:
    - the composite commit identity and parent;
    - the home identity and the publishing home incarnation;
    - the product branch identity, incarnation and generation;
    - the attempt identity and provenance;
    - the program revision, populated from this phase on;
    - caller correlation;
    - an extensible, owner-tagged list of component evidence.

    Phase 4 defines the Relational entry: commit id, position and digest.
    Later phases add tags, never fields.
  - `RuntimeWorldProductBranchArtifact`: create, retire and incarnation.
  - `RuntimeWorldCustodyDisposition`: continue or release (D5).
- World's identities become home-scoped (D6). The process-static owner
  ordinal no longer reaches any artifact or row.
- Product movement is durable before live (D4).
- On reopen, World rebuilds its history catalog, product heads, ancestry and
  open custody from its checkpoint section plus its journal tails.
  Pre-crash history pages, ancestry tracing and `recover_performed` work
  after reopen.
- Reconciliation follows D5. Pending settlement is covered: a Relational
  commit that is durable but unsettled is recovered by Relational, and World
  holds its custody. `continue_effects`, `release_effects` and
  `prepare_settled_relational_adoption` operate on recovered custody.
- Milestone 2's unresolved forks retained as `ProductUnpublished` survive
  reopen as the same custody, with their native fate decided by recovery.

*Relational association*

- **Original occurrence backing.** Before Native effects, reserve the actual
  occurrence frame's charged backing/capacity through existing
  `ExecutionByteBuffer` under the original execution/allocation policy. Retain
  that same buffer in the original `ActiveAttemptResources` beside its durable
  OpenAttempt and independently issued World identities. Candidate-known
  receipt/root data may inform admission; Native's actual position and artifact
  digest are unavailable until publication and must never be predicted.
  After actual Native settlement, fill and seal that same buffer from the
  original positioned result and retained original successor basis using
  Native's descriptive retained-component export. A later Native head advance
  cannot rebind the occurrence. Do not replace stopped backing, request fresh
  grants to revive it, issue replacement World identities, or substitute the
  latest Native basis. Refusal/unwind retains original unpublished custody and
  actual owner evidence. Publish the World occurrence durably before live CAS;
  recovery uses distinct owner capability adoption, never a fabricated live
  `CommitResult`.

- The canonical envelope gains caller association and the original semantic
  result under artifact v3; both authoritative and embedded checkpoint
  projections carry them. CHK16 requires association discovery in its existing
  commit directory. Earlier formats refuse before effects (clean cutover).

**Native semantic result and recovery admission**

The adversarial case is a Native effect that advanced its branch but never
moved the product. After a kill, World must keep its original predecessor
product basis while recovering that exact effect. Substituting Native's latest
basis, fabricating a live result, or loading all historical bodies to discover
associations fails this contract.

One immutable, Native-owned `RelationalCommitSemanticResult` is assembled
during original preparation, before envelope sealing. The same semantic value
feeds live result assembly and the authoritative recorded image; live-only
fields stay separate. Do not serialize `CommitResult`, `CommitLog` or
`CommitSummary` wholesale, or introduce another result policy or ledger.

The existing producer already has the needed data. `AssembledCommitExecution`
owns structural summary, invariant executions, version, history and create
bindings. `prepare_commit_publication_execution` combines these with the
admitted log's semantic summaries and canonicalized changed records, then
installs the semantic value before the existing envelope `Arc::new` seal.
`CommitResultMaterial` and `CommitResultSeal` consume that same value for the
live result. Source-backed recovery exposes a distinct
`RecoveredRelationalSemanticResult`, not a live seal or result handle.

The durable projection has this explicit field boundary:

| Original source | Recorded semantic meaning | Excluded live or diagnostic data |
|---|---|---|
| Commit reference and version | Exact original receipt/version, bound to the enclosing envelope | Process-local transaction identity |
| Created entity bindings | Every canonical correspondence: partition, kind, resolved client-key text to assigned `EntityId` | Process-local symbol representation and lookup estimates |
| Created relation bindings | Every canonical correspondence: partition, kind, resolved client-key text, exact endpoints to assigned `RelationId` | Process-local symbol representation and lookup estimates |
| Created endpoint reference | `Existing(EntityId)` or `Created(partition, kind, resolved client-key text)` | Guesses from allocation order, later state or patch contents |
| Prepared changed records | Exact canonical ordered `Vec<RecordRef>` | A later graph scan |
| Structural summary | Invariant groups, commit topology, ordered touched partitions, descriptor semantics version | Bulk entity/relation slots reserved |
| Schema summary | Original transition and descriptor semantics version, consistent with the schema artifact | Current-policy reconstruction |
| History summary | Target branch, requested/effective merge-parent counts, parent/merge-base counts, previous-head presence | Later branch state |
| Change/aspect summaries | Original changed-record/adjacency and entity/relation aspect counts, touched aspects, opaque/zero-aspect structural deltas | Evaluation/emission traces |
| Patch/publication summaries | Actual patch-record count represented once, lineage-event and merge-parent counts | Preparation limit, diagnostic-entry count, snapshot ID |
| Validation executions and decisions | Original ordered metadata, checks, results and verdict meaning described below | Validator re-execution and process-local proposal identity |
| Validation summary | Read-only projection of the retained original validation meaning | Independent counters or another accounting authority |
| Retention | Authoritative root/member references and real owner pins remain in their existing owners | Timing, visit counts, charges and residency bookkeeping |
| Execution, snapshot and diagnostics | Nothing | Snapshot handles/IDs, timing, complexity, work/CPU/allocation measurements, late interruption and omitted traces |

Validation retains original execution point, observation kind, version/current
version, consumed/applicable groups, declared cost class, disposition, plan
contract and merged-plan presence. Preparation strategy/failure provenance and
proof scope/widening causes may be retained as historical categories, never as
current execution authority. Packet/traversal bookkeeping is excluded. Checks
retain execution point, failure effect, rule, canonical witness, groups, declared
cost class, custom provenance and complete verdict. Decisions retain those
meanings and their original decision. Declared cost classes describe the original
validation contract; they are not measured work. Process-local
`proposal_identity` is excluded. Numerical verdict output keeps its original
semantic value.

Canonical reference text comes from the original admitted symbol dictionary,
never display formatting of an unresolved symbol. Preserve every original
correspondence's meaning. Exact/stable aliases of one canonical key must agree;
conflicting assignments, missing text or incomplete semantics refuse preparation
before effects. Readmission validates ordering, correspondence consistency,
receipt/version/schema agreement, changed-record/count agreement and validation
result/decision agreement. It does not run handlers or validators.

There is no serialized `Published` flag or pre-recorded completion log event.
Receipt/version can bind before sealing; assigned journal position comes from
actual positioned evidence. Artifact digest is computed after encoding and is
not placed inside its own semantic image. Actual publication or verified recovery
source establishes commitment. Excluded live fields are unavailable after
recovery, never zero-filled or invented.

**Lazy association discovery (D12)**

The existing `RelationalCommitIndexEntry` gains one required, non-optional
`RelationalAssociationDiscovery` value, with variants `Unassociated` and
`Associated(RelationalCallerAssociation)`. It is derived from the actual
envelope when publication becomes indexed or its journal tail is readmitted.
The existing checkpoint commit directory carries it without demanding cold
envelopes. Missing metadata is malformed current-format data; no default can
hide an associated effect. Existing index/envelope checks compare discovery
whenever both are available.

World selects original subjects from this admitted directory and its opens,
occurrences and dispositions. Only selected artifact bodies and the predecessor
and successor roots actually required are demanded. Their source artifact must
match receipt, position, digest and exact association before capability issuance.
Neither discovery metadata nor a memory-exported artifact certifies durable
origin. Reopen cost is proportional to the manifest, bounded tails and demanded
root/artifact material. Unrelated historical bodies stay cold. Ordinary
publication performs no reconstruction scan. This adds no registry, control
publication or accounting framework.

**Exact Native references and recovered capability**

Recorded component references distinguish:

- `Empty`: branch identity, reference generation, truth version, lifecycle,
  schema and visibility commitments; no commit, position, digest or zero receipt.
- `Canonical`: original full receipt, position, artifact digest, root descriptor,
  branch/reference axes, truth version, lifecycle, schema and visibility.

Reuse Native's canonical runtime-frame conversion and
`FoundationalBranchTarget::Empty`/`Basis` meanings. World records its predecessor
before Native preparation; it cannot predict a future Native commit ID.
Descriptions never admit a basis. No path rewrites an old runtime ID or falls
back to the latest root.

**Checkpoint coverage of exact Native components**

Every canonical `RelationalCheckpointRoot` carries required, nonempty
`recorded_components: Vec<RelationalBranchBasisDescriptor>`. These are exact
current-cell or external-component observations in Native's existing canonical
runtime frame. Distinct observations may select the same immutable root;
identical observations are deduplicated and encoded in canonical order. Each
must agree with that root's actual commit descriptor and schema authority.

The manifest separately carries required `retained_empty_components`. Each
entry contains an exact Empty descriptor and `schema_carrier_digest`. Empty
has no invented commit, position, artifact digest or canonical root target.
The referenced schema carrier must exist and match its schema commitment.

The root directory contains exactly current canonical heads plus canonical
roots required by captured external-component obligations. Every current
canonical head is present; every additional root has an exact recorded
component. Missing fields, duplicate or conflicting observations, foreign
frames, target/root/schema disagreement and unbacked extra roots refuse.
Capture obtains external descriptors from the same actual observations that
created their existing retention obligations. Root equality, weak-registry
membership and copied caller descriptions cannot replace this provenance.
Extend existing retention records and capture ownership; add no registry,
control-journal publication or independent authority.

Readmission admits the directory and current heads without loading historical
retained artifact bodies. The existing recovered-root inventory combines
checkpoint component headers, current images and actual tail replay
observations. Historical entries retain exact admitted directory/source
identity and schema commitments; decoding does not make them operational bases.
Exact component admission selects the original header, verifies its complete
source artifact against receipt, position, digest, association and
root/schema/reference commitments, then materializes only the demanded
immutable region directory through existing root readmission. Real observation
and component pins are acquired before exposure. Refusals retain original
evidence and acquired custody. Unselected historical bodies and region payloads
stay cold. Checkpoint coverage neither proves settlement nor authorizes World
adoption.

Capture remains in `durability/authority/checkpoint_capture/`, with exact
obligation selection in existing `history/retention/`. Required headers and
validation remain in `durability/{data,log/native_checkpoint_codec}/`. Lazy
checkpoint/tail inventory and selected materialization belong under existing
`durability/authority/runtime_rebuild/`. Existing recorded-component/publication
admission remains the sole capability producer. Native README and capability
index explain this lifetime, selective admission and version cutover.

The adversarial proof externally retains an exact canonical predecessor,
advances Native without publishing World's occurrence, checkpoints, kills and
reopens. It admits the original predecessor and successor with original
semantics and real pins, releases or continues actual World custody, and
reopens again. Repeat with an externally retained Empty predecessor. Deleting
an unrelated historical body cannot prevent selected admission; missing or
corrupt selected material must refuse. Altered component axes/schema/root/source,
missing headers, latest-head substitution or checkpoint-as-settlement must
fail. Releasing the last external obligation permits a subsequent checkpoint
to omit that historical component. Owner tests support these boundaries but
do not replace the real Home filesystem journey.

Extend the existing `DurabilityRecoveryAuthority` with
`recover_with_authority(execution, plan)`, issuing existing
`RecoveredRelationalRuntimeAuthority` only after actual reconstruction. Extend
that owner with exact recorded-component/publication admission. Its existing
current-image admission remains distinct. Select the exact Journal entry/source,
release owner locks, demand and readmit that source's artifact, and compare all
receipt/position/digest/association/schema/root/reference commitments. Admit the
actual empty or canonical predecessor and successor through existing root/member
owners, acquiring real observation/component pins before exposure. Missing
historical material refuses; it does not justify a synthetic basis.

World may have only its recorded predecessor when a kill precedes occurrence
publication. Existing recovered Native authority therefore exposes
`recorded_publication_successor(execution, runtime, readmitted_effect)`: a
description derived from that exact effect's source/index and this
reconstruction's actual tail/current/checkpoint component image. It returns
Native's existing recorded-reference type, not an operational basis. Exact
original receipt, position, digest, target branch and reference/root/schema
axes must agree. Missing or ambiguous images refuse while retaining the
original effect; neither latest-current export nor guessed axes are allowed.
The existing recovered-publication capability still performs real owner
admission and pin acquisition. Its proof must recover without a caller-held
successor reference, including later Native head advancement.

Use one named move-only proof phase in `worth-proof`,
`RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>`.
Native owns the concrete private payload, its actual recovering runtime/source
binding, original semantic result and admitted predecessor/successor bases/pins.
Store/World bind home identity and incarnation through the actual private Home
stage; Native need not interpret World's association or invent a Home binding.
Only successful Native owner admission constructs the payload. World consumes
that concrete specialization through a distinct recovered-adoption branch and
moves custody into its existing retained owner. Generic markers, receipt data,
index entries and caller-provided fate pairs cannot construct it. Refusal retains
the original subject/evidence and any acquired custody.

Settlement observations distinguish actual matching performed-but-pending and
performed-and-settled route states. Existing pending repair acts on that same
record/route and remains receipt-only; World's live result stays `None`.
`CheckpointReadmitted` reports exact admitted directory/source coverage only,
not a preserved live result or settlement event. It authorizes no adoption by
itself. The capability requires actual source/body/root admission and the required
recovered owner state and pins. Pending work must be settled by its existing owner
before adoption; pending-record absence is never settlement proof.

Memory-only backing, interruption, stale/foreign binding, source/index mismatch,
wrong roots/schema/references, missing semantics and unavailable roots have typed
refusals. Recovery neither republishes an effect nor executes a handler, constructs
`PerformedRelationalCommit`, fabricates `CommitResultSeal`, or supplies a recovered
`Arc<CommitResult>`. Arbitrary recovery bytes or public traits do not manufacture
the concrete Store-private `WorldReconciled` Home stage.

The destination within the existing owners is:

```text
worth-relational/src/
  transactions/data/outcomes/
    commit_semantic_result.rs                 # shared immutable meaning
    commit_semantic_result/{creation_correspondence,validation}.rs
    commit_result.rs                          # live-only seal plus same meaning
  authority/commit/pipeline/
    semantic_result.rs                        # original producer, before seal
    publication_execution/preparation.rs      # install image before Arc seal
    result_assembly.rs                        # consume same value
  history/commit/index_entry.rs               # required discovery metadata
  history/data/canonical_commit_envelope/     # both existing projections
  durability/canonical_commit_artifact/       # v3 strict readmission
  durability/log/native_checkpoint_codec/     # CHK16 strict directory/image
  durability/authority/recovery/
    recorded_component.rs                    # exact source-backed roots
    recovered_publication.rs                 # private capability custody/minter
  branch/{recorded_basis,canonical_runtime_frame}.rs
  facade/durability.rs                        # narrow owner admission exports
worth-proof/src/recovered_publication.rs      # legal consuming phase shape
worth-runtime-world/src/
  durability/                                # recorded basis/reconciliation
  publication/recovered_adoption.rs           # consumes Native capability
  recovery/product_unpublished/               # original composite custody
worth-store-application/src/world_section/readmission.rs
```

These are current-slice destinations, not empty placeholders. Pure semantic
definitions remain Native-owned; legal phase shape belongs in Proof; clocks,
source, pins and `Drop` stay in Native runtime custody. World owns composite
reconciliation; Store owns filesystem sequencing. No generic recovery bag or
second authority is introduced.

The intended new facade sequence is:

```rust
let (_, mut recovered) = native.durability_recovery()
    .recover_with_authority(execution, plan)?;
let subjects = recovered.associated_publication_subjects()?;
let admitted = recovered.readmit_publication(
    execution, original_subject, recorded_predecessor,
)?;
world.recovery().prepare_recovered_relational_adoption(
    original_custody, admitted,
)?;
```

Names describe the required new contract, not APIs that already exist. Discovery
does not load historical bodies; only owner admission grants the capability.
Continuation/release then follows the existing durable-disposition-before-live
protocol. Recovery itself publishes no occurrence or disposition.

Artifact v3/CHK16 are a clean undeployed-format cutover: older versions,
including CHK15, refuse before effects; no missing-field default,
reinterpretation, in-place repair or automatic reinitialization. Canonical
artifact v3 remains unchanged by the exact-component checkpoint amendment.
Implement shared semantic production/versioning,
then exact source/historical admission, then World adoption, then the real
filesystem journey within existing Phases 3-4.

The decisive proof kills after Native commitment and before occurrence on both
sides of the first Home checkpoint. Cover empty and cold canonical predecessors,
a journal-tail effect, entity/relation correspondence and nontrivial validation
meaning. Compare authored expectations and the original live semantic projection
with recovered output; a same-encoder round trip is insufficient. Observe original
identities/custody, unchanged handler invocation, no recovery-time republication,
continuation or release, and a second reopen. Missing discovery/semantics,
memory backing, stale/foreign sources, wrong roots and altered correspondence or
validation must fail for the intended reason. Unrelated cold bodies stay cold;
excluded live fields must be unavailable. Local Native tests do not qualify the
filesystem World journey or complete either phase.

*Idempotency and receipts (D9)*

- The binding records affinity and association. Resolution reads only durable
  state.
- Each original receipt field has a durable source:

| Receipt field | Durable source |
|---|---|
| Commit receipt/id, version, journal position and artifact digest | Exact source-backed Relational artifact, readmitted and bound to admitted index/root evidence |
| Original durable semantic result: create correspondences, changed records, structural/schema/history/change/aspect and validation meaning | `RelationalCommitSemanticResult` sealed in the original artifact |
| `changed_record_count` | Exact recorded ordered changed records, consistent with recorded change summary |
| Live `CommitResult`, process transaction/proposal identity, snapshot handle/ID, timings/complexity and late interruption | Not reconstructed. Recovered receipts expose distinct `RecoveredRelationalSemanticResult`; live-only fields are unavailable |
| commit evidence: idempotency binding | Query idempotency binding row |
| commit evidence: outbox seal | durable outbox rows in the same commit |
| commit evidence: provider session binding | Not recovered: sessions are process-local and incarnation-fenced (D7). A recovered receipt carries the publishing home incarnation in its place, and nothing resolves through the old session. |
| `basis_descriptor` | Relational artifact (runtime-neutral, Milestone 2 D11) |
| occurrence, old and new product head, attempt | World occurrence |
| outcome identity, emitted effect count | Query idempotency binding row |
| exact output settlement | Query outcome row in the same commit (new field) |
| aftermath causality | Relational causality rows |
| publishing runtime | home identity and incarnation, recorded in the occurrence |

Recovered equality means equality of this explicit durable semantic projection,
not live execution observations. Receipt constructors consume the readmitted
binding, World occurrence/custody and Native owner-admitted inputs; the derived
evidence index grants no constructor authority. The existing capability index,
build guide and Host README must explain this distinction and use the real facade.

- These are deleted: the receipt authority of the evidence store,
  `CommittedReceiptNotRetained`, and the Server retry caches.
- **Changed intent and foreign affinity.** A retry with changed intent refuses
  with `IdempotencyIntentDrift`. A foreign product branch or incarnation
  refuses with the existing affinity refusal, now evaluated against durable
  affinity.
- **Custody and fates.** A retry whose commit is in recovered custody answers
  `ProductUnpublished` with its next actions. An attempt with proven no
  effect executes once on retry. An unresolved attempt answers its unresolved
  posture until recovery decides it.

**Interim refusals.** These are the same as Phases 3–4 above.

**Required tests**

- **The decisive journey**, on the production host and filesystem media:
  1. Perform an ordinary mutation with an idempotency key.
  2. Let it durably complete, then drop its response.
  3. Kill the process without closing it.
  4. Reopen in a fresh process and retry the same request.

  The retry returns the original occurrence and exact branch state. The
  mutation handler's invocation count is unchanged, and there is no duplicate
  domain effect. The journey runs on both sides of the first home checkpoint.
- **Retry refusals:**
  - a changed-intent retry after reopen refuses;
  - a retry on a sibling product branch refuses;
  - a stale capability from before the reopen refuses.
- **Durable data across reopen:**
  - a pre-crash receipt used for history and ancestry lookups succeeds after
    reopen;
  - recovered service then performs another ordinary mutation.
- **Every receipt field** in the table equals the reference after reopen.
- **Crash between the Relational commit and product movement.** Recovery shows
  custody: the Relational effect retained, the product unmoved, and a retry
  answering `ProductUnpublished`. Continuing publishes the original commit
  without re-execution.
- **Release, kill, reopen.** The custody stays released.
- **Caller abandonment and cancellation.** These after an owner effect leave
  custody that survives reopen.
- **Proven no effect.** An attempt with proven no effect executes exactly once
  on retry after reopen.
- **Indeterminate product movement.** It blocks product movement on every
  branch until reopen (D4), and recovery decides it.
- **Concurrent same-key retries** produce exactly one occurrence, and every
  caller receives it.
- **History and identities.** World history pages and ancestry after reopen
  equal the reference. Identities issued after reopen never collide with
  recovered ones.
- **Server.** The retry route survives a restart of the Server process.

**Oracle extension**

- **Reference and script.** A memory-only host running the same program
  executes a seeded host script. The script covers keyed mutations, including
  distinct keys, same-key retries, changed-intent retries and concurrent same-key
  retries; response loss, branch creation, product movement on several
  branches, settlement failure, custody continue and release, caller
  abandonment, and checkpoints. Every seed in the tested range generates.
  Application mutations require an idempotency key by contract (build §5.3), so
  there is no unkeyed route to exercise.
- **Interruption points.** The oracle interrupts at every durable boundary:
  - before commitment;
  - after durable Relational publication;
  - between the owner effect and product movement;
  - after product movement, before the response;
  - during custody dispositions;
  - during checkpoints.
- **Comparison after each reopen.** Graph state, occurrences, custody, product
  heads and idempotency outcomes are compared with the reference prefix. The
  oracle checks the distinct no-effect, committed and unresolved outcomes. It
  never expects a committed receipt for an attempt that never committed.
- **Continuation.** The remaining script then runs to the reference's final
  state.
- **Fault coverage.** A media sweep of every Store operation of a mutation, a
  product movement, a disposition and a checkpoint; real process kills at
  every Store seam; and recovery interruption at every stage, including World
  reconciliation.

**Docs**

- `capability-index.md`, rows "Retry a mutation safely" and "Survive a
  restart".
- `build-an-application.md`: commitment, retry, close, unsupported homes and
  recovery, at the caller's level.
- The host README.
- The bank docs' process-death disclaimers, for Relational-only flows.

**Exit criteria (Phases 3 and 4 together)**

- **Landing 2's Tier-2 run is green on the combined head.** This diff
  reaches nearly every workspace except worth-ui, so the run is close to the
  full matrix by construction.
- **Oracles.** The cumulative oracle runs exhaustively, because the diff
  reaches the home and owner durability.
- **Certification.** The coordinator certifies at the final gate.

---

## Phase 5: Signal and Bridge authored state; mixed publication

**Goal.** Signal and Bridge recover their authored state through their own
readmission. Mixed Relational/Signal operations recover at the true owner
boundary.

**Requirements**

*Authored and derived state*

- Signal separates authored state from derived and scheduling state:
  - **Authored:** installed definitions, runtime-published definitions,
    topology and branch structure.
  - **Derived:** values, dirty and aspect versions, pending causes, traces,
    telemetry, and the resource and temporal runtime.
- The authored image is a Signal-owned artifact with a versioned, checksummed
  byte codec, not serde snapshots.

*Signal port and artifacts*

- Signal defines `SignalDurableMedium`, a wasm-safe trait with the D2
  answers.
- `SignalDefinitionTransitionArtifact` records a change to authored
  definitions. It includes Bridge's conditional definitions and lowering
  claims as a Bridge-defined section.
- `SignalBranchAdvanceArtifact` records the branch, generation, attempt
  association (D5) and definition-transition reference. It never carries
  derived values.

*Identities*

- Signal's durable identities become home-scoped (D6): owner, branch, cell
  incarnation, generation, snapshot, admission and conditional contract
  occurrence.
- Descriptor readmission accepts a recovered owner and refuses a foreign one.

*Readmission*

- Multi-branch owner readmission installs program definitions and applies
  durable definition transitions in order. It restores branch structure and
  generation frontiers for every World branch, and leaves derived state cold,
  recomputed on demand.
- A derived value never stands in for a lost definition.

*Mixed publication*

- World occurrences carry a Signal component-evidence tag.
- A crash between the Relational and Signal effects recovers custody naming
  exactly the acting owners, with the product unmoved.

*Bridge*

- Conditional definitions recover through the Signal transition section.
- Bridge's lowering registry and retention ledger are derived, rebuilt at
  installation.
- Writeback replay records follow their audit classification.
- Owned-async custody belongs to Phase 7.

*Build identity*

- Signal derived state recorded under one build is stale under another. It is
  recomputed, never reused unconditionally.

**Interim refusals lifted.** Signal-touching and mixed operations, and
conditional-definition publication.

**Still refused until later phases.** Conditional and temporal operations,
workflow runs, managed and owned-async work, dispatch, corrections and
program adoption.

**Required tests**

- **Signal readmission refuses** a damaged frame, a foreign owner, an unknown
  version and a definition-basis mismatch.
- **Reopen with several branches.** A home with several branches, published
  definitions and conditional definitions reopens to authored state equal to
  the reference.
- **Derived values.** They recompute to the reference values on first demand,
  and none is read from durable bytes.
- **Mixed operation killed between owner effects.** It recovers custody with
  exactly the acting owners. Continuing completes it without re-executing
  either effect.
- **wasm.** `worth-signal-wasm` checks for wasm32 with the port present.

**Oracle extension.** The host script adds Signal-only and mixed operations,
conditional-definition publication, and Signal forks with branch creation. It
interrupts at every Store seam of each Signal publication, and between owner
effects.

**Docs**

- The Signal README durability section and the Bridge README.
- The capability index.
- The derived-recovery section of `build-an-application.md`.

**Exit criteria**

- The phase's own reviewer approves, and its Tier-2 run against the
  integration branch is green. The cumulative oracle runs with the phase's new
  cases included.
- Landing verification happens once, at Landing 3 (Phase 10).

---

## Phase 6: Programs, adoption and build identity

**Goal.** Program lifecycle and adoption obligations survive restart, and a
changed build identity is an oracle dimension rather than a single case.

**Requirements**

- **Support lifecycle.** Program support status (Active, Retiring, Retired)
  and its custody counts become durable obligations in `query.obligations`.
  Counts derive from durable obligations and are never reset to Active at
  open.
- **Program activation.** Activation is verified on every branch at open, not
  only on main. Each occurrence's program revision (recorded since Phase 4) is
  checked against the branch's activation history.
- **Adoption custody.** These become durable custody:
  - `WorthQueryBranchAdoptionRecovery`, with its support custody and
    unpublished recovery;
  - `WorthQueryStoppedBranchSetAdoption`.

  Both resume through `recover_branch_adoption` and the branch-set resume
  after reopen. They are no longer caller-held values whose loss forgets the
  obligation.
- **Build identity.**
  - Authored source and references survive a changed build.
  - Incompatible derived outputs are marked stale under their reuse
    contracts. They are recovered or regenerated through the normal demand,
    dependency, invalidation and settlement owners, never eagerly at open.
  - Same-build reuse keeps its equivalence and ownership checks.
- **Derived caches.** Discarding derived caches or materializations loses no
  authored truth, history or custody.

**Interim refusals lifted.** Program adoption and support retirement.

**Required tests**

- **Producer directory build identity.** Restore
  `directory_changed_build_reuses_original_output_until_new_implementation_is_demanded`
  (restore from `5358b31d3083dc88eea961af716e948f759eac96:workspaces/worth-query/crates/worth-query-court-decl-host-package-archive-replay-application-graph/tests/worth_query_court_decl_host_package_archive_replay_application_graph/certification/application_graph/producer_predicate_checkpoint/changed_build.rs`): produce under A, reuse the
  original accepted output under A, reopen under B without producer execution,
  then regenerate only on demand.
- **Retirement survives restart.** Retire support, kill, reopen: the status is
  still Retired, and interpretation refuses where the retirement requires it.
- **Adoption resumes.** Adoption interrupted at each stage across a kill
  resumes to the reference result, with no repeated effect.
- **Unrostered activation.** A non-main branch whose activation names an
  unrostered revision refuses at open.
- **Changed build.**
  - Stale outputs are reported stale.
  - Outputs regenerate only on demand, and keep their original entities.
  - Required-output obligations keep their execution and failure semantics.
- **Discarded derived state.** Discard derived state (D14 door), reopen:
  authored truth and custody are exact.

**Oracle extension.**

- From this phase on, two dimensions apply to every generated sequence:
  - build identity changes between reopens;
  - derived material is discarded between reopens.
- Adoption and retirement operations join the host script.

**Docs.**

- The program adoption guide.
- The "Build identity" sections of `build-an-application.md`.
- The capability index.

**Exit criteria.**

- The phase's own reviewer approves, and its Tier-2 run against the
  integration branch is green. The cumulative oracle runs with the phase's new
  cases included.
- Landing verification happens once, at Landing 3 (Phase 10).

---

## Phase 7: Workflows, conditional operations and long-running work

**Goal.** Workflows, conditional and temporal operations, managed recovery
and owned-async work resume after process death, or terminate exactly where
their existing lifecycle contracts define a terminal posture.

**Requirements**

- **Workflows.**
  - Workflow definitions, instances, transitions, approvals and coverage are
    graph records, and recover with Relational.
  - Progress and replay retention are derived, rebuilt by reconstruction.
  - A workflow interrupted mid-run resumes from durable state under its
    original keys. Its required approvals remain required.
- **Workflow migration on transition.** A home holding workflow records
  transitions (D16) under the same workflow dispositions that live program
  adoption applies (`program_adoption/preparation/dispositions.rs`).
  `checkpoint_transition/selection.rs` refuses workflow records today; this
  phase deletes that refusal.
- **Managed cleanup.** Cleanup required after a failed commit becomes a
  durable obligation, recording the run identity and failure kind. Its
  process resources are reacquired through fresh authority. It is persisted
  even when clean close would have refused.
- **Conditional decisions.**
  - These retained decisions become durable obligations: CleanupRequired,
    Retryable, CommitRetryable and ProductStale.
  - Eligible, Deferred and Suppressed wakes are process-local. They are
    re-derived from durable state and host-fed time after reopen.
  - Clock leases are reacquired.
- **Temporal operations.** The temporal idempotency key derives only from
  durable, home-scoped identities, never from per-process runtime binding
  identity. A temporal intent re-executed after restart resolves to its
  original occurrence.
- **Managed runs and owned-async custody (Bridge and Query).**
  - Each yield, suspension and mandatory in-flight request is resumed from a
    durable record.
  - Termination at reopen is allowed only where the existing lifecycle
    contract defines a terminal posture for that kind. The termination is
    then typed and observable.
  - Loss of memory is never success.
- **Resources.** Resources are process-local and are reacquired through fresh
  authority.

**Interim refusals lifted.** Workflow runs, conditional and temporal
operations, and managed and owned-async work.

**Required tests**

- **Workflow killed mid-run.** It reopens, resumes under its original keys and
  reaches the reference terminal state. Its approvals survive.
- **Workflow migration.** A transition of a home holding workflows applies the
  migration dispositions. A kill at each transition seam resumes correctly.
- **Managed cleanup.** Kill with cleanup outstanding. The reopened home shows
  the obligation. Discharge it, and close then succeeds.
- **Temporal operation.** Kill it between its commit and its response.
  Re-execution after reopen yields one occurrence.
- **Managed-run suspensions.** Each kind either resumes, or terminates with
  the typed posture its contract defines.
- **Owned-async requests.** Request custody survives a kill and resumes or
  terminates per its contract.

**Oracle extension.** The host script adds:

- workflow runs stopped mid-progress;
- conditional and temporal operations;
- managed cleanup;
- owned-async requests;
- workflow-bearing transitions.

It interrupts at every durable boundary, including between workflow steps.

**Docs.**

- The workflow and conditional-operation guides.
- The capability index rows for managed and long-running work.

**Exit criteria.**

- The phase's own reviewer approves, and its Tier-2 run against the
  integration branch is green. The cumulative oracle runs with the phase's new
  cases included.
- Landing verification happens once, at Landing 3 (Phase 10).

---

## Phase 8: External dispatch and custody

**Goal.** External work keeps its custody across process death. Unknown
external effects stay unknown until their owner resolves them, and completed
external effects are never repeated.

**Requirements**

- **Outbox contract on reopen.** The outbox's inbound contract is rebound from
  the installed program on reopen. It is no longer `None` after restore.
  Outbox rows, transport causal digests and runtime source commitments carry
  no process-local identity. The audit's identity table for Phase 8 lists
  them.
- **Dispatch custody.** A dispatch attempt is recorded durably in
  `query.dispatch` before the external call. Its outstanding custody,
  correlation and idempotency association survive.
- **Crash after dispatch.** If the process dies after dispatch and before the
  response is recorded, the external posture is indeterminate. That posture
  survives reopen until the owner resolves it.
- **Proven no effect.** Proven no effect (for example `PayloadRejected`) is a
  first-class durable outcome. It is distinct from unresolved, compensation
  and completed.
- **Inbound custody and deduplication** are durable: message identity,
  terminal, pending and expiry.
  - A duplicate delivery after reopen is recognized as a duplicate.
  - Terminal acknowledgement settlement is durable.
  - An authentic callback for a pre-crash dispatch completes it, rather than
    answering "unknown correlation".
- **Recovery handles.** Their claim status is a durable obligation. Recovery
  progression (compensate, reconcile, redispatch, safe retry, resolve, expiry)
  operates on recovered custody.
- **Rebuilt indexes.** The inbound terminal index and the evidence store's
  correlation index are rebuilt from durable World history and Relational
  rows.
- **Deleted postures.** The `StoreCapabilityRequired` postures are deleted
  from `WorthQueryDispatchOutboxDurabilityPosture` and
  `WorthQueryRecoveryDurabilityPosture`.
- **No exactly-once claim.** There is no claim of remote exactly-once beyond
  the external owner's admitted contract.

**Interim refusals lifted.** Dispatch and inbound completion.

**Required tests**

- **Bank external rail under application kills.** With the bank external rail
  as the external party, these cases pass while the application process is
  killed:
  - exactly-once completion;
  - lost response;
  - duplicate acknowledgement;
  - completion after timeout;
  - disappearance mid-dispatch;
  - semantic rejection, as proven no effect.
- **Crash between dispatch and response recording.** Reopen shows the
  indeterminate posture. Reconcile resolves it without redispatching a
  completed effect.
- **Duplicate inbound delivery across a reopen** is deduplicated.
- **Authentic callback after reopen** completes the original dispatch.

**Oracle extension.** The host script adds dispatching operations and inbound
completions against a deterministic in-test external party. It interrupts at
every durable boundary, both before and after the external call, and before
inbound results are recorded.

**Docs.**

- The external work and recovery guides.
- The process-death disclaimers in `http-owning-money.md` and
  `public-consumer-contract.md`, rewritten to describe the supported behavior.
- The capability index.

**Exit criteria.**

- The phase's own reviewer approves, and its Tier-2 run against the
  integration branch is green. The cumulative oracle runs with the phase's new
  cases included.
- Landing verification happens once, at Landing 3 (Phase 10).

---

## Phase 9: Correction and original-owner advancement

**Goal.** 9.18 corrections and their retained material survive restart, and
the original owner keeps editing after reopen.

**Requirements**

- **Lineage readmission.** Correction occurrences, alternatives, causal links
  and lineage are Relational causality rows. They readmit against durable
  World identities, which they record as home-scoped identity since Phase 4.
- **Retained material.** Retained governed input, pre-images and original
  authority become durable in `query.corrections`, under their retention and
  disclosure contracts. Each is optional or an obligation, as its contract
  says. Expired optional material answers typed unavailability (D10). Recovery
  never guesses an inverse or reinterprets retained bytes.
- **Custody.**
  - Correction idempotency resolves through Phase 4's durable idempotency.
  - Required-output custody for corrections, meaning the pending publication
    registry's correction entries, is a durable obligation.
  - Authored correction and required derived settlement stay distinct,
    including recovery when only one has completed.
- **Divergent branches.** Correction meaning and the selected lineage are
  preserved through reopen. Recovery never consumes or substitutes an
  alternative from another branch.
- **Original-owner advancement contract.**
  - **The original owner** is the retained native operation owner, in the
    platform's existing sense: the continuation that fresh authenticated
    commands advance and close. Examples are the bank's
    `BankHttpMutationContinuation`, `capability-index.md` "advance and close
    each original owner", and `build-an-application.md`'s original-owner
    guidance. Today it is a caller-held process value.
  - **Durable custody.** The owner's continuation becomes durable custody in
    `query.obligations`. It records:
    - its original operation identity and its occurrence;
    - its native ownership material, meaning the aggregate identities and
      incident relation facts it governs;
    - its lifecycle position.

    Phase 7 supplies the managed and owned-async custody it builds on.
  - **After reopen.** The custody readmits, and the host issues a fresh owner
    value, bound to the new incarnation (D7), for the same original identity.
    Fresh authenticated commands advance and close it exactly as before the
    crash. The old process's value refuses with `StaleIncarnation`.
  - **A dependency edit followed by advancement.** Advancement continues the
    same original owner. Aggregate identities stay continuous under the
    existing keeper, split and merge laws (`aggregate_identity/transition.rs`):
    membership may change exactly as those laws allow, and no identity is
    re-minted outside them.
  - **What reconstruction must not do.** It never replaces the owner, and it
    never creates a new authoritative object to make the edit succeed. No new
    lineage kind is added.
  - **Missing material.** If the required ownership material is unavailable,
    advancement refuses with `OwnershipMaterialUnavailable`.

**Interim refusals lifted.** Corrections.

**Required tests**

- **Correct, kill, reopen.** Correction history, alternatives and lineage equal
  the reference.
- **Reapply after reopen** selects the recovered alternative.
- **Expired optional pre-image** answers typed unavailability.
- **Divergent branch.** A correction on a divergent branch, killed and
  reopened, keeps its branch's lineage.
- **Split completion.** A correction killed with the authored correction
  complete but derived settlement pending recovers that exact split.
- **Advancement after reopen.** An original owner is held across a kill and
  reopen. A fresh authenticated command, then a dependency edit, then
  advancement and close of the original owner, all match the reference:
  aggregate identities, membership and incident relations are equal, and no
  identity is re-minted outside the keeper, split and merge laws.
- **Stale owner value.** The pre-crash owner value refuses with
  `StaleIncarnation`.
- **Missing ownership material** refuses with
  `OwnershipMaterialUnavailable`.

**Oracle extension.** The host script adds corrections, reapplies and
divergent-branch corrections, and a dependency edit plus original-owner
advancement after a reopen.

**Docs.**

- The correction guide.
- The capability index rows.

**Exit criteria.**

- The phase's own reviewer approves, and its Tier-2 run against the
  integration branch is green. The cumulative oracle runs with the phase's new
  cases included.
- Landing verification happens once, at Landing 3 (Phase 10).

---

## Phase 10: Public cutover and milestone certification

**Goal.** One supported durable path, honest public guidance, and
certification of the whole milestone.

**Requirements**

- **Retire the interim checkpoint path (D13).** Delete it entirely:
  - capture;
  - byte transport;
  - the `*_from_checkpoint*` constructors;
  - `repair_to_checkpoint*`;
  - Relational's embedded-checkpoint placement.

  Its callers move to the open call.
- **Delete `UnsupportedDurableFamily`.** Every family in the roadmap's
  Milestone 3 requirements is supported. No family within that set remains
  refused.
- **Re-audit every Phase 1 family against the code.** Confirm three things:
  - no competing durable publication or recovery path remains;
  - no durable row records a process-local identity;
  - no capability claim overstates what reopening resumes.
- **Update every ordinary guide** with supported restart behavior, failures
  and valid next actions:
  - the application, workflow, external recovery and correction guides;
  - the host README;
  - the capability index, including "Known gaps".
- **Executable examples** use the public host and its real owners:
  `observable_workflow`, `authored_workflow`, `shared_execution` and the bank
  world.

**Required tests**

- **The full journey,** on the production host and filesystem media, in one
  run:
  1. mixed-owner publication;
  2. a deferred settlement;
  3. a workflow mid-run;
  4. an ambiguous external completion;
  5. a correction on a divergent branch;
  6. kill and reopen;
  7. recovery of original occurrences and custody, without repeated effects;
  8. a dependency edit, then advancing the original owner.
- **No caller of a deleted path remains.** Boundary-check enforces this.

**Oracle.** The cumulative oracle runs once, exhaustively.

- Generated sequences mix every family, interrupt at every durable boundary,
  and reopen in a fresh process.
- Build-identity change and discarded derived state are dimensions across the
  whole run.
- Recovery interruption runs at every stage.

**Completion.** Milestone 3 is complete when:

1. **One durable home.** The host opens one durable home. No
   application-owned checkpoint transport, in-memory receipt authority or
   parallel retry cache remains.
2. **Every audited family is accounted for.** Each is either durable, or
   process-local with reacquisition through fresh authority that is
   documented and tested.
3. **Journeys pass.** The decisive journey (Phase 4) and the full journey
   (Phase 10) pass on both sides of the first home checkpoint.
4. **Oracle passes.** The cumulative oracle's generated sequences pass across
   build-identity change and discarded derived state.
5. **Every lane is green.**
   - Not merely free of new failures. The lanes are Relational, World, Signal
     (including wasm), Bridge, Server, Query (library, integration, examples,
     doc and fixture gates), bank world and Store, plus boundary-check,
     agent-context, clippy, formatting and line caps.
   - Failures that existed before the milestone are fixed honestly.
6. **Certified.** An independent reviewer certifies the milestone against this
   specification and the roadmap.

## Out of scope: worth-ui

`workspaces/worth-ui` is not ready, so it is left out of this milestone
entirely. No phase migrates `worth-ui-query-binding`, and no landing gate
includes the UI lanes: the `ui`, `check-ui-workspace` and `clippy-ui-binding`
steps of `query_gate.py`. The binding stops compiling against the host once
Phase 3 replaces the `in_memory*` constructors. When worth-ui is picked up,
its own work adopts the open call; this milestone adds no shim for it.

## Verification tiers (every phase)

Verification cost is the milestone's main expense. It is spent where a diff
can actually break something.

| Tier | When | What runs |
|---|---|---|
| 1 | Each slice | `cargo check` on touched crates, plus the tests the slice adds or changes, selected by name. |
| 2 | Each phase merge into the integration branch, and each landing | Only the lanes the diff can reach, computed mechanically by `affected_lanes.py` in the orchestration tooling. A lane is reached when the `cargo metadata` resolution of its workspace includes a changed package, or when the diff changes the lane's non-crate inputs: docs for the doc gates, scripts and workflow steps for their lanes. The constitution check (boundary-check and agent-context), line caps in diff mode, formatting of touched workspaces, and lockfile alignment when Cargo files change are always included. Tool self-tests run only when the tool changed. |
| 3 | Landing 3 (Phase 10), and idle background runs on master | The full lane matrix and the exhaustive cumulative oracle. |

- **Crash oracle at Tier 2.** The oracle runs exhaustively when the diff
  reaches its subject: Store publication, recovery, root or journal code, the
  home, or any owner's durable artifacts, readmission or recovery. Otherwise it
  runs its focused seed range plus every case the diff adds or changes.
  Failing seeds reproduce deterministically.
- **Evidence.** Logs record the command, HEAD, a clean-tree flag, times and
  the exit code. A lane with a green log on the same HEAD is reused, never
  rerun. Reviewers verify evidence and probe; they do not rerun suites.
- **Every lane green.** This is enforced at Tier 3. A Tier-3 failure is fixed
  before the milestone closes, whichever phase caused it.
- **Merging to master.** Each landing merges the current master in, runs the
  Tier-2 lanes that merge reaches, then fast-forwards master. Nothing is ever
  force-pushed.
