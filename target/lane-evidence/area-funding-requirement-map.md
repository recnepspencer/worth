# Funding requirement map

These tests are selected by the exact expressions in `area-funding-proof-batch.json`; the corrected cold case and unchanged bounded case have individual rerun logs. Names state behavior; requirement identifiers remain here.

| Requirement | Behavioral evidence |
|---|---|
| D12 automatic checkpoints preserve ordinary caller results and use the declared Host allowance | `automatic_capture_at_head_forty_one_preserves_ordinary_request_results`; `automatic_checkpoint_exhaustion_keeps_business_results_and_explicit_capture_uses_its_own_allowance`; `automatic_checkpoint_progresses_after_caller_cancellation_without_replacing_the_body_value` |
| D11/D12 close upkeep uses Host funding; exhaustion retains recoverable tails | `close_checkpoint_uses_host_allowance_with_an_ordinary_close_caller`; `close_checkpoint_exhaustion_reports_failure_and_preserves_journal_reopen` |
| D12 bounded restart at ordinary8M policy/default maintenance allowance | `automatic_checkpoints_bound_host_reopen_journal_reads_with_headroom` (unchanged <=40 assertion) |
| D12 no covered canonical reads for live and cold recapture | `live_checkpoint_recapture_keeps_all_covered_canonical_bodies_cold`; `cold_image_plus_tail_recapture_reads_only_the_uncovered_original_frames`; `checkpoint_image_restores_publication_bases_and_creations_without_canonical_body_reads` |
| D12 captured-state bound independent of discarded covered updates | `selected_state_recapture_work_does_not_follow_discarded_covered_update_bytes` |
| D12 selected capture has one source; no missing-base replay or selected Embedded export | `selected_checkpoint_without_captured_original_base_refuses_without_body_demand`; `selected_embedded_capture_refuses_before_canonical_body_demand` |
| D12 selected member and index custody survives another Complete | `complete_source_recovery_restores_original_inputs_for_another_complete_capture`; `cold_complete_recapture_preserves_original_index_generations_and_live_tail`; `cloned_complete_plan_retries_after_member_read_cancellation_without_directory_contamination` |
| D17 resource stops before effects; real retained owners retain their funding | `head_materialization_budget_survives_edit_and_last_basis_drop`; `full_materialization_budget_survives_mirror_edits_until_runtime_drop`; `checkpoint_image_recovery_respects_memory_and_cancellation_and_releases_its_retained_budget`; `complete_capture_stopped_request_precedes_proven_selected_damage` |
| D17 malformed scalar structure is established before schema use/allocation | `excess_scalar_columns_refuse_before_schema_lookup` |
| D17 cold wide-contract work refusal preserves retry and refunds temporary backing | `cold_head_work_exhaustion_keeps_a_wide_contract_region_retryable`; direct source inspection proves contract/field traversal admission before its walk |
| D12 original tail, checkpoint plus tail, and interrupted ACK recovery | `journal_only_killed_home_reopens_original_inputs_and_commits_without_initializing`; `killed_home_reopens_normalized_image_plus_nonempty_tail`; `interrupted_owner_ack_resumes_original_attempt_and_reopens_unacknowledged_selection`; `selected_checkpoint_survives_observer_drop_online_history_and_mutation_before_ack_recovery` (includes zero member reads before World admission) |
| D12 selected image binding/predecessor/missing/foreign/damage refusals | `selected_home_admission_refuses_predecessors_missing_foreign_and_damaged_images`; `selected_home_image_preserves_original_binding_fields_and_world_publication` |
| D8/D12 checkpoint accelerator preserves partial-Home semantics | `initializer_refusal_reopens_at_missing_world_origin_without_replaying_initializer`; `selected_completion_and_activation_failures_keep_owned_home_without_initializing` |
| D12 reconstruction preserves actual CDC names, keys and lineage | `cdc_property_persisted_random_operation_matrix_recovers`; `seeded_full_key_differential_fork_recovery_and_replay_keep_actual_names_and_lineage` |

Private owner structure is reviewed against the allocation preflight and same-backing ticket contract. The two lifetime courts observe the public request's memory and the public edit/retention/runtime owners; they do not demand test-only getters into the ticket implementation.
