# Signal and Bridge authored recovery evidence

Authority: [Milestone 3](milestone-3.md), Phase 5 and Delivery tracks, track H.
This map distinguishes accepted owner-level evidence from the Home serving cutover and the remaining public journeys. Passing historical owner courts do not certify a later Home route.

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
| D2: Signal owns the definition envelope, including an opaque Bridge-owned section | `definition_transition_retains_exact_authored_image_bridge_section_and_predecessor` | Accepted owner artifact and Bridge section courts; fresh host-provider contracts remain a separate open requirement |
| D5: original association and exact branch coordinates travel in branch bytes | `branch_effect_preserves_original_attempt_association_and_all_recorded_axes` | Passing descriptive artifact court; standalone publication deliberately has no World association; World owner composition is implemented, and Query mixed publication remains pending |
| D6/D17: Home-scoped descriptive identity vocabulary cannot carry process identity | Sealed `DurableIdentity` implementations; exact typed descriptor fields; `foreign_definition_owner_and_foreign_attempt_home_refuse_as_typed_data` | Passing descriptive artifact court; recovered standalone advance/admission/definition issuers resume recorded bounds |
| D2: wasm-safe port, shared three-way publication fate | `SignalDurableMedium` has no Store/filesystem dependency; `SignalPublicationCompletion` has private fields | Passing `worth-signal-wasm` check for `wasm32-unknown-unknown` with the port present |
| D17: durable before live | Private concrete `DurablyCommitted<SignalCommittedBranchAdvance>` consumption; `candidate_unwind_and_proven_absence_preserve_the_head_and_release_the_original_reservation` | Passing Store-backed standalone owner court |
| Phase 5: multi-branch owner readmission and cold derived state | `several_branches_reopen_with_authored_definitions_and_values_recompute_on_first_demand` | Passing Store-backed standalone owner court; actual memory owner supplies the reference |
| D6/D8: recovered-owner descriptor admission and incarnation fencing | `descriptors_refuse_foreign_owners_unknown_versions_and_old_live_incarnations` | Passing Store-backed owner court |
| D6/D7/D17: one live canonical owner per physical Signal attachment, including failure and retirement | `second_live_owner_refuses_before_publication_and_original_owner_can_continue`; `failed_readmission_releases_its_original_exclusive_claim_before_retry`; `alternate_adapters_require_released_unshared_serving_and_attachment_refusals_retain_it` | Accepted Store-backed owner courts; stopped Home admission is qualified separately below |
| D6: every recorded node incarnation bounds future allocation without changing recovered source | `runtime_published_keyed_definitions_reopen_without_reissuing_unused_body_incarnations`; `recovered_node_bounds_change_future_issuance_without_changing_authored_source`; `exhausted_recorded_node_slots_are_retired_only_when_allocation_reaches_them`; `unused_body_vacancies_and_retired_slots_bound_future_node_issuance` | Passing Store-backed allocator courts |
| Phase 5 / R4: Bridge conditional definitions, fresh lowering/retention and typed diagnostic absence | `cold_owner_rebuilds_conditional_definitions_and_retention_on_every_branch_for_either_build`; `cold_correspondence_rebinds_full_semantics_and_preserves_the_selected_slot_without_source_effects`; writeback compiler contracts | Accepted owner evidence in `25a4292388` and `2d1dec42c7`; this does not establish portable host-provider parameters |
| Phase 5: same/changed build, graceful close and quiescent kill | `several_branches_reopen_with_authored_definitions_and_values_recompute_on_first_demand`; `quiescent_kill_reopens_all_branch_sources_and_recomputes_values` | Passing Store-backed standalone owner courts; Home admission remains pending |
| D15 / Phase 5: mixed publication and Home/World/Host admission | World's original `WithSignal` preparation and exhaustive owner evidence; Store's admitted Signal journal class | World owner route implemented; Query mixed callers and Host lift remain pending |
| D8/D17: Signal precedes World admission | Concrete `SignalReadmitted<SignalReadmittedApplicationHome>`; genuine Query fresh-program/installation binding | Compiler checks green; full several-branch public Home journey remains pending |
| D16/D17: an original open carries the selected branch programme | `AdmittedBranchProgramSource` retained from `WorthQuerySelectedProgramInspection`; `host_transition_plans_on_nonempty_a_then_serves_writes_closes_and_reopens_b` | Original-source typing implemented; named Host A-to-B close/reopen court passes after retirement correction; full H5 journeys remain open |
| D13/D17: stopped owner admission and medium retirement retain the original allocation | Concrete same-owner `SignalStoppedOwnerGuard`; `stopped_signal_owner_retires_its_medium_before_original_allocations_are_reclaimed`; `authored_retirement_refuses_an_open_owner_and_completes_after_the_same_owner_stops` | Zero-budget original retirement and same-owner stop courts pass; independent serving-slice review pending |
| D13: checkpoint capture failure cannot withhold physical close after admitted retirement | `host_checkpoint_memory_failure_is_reported_while_original_retired_owners_remain_physically_closed` | Passing named Store Host court with the installed maintenance allowance derived from the actual worker allocation bill; independent serving-slice review pending |
| Phase 5: conditional provider selection and parameters survive fresh installation | Required A-initial/B-published, branch-specific, same/changed-build provider courts | Open: portable provider description/decoder contract and Bridge section format decision; existing Signal condition codec courts do not prove this |
| Phase 5: static conditional authoring does not install temporal runtime | Required authored-only public definition publication court | Open: separate static authoring from the H7 temporal operation handle |
| Phase 5: public durable siblings reopen and continue creating branches | Required public create/edit/reopen/create Home court | Open: World creation intent/selection, shared history versus references, and owner-issued generated names |

The graph-section codec does not encode values, dirty/aspect versions, cause
sets, traces, telemetry, resource or temporal runtime. Its bytes are descriptive
source and do not admit a durable Signal owner. Store-backed owner recovery is
proved by the separately named owner courts. No Home reopen, mixed operation or
oracle result is inferred from codec or standalone owner courts. The wasm result
comes from its separate target check. Fresh durable bootstrap, fork and Bridge readmission have accepted owner evidence. Public Home sibling creation, mixed publication and the full Home oracle remain pending.

Logs live under `target/lane-evidence/`. Accepted owner slices and the reviewed
Root owner cutover are distinct from the currently uncommitted serving slice.
The early authored-retirement/declaration corrections have approving owner reviews;
a fresh review and green affected Home courts are still required for this slice.
The complete H5 declaration requires all open rows above and its final wasm check.

The three allocator tests moved from a source unit module to the Store owner
court under test-convention v3's runtime-root rule. Their names and claims are
preserved. The external court checks exact live-set membership through public
owner contracts and node-count/liveness APIs, and produces the exhausted source
slot through an actual canonical allocation before testing unused-body replay.
