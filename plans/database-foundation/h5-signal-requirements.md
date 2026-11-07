# Signal and Bridge authored recovery evidence

Authority: [Milestone 3](milestone-3.md), Phase 5 and Delivery tracks, track H.
This map separates the preparatory byte contracts from operational readmission.

| Requirement | Enforcement or named court | Status |
|---|---|---|
| R2: static declarations survive evaluation independently of discovered edges | `authored_bytes_exclude_evaluated_edges_versions_and_node_state` | Passing public Signal graph court |
| R2: retirement removes authored edges even after discovered adjacency changes | `retiring_a_source_removes_declarations_after_evaluation_changes_the_read_set` | Passing; includes slot reuse, serde memory reconstruction and checkpoint memory reconstruction |
| R2: declaration cycles cannot hide behind a changed discovered read set | `declaration_cycles_refuse_when_evaluation_has_removed_the_runtime_edge` | Passing |
| Phase 5: explicit versioned, checksummed Signal byte format | `authored_image_has_explicit_empty_payload_and_refuses_unknown_versions`; `checksum_and_exact_length_cover_the_authored_frame` | Passing graph-section codec courts |
| Phase 5: authored policy and declaration fields are retained | `definition_policy_changes_are_source_even_when_cached_versions_match`; private `declaration_codec_preserves_nondefault_contracts_schema_policies_and_scopes` | Passing graph-section codec courts |
| Phase 5: full conditional source, including literal threshold parameters and provider roles | `full_conditional_definition_round_trips_literal_parameters_and_provider_roles`; `literal_threshold_payloads_round_trip_without_node_result_storage` | Passing pure codec courts |
| Phase 5: process-local binding counters are excluded from source | `authored_declarations_exclude_process_local_conditional_binding_counters` | Passing pure codec court |
| Phase 5: missing conditional definitions and foreign comparator bindings cannot stand in for source | `lowered_conditional_roles_cannot_replace_a_missing_definition`; `runtime_comparator_readmission_requires_its_full_definition_and_rebinds_current_installation` | Passing graph and configuration source courts |
| Phase 5: fresh source graph installs conditional contracts and derives values on first demand | `conditional_source_reinstalls_with_fresh_bindings_after_graph_reconstitution`; `authored_readmission_keeps_values_cold_until_first_demand` | Passing source reconstruction courts; standalone owner replay is covered below |
| Phase 5: runtime registrations, keyed nodes and stable tier names survive with cold memoization | `runtime_declarations_rebind_stable_tier_names_and_keep_memo_values_cold` | Passing pure configuration table court |
| Phase 5: full conditional source survives node memory/checkpoint projections and unrelated metadata changes | `node_memory_and_checkpoint_projections_retain_the_full_conditional_source`; `unrelated_node_metadata_preserves_conditional_source_and_a_changed_condition_retires_it` | Passing storage courts |
| Source reconstruction: decoder and constructor snapshot invariants agree | `branch_decoder_refuses_snapshot_effects_without_a_snapshot`; existing valid snapshot round trips | Passing |
| Source reconstruction: declared topology stays legal | `graph_readmission_refuses_stale_declarations_and_declaration_cycles`; `graph_readmission_reconstructs_valid_scoped_declarations` | Passing |
| D2: Signal owns the definition envelope, including an opaque Bridge-owned section | `definition_transition_retains_exact_authored_image_bridge_section_and_predecessor` | Passing descriptive artifact court; Bridge section interpretation pending |
| D5: original association and exact branch coordinates travel in branch bytes | `branch_effect_preserves_original_attempt_association_and_all_recorded_axes` | Passing descriptive artifact court; standalone publication deliberately has no World association; mixed publication pending |
| D6/D17: Home-scoped descriptive identity vocabulary cannot carry process identity | Sealed `DurableIdentity` implementations; exact typed descriptor fields; `foreign_definition_owner_and_foreign_attempt_home_refuse_as_typed_data` | Passing descriptive artifact court; recovered standalone advance/admission/definition issuers resume recorded bounds |
| D2: wasm-safe port, shared three-way publication fate | `SignalDurableMedium` has no Store/filesystem dependency; `SignalPublicationCompletion` has private fields | Passing `worth-signal-wasm` check for `wasm32-unknown-unknown` with the port present |
| D17: durable before live | Private concrete `DurablyCommitted<SignalCommittedBranchAdvance>` consumption; `candidate_unwind_and_proven_absence_preserve_the_head_and_release_the_original_reservation` | Passing Store-backed standalone owner court |
| Phase 5: multi-branch owner readmission and cold derived state | `several_branches_reopen_with_authored_definitions_and_values_recompute_on_first_demand` | Passing Store-backed standalone owner court; actual memory owner supplies the reference |
| D6/D8: recovered-owner descriptor admission and incarnation fencing | `descriptors_refuse_foreign_owners_unknown_versions_and_old_live_incarnations` | Passing Store-backed owner court |
| D6/D7/D17: one live canonical owner per physical Signal attachment, including failure and retirement | `second_live_owner_refuses_before_publication_and_original_owner_can_continue`; `failed_readmission_releases_its_original_exclusive_claim_before_retry`; `alternate_adapters_require_released_unshared_serving_and_attachment_refusals_retain_it` | Passing Store-backed owner courts; exclusive-admission correction awaits independent review |
| D6: every recorded node incarnation bounds future allocation without changing recovered source | `runtime_published_keyed_definitions_reopen_without_reissuing_unused_body_incarnations`; `recovered_node_bounds_change_future_issuance_without_changing_authored_source`; `exhausted_recorded_node_slots_are_retired_only_when_allocation_reaches_them`; `unused_body_vacancies_and_retired_slots_bound_future_node_issuance` | Passing Store-backed allocator courts |
| Phase 5 / R4: Bridge conditional definitions, fresh lowering/retention and typed diagnostic absence | Bridge installation and replay courts | Pending |
| Phase 5: same/changed build, graceful close and quiescent kill | `several_branches_reopen_with_authored_definitions_and_values_recompute_on_first_demand`; `quiescent_kill_reopens_all_branch_sources_and_recomputes_values` | Passing Store-backed standalone owner courts; Home admission remains pending |
| D15 / Phase 5: mixed publication and Home/World/Host admission | Granted integration seams and mixed-operation courts | Deferred to coordinator's exact seam grant |

The graph-section codec does not encode values, dirty/aspect versions, cause
sets, traces, telemetry, resource or temporal runtime. Its bytes are descriptive
source and do not admit a durable Signal owner. Store-backed owner recovery is
proved by the separately named owner courts. No Home reopen, mixed operation or
oracle result is inferred from codec or standalone owner courts. The wasm result
comes from its separate target check. Fresh durable bootstrap, fork and Bridge
readmission remain pending.

Logs live under `target/lane-evidence/`. The first codec review rejected missing
authored retirement propagation; the reverse declaration index and the named
retirement court above address that finding. An approving review of the corrected
range is still required before a safe point is reported.

The three allocator tests moved from a source unit module to the Store owner
court under test-convention v3's runtime-root rule. Their names and claims are
preserved. The external court checks exact live-set membership through public
owner contracts and node-count/liveness APIs, and produces the exhausted source
slot through an actual canonical allocation before testing unused-body replay.
