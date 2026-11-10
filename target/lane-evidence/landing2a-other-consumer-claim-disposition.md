# Remaining non-history consumer claim boundary review

Read-only source review at the working tree based on `45b6e87a0a`, 2026-11-09. No Rust edits, formatting, builds or tests. A disposition is not passing evidence.

## Finite direct scope

Exact normalized rg inventory is in `landing2a-other-direct-inventory.json`: 73 files,154 named roots,0 unclassified; excludes tests/history, tests/durability, tests/support. Includes production history owner units. Full extracted assertions/calls in other-root-claims.json; classification JSON preserves those originals. Helper-only files are not independent claims.

HAPPY: public owner port with actual original medium. OWNER: local value/legality unit with coherent actual Native Memory data only when physical origin is irrelevant. UNCHANGED: non-adapter Memory/pure sibling, no migration inferred from module import. C2: private residency, pin/custody, exact allocation/work. C4: actual injected source/recovery fault. Mixed rows must split. HAPPY-GAP remains an exact happy claim needing the genuine producing path; no private setter becomes public. Closed grammar negative-value units are distinct from mutating an actual source and running runtime recovery.

## Existing public contracts and nearest working analogues

- `worth_testkit_relational::durable_medium::TestDurableMedium` is external.
  Retain its actual Arc at construction and use `.build_durable(medium.attachment())`.
  `durable_publication::{durable_runtime,capture,recover,perform}` and the existing
  external `durable_publication/branch_history_{publications,recovery}.rs` courts
  demonstrate actual authoring and original-source recovery without discovery.
- `observe_branch(identity)` / `admit_branch_basis(identity)` and
  `begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())`
  replace private fixture transaction factories. Public publication_port
  compare_and_publish issues the actual performed witness. settlement_port is
  cloneable/shared-borrow and supports retains_pending_settlement(id), settle and
  repair-by-identity. Medium offered/committed artifacts prove exact durable append
  counts, not a later head equality alone.
- `recovery_plan_from_source(request, medium.recovery_source(), mode)` then
  `durability_recovery().recover(request, plan)` on a matching receiver captures
  actual durable evidence. `RecoveryPlan.tail_log`, `RecoveryOutcome.coverage`,
  public patch/subscriber batches, recorded receipts and canonical replay
  envelopes already expose the semantic observations. Select branch receipts
  via branch_head_for_observation; historical_latest_commit only where the claim
  actually is catalog latest, not branch currentness.
- `begin_branch_schema_transition(&basis, ProposedSchemaTransition,
  Option<SchemaReconciliationPolicy>, target_registry)` is the existing public
  basis-only transition, at runtime/schema_transition_admission.rs:104. It checks
  actual source/target and complete executable-contract preservation. It replaces
  private schema registry reconfiguration for basis-only schema/CDC tests.
- For actual executable contract changes, the owner already has
  prepare_home_program_contracts(request, predecessor, registry, invariants,
  sources) and begin_home_program_step(request,basis,contracts,plan,step,original,
  association,admission), with `RelationalHomeProgramStepContracts::Target`. Admission and
  envelope authenticate actual Home origin; target_step.rs validates original
  source/target and installs whole successor execution meaning. Follow the actual
  Home program installation service. A parallel arbitrary registry setter would
  be a second authority path.
- Public authored write history selector/generation discard/rebuild, index
  authority/generation reads, pinned derived relation reads, aggregate identity
  reads, current/historical inspection and lineage/resolution APIs cover the
  remaining ordinary semantic observations. Exact private residency or work
  census is not equivalent to a semantic read and stays C2.

## Exact gaps and precautions

1. Aggregate logical checkpoint's
   `physical_membership_recipe_and_alternating_derived_catalog_restore_from_real_checkpoints`
   ends with assert_durable_commit_meaning_eq across original/restored envelopes.
   Its private recorded_original_inputs() includes optional result-image,
   prestate/result schema and original MutationConfig. Public Serialize exposes
   all body/semantic-result fields but omits this attribution. Same gap as the
   three reviewed history roots. Preserve exact attribution with a genuine owner
   durability proof or the authorized minimal immutable owner data observation
   through facade. Memory None attribution would vacuously weaken the assertion.
2. `recovered_related_ordering_uses_each_exact_roots_schema_contract` changes v1
   name contract to v2 display through private reconfiguration. Basis-only public
   transition refuses TargetContractMismatch; the real Home program target-step
   path above is the owner. Port through it or retain exact genuine owner recovery
   evidence. No arbitrary setter, contract default or removed old/current contrast.
3. Structural identity follow-up: the exact setter implementation in
   runtime/state/runtime_state/test_support.rs:34-53 writes arena.extra directly
   using EntityExtra::from_unbudgeted_parts, outside original publication. This
   corrects the initial missing-producer suspicion: its recovered arbitrary
   sidecar claim is private resident-only state, class2/C2. Existing Memory
   inspection comparison siblings remain unchanged; ordinary inspection truth
   bundle recovery is HAPPY. A public fingerprint writer is not warranted.
4. Selected ACK mixes actual DATA availability/stop/resume/later-write behavior
   with private source/directory pointers, exact partial cursor, acknowledged
   catalog frontier and Weak basis/root release. Split those observations; keep
   the named public Complete/ACK proof and restore exact census/custody in C2.
   Neither caller cancellation nor observer drop is completed acknowledgement.
5. Cache eviction is setup, not a missing capability. Use actual cold external
   source/reopen for fallback CDC semantic ports, retaining the explicit
   DurableCanonicalRecovery choice. Exact no-covered-body demand is C2. Warm
   Memory reads do not prove fallback.
6. Closed wire/header/proof grammar decoder negatives are owner value tests,
   equivalent to earlier packet serde owner units. Coherent Memory-produced
   originals are honest inputs for those exact predicates. Actual resealed cold
   source corruption, failed-runtime-before-effects and scripted provider fates
   remain C4; do not turn them into value-only success tests.

## Named roots

### `history/retention/component_cut_tests.rs`

- `native_custody_correction_cross_lane_acquire_release_has_one_actual_capture_cut` (line 20): **C2**. Exact capture-cut mutex, lane inventory and original-root custody; public reads perturb custody.
- `native_custody_correction_current_lane_inventory_is_collected_inside_actual_cut` (line 93): **C2**. Exact capture-cut mutex, lane inventory and original-root custody; public reads perturb custody.

### `history/commit/envelope_residency_tests.rs`

- `an_indexed_envelope_remains_cold_until_explicit_readmission_and_loads_once` (line 80): **C2**. Exact private cell residency/read count/pointer/backing lifetime.
- `an_indexed_envelope_storage_failure_is_typed_and_sticky` (line 103): **C4**. Damaged cold source and exact sticky refusal/read count.
- `relinking_other_envelopes_preserves_an_index_even_while_its_body_is_in_flight` (line 134): **C2**. Exact private cell residency/read count/pointer/backing lifetime.
- `settlement_releases_a_journal_flight_but_preserves_memory_history_and_position` (line 168): **C2**. Exact private cell residency/read count/pointer/backing lifetime.
- `positioned_demand_owns_its_result_across_journal_settlement` (line 230): **C2**. Exact private cell residency/read count/pointer/backing lifetime.
- `unpositioned_memory_envelopes_and_cold_digests_refuse_explicitly` (line 250): **OWNER**. Artifact digest/positioning legality; actual Native Memory envelope suffices; keep exact negative twins.

### `history/commit/envelope_residency/memory_publication_tests.rs`

- `memory_original_frame_survives_export_checkpoint_and_last_alias` (line 18): **UNCHANGED**. Already genuine Memory original-frame/quota/tuple owner unit; no adapter dependency in this root.
- `memory_frame_quota_and_cancel_refuse_before_position_or_head` (line 98): **UNCHANGED**. Already genuine Memory original-frame/quota/tuple owner unit; no adapter dependency in this root.
- `memory_frame_preserves_original_payload_and_scalar_tuple_observations` (line 234): **UNCHANGED**. Already genuine Memory original-frame/quota/tuple owner unit; no adapter dependency in this root.
- `journal_publication_keeps_journal_placement_without_memory_frame` (line 309): **C2**. Exact private journal placement, zero payload bytes, same source owner/pointer; exported bytes alone do not prove placement.

### `commit_strategies/facade_tests/replay_recovery_failures.rs`

- `replay_commit_reports_strategy_executor_unavailable_when_recovered_runtime_lacks_executor` (line 5): **C4**. Recovery plan executor registry deliberately removed/replaced with rejecting executor; exact strategy mismatch and unchanged head.
- `replay_commit_reports_strategy_execution_failure_when_recovered_executor_rejects` (line 62): **C4**. Recovery plan executor registry deliberately removed/replaced with rejecting executor; exact strategy mismatch and unchanged head.

### `history/commit/artifact_tests.rs`

- `fresh_scalar_commit_seals_exact_native_lifecycle_and_field_touches` (line 14): **HAPPY**. Actual ordinary authoring/index registration; public canonical descriptive_touches expose lifecycle/field/index exact assertions.
- `installed_field_index_commit_touch_carries_native_old_and_new_keys` (line 62): **HAPPY**. Actual ordinary authoring/index registration; public canonical descriptive_touches expose lifecycle/field/index exact assertions.
- `recovery_relinks_an_already_sealed_payload_without_reencoding_or_weakening_root_checks` (line 108): **OWNER+C2**. RootLinkage/identity/parentage/roots/digest legality stays Native Memory owner unit; exact Arc residency/no-reencode sharing C2.
- `canonical_artifact_accounting_excludes_nested_diagnostic_perturbation` (line 162): **C2**. Exact allocation inventories and nested diagnostics/cache perturbation; preserve payload independence negative twin.
- `populated_derived_index_artifacts_only_change_the_optional_cache_lane` (line 268): **C2**. Exact allocation inventories and nested diagnostics/cache perturbation; preserve payload independence negative twin.

### `history/commit/artifact_tests/strategy_readmission.rs`

- `strategy_input_clone_retains_only_its_original_decoded_bytes_grant` (line 10): **C2**. Exact retained grant, pointer sharing, quote and last-drop refund; not equivalent to public byte read.
- `strategy_descriptor_clone_retains_only_its_original_decoded_locator_grant` (line 44): **C2**. Exact retained grant, pointer sharing, quote and last-drop refund; not equivalent to public byte read.

### `history/commit/artifact_tests/shared_readmission.rs`

- `shared_readmitted_envelope_clones_keep_the_original_allocation_and_grant_until_last_drop` (line 10): **C2**. Exact retained grant, pointer sharing, quote and last-drop refund; not equivalent to public byte read.
- `actual_weak_position_retains_only_the_fixed_original_control_bill_until_last_drop` (line 33): **C2**. Exact retained grant, pointer sharing, quote and last-drop refund; not equivalent to public byte read.
- `embedded_checkpoint_preserves_ignored_unknown_nested_fields_without_retaining_their_heaps` (line 117): **C2**. Exact retained grant, pointer sharing, quote and last-drop refund; not equivalent to public byte read.

### `publication/cdc/planning/checkpoint_resolution/tests.rs`

- `subscriber_stream_recovers_from_durable_canonical_envelopes_when_patch_stream_index_outlives_envelope` (line 9): **HAPPY**. Public exact resumed suffix and DurableCanonicalRecovery from genuinely cold external originals; replace private eviction setup, not evidence.

### `durability/log/native_checkpoint_codec/tests.rs`

- `member_directory_preserves_divergent_sibling_roots_and_pinned_reader` (line 10): **HAPPY+OWNER**. External real divergent-root/pinned-read reopen; private directory member dedup stays codec owner value unit.
- `current_manifest_deduplicates_equal_region_members_and_round_trips` (line 148): **OWNER**. Closed wire encode/decode/grammar/accounting unit; coherent Native Memory capture is legitimate value source, no physical reopen claim.
- `current_directory_rejects_damaged_duplicate_and_missing_members` (line 191): **OWNER**. Closed wire encode/decode/grammar/accounting unit; coherent Native Memory capture is legitimate value source, no physical reopen claim.
- `native_capture_sections_count_current_directory_values` (line 236): **OWNER**. Closed wire encode/decode/grammar/accounting unit; coherent Native Memory capture is legitimate value source, no physical reopen claim.
- `chk6_and_retired_inner_partition_grammars_are_refused` (line 316): **OWNER**. Closed wire encode/decode/grammar/accounting unit; coherent Native Memory capture is legitimate value source, no physical reopen claim.

### `durability/log/native_checkpoint_codec/manifest_validation/components/tests.rs`

- `recorded_component_checkpoint_header_tamper_refuses_without_defaults_or_foreign_binding` (line 59): **OWNER**. Closed header grammar corruption table; coherent Memory capture value and exact CorruptCheckpoint/component diagnosis.

### `durability/log/native_checkpoint_codec/manifest_integrity_tests.rs`

- `shared_members_with_inconsistent_leaf_facts_refuse_the_manifest` (line 5): **OWNER**. Closed leaf-fact manifest consistency and sparse Page mapping validation; preserve all negative value twins.
- `sparse_page_membership_accepts_late_holes_and_refuses_duplicate_or_forged_ids` (line 75): **OWNER**. Closed leaf-fact manifest consistency and sparse Page mapping validation; preserve all negative value twins.

### `durability/log/native_checkpoint_codec/kind_authentication_tests.rs`

- `checkpoint_readmission_refuses_retired_integer_array_digest_before_restore` (line 13): **OWNER**. Retired digest representation is closed byte grammar decoder refusal, Memory capture value suffices.
- `forged_cross_region_kind_commitment_is_refused_before_member_reads` (line 55): **C4**. Resealed forged kind manifest in actual cold source; runtime StorageContentMismatch and zero member reads.

### `durability/log/native_checkpoint_codec/inspection/tests.rs`

- `inspection_compares_every_native_model_value_as_original_bytes` (line 130): **OWNER**. Inspection/closed grammar model-field validation unit; coherent Native Memory-produced values, preserve ignored/unknown/frontier negatives.
- `inspection_refuses_unknown_root_allocator_and_row_fields` (line 179): **OWNER**. Inspection/closed grammar model-field validation unit; coherent Native Memory-produced values, preserve ignored/unknown/frontier negatives.
- `restore_refuses_unknown_fields_before_publishing_any_model_state` (line 204): **C4**. Mutated fields applied to runtime restore; exact CorruptCheckpoint and unchanged model bytes for every owner.
- `the_live_census_includes_divergent_retained_branch_roots` (line 271): **HAPPY+OWNER**. Public restored divergent-root reads; exact inspection census values stay inspection owner if sealed caller rules prevent external inspection. Do not add census door.
- `burned_symbol_frontier_does_not_hide_published_symbol_damage` (line 304): **OWNER**. Inspection/closed grammar model-field validation unit; coherent Native Memory-produced values, preserve ignored/unknown/frontier negatives.

### `durability/data/native_checkpoint/tests/selected_ack.rs`

- `selected_ack_partial_transfer_preserves_later_cells_and_resumes_same_capture` (line 24): **HAPPY+C2**. Actual selected ACK stop/resume/current-write retention is HAPPY; exact per-cell source/directory pointer/cursor/Weak/frontier observations C2.
- `selected_ack_exposes_backed_data_before_retirement_and_releases_holds_only_on_completion` (line 119): **HAPPY+C2**. Actual selected ACK stop/resume/current-write retention is HAPPY; exact per-cell source/directory pointer/cursor/Weak/frontier observations C2.

### `durability/data/native_checkpoint/component_custody/tests.rs`

- `native_custody_correction_delayed_canonical_selection_retains_actual_root_and_interrupted_ack` (line 101): **HAPPY+C2**. Actual delayed selection, interruption/resume/readability HAPPY; same Weak basis/root/schema pointer custody release C2. Preserve both delayed_selection helper branches.
- `native_custody_correction_delayed_empty_selection_retains_actual_schema_and_interrupted_ack` (line 105): **HAPPY+C2**. Actual delayed selection, interruption/resume/readability HAPPY; same Weak basis/root/schema pointer custody release C2. Preserve both delayed_selection helper branches.
- `native_custody_correction_last_actual_capture_abandonment_releases_shared_root_custody` (line 109): **C2**. Same original captured basis Weak lifetime after lease release, movement and abandonment.

### `runtime/state/subsystems/visibility/snapshot_handles/published_registry/original_cleanup/tests.rs`

- `result_snapshot_cleanup_registry_contention_preserves_exact_cell_and_competing_task` (line 10): **C2**. Held private registry lock, exact occupied/count/tombstone and original-root Weak survival/reclamation.

### `storage/residency/cross_region_forgery_tests.rs`

- `rehashed_cross_region_kind_forgery_is_refused_by_the_canonical_root` (line 184): **C4**. Resealed cross-region kind forgery through actual cold source/runtime recovery; exact StorageContentMismatch.

### `durability/data/checkpoint_images.rs`

- `streamed_branch_root_image_digest_matches_buffered_wire_for_divergent_roots` (line 319): **OWNER**. Streamed versus independent buffered digest on divergent captured values; actual Memory roots or pure value fixture, preserve altered-value contrast.

### `durability/authority/runtime_rebuild/checkpoint_restore_tests.rs`

- `equal_exact_root_reuses_the_readmitted_partition_substrate` (line 12): **C2**. Exact private first-generation substrate Arc reuse.
- `native_restore_work_counts_revised_history_sibling_roots_and_partition_images` (line 35): **HAPPY+C2**. Public original pinned read remains; exact byte/work/readmitted/reused count and history growth claims C2.
- `shared_restored_substrate_keeps_a_pinned_reader_exact_after_next_write` (line 132): **HAPPY**. External actual recovery preserves pinned/current reads and canonical identity/parentage/roots/payload digest.
- `divergent_sibling_is_not_reused_as_the_storage_mirror` (line 179): **HAPPY+C2**. Correct restored entity count HAPPY; exact mirror/root sharing/nonsharing C2.
- `divergent_mirror_partition_rebuilds_cross_partition_adjacency_after_reuse` (line 235): **C2**. Exact partition generation/relation arena pointer reuse after divergent mirror rebuild.
- `recovered_main_and_sibling_catalogs_reuse_exact_canonical_payloads` (line 299): **HAPPY**. External actual recovery preserves pinned/current reads and canonical identity/parentage/roots/payload digest.
- `equal_image_cannot_bypass_global_contract_readmission` (line 348): **C4**. Privately altered checkpoint contract then exact aspect-readmission CorruptCheckpoint refusal.

### `runtime/state/subsystems/services/symbol_table/reservations/tests.rs`

- `publication_excludes_abandoned_preview_meanings_adopted_for_reading` (line 4): **UNCHANGED**. Already Native pending-meaning owner units; durable sibling does not make all roots provider-dependent.
- `concurrent_pending_meanings_deduplicate_and_publish_only_the_accepted_candidate` (line 33): **UNCHANGED**. Already Native pending-meaning owner units; durable sibling does not make all roots provider-dependent.
- `abandoning_the_last_custodian_spends_the_identity_and_removes_pending_text` (line 67): **UNCHANGED**. Already Native pending-meaning owner units; durable sibling does not make all roots provider-dependent.
- `independent_admission_uses_the_pending_identity` (line 93): **UNCHANGED**. Already Native pending-meaning owner units; durable sibling does not make all roots provider-dependent.
- `parallel_candidates_share_pending_meaning_without_publishing_it` (line 119): **UNCHANGED**. Already Native pending-meaning owner units; durable sibling does not make all roots provider-dependent.
- `checkpoint_recovery_preserves_abandoned_symbol_identity_frontiers` (line 147): **C2**. Private pending symbol burns identity; actual recovery/private next admission and abandoned text absence. Public write perturbs frontier.
- `canonical_symbol_sealing_and_cross_owner_adoption_refuse_before_publication` (line 191): **UNCHANGED**. Already Native pending-meaning owner units; durable sibling does not make all roots provider-dependent.

### `branch/root_tests.rs`

- `relink_rejects_schema_authority_mutation` (line 35): **OWNER**. Actual implementation hash/relink legality; honest Memory-sealed root value, exact schema/visibility and every tuple axis, no copied predicate or mutable public root.
- `relink_rejects_visibility_tuple_mutation` (line 60): **OWNER**. Actual implementation hash/relink legality; honest Memory-sealed root value, exact schema/visibility and every tuple axis, no copied predicate or mutable public root.
- `visibility_commitment_binds_every_visible_tuple_axis` (line 86): **OWNER**. Actual implementation hash/relink legality; honest Memory-sealed root value, exact schema/visibility and every tuple axis, no copied predicate or mutable public root.

### `tests/transactions/core/owner_lifecycle_phase3b.rs`

- `phase3b_owner_drop_during_off_thread_settlement_completes` (line 22): **HAPPY+OWNER**. Actual close-vs-settlement HAPPY. Deterministic private settlement pause/admission probe stays owner scheduling unit on genuine Memory, paired with external actual settle/close journey; not storage fault.
- `phase3b_settlement_admitted_after_owner_drop_is_denied` (line 95): **HAPPY+OWNER**. Actual close-vs-settlement HAPPY. Deterministic private settlement pause/admission probe stays owner scheduling unit on genuine Memory, paired with external actual settle/close journey; not storage fault.
- `phase3b_recovery_replacement_closes_the_prior_runtimes_ports` (line 169): **HAPPY**. Real recovery replaces runtime; public prior settlement service reports exact OwnerUnavailable instance, not merely a newly constructed unrelated runtime.
- `phase3b_removed_lifecycle_premises_stay_removed` (line 208): **UNCHANGED**. Static source-law unit; no runtime/provider.

### `tests/transactions/core/pending_settlement_recovery.rs`

- `dropping_the_performed_witness_leaves_settlement_recoverable` (line 11): **HAPPY+C2**. Public drop/repair/repeated receipt/exactly-once durable append HAPPY; exact abandoned capability/pending/snapshot census C2. retains_pending_settlement observes known identity, not arbitrary global count.
- `repeated_terminal_settlement_returns_one_receipt_and_one_durable_append` (line 70): **HAPPY+C2**. Public drop/repair/repeated receipt/exactly-once durable append HAPPY; exact abandoned capability/pending/snapshot census C2. retains_pending_settlement observes known identity, not arbitrary global count.
- `repair_and_witness_settlement_execute_exactly_once` (line 100): **HAPPY+C2**. Public drop/repair/repeated receipt/exactly-once durable append HAPPY; exact abandoned capability/pending/snapshot census C2. retains_pending_settlement observes known identity, not arbitrary global count.
- `owner_loss_retires_every_pending_settlement_exactly_once` (line 135): **UNCHANGED**. Already Memory registry owner unit; no adapter-causal instance.
- `interruption_after_movement_retains_its_pending_settlement` (line 168): **UNCHANGED**. Already Memory injected interruption unit; no adapter instance. Preserve exact late interruption/pending claims if separately moved.

### `tests/transactions/core/authored_write_history/indexed/recovery.rs`

- `index_rebuild_requires_live_verified_canonical_evidence` (line 5): **HAPPY+OWNER+C2**. Public index incomplete/fabricated selectors, discard/rebuild, exact no-op cold history and subsequent live coverage HAPPY; zero-budget rebuild C2; optional-field/unverified description serde stays owner value unit.

### `tests/transactions/core/owner_service_phase3.rs`

- `phase3_settlement_receivers_take_a_shared_borrow` (line 13): **UNCHANGED**. Compile-only receiver signature assertion; no provider constructed.
- `phase3b_owner_authority_and_owner_services_keep_their_receivers` (line 41): **UNCHANGED**. Compile-only receiver signature assertion; no provider constructed.
- `phase3_settlement_port_is_a_cloneable_shared_borrow_service` (line 59): **UNCHANGED**. Already Native Memory owner/service operation; no durable adapter used by root.
- `phase3_settlement_port_denies_after_its_runtime_owner_closes` (line 96): **UNCHANGED**. Already Native Memory owner/service operation; no durable adapter used by root.
- `phase3_settlement_port_settles_a_performed_publication_from_another_thread` (line 132): **HAPPY**. Existing public settlement_port/perform/settle/repair and actual transaction; external original medium append identity/count, independent public head/parent receipts.
- `phase3_settlement_port_repairs_a_lost_witness_by_identity` (line 168): **HAPPY**. Existing public settlement_port/perform/settle/repair and actual transaction; external original medium append identity/count, independent public head/parent receipts.
- `phase3_foreign_settlement_denial_preserves_source_recovery` (line 205): **HAPPY**. Existing public settlement_port/perform/settle/repair and actual transaction; external original medium append identity/count, independent public head/parent receipts.

### `tests/transactions/core/authored_write_history/adjacency/recovery.rs`

- `adjacency_occurrence_identities_rebuild_from_cold_canonical_history` (line 5): **HAPPY**. Ordinary endpoint change/no-op/delete and public discard/rebuild; exact outgoing/incoming commit-ID sequences after actual reopen.

### `tests/lineage/resolution.rs`

- `historical_lineage_resolution_follows_replace_events` (line 18): **UNCHANGED**. Already Memory ordinary lineage/aspect history/traces; do not absorb unrelated siblings.
- `refused_durable_publication_leaves_no_replacement_lineage_behind` (line 106): **C4**. Actual ProvenNoEffect provider script and exact typed refused replacement leaves graph/head/resolution unchanged.
- `historical_lineage_resolution_does_not_scan_unrelated_branch_events` (line 190): **UNCHANGED**. Already Memory ordinary lineage/aspect history/traces; do not absorb unrelated siblings.
- `lineage_aspect_history_keeps_origin_events_and_marks_resolution_context` (line 262): **UNCHANGED**. Already Memory ordinary lineage/aspect history/traces; do not absorb unrelated siblings.

### `tests/transactions/core/publication_stream_order.rs`

- `branch_head_index_moves_at_cutover_before_reverse_settlement` (line 5): **UNCHANGED**. Already Native Memory publication-order owner test; private index/gate assertions not made durable by sibling.
- `preparation_order_does_not_reserve_a_subscriber_stream_gap` (line 82): **UNCHANGED**. Already Native Memory publication-order owner test; private index/gate assertions not made durable by sibling.
- `checkpoint_tail_recovery_follows_publication_order_with_exact_reserved_identities` (line 164): **HAPPY+C2**. Public real checkpoint+tail positions, reserved IDs, branch reads and continued write HAPPY; exact private settlement route-visit count C2.

### `tests/transactions/core/authored_write_history/adjacency/audit.rs`

- `audit_lifecycle_coverage_survives_unrelated_history_discard_and_durable_recovery` (line 14): **HAPPY**. Actual audit policy, support/UnsupportedVisibility, canonical touches without fabricated revision, selector history and exact original payload/digest after reopen.

### `tests/transactions/core/publication_settlement.rs`

- `direct_publication_settles_before_a_child_can_acknowledge` (line 6): **HAPPY**. Actual performed visibility, unsettled fork/child refusal, explicit settle, exact parent/child durable positions, checkpoint+tail reopen and continuation. Medium originals/public plan tail replace private log access.

### `branch/root_authoritative_payload_tests.rs`

- `content_old_retained_contract_cannot_hide_a_reference_after_current_binding_is_removed` (line 13): **OWNER**. Sealed-root relink legality for strategy/merge/schema/content payload variants; actual Memory original suffices for local predicate, no claim of physical recovery.
- `strategy_artifacts_are_bound_into_the_committed_root` (line 150): **OWNER**. Sealed-root relink legality for strategy/merge/schema/content payload variants; actual Memory original suffices for local predicate, no claim of physical recovery.
- `merge_execution_authority_is_bound_into_the_committed_root` (line 174): **OWNER**. Sealed-root relink legality for strategy/merge/schema/content payload variants; actual Memory original suffices for local predicate, no claim of physical recovery.
- `schema_transition_descriptors_are_each_bound_into_the_committed_root` (line 225): **OWNER**. Sealed-root relink legality for strategy/merge/schema/content payload variants; actual Memory original suffices for local predicate, no claim of physical recovery.

### `durability/canonical_commit_artifact/retained_export/skip_tests.rs`

- `seeking_past_an_acknowledged_index_prefix_never_demands_its_journal_bodies` (line 35): **C2**. Exact skipped prefix body demand bound/read count1; public export bytes are companion, not equivalent cost proof.

### `transactions/data/outcomes/commit_result/snapshot_cleanup_tests.rs`

- `result_snapshot_cleanup_refuses_an_actual_admitted_runtime_copy` (line 10): **OWNER**. Actual admitted service-copy tenure is constructed privately then exact NotOwner and original Arc/result cleanup are checked. This is owner tenure legality, not durable source semantics. Keep honest Native Memory-origin result and exact crate-private admitted-copy input; no public runtime-copy constructor.
- `native_custody_correction_result_cleanup_is_explicit_idempotent_and_keeps_external_read` (line 61): **HAPPY+C2**. Public cleanup/idempotency/external-read close refusal/OwnerUnavailable; exact private published handle and retention census C2.
- `native_custody_correction_result_release_cell_tracks_public_release_closeout_and_transfer` (line 106): **HAPPY+OWNER**. Public Released/AlreadyReleased actual cleanup and closeout are HAPPY; exact internal transfer_release_obligation accounting stays result-owner unit on genuine Memory-origin result if no public shipping transfer path. Do not expose private transfer control.
- `native_custody_correction_result_cleanup_refuses_foreign_and_private_altered_pairing` (line 149): **HAPPY+C4+C2**. Public foreign owner refusal; private altered result registration pairing C4; exact handle/retention inventory C2.

### `tests/inspection/structural_identity.rs`

- `structural_identity_comparison_only_uses_fingerprint_truth` (line 5): **UNCHANGED**. Already Memory inspection owner units, not adapter-causal merely because sibling is durable.
- `structural_identity_evidence_exposes_declared_fingerprint_and_lineage_for_entities_only` (line 29): **UNCHANGED**. Already Memory inspection owner units, not adapter-causal merely because sibling is durable.
- `structural_identity_comparison_distinguishes_equal_mismatch_and_family_mismatch` (line 80): **UNCHANGED**. Already Memory inspection owner units, not adapter-causal merely because sibling is durable.
- `structural_identity_query_is_family_scoped_and_entity_only` (line 154): **UNCHANGED**. Already Memory inspection owner units, not adapter-causal merely because sibling is durable.
- `structural_identity_historical_scope_does_not_leak_reused_slot_sidecars` (line 203): **UNCHANGED**. Already Memory inspection owner units, not adapter-causal merely because sibling is durable.
- `structural_identity_recovery_preserves_current_evidence_and_queries` (line 259): **C2**. Exact privately authored resident-only sidecars: setter directly writes from_unbudgeted_parts into arena.extra outside publication/original authority. Coordinator class2 applies. Restore exact evidence/query/recovery assertions with setter helper; do not invent public fingerprint writer or substitute default empty data.
- `inspection_truth_bundle_recovery_parity_holds_for_current_and_historical_surfaces` (line 342): **HAPPY**. Ordinary entity/relation actual reopen and full current/historical inspection bundle parity; retain all bundle fields, not only entities.

### `tests/transactions/core/aggregate_identity/logical_components/checkpoint.rs`

- `physical_membership_recipe_and_alternating_derived_catalog_restore_from_real_checkpoints` (line 13): **HAPPY**. Actual schema-derived membership/handles/domain and foreign/schema mismatch before effects; public pinned reads. Exact original attribution subclaim needs observation noted below.
- `changed_configured_domain_cannot_replace_the_actual_recovered_root_or_its_handles` (line 124): **HAPPY**. Actual schema-derived membership/handles/domain and foreign/schema mismatch before effects; public pinned reads. Exact original attribution subclaim needs observation noted below.

### `tests/query/indexes/related_entity_schema_recovery.rs`

- `recovered_related_ordering_uses_each_exact_roots_schema_contract` (line 14): **HAPPY-GAP**. Old v1 name ordering versus current v2 display executable contract after recovery. Public schema basis transition rejects changed contracts; real program installation or owner proof needed, no registry setter door.

### `tests/query/indexes/discard_rebuild/checkpoint.rs`

- `discarded_ordering_stays_cold_after_native_checkpoint_restore_then_rebuilds` (line 4): **HAPPY**. Public registration/build/discard/restore preserves definition, absent generation/ExactGenerationUnavailable, exact explicit rebuilt entries/page and continued write; explicit receiver medium.

### `branch/owner_services/materialization_port_tests.rs`

- `durability_replays_suspended_and_restored_materialization_roots` (line 19): **HAPPY**. Public suspend/rematerialize services, actual checkpoint/recovery, MaterializationUnavailable and exact entity/relation IDs.

### `durability/authority/fresh_source/tests.rs`

- `fresh_source_keeps_actual_runtime_root_and_installed_schema_for_complete` (line 23): **HAPPY+C2**. Public installed schema/unused constraints in Complete HAPPY; exact source/root/schema Arc identity C2.
- `fresh_source_refuses_foreign_and_nonempty_actual_sources_without_rebuild` (line 168): **HAPPY**. Public fresh admission/installation receipt validation against actual source/runtime; exact foreign/nonempty/stale/cancel/selected-empty refusals.
- `fresh_receipt_refuses_actual_movement_and_preserves_original_cancellation` (line 196): **HAPPY**. Public fresh admission/installation receipt validation against actual source/runtime; exact foreign/nonempty/stale/cancel/selected-empty refusals.
- `fresh_source_refuses_real_selected_empty_checkpoint` (line 242): **HAPPY**. Public fresh admission/installation receipt validation against actual source/runtime; exact foreign/nonempty/stale/cancel/selected-empty refusals.

### `durability/authority/checkpoint_transition/tests.rs`

- `acknowledged_transition_admits_only_its_exact_successor_and_releases_snapshot` (line 59): **HAPPY+C2**. Public recovered authority transition, exact predecessor/successor/stale/foreign effects; medium originals prove append count; private snapshot census C2.
- `foreign_recovery_and_foreign_candidate_are_refused_before_effects` (line 113): **HAPPY+C2**. Public recovered authority transition, exact predecessor/successor/stale/foreign effects; medium originals prove append count; private snapshot census C2.
- `moved_source_and_stale_candidate_cannot_extend_recovery_lineage` (line 177): **HAPPY+C2**. Public recovered authority transition, exact predecessor/successor/stale/foreign effects; medium originals prove append count; private snapshot census C2.
- `refused_durable_transition_returns_its_authority_and_moves_nothing` (line 212): **C4**. Provider ProvenNoEffect/Unresolved script; exact typed refusal/fate, unchanged effects, returned authority and retry.
- `unresolved_durable_transition_installs_nothing_and_names_its_fate` (line 254): **C4**. Provider ProvenNoEffect/Unresolved script; exact typed refusal/fate, unchanged effects, returned authority and retry.

### `durability/authority/checkpoint_complete/tests/empty_schema.rs`

- `complete_empty_archive_and_initial_installation_preserve_unused_original_schema` (line 9): **HAPPY+C2**. Public Complete original empty schema/unused constraint recovery HAPPY; exact old Weak and backing charge lifetime C2.
- `complete_refuses_recovered_empty_without_original_birth_authority` (line 130): **HAPPY**. Actual byte-only standalone restore lacks birth authority; existing OriginalAuthorityUnavailable guard must remain, no fresh proof substitution.

### `durability/authority/checkpoint_complete/tests/partial_aspect_update.rs`

- `complete_preserves_partial_update_without_demanding_untouched_aspect_reference` (line 11): **HAPPY+C2**. Actual partial name/lifecycle update/capture/recovery HAPPY; private resident binding/revision/symbol inventory and axes C2.

### `durability/authority/checkpoint_complete/tests/pinned_delete.rs`

- `complete_fold_matches_original_relation_delete_with_snapshot_and_replay_pins` (line 9): **HAPPY+C2**. Public old snapshot/replay relation read and current deletion HAPPY; exact root pointer/arena pin counters/retired-at/axes C2.

### `durability/authority/checkpoint_concurrency_tests.rs`

- `completing_an_older_capture_cannot_replace_the_compacted_transition_basis` (line 9): **HAPPY**. Freeze actual public prepared Complete/native capture, publish/fork later, select via real medium; preserve exact coverage/tail/recovered head. No new capture authority door.
- `checkpoint_reconstruction_does_not_exclude_publication_after_capture` (line 43): **HAPPY**. Freeze actual public prepared Complete/native capture, publish/fork later, select via real medium; preserve exact coverage/tail/recovered head. No new capture authority door.

### `tests/schema/certification.rs`

- `schema_evolution_cdc_contract_test` (line 60): **HAPPY**. Basis-only public schema transitions preserve exact descriptor/CDC/continuation digests across actual reopen.

### `tests/publication/patch/batching.rs`

- `patch_stream_resume_batches_commits_without_duplication` (line 5): **UNCHANGED**. First two roots already Memory pagination/resume request validation.
- `patch_stream_rejects_unknown_resume_position` (line 44): **UNCHANGED**. First two roots already Memory pagination/resume request validation.
- `patch_stream_uses_durable_canonical_history_when_retained_envelope_is_missing` (line 69): **HAPPY**. Exact patch positions/count/resume from actual cold external canonical originals; private eviction setup replaced, no warm Memory substitute.

### `branch/coordination/managed_fork_cleanup.rs`

- `managed_fork_refusal_and_success_drop_lookup_outside_native_coordination` (line 92): **C4+C2**. Provider transition refusal then success, exact lookup-drop under coordination lock and charged backing; actual refusal campaign C4 and private destructor/lock/funding observations C2.

### `tests/publication/cdc/certification/schema_boundary_parity.rs`

- `cdc_certification_schema_boundary_continuation_is_explained_and_counted` (line 12): **UNCHANGED**. Already Memory semantic diagnostic/counter owner unit; no adapter instance in root.
- `diff_cdc_truth_parity_test` (line 79): **HAPPY+C2**. Actual schema CDC/diff/diagnostic parity HAPPY; exact reset/work/zero replay evaluation counters C2. Basis-only transition already public.

### `tests/publication/cdc/certification/durable_recovery.rs`

- `cdc_certification_durable_recovery_matches_head_and_midstream_consumers` (line 20): **HAPPY**. Actual durable head/midstream consumers, seeded ordinary commits/checkpoint/reopen and exact suffix after retention; public scenario authoring and explicit actual medium.
- `cdc_certification_persisted_seeded_matrix_survives_checkpoint_compaction_and_recovery` (line 87): **HAPPY**. Actual durable head/midstream consumers, seeded ordinary commits/checkpoint/reopen and exact suffix after retention; public scenario authoring and explicit actual medium.
- `cdc_certification_retention_truncation_recovers_exact_suffix_from_old_checkpoint` (line 128): **HAPPY**. Actual durable head/midstream consumers, seeded ordinary commits/checkpoint/reopen and exact suffix after retention; public scenario authoring and explicit actual medium.

### `tests/publication/cdc/subscriber_failures/durable_descriptor_version.rs`

- `subscriber_stream_rejects_durable_only_checkpoint_with_descriptor_version_mismatch` (line 6): **OWNER+HAPPY**. Incoherent continuation constructed by private helper stays exact owner mismatch value unit; pair with public stale/unsupported request on actual cold source where admitted. No public incoherent constructor.

### `tests/publication/cdc/subscriber_failures/checkpoint_identity.rs`

- `subscriber_stream_rejects_schema_unsupported_checkpoint` (line 5): **UNCHANGED**. Memory request validation siblings unaffected; only missing durable coverage root uses adapter.
- `subscriber_stream_rejects_checkpoint_without_history_or_durable_coverage` (line 26): **HAPPY**. Actually empty durable source and public resume checkpoint42; exact DurableCoverageGap and Replay diagnostic scopes, no injection.
- `subscriber_stream_rejects_checkpoint_with_mismatched_contract_identity` (line 46): **UNCHANGED**. Memory request validation siblings unaffected; only missing durable coverage root uses adapter.

### `tests/publication/cdc/replay_parity.rs`

- `subscriber_stream_matches_patch_stream_for_committed_history` (line 5): **UNCHANGED**. Already Memory ordinary subscriber/patch parity; no adapter instance.
- `durable_source_subscriber_stream_matches_recovered_runtime_patch_stream` (line 32): **HAPPY**. Actual durable-only source and recovered patch parity including later original dependency; genuine cold source replaces cache eviction.
- `subscriber_stream_keeps_patch_parity_when_later_retained_envelope_is_missing` (line 80): **HAPPY**. Actual durable-only source and recovered patch parity including later original dependency; genuine cold source replaces cache eviction.

### `tests/publication/cdc/recovery.rs`

- `subscriber_stream_recovers_from_durable_canonical_envelopes_when_checkpoint_is_not_in_memory` (line 4): **HAPPY**. Public exact suffix/durable latest checkpoint after cold original checkpoint/head; genuine external originals, no private eviction hook.
- `subscriber_stream_reports_durable_latest_available_checkpoint_when_latest_head_is_not_in_memory` (line 39): **HAPPY**. Public exact suffix/durable latest checkpoint after cold original checkpoint/head; genuine external originals, no private eviction hook.

### `tests/complexity/contracts/commit_budgets/schema_continuity_boundaries.rs`

- `complexity_budget_schema_transition_classification_is_changed_atom_bounded` (line 5): **C2**. Exact atom/subtree/descriptor/continuation/replay-layer counters and bounds; companion semantic parity does not prove these counters.
- `complexity_budget_subscriber_resume_continuity_is_boundary_local` (line 57): **C2**. Exact atom/subtree/descriptor/continuation/replay-layer counters and bounds; companion semantic parity does not prove these counters.
- `complexity_budget_milestone5_closeout_keeps_schema_cdc_and_recovery_boundary_local` (line 110): **C2**. Exact atom/subtree/descriptor/continuation/replay-layer counters and bounds; companion semantic parity does not prove these counters.

### `tests/complexity/contracts/merge_budgets.rs`

- `complexity_budget_merge_planning_reports_request_shaped_work` (line 10): **C2**. Exact planning/execution/elapsed/scanned/admitted work counters. Public emitted entities/mutations are companion HAPPY claims; split exact accounting.
- `complexity_budget_merge_execution_reports_admitted_records_and_emitted_mutations` (line 81): **C2**. Exact planning/execution/elapsed/scanned/admitted work counters. Public emitted entities/mutations are companion HAPPY claims; split exact accounting.

### `tests/domains/cad/topology_identity_survival.rs`

- `topology_identity_survival_preserves_reidentification_truth_across_recovery` (line 10): **HAPPY**. Ordinary replacement lineage exact historical resolution digest/basis and resolved/event IDs survive actual recovery.

### `tests/domains/fintech/workflows/persistence/checkpoint_fallback.rs`

- `fintech_recovery_refuses_a_damaged_checkpoint_instead_of_losing_truth` (line 6): **C4**. Actual filed checkpoint deliberately damaged; exact CorruptCheckpoint without lost truth.

## Helper-only direct hits

- `commit_strategies/facade_tests/native_strategy_fixtures.rs`
- `durability/medium.rs`
- `durability/medium/binding.rs`
- `tests/inspection/mod.rs`
- `tests/transactions/core/aggregate_identity/fixtures.rs`
- `tests/transactions/core/aggregate_identity/components/fixtures.rs`
- `tests/transactions/core/aggregate_identity/cold_recovery_fixture.rs`
- `tests/query/indexes/mod.rs`
- `durability/access/recovery_plan_entrypoint.rs`
- `durability/access/mod.rs`
- `durability/access/medium_recovery_plan.rs`
- `tests/harness/scenario/runner/property_runtime.rs`
- `tests/harness/fixtures/runtime.rs`
- `tests/domains/fintech/failure_injection/durability.rs`
- `tests/domains/fintech/actions/recovery.rs`
- `tests/domains/fintech/fixture/mod.rs`

Counts: C2 26, C4 11, C4+C2 1, HAPPY 34, HAPPY+C2 19, HAPPY+C4+C2 1, HAPPY+OWNER 5, HAPPY+OWNER+C2 1, HAPPY-GAP 1, OWNER 21, OWNER+C2 1, OWNER+HAPPY 1, UNCHANGED 32.

## Indirect fixture closure (sixteen additional causal roots)

The exact direct search is not a transitive call graph. These source reads close
its persisted-only helper consumers; unrelated Memory siblings stay unchanged.
Every path below is relative to crates/worth-relational/src/.

| Root and source | Disposition | Exact claim / cutover |
|---|---|---|
| `checkpoint_schema_readmission_and_tail_replay_rederive_identical_private_records`, tests/transactions/core/aggregate_identity/recovery.rs | HAPPY | Actual prepared candidate discard burns allocation occurrence, then checkpoint plus tail and second checkpoint recovery preserve exact aggregate records and canonical history. Use external original medium and real public aggregate reads; do not fake private records. |
| `cold_split_and_warm_split_publish_identical_aggregate_names_without_history_loads`, aggregate_identity/cold_recovery.rs | HAPPY+C2 | Actual cold versus live split preserves exact aggregate names and emitted envelope; no-history-member-demand bound C2. |
| `cold_aggregate_source_refusal_publishes_nothing_and_can_retry`, aggregate_identity/cold_recovery.rs | C4 | fail_regions source script, typed storage cause, unchanged head, pending allocation empty and retry. Preserve helper/source refs. |
| `checkpoint_tail_recovery_and_cert_replay_keep_component_memberships_and_split_merge_lineage`, aggregate_identity/components.rs | HAPPY | Public pinned aggregate identity/membership/exact lineage tuples, actual cert replay, canonical history and checkpoint-tail recovery. Other two component roots are Memory-only. |
| `seeded_full_key_differential_fork_recovery_and_replay_keep_actual_names_and_lineage`, aggregate_identity/logical_components/seeded.rs | HAPPY | Finite64 ordinary authored transitions, independent literal grouping oracle, actual split and merge lineage, retained fork and recovered full key state/canonical history. No fault or cost assertion; do not move merely for a module named exhaustive. Never run an exhaustive oracle during this review. |
| `required_content_authority_keeps_exact_shape_and_positional_empty_tails`, aggregate_identity/logical_components/serialization.rs | OWNER | Independent exact positional/named authority schema serialization, explicit empty tail slots and image readmission. Pure mapping/wire owner unit can use coherent Native Memory capture; no physical journal claim. |
| `fintech_persisted_workflow_recovers_checkpoint_tail_and_keeps_queryable_portfolio_state`, tests/domains/fintech/workflows/persistence.rs | HAPPY | Actual portfolio query entities, analysis branch/latest receipt and full recovery probe after checkpoint/tail. |
| `fintech_persisted_workflow_recovers_from_its_filed_checkpoint`, same path | HAPPY | Actual filed checkpoint selected, full query entity count/analysis head/latest probe after reopen. |
| `fintech_index_and_lineage_authority_survive_hostile_workflow_recovery`, same path | HAPPY | Ordinary trade correction/audit/index registration, exact recovered generation ID/source commit/branch and lineage. Its name says hostile but no injected fault occurs. |
| `fintech_failure_injection_helpers_cover_savepoints_replay_and_checkpoint_corruption`, same path | HAPPY+C4 | Valid/invalid savepoint and public wrong-branch replay checks HAPPY; private envelope removal and actual filed checkpoint corruption C4. Split, preserving each assertion. |
| `cdc_property_persisted_random_operation_matrix_recovers`, tests/publication/cdc/certification/property_matrices.rs | HAPPY | Actual generated ordinary operation16..64 sequence, multi-subscriber suffix convergence and whole visible truth after checkpoint/recovery. Other two proptest roots are Memory-only. |

Aggregate shared fixtures contain both runtime().build() and persisted().build_durable().
Only the six causal roots above use persisted branches; logical checkpoint's two
roots are already in the direct154. Preserve unrelated Memory consumers. The
aggregate cold_recovery_fixture defines another local ColdSource: do not recreate
it in Native. When porting its HAPPY half, move actual provider behavior into
external testkit through the Native source trait; C4/C2 restorations retain exact
old helper refs. Fintech fixture bool/persisted enum and harness mode similarly
must keep ordinary Memory users while persisted courts take explicit medium.
Helper dispatch must not rediscover a runtime's source or late-bind a receiver.

Total finite reviewed scope: direct154 plus indirect16 =170 roots. The
73 direct file count includes helper-only and unrelated Memory sibling roots;
this count is not170 durable scenarios and does not cover tests/history or
 tests/durability, which have separate reviews. No capability is declared absent
without the scoped implementation search described above.

### Index fixture transitive closure correction

A second scoped helper search for `durable_runtime_with_index_field_aspects`
finds five more causal roots beyond its direct constructor and checkpoint child:

| Root / tests/query/indexes path | Disposition | Exact preservation |
|---|---|---|
| `restored_catalog_allocates_fresh_identities_and_preserves_old_observations`, generation_identity_recovery.rs | HAPPY+C2 | Public restored definition/generation monotonic IDs, old/current exact generations and candidate entity IDs HAPPY; exact generation_payload_reads delta1/history_inventory_entries delta0 C2. |
| `discard_all_bases_keeps_readers_and_rebuilds_complete_ordered_content`, discard_rebuild.rs | HAPPY+C2 | Public historical/sibling/current generation discard, installed definition/current version unchanged, ExactGenerationUnavailable, cancellation and explicit rebuild/ExpectedGenerationMismatch/page parity HAPPY; private generations_snapshot exact census, untouched Arc pointer and last retained Weak release C2. |
| `discard_requires_installed_definition_and_empty_discard_is_idempotent`, discard_rebuild.rs | HAPPY | Exact public IndexNotInstalled and repeated empty discard removed_count0; no private observer required. |
| `derived_index_contract_relation_field_equals_persisted_recovery_preserves_parity`, recovery_and_execution_models.rs | HAPPY | Actual relation-field index/query execution, recovered access path, full result and reduction digest parity. |
| `derived_index_contract_persisted_recovery_preserves_entity_field_equals_parity`, same path | HAPPY | Same exact entity-field query/index recovery parity; its leased-build sibling remains a separate Memory fixture. |

The corrected transitive scope is170 roots. It includes unrelated Memory/pure
siblings in the direct-file inventory; no claim of170 independent durable
journeys is made. No unknown transitive source caller was silently defaulted:
queries for aggregate persisted/reopen, index durable factory, Fintech
PersistedSmokeBook/setup_persisted_world and harness Persisted/persisted_geometry
supply the listed sixteen causal roots. Any new call site discovered during
implementation must receive its own claim disposition before deletion.
