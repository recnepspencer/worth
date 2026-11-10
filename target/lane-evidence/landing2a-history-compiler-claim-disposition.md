# Native history compiler claim disposition

Read-only source review at HEAD 50d691f03d. No Rust edits, builds or test runs. Scope: milestone_7b, 7c phases a/d/e/g, 7d phases b/c/d/e and their mounted children. The finite module traversal found **39 files, 65 named test roots, 0 missing mounts, 0 unclassified roots**. All 65 are listed below, including already-Memory/non-provider roots and two runtime-free schema units; these are not all attachment-caused.

Disposition: **33 wholly HAPPY, 12 C4, 3 wholly C2, 8 owner compiler/value, 9 mixed**. Mixed: six HAPPY+C2 and three public/owner splits (schema hash, execution-ready copied records, direct rebuild). Three named roots also inherit an exact private original-input attribution assertion; its public observation gap is recorded separately below, not hidden in the classification.

HAPPY means normal shipping owner-bound planning/preparation/publication/inspection; it does not imply every current helper is externally callable. Owner compiler/value means exact intermediate compiler or derivation evidence, not physical durability. C2 is actual measured work or original resident Root evidence. C4 is injected malformed state/reference/plan. No test is C2 merely because it reads a private field.

## Mechanical cutover rules

- Public durable tests use the **existing external** worth-testkit-relational TestDurableMedium and retain its owned Arc through checkpoint/cold source/recovery. Reuse existing mounted court helpers listed below. No new local publisher/trait bridge/bind/discovery replacement.
- Pure compiler units stay in their existing baselined owner files. runtime_with_test_schema() builds actual Native Memory with CertificationCore and test_schema_registry(); for custom schemas, preserve the existing schema registry and .build(). These units assert compiled values, not reopen persistence. Do not add runtime-root baseline rows or a new proof/merge-parent transaction door.
- C4 retains actual original mutation controls and exact caller references. Moving an injected plan into public execute does not turn it into ordinary publication. Fault medium/source custody, where required by a fault claim, belongs to the already ruled C4 campaign; Memory is allowed only when the injected input claim is purely compiler/value state.
- C2 retains setup + oracle + counters/Root producer together. Public semantic returned summaries and neighborhood sizes are distinct from instrumentation counters.
- Split mixed assertions before retiring their shared durable fixture. Keep genuine durability claims genuine; no Memory fake reopen.

## Named roots and exact claim split

All file paths below are relative to crates/worth-relational/src/tests/history/. Source lines identify the named roots at the reviewed revision.

### milestone_7b.rs

- **merge_planning_artifact_certification_is_stable_across_recovery** (line 63) — **HAPPY**. Genuine checkpoint/recovery equality of planning schema snapshot, decision log, lowered records and digests. Keep real medium across source/recovered constructors; inspect_recovered_merge_planning_artifact must accept/return the owned medium, not discover an attachment.
- **merge_planning_schema_snapshot_changes_when_schema_semantics_change** (line 88) — **HAPPY + owner digest (already Memory)**. Two actual Memory schema registries (PreferRicher / FailOnConflict), each builder.build(), produce different canonical schema snapshot hashes via private schema_snapshot_digest. Public schema_snapshot data can preserve semantic declaration differences; retain this exact private hash assertion as a genuine Memory owner value/compiler check. Do not silently replace hash inequality with data inequality or add a hashing authority door. No medium dependency in this root.

### milestone_7c_phase_a/compilation_denials.rs

- **compile_execution_ready_merge_plan_rejects_missing_source_record** (line 12) — **C4**. Deletes prepared.execution_ready_plan_mut_for_test().source_records, then compile_execution_ready_merge_plan_for_test reports MissingSourceRecord. Preserve injected input and exact named controls/caller refs; actual Native Memory setup suffices for this compiler fault, no durable assertion.
- **verify_prepared_merge_execution_rejects_corrupted_compiled_plan** (line 48) — **C4**. Clears bound_executable_plan_mut_for_test().record_plans; public verify returns Compilation(PreparedAuthorityBindingMismatch). Keep corruption and typed denial in C4; not ordinary foreign-runtime or head-drift coverage.

### milestone_7c_phase_a/prepared_merge_verification.rs

- **verify_prepared_merge_execution_accepts_fresh_prepared_merge** (line 14) — **HAPPY**. Fresh genuine prepared merge accepted by public verify_prepared_merge_execution.
- **verify_prepared_merge_execution_rejects_runtime_instance_mismatch** (line 46) — **HAPPY**. runtime.fork() yields a real different runtime instance; verify original prepared object against fork yields RuntimeInstanceMismatch. Public fork exists; no forged authority needed.
- **verify_prepared_merge_execution_rejects_target_head_drift** (line 82) — **HAPPY**. Actual target transaction advances main after preparation; public verification returns StaleBranchHead for main.
- **verify_prepared_merge_execution_rejects_source_head_drift** (line 121) — **HAPPY**. Actual source transaction advances feature after preparation; public verification returns StaleBranchHead for feature.
- **verify_prepared_merge_execution_rejects_schema_semantic_drift** (line 165) — **C4**. set_schema_registry_for_test replaces runtime schema after preparation; SchemaSemanticDrift is the injected-state claim. Preserve exact control/caller; do not substitute two independent schema worlds.
- **verify_prepared_merge_execution_rejects_merge_base_drift** (line 232) — **C4**. authority_binding_mut_for_test changes the retained merge-base commitment; verify returns MergeBaseDrift. Preserve forged prepared binding, not a new public proof door.

### milestone_7c_phase_a/readiness_admission.rs

- **prepare_merge_execution_admits_fully_ready_source_only_addition** (line 20) — **HAPPY + owner compiler**. Public: fully-ready artifact/admission and source-addition record counts. Owner: exact execution_ready_plan().lowered_records all Execute. Public Core.lowered_plan.records can prove semantic Execute decisions; preserve the exact intermediate copied-array assertion in an actual Native Memory owner fixture if retained.
- **prepare_merge_execution_rejects_blocked_merge_plans** (line 64) — **HAPPY**. Declared manual conflict yields NotExecutionReady with blocked=1, rejected=0, denied=1 and Block decision. Public preparation error report carries these semantics.
- **prepare_merge_execution_rejects_rejected_merge_plans** (line 107) — **HAPPY**. Declared FailOnConflict schema yields rejected readiness/Reject decision in the public preparation error; ordinary declaration, not forced runtime schema drift.
- **runtime_prepare_merge_execution_matches_merge_access_surface** (line 183) — **HAPPY**. Runtime preparation and MergeAccess preparation produce equal normalized requests/artifacts. Bind the descriptive request independently for each consumed owner-bound call.

### milestone_7c_phase_a/record_plan_compilation.rs

- **prepare_merge_execution_compiles_source_addition_record_plan** (line 25) — **owner compiler**. Exact bound_executable_plan().record_plans len/AdoptSource variant and source record/name binding. Retain full compiler assertion under actual Native Memory with runtime_with_test_schema; public merged_intents can provide a separate semantic companion, not replace the compiled variant.
- **prepare_merge_execution_compiles_exact_shared_record_plan** (line 66) — **owner compiler**. Exact PreserveShared compiled record, equality witness digest, ExactSharedTruth provenance, aspect plan and EqualityWitnessDigest materialization. Keep owner-only evidence, same actual Memory preparation; no physical pin/dictionary/cost assertion.
- **prepare_merge_execution_compiles_reconcile_record_plan** (line 152) — **owner compiler (already Memory)**. Existing custom-registry builder.build() compiles Reconcile identity/aspect plans and source/target/base SnapshotPinnedRead policies; helper matches PinnedVisibleAspect with Authoritative WholeAspect locator. These are compiler evidence variants, not retained pin census/lifecycle. Preserve helper and exact intermediate assertions; no C2 move based on the word pinned.

### milestone_7c_phase_a/verification_counters.rs

- **verify_prepared_merge_execution_does_not_increment_planning_counters** (line 9) — **C2**. Actual work: verification leaves planning request/schema-kind/elapsed-nanos counters unchanged and increments exact verification/branch/base/digest work. Keep whole setup and guard assertions in C2.
- **verify_prepared_merge_execution_reports_verification_counters_without_planning_work** (line 76) — **C2**. Counter reset then zero planning work/elapsed and exact verification work increments. This is the cost claim, not a private-field classification.

### milestone_7c_phase_d.rs

- **derive_merge_commit_mutation_plan_emits_source_authorized_create_intent** (line 32) — **owner compiler**. bind_mutation_plan_for_test(TransactionId(77)) asserts exact binding, source-authorized create intent and structural counts. Actual Native Memory preparation removes durable fixture dependency. Public merged_intents() covers create semantics; actual public execute outcome covers structural counts, but arbitrary private TransactionId binding stays owner unit.
- **derive_merge_commit_mutation_plan_preserves_exact_shared_truth_without_mutation** (line 79) — **owner compiler**. bind_mutation_plan_for_test(TransactionId(88)) preserves exact shared truth, preserved-shared=1, emitted intents=0, empty mutation intents. Native Memory setup preserves complete private derivation claim; public merged_intents/outcome can separately prove semantic no-op. Its custom-registry builder.build() is already Memory.
- **derive_merge_commit_mutation_plan_reconciles_target_with_source_authorized_aspects** (line 143) — **owner compiler (already Memory)**. bind_mutation_plan_for_test(TransactionId(99)) derives reconciled target update/status=active/name absent with exact structural counts. Custom schema builder.build() already Memory. Retain test_patch_target's declared locator and private bound-plan assertions.
- **derive_merge_commit_mutation_plan_does_not_rely_on_raw_lowered_record_arrays** (line 296) — **C4**. Clears execution_ready_plan_mut_for_test().lowered_records, then binds TransactionId(123) and proves materialized intents survive. This deliberately inconsistent prepared object belongs C4 with exact control refs; no public plan mutation door.

### milestone_7c_phase_e/canonical_envelope_publication.rs

- **execute_prepared_merge_publishes_ordered_multi_parent_commit_through_canonical_envelope** (line 11) — **HAPPY**. Actual prepared execution publishes canonical ordered parents [main, feature], execution summary head IDs, merge-parent metadata; main advances and feature does not. Use genuine public preparation/execute, never raw merge-parent transaction creation.
- **execute_prepared_merge_preserves_reserved_summary_when_optional_diagnostics_budget_is_zero** (line 62) — **HAPPY (public configuration cutover)**. Original configure_diagnostics_for_test sets max_entries_per_artifact=0; assertion is one reserved MergeExecutionPublished entry. Public builder.diagnostics accepts RelationalDiagnosticsProfile with that public field, so configure before construction and preserve exact envelope assertion. No injected fault or new hook needed.

### milestone_7c_phase_e/commit_context_admission.rs

- **merge_commit_context_rejects_mismatched_parent_branch_metadata** (line 9) — **HAPPY actual claim; name overclaims**. The only exercised denial is unknown source branch wrong. _mutation_plan from bind_mutation_plan_for_test(TransactionId(999)) is unused and no metadata is corrupted. Remove dead setup; production bind_merge_execution_request reports UnknownBranch before prepare (cfg(test) convenience currently translates to MissingSourceHead). Preserve source ID assertion, name the real claim. No mismatched parent-context claim is currently evidenced.

### milestone_7c_phase_e/durability_recovery_diagnostics.rs

- **execute_prepared_merge_survives_durability_append_and_recovery** (line 11) — **HAPPY durable**. Actual merge append/checkpoint/recovery, visible truth and full durable commit meaning parity, emitted publication diagnostics/commit ID/execution and diagnostics digests. Keep explicit durable medium. Do not replace durable_commit_meaning projection with receipt-only equality.

### milestone_7c_phase_e/execution_counters_diagnostics.rs

- **execute_prepared_merge_reports_execution_counters_and_structural_summary** (line 10) — **HAPPY + C2**. Public: emitted semantic structural summary, diagnostic code/fields and merge result. C2: exact attempts/requests/records/intents counters and complexity_delta. Split the counter assertions with the same original setup; structural record/intent counts in returned outcome are semantic, not automatically cost.

### milestone_7c_phase_e/failure_attempt_diagnostics.rs

- **execute_prepared_merge_records_attempt_without_success_on_failure** (line 10) — **HAPPY + C2**. Public: actual target transaction makes plan stale; typed failure diagnostic exists and successful publication/detailed trace absent. C2: attempts=1, successes=0, records=0, intents=0. Keep failure behavior public; instrumented work proof C2.

### milestone_7c_phase_e/merge_ready_history_shape.rs

- **execute_prepared_merge_produces_merge_ready_history_shape** (line 7) — **HAPPY**. Published ordered parents, replay authority, source/target/merge-base summary and postmerge ancestry. Replace private latest_common_ancestor/inspect_merge aliases with public owner-observed branch basis: merge_base.commit ID and supporting source-ancestor minus target-ancestor set is empty. Retain replay parent equality; no graph graft or public authority mint.

### milestone_7c_phase_g/mod.rs

- **authoritative_merge_execution_certification_emits_machine_checkable_artifacts** (line 16) — **HAPPY durable + owner rebuild**. Three actual exact-shared/source-only/prefer-richer cases emit checked digests/truth counts and certify public replay, real checkpoint/recovery, full envelope meaning/truth/heads. Keep explicit real medium in every case and recovered factory. Private helper also calls rebuild_runtime_from_plan and only asserts successful result: preserve that direct private rebuild entry coverage in a separate Native Memory reconstruction owner lane; public recovery/replay is the external claim.

### milestone_7c_phase_g/stale_prepared_merge_denials.rs

- **authoritative_merge_execution_certification_rejects_stale_prepared_merge_after_target_advances** (line 11) — **HAPPY**. Actual target transaction advances after planning; stale prepared execution denied.
- **authoritative_merge_execution_certification_rejects_stale_prepared_merge_after_source_advances** (line 47) — **HAPPY**. Actual source transaction advances after planning; stale prepared execution denied.
- **authoritative_merge_execution_certification_rejects_metadata_reference_movement** (line 88) — **C4**. Direct branch_cell_mut(target).advance_metadata() keeps head constant, increments metadata generation and produces StaleBranchReference. Preserve exact reference-state and typed denial assertions; a public head advance would weaken/change the fault.
- **authoritative_merge_execution_certification_rejects_schema_semantic_drift_after_planning** (line 161) — **C4**. set_schema_registry_for_test injects schema drift after planning; preserve exact named control and SchemaSemanticDrift denial.

### milestone_7d_phase_b/deletion_block_reasons.rs

- **lowered_plan_preserves_source_deleted_target_live_block_reason** (line 13) — **HAPPY**. Genuine source delete/target live history produces SourceDeletedTargetLive, deletion resolution and no executable class. Shared helper checks corresponding digest-basis denial row too.
- **lowered_plan_preserves_source_live_target_deleted_block_reason** (line 33) — **HAPPY**. Genuine source live/target delete produces SourceLiveTargetDeleted with matching public lowered record and digest-basis rows.
- **lowered_plan_preserves_deleted_vs_modified_block_reason** (line 60) — **HAPPY**. Genuine deletion versus modification produces DeletedVsModified with matching lowered resolution, executable=None and denial bundle kind.

### milestone_7d_phase_b/executable_admission.rs

- **admitted_source_addition_carries_executable_class** (line 13) — **HAPPY**. Actual source addition public Core carries SourceOnlyAddition and AdoptSourceRecord executable class.
- **compile_rejects_corrupted_non_executable_resolution_class** (line 59) — **C4**. Mutates prepared execution-ready resolution/class and invokes compile_execution_ready_merge_plan_for_test; exact invalid compiled-resolution denial. Keep control and mutation in C4 with original caller reference.

### milestone_7d_phase_b/policy_boundaries.rs

- **lowered_plan_carries_explicit_manual_resolution_policy_boundary_for_generic_denial** (line 15) — **HAPPY (Memory policy fixture)**. Declared manual generic conflict carries RuntimeOnly manual-resolution boundary and digest-basis boundary parity. runtime_with_name_merge_policy already builder.build().
- **lowered_plan_carries_explicit_hard_reject_policy_boundary_for_fail_on_conflict** (line 62) — **HAPPY (Memory policy fixture)**. Declared FailOnConflict carries RuntimeOnly hard-reject boundary, Reject decision and exact digest-basis parity. No schema mutation control.

### milestone_7d_phase_c.rs

- **promoted_deleted_on_both_sides_compiles_to_an_explicit_executable_record_variant** (line 85) — **C4**. Helper changes source-addition prepared record to DeletedOnBothSides, ConvergeDeleted class/decision/action, compiles it and replaces bound plan. Assert explicit ConvergeDeleted variant, authoritative mutual-deletion semantics/provenance and Unchanged lineage; this is synthetic prepared-plan coverage.
- **promoted_deleted_on_both_sides_derives_zero_mutation_intent_execution_plan** (line 118) — **C4**. Same forged promoted fixture binds TransactionId(701), checks exact zero mutation-intent plan and semantic counters. Retain complete injected helper; not evidence of a genuine both-sides deletion history.
- **promoted_deleted_on_both_sides_executes_through_authoritative_merge_publication** (line 153) — **C4**. Executes the same forged promoted fixture through public execute, checking zero emitted intents and publication diagnostics. Public execution method does not make the input ordinary. Preserve as injected prepared-plan publication campaign.
- **real_feature_branch_delete_after_main_delete_is_authorable_and_classifies_as_deleted_on_both_sides** (line 241) — **HAPPY**. Real create/fork/delete main/delete feature is authorable and public planning classifies DeletedOnBothSides with actual convergence executable class. This is the genuine counterpart to the three promoted fixtures.
- **branch_local_delete_allowance_does_not_make_same_branch_stale_delete_legal** (line 286) — **HAPPY**. Actual stale same-branch delete is denied StaleHandle; real branch-local delete permission does not grant stale same-branch mutation. Public admitted transaction/validation path.

### milestone_7d_phase_d/aspect_merge_policy_denials.rs

- **built_in_additive_set_merge_policy_is_rejected_without_native_foundational_set_contract** (line 4) — **pure owner value**. Pure schema registration rejects AdditiveSet without native foundational set contract; typed InvalidAspectDeclaration/detail. No runtime or medium in root; keep in place.
- **monotonic_counter_merge_policy_rejects_struct_shape_during_schema_planning** (line 25) — **pure owner value**. Pure schema registration rejects MonotonicCounter struct shape with typed InvalidAspectDeclaration/detail. No runtime/medium; keep in place.

### milestone_7d_phase_d/deleted_convergence.rs

- **deleted_on_both_sides_merge_commit_has_replay_and_recovery_parity** (line 4) — **HAPPY durable**. Genuine two-branch delete convergence, public prepared execute, replay and actual durable recovery; truth/envelope/history parity, semantic zero-intent counts, Unchanged lineage and equality witness digest. Do not replace with milestone_7d_phase_c promoted helper.

### milestone_7d_phase_d/last_writer_wins_resolution.rs

- **built_in_last_writer_wins_reject_retained_envelope_is_stable_across_recovery** (line 4) — **HAPPY durable**. Declared LastWriterWins causal conflict rejects with policy reason, no resolved value and Reject record; retained planning envelope/lowered data equal after actual recovery.
- **built_in_last_writer_wins_auto_resolution_is_stable_across_recovery** (line 120) — **HAPPY durable**. Declared LastWriterWins auto-resolves to SourceVisibleValue, public Execute decision and actual feature-change visible after merge and real recovery; diagnostic summary parity.
- **auto_resolved_merge_reads_pinned_visible_value_through_declared_aspect_binding** (line 262) — **HAPPY (Memory policy fixture)**. Different declared display_name aspect/field binding reads actual chosen visible value after execution. Existing policy runtime is Memory. Pinned in name refers to selected materialized value, no pin-count/retention claim.

### milestone_7d_phase_d/merge_descriptive_touches.rs

- **native_merge_touches_only_changed_revision_and_both_index_membership_keys** (line 7) — **HAPPY + C2**. Public: exact changed status revision/path, untouched name absence, old/new index membership keys; target visibility and actual built index semantic membership; Complete preserves built generation/source axes. C2: independent original resident Root producer, retained Root commit/axes and every decoded Root column equality against Complete. Preserve native_checkpoint/decode_checkpoint/root capture together in C2; a public receipt comparison cannot replace this private resident oracle.

### milestone_7d_phase_d/monotonic_counter_resolution.rs

- **built_in_monotonic_counter_merge_is_auto_resolved_with_inline_value_and_recovery_parity** (line 4) — **HAPPY durable + C2**. Public: declared counter resolves inline Int64(18), read truth=18 before/after real recovery; diagnostics/digests and semantic execution/patch tags. C2: exact source-state=1, target-state=1, base-state=0, base-patch-authority=1 lookup counters. Split those counters without downgrading genuine durable parity.

### milestone_7d_phase_d/prepared_merge_denials.rs

- **deleted_on_both_sides_prepared_merge_rejects_target_head_drift** (line 4) — **HAPPY**. Actual target transaction after genuine mutual-deletion preparation returns StaleBranchHead(main) and failure diagnostics identify source/target.
- **deleted_on_both_sides_prepared_merge_rejects_schema_semantic_drift** (line 55) — **C4**. set_schema_registry_for_test injects post-prepare drift into mutual-deletion fixture; preserve exact control/caller and denial.
- **non_executable_deletion_denial_is_stable_across_recovery** (line 84) — **HAPPY durable**. Actual DeletedVsModified non-executable conflict: blocked class/reason, executable=None, denial kinds and whole lowered/digest parity after real recovery.

### milestone_7d_phase_d/topology_local_rewire/neighborhood_detection.rs

- **unrelated_relation_additions_do_not_inflate_topology_region_detection_counters** (line 4) — **C2**. Unrelated relation-addition scale is a cost guard: local semantic record=1, relationCandidatesScoped=2, endpointIncidences=5, regionConflicts=0, recordsEscalated=0. Keep entire setup, scale/noise input and local-record correctness guard in C2.

### milestone_7d_phase_d/topology_local_rewire.rs

- **topology_endpoint_divergence_denial_is_stable_across_recovery** (line 4) — **HAPPY durable**. Actual declared-identity endpoint rewires yield RelationEndpointRewiredEscalated with LocalRewireCandidate propagation, neighborhood records=2, rewired=1 and no topology conflict reason; compare all recovered record/digest evidence. These semantic neighborhood counts are not instrumentation.
- **disjoint_rewire_neighborhoods_do_not_escalate_to_topology_region_conflict** (line 183) — **HAPPY + C2**. Public: two disjoint local records, zero semantic region records, each blocked reason/LocalRewire propagation/no region reason and neighborhood evidence. C2: exact region conflict/escalation instrumentation stays zero.

### milestone_7d_phase_d/topology_region.rs

- **topology_region_conflict_detection_reports_bounded_neighborhood_counters** (line 4) — **HAPPY + C2**. Public: two BlockedTopologyRegionConflict records, Escalates propagation, ConnectedRewireNeighborhood reason, four-record neighborhood and exact rewired set. C2: candidatesScoped=4, endpointIncidences>=8, conflicts=1, escalatedRecords=2.
- **topology_region_conflict_denial_is_stable_across_recovery** (line 134) — **HAPPY durable**. Real connected rewires produce two region conflicts; public propagation/reason/neighborhood four/rewired two and whole conflict/lowered-plan/record parity after recovery. No injected prepared state.

### milestone_7d_phase_e.rs

- **inspection_input_round_trips_deleted_vs_modified_without_host_projection** (line 22) — **HAPPY**. Real DeletedVsModified planning.inspection_input round-trips via public RelationalMergeInspectionArtifact::from_input; normalized request, Core.lowered_plan, row count and typed ExecutionDenied/blocked reason equal public inspect_execution_surface.
- **execution_surface_preserves_topology_region_conflict_authority_rows** (line 74) — **HAPPY**. Actual topology identity fixture yields two ExecutionDenied topology-region inspection authority rows. Preserve real schema and ordinary relation transactions; no need to expose compiler-private fields.

## Mounted helper closure

These are all files in the 39-file mount closure that define non-test functions. Other files contain the named roots above or module mounts/imports; their inherited fixture dependency is not permission to move every assertion wholesale.

- **milestone_7b.rs**: `inspect_recovered_merge_planning_artifact`, `runtime_with_registry`, `snapshot_digest`. Split durable inspect_recovered_merge_planning_artifact from already-Memory runtime_with_registry/snapshot_digest. Durable helper takes explicit medium; exact private hash remains owner value check.
- **milestone_7c_phase_a/record_plan_compilation.rs**: `assert_pinned_visible_aspect_evidence`. Keep assert_pinned_visible_aspect_evidence with private compiler roots in their current baseline file. Source/exact-shared fixtures can use runtime_with_test_schema; reconcile fixture already uses .build().
- **milestone_7c_phase_d.rs**: `test_patch_target`. test_patch_target is a pure declared locator constructor; preserve it with private mutation-plan roots. First source-addition fixture can use runtime_with_test_schema; remaining custom-schema pure roots already Memory. Last corrupted-array root and its controls are C4.
- **milestone_7c_phase_e/fixtures.rs**: `execute_feature_into_main_merge`. execute_feature_into_main_merge returns runtime/outcome/original heads; public durable version must also retain/pass Arc<TestDurableMedium> (or accept it from caller). Genuine execute creates merge parents. Share value declarations, not a local durable publisher.
- **milestone_7c_phase_g/certification_cases.rs**: `certify_exact_shared_merge_execution`, `certify_source_only_addition_merge_execution`, `certify_prefer_richer_merge_execution`. Each of three case constructors retains its own actual medium, schema and original commit results. Pass explicit medium to recovery helper. Exact-shared no-op counts, source-addition create counts and prefer-richer chosen values remain semantic public assertions.
- **milestone_7c_phase_g/mod.rs**: `assert_machine_checkable_merge_execution_artifact`. assert_machine_checkable_merge_execution_artifact checks populated digest strings, truth counts and certification booleans; artifact structs are plain test report data, not authority. Keep all three cases mounted.
- **milestone_7c_phase_g/recovery_certification.rs**: `certify_merge_execution_with_recovery`. Separate private direct rebuild successful-entry assertion from public replay/recovery parity. Public recovery plan comes from original medium.recovery_source(), not discovered runtime attachment. Preserve exact attribution caveat below; branch heads/basis use observation APIs.
- **milestone_7c_phase_g/schema_fixtures.rs**: `prefer_richer_registry`, `drifted_schema_registry`, `durable_runtime_with_registry`. prefer_richer_registry pure declaration; drifted_schema_registry is C4 input; durable_runtime_with_registry accepts explicit real external medium only in public court. Owner compiler fixture uses same declared registry + .build(), no local medium implementation.
- **milestone_7d_phase_b/deletion_block_reasons.rs**: `assert_blocked_deletion_lowering`. assert_blocked_deletion_lowering reads actual public Core lowered/digest data; move intact with its three named public behavior roots, not to a private-plan bridge.
- **milestone_7d_phase_b/fixtures.rs**: `runtime_with_name_merge_policy`. runtime_with_name_merge_policy already builds Memory. Reuse identical schema declarations in public policy cases; no provider needed for these two policy-boundary roots.
- **milestone_7d_phase_c.rs**: `prepared_merge_promoted_to_deleted_on_both_sides`. prepared_merge_promoted_to_deleted_on_both_sides belongs exclusively to the three C4 promoted roots. It rewrites class/resolution/executable/policy/readiness/decision/action/aspect outcomes, then compile_execution_ready_merge_plan_for_test + replace_bound_merge_plan_for_test. Keep complete helper and exact refs. The two ordinary real-deletion/stale-delete roots do not require this forged fixture.
- **milestone_7d_phase_d/mod.rs**: `drifted_schema_registry`, `topology_identity_registry`, `durable_runtime_with_topology_identity_registry`, `runtime_with_aspect_field_merge_policy`, `register_aspect_field_merge_policy`, `i64_counter_value`, `string_aspect_field_patch_for_target`, `create_entity_with_aspect_fields`, `create_entity_with_aspect_fields_on_branch`, `update_entity_aspect_fields_on_branch`, `aspect_fields_with_identity_name`, `read_entity_aspect_field_display`. Split declarations/value patches from durable topology construction and C4 schema drift. runtime_with_aspect_field_merge_policy is already Memory; keep register_aspect_field_merge_policy, i64_counter_value, string_aspect_field_patch_for_target and aspect_fields_with_identity_name. Public create/update helpers use admitted branch transactions and must preserve stable declared name identity. read_entity_aspect_field_display replaces private branch_head/read_version with observation + public try_read_version (two Result layers). Keep original resident Root/native checkpoint codec closure in C2, not public test support.
- **milestone_7d_phase_e.rs**: `runtime_with_topology_identity_registry`. runtime_with_topology_identity_registry currently takes local TestDurableMedium and .build_durable(attachment); public equivalent accepts existing testkit medium attachment and same declared identity registry. Inspection rows require no private compiler input.

Shared original support boundaries: tests/support/runtime.rs (Memory constructor vs durable factory), branch transaction/create/update/delete support (ordinary admitted publication), checkpoint_and_recover_with/recovery_plan_for (currently attachment-dependent; public court uses explicit source), capture_aspect_truth_bundle (visible truth projection), and truth_comparison.rs (exact attribution + body meaning; see below). These support functions are not extra named test roots.

Current baseline evidence: src-court-baseline.json already lists milestone_7b [ExecutionRequest, RelationalRuntime], 7c_a record_plan_compilation/readiness/verification/compilation_denials and 7c_d [ExecutionRequest], 7d_c [ExecutionRequest, RelationalRuntime], 7d_d/mod [ExecutionRequest, RelationalRuntime]. Keep compiler units in their current files and reuse imported constructors: adding explicit RelationalRuntime annotations or new runtime-bearing helper files could add a root even if behavior is unchanged.

## Existing public APIs and nearest court content

Production signatures (the in-crate cfg(test) overloads auto-bind raw requests and are not the external signature):

```rust
RelationalRuntime::bind_merge_planning_request(
    &self, request: MergePlanningRequest
) -> Result<OwnerBoundMergePlanningRequest, RelationalMergeRequestBindingDenial>;
RelationalRuntime::bind_merge_execution_request(
    &self, request: MergeExecutionRequest
) -> Result<OwnerBoundMergeExecutionRequest, RelationalMergeRequestBindingDenial>;

MergeAccess::inspect_planning_scope(
    &self, execution: &ExecutionRequest<'_, '_>, request: OwnerBoundMergePlanningRequest
) -> Result<MergePlanningArtifactCore, MergePlanningError>;
MergeAccess::inspect_execution_surface(
    &self, execution: &ExecutionRequest<'_, '_>, request: OwnerBoundMergePlanningRequest
) -> Result<RelationalMergeInspectionArtifact, MergePlanningError>;
MergeAccess::normalize_merge_request(
    &self, request: OwnerBoundMergeExecutionRequest
) -> Result<NormalizedRelationalMergeRequest, RelationalMergeRequestNormalizationDenial>;
MergeAccess::prepare_merge_execution(
    &self, execution: &ExecutionRequest<'_, '_>, request: OwnerBoundMergeExecutionRequest
) -> Result<PreparedMergeExecution, MergeExecutionPreparationError>;
MergeAccess::verify_prepared_merge_execution(
    &self, execution: &ExecutionRequest<'_, '_>, prepared: &PreparedMergeExecution
) -> Result<(), MergeExecutionError>;
RelationalRuntime::execute_prepared_merge(
    &self, execution: &ExecutionRequest<'_, '_>, prepared: PreparedMergeExecution
) -> Result<MergeExecutionOutcome, MergeExecutionError>;

HistoryAccess::branch_head_for_observation(
    &self, observation: &RelationalBranchObservation
) -> Result<Option<RelationalCommitReceipt>, RelationalBranchBasisDenial>;
HistoryAccess::historical_latest_commit(&self) -> Option<RelationalCommitReceipt>;
HistoryAccess::merge_branch_basis_for_observations(
    &self, source: &RelationalBranchObservation, target: &RelationalBranchObservation
) -> Result<RelationalMergeBranchBasis, RelationalMergeBranchBasisDenial>;
VisibilityReadContext::try_read_version(
    &self, execution: &ExecutionRequest<'_, '_>, version_id: VersionId
) -> Result<Result<RelationalReadView, HistoricalVisibilityDenial>,
            RelationalExecutionDenialCause>;
```

Use runtime.read_truth().try_read_version; its source impl is VisibilityReadContext. The two layers are real: .expect("request remains live").expect("actual retained version"). Public head recipe: branch_identity(branch), observe_branch(&identity), then history().branch_head_for_observation(&basis.observation()). Do not substitute historical_latest_commit for a selected branch head; it is the explicit immutable global-latest observation. Source evidence: runtime/state/runtime_state/merge_authority.rs:11/28; merge/access.rs:54/105/167; merge/execution/mod.rs:63/115; merge/facade.rs:23/34; history/access/commit_surfaces.rs:12/92; history/access/merge_branch_basis.rs:12; visibility/materialization/read_records/reader/snapshot_reads.rs:94.

Public PreparedMergeExecution request(), artifact(), merged_intents() expose original semantic inputs/intents. Core.lowered_plan.records and Core.digest_basis rows carry the semantic class/resolution/readiness/decision/policy data used by HAPPY roots; Core.branch_basis and Core.ancestry carry admitted ancestry/base and unique-commit counts. Exact bound_executable_plan/execution_ready_plan copies remain owner assertions. MergeExecutionOutcome.structural_summary/execution_summary and canonical envelope merge_execution_authority retain returned publication semantics; semantic counts here are not instrumented cost.

Already-mounted court directory: crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/:
- merge_history_planning.rs::inspect_feature_reconciliation binds the actual branches and calls public inspect_planning_scope.
- branch_history_recovery.rs::checkpoint/recover/current_head keeps Arc<TestDurableMedium>, files captured checkpoint, calls recovery_plan_from_source(medium.recovery_source()), and observes an exact branch head.
- merge_policy_evidence.rs already asserts public digest policy/visibility evidence rows.
- merge_visibility_evidence.rs:184 already demonstrates public try_read_version's two-result pattern.
- branch_history_publications.rs contains admitted create/fork/update branch publication support. Reuse it instead of moving Native private transaction helpers across the boundary.
- worth-testkit-relational/src/durable_publication.rs::durable_runtime(&Arc<TestDurableMedium>) builds CertificationCore with a real attachment; memory_runtime() is explicit Memory. Custom registries preserve their declared identity/aspect contracts with the same public builder/real attachment.
- RelationalRuntimeBuilder::diagnostics(self, RelationalDiagnosticsProfile) -> Self already exists at runtime/builder/mod.rs:93; profile.max_entries_per_artifact is public, facade re-export facade.rs:70. The zero-budget root can configure this before build and remove its test-only control.

## Exact private original-input attribution gap

Three roots invoke the same hidden subclaim:
1. 7c_phase_e/durability_recovery_diagnostics.rs::execute_prepared_merge_survives_durability_append_and_recovery (assert_durable_commit_meaning_eq).
2. 7c_phase_g/mod.rs::authoritative_merge_execution_certification_emits_machine_checkable_artifacts, all three cases via recovery_certification.rs:58 (durable_commit_meaning_matches).
3. 7d_phase_d/deleted_convergence.rs::deleted_on_both_sides_merge_commit_has_replay_and_recovery_parity (assert_durable_commit_meaning_eq).

tests/support/truth_comparison.rs:60-105 compares **both** recorded_original_inputs() and a copied complete body with diagnostics entries cleared and derived index artifacts defaulted. Existing public Serialize for CanonicalCommitEnvelope exhaustively serializes every body field plus semantic_result (history/data/canonical_commit_envelope/wire_projection.rs:18-144). An external read-only serialized projection can preserve the entire body equality with exactly those two omissions; do not shrink it to receipt/head/digest equality or selected visible fields. This comparison is observation only and requires no publisher or body mutation control.

However recorded_original_inputs() is pub(crate), absent from that serialization, and stores original attribution outside the body. Scoped search found no public facade/replay observation of this per-envelope record. Source: history/data/canonical_commit_envelope/original_inputs.rs:5; durability/source_initial_inputs/recorded.rs:19. Data: optional result_image (DurableBranchRootSchemaImage), prestate_schema [u8;32], result_schema [u8;32], original MutationConfig. Initial-source artifact bytes are not this exact per-commit retained attribution.

Nearest existing observation: RelationalCompleteSourceArtifact::input_commitment_leaves_for_test(&ExecutionRequest) -> Result<ExecutionArray<(PatchStreamPosition, [u8;32])>, RelationalInitialInputsRefusal>, source_initial_inputs/commitment_observation.rs:12. Its existing external caller is cost/relational/source_initial_inputs.rs:43. It computes normalized source input leaves, including optional result-image prefix/prestate/result schema/mutation (input_commitments.rs:57); it does not return the retained per-envelope Option. It is a sealed named test-door and observes a different object, so do not add HAPPY callers or claim this already proves retained envelope attribution. The court already depends on serde_json and rmp-serde, so the full public envelope serialized comparison needs no new package dependency.

This is an ordinary durable DATA parity assertion, **not C2 and not injected C4**. Native Memory normally has no positioned original-input attribution: comparing None to None would erase this claim. Two honest choices remain: preserve the exact assertion inside the durability/history owner's real-journal proof, or add the minimal immutable history/replay observation of the original per-envelope attribution (presence plus those exact original fields / canonical semantic image). It must expose no mutation, admission, proof, merge-parent creation, or fresh authority. This artifact does not authorize an implementation or pretend the observation already exists.

## Complete finite mount inventory

```text
milestone_7b.rs
milestone_7c_phase_a/compilation_denials.rs
milestone_7c_phase_a/mod.rs
milestone_7c_phase_a/prepared_merge_verification.rs
milestone_7c_phase_a/readiness_admission.rs
milestone_7c_phase_a/record_plan_compilation.rs
milestone_7c_phase_a/verification_counters.rs
milestone_7c_phase_d.rs
milestone_7c_phase_e/canonical_envelope_publication.rs
milestone_7c_phase_e/commit_context_admission.rs
milestone_7c_phase_e/durability_recovery_diagnostics.rs
milestone_7c_phase_e/execution_counters_diagnostics.rs
milestone_7c_phase_e/failure_attempt_diagnostics.rs
milestone_7c_phase_e/fixtures.rs
milestone_7c_phase_e/merge_ready_history_shape.rs
milestone_7c_phase_e/mod.rs
milestone_7c_phase_g/artifacts.rs
milestone_7c_phase_g/certification_cases.rs
milestone_7c_phase_g/mod.rs
milestone_7c_phase_g/recovery_certification.rs
milestone_7c_phase_g/schema_fixtures.rs
milestone_7c_phase_g/stale_prepared_merge_denials.rs
milestone_7d_phase_b/deletion_block_reasons.rs
milestone_7d_phase_b/executable_admission.rs
milestone_7d_phase_b/fixtures.rs
milestone_7d_phase_b/mod.rs
milestone_7d_phase_b/policy_boundaries.rs
milestone_7d_phase_c.rs
milestone_7d_phase_d/aspect_merge_policy_denials.rs
milestone_7d_phase_d/deleted_convergence.rs
milestone_7d_phase_d/last_writer_wins_resolution.rs
milestone_7d_phase_d/merge_descriptive_touches.rs
milestone_7d_phase_d/mod.rs
milestone_7d_phase_d/monotonic_counter_resolution.rs
milestone_7d_phase_d/prepared_merge_denials.rs
milestone_7d_phase_d/topology_local_rewire/neighborhood_detection.rs
milestone_7d_phase_d/topology_local_rewire.rs
milestone_7d_phase_d/topology_region.rs
milestone_7d_phase_e.rs
```

0 unclassified named roots; one precisely scoped missing public DATA observation affects three named roots. This is a source disposition, not compile/behavior verification or Complete ACK.

