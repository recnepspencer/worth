# Remaining attachment claim classification

Read-only source review artifact; no Rust/manifests, builds, formatting or tests changed by this review. This is not execution evidence or Complete ACK.

## Coverage and unresolved decisions

Input inventory: 224 files. Present named test declarations: 459. Direct assertion expressions: 2055.

Direct assertions: C2=336, C4=362, HAPPY=76, UNCLASSIFIED=1274, pureowner=7.

Named tests carrying each class (mixed tests count more than once): C2=93, C4=81, HAPPY=17, UNCLASSIFIED=337, pureowner=1.

**Unclassified named tests: 337. Unclassified non-cost helper boundaries: 37.** No blanket migration clearance is claimed. Explicit C2/C4 source observations are retained even when another claim in the same test remains unresolved.

## Governing cut

HAPPY is ordinary public authoring/publication/branch/read/history/replay/checkpoint/recovery/inspection evidence over the actual explicitly owned durable provider. C2 is exact private resident/dictionary/pin/allocation/identity/cost evidence. C4 is injected fault campaigns. pureowner is local value/algorithm semantics independent of runtime issuance. UNCLASSIFIED is an unresolved semantic decision, not permission to move, drop or convert to memory.

Remove all three adapters. No replacement local publisher, local/external trait bridge, runtime provider discovery/downcast or late bind. Split mixed claims, never whole files by name.

## Existing public implementation surfaces

The mounted `worth-court-relational-certification/.../relational/durable_publication/mod.rs` already imports `worth_testkit_relational::durable_publication::{durable_runtime,capture,recover,source_journey,create_entity_outcome,perform,current}`. Producer and receiver both build over the explicitly retained same medium. Reuse `aspect_storage::publications::{begin_main_transaction,release_test_commit_snapshot}`. `cold_recovery::{capture_source,source_with_history,recovered_with_history}` preserves actual committed bytes and external members.

Nearest mounted courts: `durable_publication/{associated_effects,recovered_associated_effects,recovered_publication,recorded_basis,recorded_component_checkpoint,recorded_component_recovery,recorded_component_validation,result_snapshot_cleanup,cold_envelope_custody,index_catalog,encoded_checkpoint_backing}.rs`; `durable_publication/source_initial_inputs/**`; `content/{recovery,checkpoint_corruption,checkpoint_controls,checkpoint_capture_controls}.rs`. These are insertion analogues, not proof an old private assertion has already moved.

Remaining provider work includes fallible source frame reads (current `recovery_source` expects intact held frames), actual required byte replacement/duplication/checkpoint damage and provider-owned publication gates. Publication/transition scripting and offered artifact observation already exist.

## Reviewed mixed cuts and execution priorities

The source-expression index below is deliberately conservative. Its C2/C4 rows identify concrete observations/controls, not clearance of the whole test. The UNCLASSIFIED total includes unresolved helper claims even when every directly asserted expression has a class. In particular, no test becomes HAPPY merely because the source scanner found no private-looking spelling. This pass does not claim to have established every semantic cut in this large closure.

### First public cases

Under `tests/durability/contracts/`, move these named claims through the external medium and ordinary public owners:

- `adjacency_checkpoint::checkpoint_rebuilds_current_and_historical_adjacency_kind_buckets`: actual historical relation membership and empty current membership after durable recovery; `read_truth().bounded_outgoing_relations_of_kind_at_version`.
- `branch_and_aspect_history::durability_contract_recovery_rebuilds_branch_heads_and_latest_commit`: actual main/feature receipts and recovery latest commit; `history().branch_head`.
- `branch_and_aspect_history::durability_contract_recovery_preserves_aspect_bearing_patch_truth_and_history`: actual recovered aspect histories/digests and canonical patch meaning; public history/replay owners.
- `branch_and_aspect_history::durability_contract_recovery_preserves_relation_aspect_history_for_retained_audit_relations`: recovered relation history and original lifecycle/aspect origins. Its assertion helper must still be checked as recorded below.
- `branch_and_aspect_history::durability_contract_recovery_preserves_branch_local_endpoint_deletion_retirement_histories`: actual schema-qualified branch audit histories/inspection. Port declared relation integrity through the schema facade; do not replace its registry with the simpler default fixture.
- `fork_tail_after_source_moved::a_fork_commits_after_a_checkpoint_taken_once_its_source_moved_on`: actual fork/checkpoint/tail receipt continuity; public fork/history/recovery owners.
- `branch_transition/replay::partial_replay_includes_an_intermediate_fork_without_its_own_commit`: original public replay and recovered replay both include the intermediate transition. Carry the real transition source through the fixture; do not fabricate a fork parent.
- `retention_inspection_store::durability_contract_recovery_preserves_inspection_truth_bundle`: public feature branch inspection bundle equality. Its two sibling tests' exact pin counters are separate C2 claims.

Also HAPPY: `tests/query/indexes/discard_rebuild/checkpoint::discarded_ordering_stays_cold_after_native_checkpoint_restore_then_rebuilds` uses public discard/read-generation/lookup/rebuild results. Replace late binding with receiver construction over the same explicit medium. Its ExactGenerationUnavailable result is a public typed consequence, not a private resident-cell observation.

`indexes/access/definition_lookup::lookup_snapshot_is_detached_from_subsequent_owner_registration` is a public registration/lookup snapshot case; `exact_lookup_preserves_restored_numeric_identity_and_disambiguates_name` is pureowner and stays local. Durable publication is irrelevant to the former setup but changing it to memory is not a general precedent for durable claims elsewhere.

### Private publication claims are C2, not missing public doors

- `history/commit/envelope_residency/memory_publication_tests::journal_publication_keeps_journal_placement_without_memory_frame`: C2 owns actual Journal placement, same retained index owner, absence of original Memory payload, zero cell payload storage, exact stored original bytes and exact source pointer. A memory runtime or an externally observed equal receipt cannot prove these assertions. Preserve this actual owner evidence in C2; the public emitted/recovered artifact journey is a paired HAPPY case.
- `tests/durability/medium/recovered_publication::a_recovered_runtime_publishes_its_next_commit_at_the_next_position`: HAPPY owns no republication, covered/tail counts, next position, parent receipt, fresh ID and second reopen head. C2 owns `recovered.durability.cold_source()` and exact `Arc::ptr_eq` across reconstruction. The raw medium can independently prove held bytes and a public cold-read court can prove read limit refusal; neither implies private binding identity.
- `tests/durability/medium/durable_before_live::a_refused_publication_was_offered_to_the_medium_and_moved_nothing_live`: C4 owns the scripted ProvenNoEffect offer/refusal and public live-head/patch/storage no-effect consequences. C2 owns `visibility.published_snapshot_handle_count`, preparation snapshot capacity and exact preparation history definition frontier. Preserve accepted-next-definition semantics through public emitted artifacts; do not drop the private census merely because the public head is equal.
- `tests/durability/contracts/caller_association::caller_association_bounds_and_rebinding_refuse_without_retaining_publication`: pure value codec/bounds portions are pureowner/public value contracts; actual rebinding/abandonment behavior is public, while exact pending route and identity-reservation inventory is C2.
- `caller_association_survives_embedded_checkpoint_and_refuses_old_semantics`: HAPPY owns original association/semantic-result artifact equality and ordinary checkpoint/recovery; C4 owns authentic old-format refusal; private decode/reencode convenience is not a reason to export the codec. Reproduce that source contradiction at actual untrusted bytes.
- `caller_association_survives_cold_checkpoint_history_and_journal_tail`: HAPPY owns actual recovered framed bytes, receipts, digests, semantic results and associations; C2 owns selected-directory presence, journal-placement absence and `positioned_if_resident().is_none()` before demand.
- `associated_effects::associated_effects_refuse_memory_and_absent_settlement_evidence`: public associated-memory refusal is HAPPY; removal of an actual canonical route is C4, and exact pending/settled private source posture remains owner evidence.
- `associated_effects::associated_effects_discover_cold_subject_without_reading_unrelated_history` and `associated_effects_discovery::native_custody_correction_enumerates_known_orphan_and_unrelated_association_metadata`: HAPPY owns returned source subjects/association order and exact semantic results; C4 owns deliberately removed original journal entries; C2 owns selected/unrelated resident-cell absence and absence of unwanted admission.
- `tests/durability/medium/single_owner::a_forked_runtime_never_inherits_the_medium`: HAPPY owns the fork publishing no offers to the original medium; C2 owns direct private medium-binding absence. `recovery_moves_the_medium_into_the_rebuilt_runtime` is public recovery/next append over the retained provider. `an_attachment_the_medium_moved_out_of_publishes_nothing` is the actual private detach/binding state machine; classify its original detach-once and MediumMoved proof with its owner, not as an external fixture bridge.

### Other exact mixed boundaries

- `branch/owner_services/materialization_port_tests::durability_replays_suspended_and_restored_materialization_roots`: HAPPY owns typed materialization admission refusal and exact rematerialized entity/relation reads. C2 owns `assert_cold_inventory`'s independent actual-root completeness/derived inventory recomputation and `assert_suspended_records`' exact arena slots, unavailable materialization, unchanged kind/no retirement. The helper queue below remains unresolved as an implementation seam, not as the semantic class of these assertions.
- `durability/authority/checkpoint_concurrency_tests::completing_an_older_capture_cannot_replace_the_compacted_transition_basis`: C2 owns the actual private capture/finalize window and selected internal coverage. Its sibling `checkpoint_reconstruction_does_not_exclude_publication_after_capture` currently constructs a memory runtime: preserve that memory posture and split mixed durability helpers without manufacturing a persisted claim.
- `durability/authority/fresh_source/tests::fresh_source_keeps_actual_runtime_root_and_installed_schema_for_complete`: C2 owns exact original root/schema/source allocation identity. HAPPY owns initialization/admission/Complete and recovered unused schema constraint meaning. Equal decoded schema values are not original schema allocation identity.
- `durability/authority/runtime_rebuild/checkpoint_restore_tests::{equal_exact_root_reuses_the_readmitted_partition_substrate,shared_restored_substrate_keeps_a_pinned_reader_exact_after_next_write,divergent_mirror_partition_rebuilds_cross_partition_adjacency_after_reuse,recovered_main_and_sibling_catalogs_reuse_exact_canonical_payloads}`: C2 owns original generation sharing/pinned reader/original payload allocations. `divergent_sibling_is_not_reused_as_the_storage_mirror` has the same private nonreuse claim plus a HAPPY recovered branch-read consequence. Preserve public CheckpointRestoreWork results without exporting substrate access.
- `transactions/data/outcomes/commit_result/snapshot_cleanup_tests::result_snapshot_cleanup_refuses_an_actual_admitted_runtime_copy`: C2 owns the actual private admitted copy; a separately built foreign runtime is a different negative. `native_custody_correction_result_release_cell_tracks_public_release_closeout_and_transfer` has C2 original release/transfer cell identity plus HAPPY explicit release/close consequences. `native_custody_correction_result_cleanup_refuses_foreign_and_private_altered_pairing` splits HAPPY foreign-owner refusal from C4 mutation of the private snapshot pairing. `native_custody_correction_result_cleanup_is_explicit_idempotent_and_keeps_external_read` has HAPPY release/idempotency/independent read consequences; exact retained cell/registration observations stay C2.
- `history/retention/component_cut_tests::{native_custody_correction_cross_lane_acquire_release_has_one_actual_capture_cut,native_custody_correction_current_lane_inventory_is_collected_inside_actual_cut}`: C2 owns actual external-cut mutex and independently collected original inventories; public coherent captured results can pair with it but cannot select this cut.
- `branch/coordination/managed_fork_cleanup::managed_fork_refusal_and_success_drop_lookup_outside_native_coordination`: C4 owns scripted refusal/success campaign; C2 owns destructor `try_lock` on the actual coordination gate and original charged-memory release. A provider offer count is not the destructor lock assertion.
- `tests/durability/medium/branch_transition_races::{durable_fork_and_source_commit_complete_without_deferral,durable_fork_and_source_archive_complete_in_lock_order}`: C4 owns actual source coordination wait-count rendezvous; public completion/order is its paired outcome. `durable_fork_waits_for_another_branch_commit` can use public reservation contacts plus an external provider gate. `memory_commit_waits_behind_a_transition_without_consuming_its_candidate` is memory, not a durable provider claim.
- `tests/durability/medium/lifecycle_lock_order::{fork_against_checkpoint_capture,fork_against_runtime_fork,fork_against_reclamation,archive_against_checkpoint_capture,archive_against_runtime_fork,archive_against_reclamation}`: C4 owns exact private exclusive lifecycle contact/gate; C2 retains original lock/capture observations. A later medium pause is not the same window. `checkpoint_registry_is_readable_while_a_fork_is_durable_not_yet_live` can use an externally owned durable transition gate for its public consequence; verify its private helper observations before clearance.

### Ignored performance caller roots: C2

All paths below are under `crates/worth-relational/src/tests/performance_profiles/`; source mounts were checked through `tests/mod.rs` -> `performance_profiles.rs` -> named child module. Retain ignored/cost categories and actual durable handles.

- `artifact_recoverability_matrix.rs::perf_artifact_recoverability_matrix` and `recoverability_policy_matrix.rs::perf_recoverability_policy_matrix` directly execute the omitted durable branches.
- `commit_delta_matrix.rs::perf_commit_delta_matrix`, `durability_append_matrix.rs::perf_durability_append_matrix`, `index_parity_matrix.rs::perf_index_parity_matrix`, `replay_recovery_matrix.rs::perf_replay_recovery_matrix` directly contain cost campaigns in the original inventory.
- `chip_simulator_matrix/checkpoint_recover_compile.rs::certify_checkpoint_window_recover_compile_round_trip` -> `chip_simulator_matrix/mod.rs::perf_chip_simulator_matrix`.
- `geometry_kernel_matrix/identity_survival.rs::certify_topology_identity_survival_recovery_round_trip` -> `geometry_kernel_matrix/mod.rs::perf_geometry_kernel_matrix`.
- `hot_cold_path_matrix/{geometry_replay_reconstruction,geometry_publication_replay_truth,chip_rich_recovery_compile,chip_recovery_compile}.rs` -> `hot_cold_path_matrix/mod.rs::perf_hot_cold_path_matrix`.
- `sustained_load_matrix/replay_window_drift.rs::certify_replay_window_drift_stability` -> `sustained_load_matrix/mod.rs::perf_sustained_load_matrix`.
- All seven inventoried `merge_lineage_matrix/*.rs` helpers -> `merge_lineage_matrix/mod.rs::perf_merge_lineage_matrix`.
- Both inventoried `workflow_matrix/{persisted_recovery_replay,trade_correction_analysis}.rs` helpers -> `workflow_matrix/mod.rs::perf_workflow_matrix`.

Private semantic planning observations outside resident/dictionary/pin/allocation/cost categories still require an owner/value-boundary decision. For example, `merge/data/execution.rs` declares `execution_ready_plan` and `bound_executable_plan` as `pub(crate)`; importing their data type through a facade does not make the prepared accessor public. Do not blindly move `readiness_admission`, `record_plan_compilation`, or mutated prepared-plan tests into the public court. Their unresolved rows are intentional.

## Named test claims

Source expressions below are direct assertions in actual bodies. The source line identifies the full expression where an excerpt is shortened. C4 action cues identify concrete source controls. Unlisted semantic decisions remain unresolved. Assertions in called helpers cannot disappear when a caller moves.

### `branch/coordination/managed_fork_cleanup.rs`

#### `managed_fork_refusal_and_success_drop_lookup_outside_native_coordination`

Source: `crates/worth-relational/src/branch/coordination/managed_fork_cleanup.rs:92`.

- C4 action 123: `medium.script_next_transition(ScriptedPublication::ProvenNoEffect);`.
- **C4** 126: `assert!(matches!( stopped, Err(crate::branch::RelationalBranchTransitionStop::Denied( crate::branch::RelationalForkDenial::DurablePublicationRefused( crate::durability::medium::RelationalDurablePublicationRefusal::ProvenNoEffect { .. } ) )) ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 135: `assert_eq!(medium.offered_transitions().len(), 1)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 136: `assert!(medium.transition_sequences().is_empty())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 137: `assert_eq!(drops.load(Ordering::SeqCst), 1)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 138: `assert!(targets.lock().unwrap().is_empty())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 139: `assert!(runtime.branch_identity(&target).is_err())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C2** 142: `assert!(memory.charged_memory_bytes() > 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C4** 146: `assert_eq!(drops.load(Ordering::SeqCst), 2)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 147: `assert!(targets.lock().unwrap()[&target].admitted().is_some())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C2** 148: `assert!(memory.charged_memory_bytes() > 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 152: `assert_eq!(memory.charged_memory_bytes(), 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.

### `branch/owner_services/materialization_port_tests.rs`

#### `durability_replays_suspended_and_restored_materialization_roots`

Source: `crates/worth-relational/src/branch/owner_services/materialization_port_tests.rs:19`.

- **HAPPY** 50: `assert!(matches!( recovered .begin_branch_transaction(&recovered_basis, RelationalTransactionIntent::ordinary()), Err(RelationalBranchTransactionAdmissionDenial::MaterializationUnavailable) ))` - reviewed public owner result over explicit durable provider.
- **HAPPY** 99: `assert_eq!( read.entities() .iter() .map(|record| record.entity_id) .collect::<Vec<_>>(), entities )` - reviewed public owner result over explicit durable provider.
- **HAPPY** 106: `assert_eq!(read.relations()[0].relation_id, relation)` - reviewed public owner result over explicit durable provider.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/branch/owner_services/materialization_port_tests.rs:191 assert_suspended_records`, `crates/worth-relational/src/branch/owner_services/materialization_port_tests.rs:109 assert_cold_inventory`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

### `branch/root_authoritative_payload_tests.rs`

#### `content_old_retained_contract_cannot_hide_a_reference_after_current_binding_is_removed`

Source: `crates/worth-relational/src/branch/root_authoritative_payload_tests.rs:13`.

- C4 action 100: `envelope.body_mut_for_test().schema_authority = changed.authority_snapshot();`.
- **C4** 138: `assert!( matches!( refusal, RelationalBranchRootCaptureDenial::Content( RelationalContentDenial::FormatBindingMissing ) ), "{refusal:?}" )` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `strategy_artifacts_are_bound_into_the_committed_root`

Source: `crates/worth-relational/src/branch/root_authoritative_payload_tests.rs:150`.

- C4 action 165: `.body_mut_for_test()`.
- **C4** 164: `assert!(envelope .body_mut_for_test() .strategy_artifacts .take() .is_some())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/branch/root_authoritative_payload_tests.rs:317 assert_axis_mutation_rejected`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `merge_execution_authority_is_bound_into_the_committed_root`

Source: `crates/worth-relational/src/branch/root_authoritative_payload_tests.rs:174`.

- C4 action 210: `.body_mut_for_test()`.
- **C4** 209: `assert!(envelope .body_mut_for_test() .merge_execution_authority .take() .is_some())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/branch/root_authoritative_payload_tests.rs:317 assert_axis_mutation_rejected`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `schema_transition_descriptors_are_each_bound_into_the_committed_root`

Source: `crates/worth-relational/src/branch/root_authoritative_payload_tests.rs:225`.

- C4 action 231: `runtime.set_schema_registry_for_test(`.
- C4 action 261: `.body_mut_for_test()`.
- C4 action 268: `.body_mut_for_test()`.
- C4 action 281: `.body_mut_for_test()`.
- **C4** 260: `assert!(transition .body_mut_for_test() .schema_transition .take() .is_some())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 267: `assert!(continuation .body_mut_for_test() .schema_continuation_descriptor .take() .is_some())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 280: `assert!(reconciliation .body_mut_for_test() .schema_reconciliation_descriptor .take() .is_some())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/branch/root_authoritative_payload_tests.rs:317 assert_axis_mutation_rejected`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

### `branch/root_tests.rs`

#### `relink_rejects_schema_authority_mutation`

Source: `crates/worth-relational/src/branch/root_tests.rs:35`.

- C4 action 41: `.body_mut_for_test()`.
- **C4** 53: `assert!(matches!( denial, RelationalBranchRootCaptureDenial::SchemaRootMismatch { .. } ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `relink_rejects_visibility_tuple_mutation`

Source: `crates/worth-relational/src/branch/root_tests.rs:60`.

- C4 action 66: `.body_mut_for_test()`.
- **C4** 79: `assert!(matches!( denial, RelationalBranchRootCaptureDenial::VisibilityCommitmentMismatch { .. } ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `visibility_commitment_binds_every_visible_tuple_axis`

Source: `crates/worth-relational/src/branch/root_tests.rs:86`.

- C4 action 111: `commit.body_mut_for_test().commit.commit_id =`.
- C4 action 112: `CommitId(commit.body_mut_for_test().commit.commit_id.0 + 1);`.
- C4 action 115: `version.body_mut_for_test().commit.version_id.0 += 1;`.
- C4 action 118: `parents.body_mut_for_test().commit.parents = vec![CommitId(41), CommitId(43)];`.
- C4 action 121: `parents.body_mut_for_test().commit.parents = vec![CommitId(43), CommitId(41)];`.
- C4 action 124: `branch.body_mut_for_test().branch_context = BranchId("visibility-mutant".to_owned());`.
- C4 action 127: `receipt_branch.body_mut_for_test().commit.branch_id =`.
- C4 action 131: `checkpoint.body_mut_for_test().branch_cell_checkpoint = None;`.
- C4 action 134: `authority_kind.body_mut_for_test().authority_kind =`.
- C4 action 139: `.body_mut_for_test()`.
- C4 action 145: `.body_mut_for_test()`.
- C4 action 150: `schema_version.body_mut_for_test().schema_version.0 += 1;`.
- C4 action 154: `.body_mut_for_test()`.
- C4 action 159: `plan.body_mut_for_test().merged_plan.transaction_id.0 += 1;`.
- C4 action 164: `.body_mut_for_test()`.
- **C4** 100: `assert_ne!( production_visibility(&envelope, [18; 32], schema_root), baseline )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 104: `assert_ne!( production_visibility(&envelope, storage_root, [30; 32]), baseline )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 162: `assert!(!allocations.record_allocations().is_empty())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 185: `assert_ne!( production_visibility(&mutant, storage_root, schema_root), baseline, "{axis} must change the visibility commitment" )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 190: `assert!( root.relink_canonical_envelope( execution, Arc::new(mutant), &runtime.services.symbols.interner_snapshot(), ) .is_err(), "{axis} mutation must not relink to the committed root" )` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `commit_strategies/facade_tests/native_strategy_fixtures.rs`

No direct test declarations. Executing helper roots and claims:


### `commit_strategies/facade_tests/replay_recovery_failures.rs`

#### `replay_commit_reports_strategy_executor_unavailable_when_recovered_runtime_lacks_executor`

Source: `crates/worth-relational/src/commit_strategies/facade_tests/replay_recovery_failures.rs:5`.

- **UNCLASSIFIED** 44: `assert_eq!(replay.failure, Some(ReplayFailureClass::ObservableMismatch))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 45: `assert!(replay .compared_surfaces .contains(&ReplayObservableSurface::Strategy))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 48: `assert!(replay.mismatches.iter().any(|mismatch| { mismatch.class == ReplayMismatchClass::StrategyExecutorUnavailable && mismatch.surface == ReplayObservableSurface::Strategy }))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 52: `assert_eq!( recovered .history() .branch_head(&BranchId("main".to_string())), branch_head_after_recovery )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 58: `assert_eq!(branch_head_after_recovery, branch_head_before)` - semantic boundary still requires direct review; no move authorized.

#### `replay_commit_reports_strategy_execution_failure_when_recovered_executor_rejects`

Source: `crates/worth-relational/src/commit_strategies/facade_tests/replay_recovery_failures.rs:62`.

- **UNCLASSIFIED** 100: `assert_eq!(replay.failure, Some(ReplayFailureClass::ObservableMismatch))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 101: `assert!(replay .compared_surfaces .contains(&ReplayObservableSurface::Strategy))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 104: `assert!(replay.mismatches.iter().any(|mismatch| { mismatch.class == ReplayMismatchClass::StrategyExecutionFailure && mismatch.surface == ReplayObservableSurface::Strategy }))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 108: `assert_eq!( recovered .history() .branch_head(&BranchId("main".to_string())), branch_head_before_replay )` - semantic boundary still requires direct review; no move authorized.

### `durability/access/medium_recovery_plan.rs`

No direct test declarations. Executing helper roots and claims:


### `durability/access/mod.rs`

No direct test declarations. Executing helper roots and claims:


### `durability/access/recovery_plan_entrypoint.rs`

No direct test declarations. Executing helper roots and claims:

- `recovery_plan_from_source` at 35: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/branch/owner_services/materialization_port_tests.rs:19 durability_replays_suspended_and_restored_materialization_roots`, `crates/worth-relational/src/commit_strategies/facade_tests/replay_recovery_failures.rs:5 replay_commit_reports_strategy_executor_unavailable_when_recovered_runtime_lacks_executor`, `crates/worth-relational/src/commit_strategies/facade_tests/replay_recovery_failures.rs:62 replay_commit_reports_strategy_execution_failure_when_recovered_executor_rejects`, `crates/worth-relational/src/durability/authority/checkpoint_concurrency_tests.rs:43 checkpoint_reconstruction_does_not_exclude_publication_after_capture`, `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:23 fresh_source_keeps_actual_runtime_root_and_installed_schema_for_complete`, `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:168 fresh_source_refuses_foreign_and_nonempty_actual_sources_without_rebuild`, `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:196 fresh_receipt_refuses_actual_movement_and_preserves_original_cancellation`, `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:242 fresh_source_refuses_real_selected_empty_checkpoint`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:132 shared_restored_substrate_keeps_a_pinned_reader_exact_after_next_write`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:179 divergent_sibling_is_not_reused_as_the_storage_mirror`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:299 recovered_main_and_sibling_catalogs_reuse_exact_canonical_payloads`, `crates/worth-relational/src/durability/log/native_checkpoint_codec/kind_authentication_tests.rs:55 forged_cross_region_kind_commitment_is_refused_before_member_reads`, `crates/worth-relational/src/durability/log/native_checkpoint_codec/tests.rs:10 member_directory_preserves_divergent_sibling_roots_and_pinned_reader`, `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs:147 checkpoint_recovery_preserves_abandoned_symbol_identity_frontiers`, `crates/worth-relational/src/storage/residency/cross_region_forgery_tests.rs:184 rehashed_cross_region_kind_forgery_is_refused_by_the_canonical_root`, `crates/worth-relational/src/tests/complexity/contracts/commit_budgets/schema_continuity_boundaries.rs:110 complexity_budget_milestone5_closeout_keeps_schema_cdc_and_recovery_boundary_local`, `crates/worth-relational/src/tests/domains/cad/topology_identity_survival.rs:10 topology_identity_survival_preserves_reidentification_truth_across_recovery`, `crates/worth-relational/src/tests/domains/fintech/workflows/persistence.rs:9 fintech_persisted_workflow_recovers_checkpoint_tail_and_keeps_queryable_portfolio_state`, `crates/worth-relational/src/tests/durability/contracts/adjacency_checkpoint.rs:4 checkpoint_rebuilds_current_and_historical_adjacency_kind_buckets`, `crates/worth-relational/src/tests/durability/contracts/associated_effects.rs:77 associated_effects_discover_cold_subject_without_reading_unrelated_history`, `crates/worth-relational/src/tests/durability/contracts/associated_effects_discovery.rs:11 native_custody_correction_enumerates_known_orphan_and_unrelated_association_metadata`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:5 durability_contract_recovery_rebuilds_branch_heads_and_latest_commit`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:38 durability_contract_recovery_preserves_aspect_bearing_patch_truth_and_history`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:95 durability_contract_recovery_preserves_relation_aspect_history_for_retained_audit_relations`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:160 durability_contract_recovery_preserves_branch_local_endpoint_deletion_retirement_histories`.

### `durability/authority/checkpoint_complete/tests/empty_schema.rs`

#### `complete_empty_archive_and_initial_installation_preserve_unused_original_schema`

Source: `crates/worth-relational/src/durability/authority/checkpoint_complete/tests/empty_schema.rs:9`.

- **UNCLASSIFIED** 23: `assert!(!old_schema.registry().entity_kinds.is_empty())` - semantic boundary still requires direct review; no move authorized.
- **C2** 68: `assert!(!std::sync::Arc::ptr_eq(&old_schema, &replacement_schema))` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 70: `assert_ne!(expected, expected_replacement)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 74: `assert!( old_root.upgrade().is_none(), "schema DATA cannot depend on the retired original Empty Root" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 86: `assert!(held > 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 96: `assert!(directory.manifest.commits.is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 97: `assert!(directory.manifest.canonical_members.entries().is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 98: `assert_eq!(directory.manifest.coverage.up_to_branch_transition.0, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 99: `assert_eq!(directory.manifest.retained_empty_components.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 100: `assert!(directory.manifest.schemas.iter().any(|image| image .readmit_registry() .unwrap() .0 .authority_snapshot() == expected))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 106: `assert!( directory.manifest.schemas.iter().any(|image| { let (schema, _) = image.readmit_registry().unwrap(); schema.authority_snapshot() == expected_replacement && schema .relation_kinds .get(&crate::identity::data::KindId(901)) .is_some_and(|registration| { registration.relation_integrity == expected_constraint }) }), "the unused physical constraint must survive in the full original image" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 119: `assert!(directory.manifest.roots.is_empty())` - semantic boundary still requires direct review; no move authorized.
- **C2** 122: `assert!( budget.charged_memory_bytes() > 0, "same real Native alias keeps the prepared output custody" )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 127: `assert_eq!(budget.charged_memory_bytes(), 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.

#### `complete_refuses_recovered_empty_without_original_birth_authority`

Source: `crates/worth-relational/src/durability/authority/checkpoint_complete/tests/empty_schema.rs:130`.

- **UNCLASSIFIED** 143: `assert!(matches!( receiver .complete_checkpoint_service() .prepare(&execution, RelationalCheckpointArtifactPolicy::Embedded), Err(Denial::OriginalAuthorityUnavailable) ))` - semantic boundary still requires direct review; no move authorized.

### `durability/authority/checkpoint_complete/tests/partial_aspect_update.rs`

#### `complete_preserves_partial_update_without_demanding_untouched_aspect_reference`

Source: `crates/worth-relational/src/durability/authority/checkpoint_complete/tests/partial_aspect_update.rs:11`.

- **UNCLASSIFIED** 20: `assert!(plan.executable_bindings.len() >= 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 26: `assert_eq!(untouched.aspect_key().as_str(), "lifecycle")` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 41: `assert!(envelope .revision_symbol_references .binary_search(&changed_symbol) .is_ok())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 45: `assert!( envelope .revision_symbol_references .binary_search(&untouched_symbol) .is_err(), "a name-only update has no lifecycle revision reference" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 67: `assert_eq!(lifecycle, RecordLifecycleState::Live)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 88: `assert_eq!(image.entity_arena.lifecycle[physical], lifecycle)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 112: `assert_eq!(text, "updated-name")` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 113: `assert_eq!(original.axes(), axes)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 114: `assert_eq!(arena.lifecycle[physical], lifecycle)` - semantic boundary still requires direct review; no move authorized.

### `durability/authority/checkpoint_complete/tests/pinned_delete.rs`

#### `complete_fold_matches_original_relation_delete_with_snapshot_and_replay_pins`

Source: `crates/worth-relational/src/durability/authority/checkpoint_complete/tests/pinned_delete.rs:9`.

- **UNCLASSIFIED** 33: `assert_eq!( creation_basis .reference() .target() .as_basis() .unwrap() .selected_commit_id(), created.commit.commit_id.0, )` - semantic boundary still requires direct review; no move authorized.
- **C2** 42: `assert!(std::sync::Arc::ptr_eq( &creation_basis.inner.root, &old_root ))` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 46: `assert_eq!(creation_basis.inner.root.axes(), old_root.axes())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 48: `assert_eq!(external.descriptor(), creation_basis.descriptor())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 52: `assert!(runtime .history_authority() .retain_version_for_replay(&execution, created.version_id) .unwrap())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 61: `assert!(runtime .inspect_what_happened() .inspect_snapshot_pinning(&execution, &created.snapshot) .unwrap() .is_some())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 66: `assert!(before.pins.replay_pins > 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 84: `assert!(pinned.pins.replay_pins > 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 95: `assert_eq!(old_relation.lifecycle, RecordLifecycleState::Live)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 96: `assert_eq!((old_relation.source, old_relation.target), (source, target))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 113: `assert_eq!(lifecycle, RecordLifecycleState::DeletedRetained)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 114: `assert_eq!(arena.snapshot_pin_count(relation.slot_index()), Some(0))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 115: `assert_eq!(arena.replay_pin_count(relation.slot_index()), Some(0))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 123: `assert_eq!( old_arena.lifecycle[old_physical], RecordLifecycleState::Live )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 149: `assert_eq!(image.relation_arena.lifecycle[physical], lifecycle)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 150: `assert_eq!( image.relation_arena.retired_at[physical], Some(deleted.version_id) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 164: `assert_eq!( retained.relation_arena.lifecycle[old_physical], RecordLifecycleState::Live )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 168: `assert_eq!(original.axes(), original_axes)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 169: `assert_eq!(arena.lifecycle[physical], lifecycle)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 171: `assert!(runtime .history_authority() .release_version_replay_retention(created.version_id))` - semantic boundary still requires direct review; no move authorized.

### `durability/authority/checkpoint_concurrency_tests.rs`

#### `completing_an_older_capture_cannot_replace_the_compacted_transition_basis`

Source: `crates/worth-relational/src/durability/authority/checkpoint_concurrency_tests.rs:9`.

- **C2** 28: `assert_eq!(new.coverage.up_to_branch_transition.0, 1)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 34: `assert_eq!(selected.coverage.up_to_branch_transition.0, 1)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 35: `assert!(runtime .durability() .branch_transition_artifacts(execution.allocation_policy()) .next() .is_none())` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.

#### `checkpoint_reconstruction_does_not_exclude_publication_after_capture`

Source: `crates/worth-relational/src/durability/authority/checkpoint_concurrency_tests.rs:43`.

- **UNCLASSIFIED** 116: `assert_eq!( checkpoint .coverage .up_to_commit .as_ref() .map(|commit| commit.commit_id), Some(baseline.commit.commit_id) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 133: `assert_eq!(plan.tail_log.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 134: `assert_eq!(plan.tail_log[0].position(), performed_position)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 135: `assert_eq!( plan.tail_log[0].envelope().commit.commit_id, performed_commit_id )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 144: `assert_eq!(outcome.coverage.replayed_tail_commits, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 145: `assert_eq!( recovered .history() .branch_head(&BranchId("main".to_owned())) .expect("recovered main head includes concurrent publication") .commit_id, performed_commit_id )` - semantic boundary still requires direct review; no move authorized.

### `durability/authority/checkpoint_transition/tests.rs`

#### `acknowledged_transition_admits_only_its_exact_successor_and_releases_snapshot`

Source: `crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs:59`.

- **HAPPY** 76: `assert_eq!(transition.commit(), &expected_receipt)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 78: `assert_eq!(runtime.history().immutable_commit_count(), count + 1)` - reviewed public owner result over explicit durable provider.
- **C2** 79: `assert_eq!(runtime.visibility.published_snapshot_handle_count(), 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 80: `assert_eq!( runtime .preparation_runtime_snapshot() .published_snapshot_count(), 0 )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **HAPPY** 86: `assert_eq!( runtime .durability() .durable_log() .iter() .filter(|entry| entry.envelope().commit.commit_id == receipt.commit_id) .count(), 1 )` - reviewed public owner result over explicit durable provider.
- **HAPPY** 103: `assert!(successor_authority .admit_basis(main_basis(&runtime)) .is_ok())` - reviewed public owner result over explicit durable provider.
- **HAPPY** 107: `assert!(successor_authority.admit_basis(predecessor).is_err())` - reviewed public owner result over explicit durable provider.

#### `foreign_recovery_and_foreign_candidate_are_refused_before_effects`

Source: `crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs:113`.

- **HAPPY** 142: `assert_eq!(main_basis(&runtime).descriptor(), &before)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 143: `assert_eq!(main_basis(&foreign).descriptor(), &before_foreign)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 144: `assert_eq!(runtime.history().immutable_commit_count(), count)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 145: `assert_eq!(foreign.history().immutable_commit_count(), foreign_count)` - reviewed public owner result over explicit durable provider.
- **C2** 146: `assert_eq!(runtime.visibility.published_snapshot_handle_count(), 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 147: `assert_eq!(foreign.visibility.published_snapshot_handle_count(), 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **HAPPY** 149: `assert!(matches!( refusal.denial(), Denial::ForeignRecoveryRuntime { .. } ))` - reviewed public owner result over explicit durable provider.
- **HAPPY** 153: `assert!(refusal .into_authority() .admit_basis(main_basis(&foreign)) .is_ok())` - reviewed public owner result over explicit durable provider.
- **HAPPY** 158: `assert!(matches!( refusal.denial(), Denial::ForeignCandidateRuntime { .. } ))` - reviewed public owner result over explicit durable provider.
- **HAPPY** 167: `assert!(acknowledged .into_parts() .1 .admit_basis(main_basis(&runtime)) .is_ok())` - reviewed public owner result over explicit durable provider.

#### `moved_source_and_stale_candidate_cannot_extend_recovery_lineage`

Source: `crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs:177`.

- **HAPPY** 199: `assert!(matches!(refusal.denial(), Denial::Publication(outcome) if matches!(outcome.as_ref(), RelationalPublicationOutcome::Stale(_))))` - reviewed public owner result over explicit durable provider.
- **HAPPY** 202: `assert!(matches!(refusal.denial(), Denial::SourceNotRecovered(_)))` - reviewed public owner result over explicit durable provider.
- **HAPPY** 204: `assert_eq!(main_basis(&runtime).descriptor(), &before)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 205: `assert_eq!(runtime.history().immutable_commit_count(), count)` - reviewed public owner result over explicit durable provider.
- **C2** 206: `assert_eq!(runtime.visibility.published_snapshot_handle_count(), 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **HAPPY** 207: `assert!(refusal.into_authority().admit_basis(predecessor).is_ok())` - reviewed public owner result over explicit durable provider.

#### `refused_durable_transition_returns_its_authority_and_moves_nothing`

Source: `crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs:212`.

- C4 action 220: `medium.script_next_publication(ScriptedPublication::ProvenNoEffect);`.
- **C4** 229: `assert!(matches!(refusal.denial(), Denial::Publication(outcome) if matches!(outcome.as_ref(), RelationalPublicationOutcome::Denied( crate::mvcc::RelationalPublicationDenial::DurablePublicationRefused(_) ))))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 233: `assert_eq!(main_basis(&runtime).descriptor(), predecessor.descriptor())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 234: `assert_eq!(runtime.history().immutable_commit_count(), count)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C2** 235: `assert_eq!(runtime.visibility.published_snapshot_handle_count(), 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C4** 236: `assert_eq!(medium.head(), PatchStreamPosition(1))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 244: `assert_eq!(medium.head(), PatchStreamPosition(2))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 245: `assert_eq!(runtime.history().immutable_commit_count(), count + 1)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 246: `assert!(transition .into_parts() .1 .admit_basis(main_basis(&runtime)) .is_ok())` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `unresolved_durable_transition_installs_nothing_and_names_its_fate`

Source: `crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs:254`.

- C4 action 262: `medium.script_next_publication(ScriptedPublication::IndeterminateWithoutEffect);`.
- **C4** 286: `assert_eq!(position, offered.position())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 287: `assert_eq!(digest, offered.payload_digest())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 288: `assert_eq!(main_basis(&runtime).descriptor(), predecessor.descriptor())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 289: `assert_eq!(runtime.history().immutable_commit_count(), count)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C2** 290: `assert_eq!(runtime.visibility.published_snapshot_handle_count(), 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.

### `durability/authority/fresh_source/tests.rs`

#### `fresh_source_keeps_actual_runtime_root_and_installed_schema_for_complete`

Source: `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:23`.

- **HAPPY** 93: `assert_eq!(runtime.runtime_instance_id(), id)` - reviewed public owner result over explicit durable provider.
- **C2** 94: `assert!(Arc::ptr_eq( &runtime.durability.cold_source().unwrap(), &source ))` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 104: `assert!(Arc::ptr_eq(&original, &actual))` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 105: `assert!(Arc::ptr_eq(&schema, &actual.retained_schema_authority()))` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 116: `assert_eq!( image .canonical_directory .as_ref() .unwrap() .manifest .retained_empty_components .len(), 1 )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 126: `assert!(image .canonical_directory .as_ref() .unwrap() .manifest .schemas .iter() .any( |image| image.readmit_registry().unwrap().0.authority_snapshot() == schema.registry().authority_snapshot() ))` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 137: `assert!( image .canonical_directory .as_ref() .unwrap() .manifest .schemas .iter() .any(|image| { let (registry, _) = image.readmit_registry().unwrap(); registry .relation_kinds .get(&crate::identity::data::KindId(901)) .is_some_and(|kind| kind.relation_integrity == expected_constraint) }), "unused physical constraint comes from original installation, not records" )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **HAPPY** 160: `assert!(runtime .validate_fresh_initial_schema_installation(&execution, &admitted, &wrong) .is_err())` - reviewed public owner result over explicit durable provider.
- **HAPPY** 163: `assert!(admitted .validate_fresh_source_runtime(&execution, &foreign) .is_err())` - reviewed public owner result over explicit durable provider.

#### `fresh_source_refuses_foreign_and_nonempty_actual_sources_without_rebuild`

Source: `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:168`.

- **UNCLASSIFIED** 179: `assert!(runtime .durability_recovery() .admit_fresh_home_source(&execution, plan) .is_err())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 183: `assert_eq!(runtime.runtime_instance_id(), original)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 188: `assert!(runtime .durability_recovery() .admit_fresh_home_source(&execution, stale) .is_err())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 192: `assert_eq!(runtime.runtime_instance_id(), original)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 193: `assert!(runtime.history().latest_commit().is_some())` - semantic boundary still requires direct review; no move authorized.

#### `fresh_receipt_refuses_actual_movement_and_preserves_original_cancellation`

Source: `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:196`.

- **UNCLASSIFIED** 217: `assert!(matches!( admitted.validate_fresh_source_runtime(&stopped_execution, &runtime), Err(Stop::Request(_)) ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 226: `assert!(matches!( runtime .durability_recovery() .admit_fresh_home_source(&stopped_execution, plan), Err(RelationalFreshSourceAdmissionStop::Recovery(Stop::Request( _ ))) ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 236: `assert!(admitted .validate_fresh_source_runtime(&execution, &runtime) .is_err())` - semantic boundary still requires direct review; no move authorized.

#### `fresh_source_refuses_real_selected_empty_checkpoint`

Source: `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:242`.

- **UNCLASSIFIED** 253: `assert!(plan.checkpoint.is_some())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 255: `assert!(runtime .durability_recovery() .admit_fresh_home_source(&execution, plan) .is_err())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 259: `assert_eq!(runtime.runtime_instance_id(), id)` - semantic boundary still requires direct review; no move authorized.

### `durability/authority/runtime_rebuild/checkpoint_restore_tests.rs`

#### `equal_exact_root_reuses_the_readmitted_partition_substrate`

Source: `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:12`.

- **C2** 20: `assert_eq!(checkpoint.branch_roots.len(), 1)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 21: `assert_eq!( checkpoint.branch_roots[0].partition_images, checkpoint.partition_images )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 31: `assert!(shares_first_generation(&mirror, root))` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.

#### `native_restore_work_counts_revised_history_sibling_roots_and_partition_images`

Source: `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:35`.

- **C2** 56: `assert!( grown.captured_sections().unwrap().partition_mirror > small.captured_sections().unwrap().partition_mirror )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 73: `assert!( native.captured_sections().unwrap().envelopes > grown.captured_sections().unwrap().envelopes )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 90: `assert_eq!(work.native_bytes_read, Some(native.bytes().len()))` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 91: `assert_eq!( work.native_envelopes_readmitted, Some(decoded.envelopes.len()) )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 95: `assert_eq!(work.root_images_verified, decoded.branch_roots.len())` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 96: `assert_eq!( work.root_partition_images_restored, decoded .branch_roots .iter() .map(|root| root.partition_images.len()) .sum::<usize>() )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 104: `assert_eq!( work.mirror_partition_images_examined, decoded.partition_images.len() )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 108: `assert_eq!( work.mirror_partitions_reused + work.mirror_partitions_reconstructed, decoded.partition_images.len() )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 112: `assert!(work.mirror_partitions_reused > 0)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 113: `assert_eq!(work.history_envelopes_routed, decoded.envelopes.len())` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 114: `assert_eq!(work.branch_cells_readmitted, decoded.branch_cells.len())` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 115: `assert_eq!( work.index_definitions_readmitted, decoded.index_definitions.len() )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 119: `assert_eq!( source .read_truth() .read_snapshot(&pinned, execution) .expect("read request remains live") .unwrap() .entities() .len(), 2 )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.

#### `shared_restored_substrate_keeps_a_pinned_reader_exact_after_next_write`

Source: `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:132`.

- **C2** 155: `assert_eq!( recovered .read_truth() .read_snapshot(&pinned, execution) .expect("read request remains live") .unwrap() .entities() .len(), 1 )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 166: `assert_eq!( recovered .read_truth() .read_snapshot(&latest, execution) .expect("read request remains live") .unwrap() .entities() .len(), 2 )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.

#### `divergent_sibling_is_not_reused_as_the_storage_mirror`

Source: `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:179`.

- **UNCLASSIFIED** 192: `assert_eq!(checkpoint.branch_roots.len(), 2)` - semantic boundary still requires direct review; no move authorized.
- **C2** 203: `assert!(shares_first_generation(&mirror, main_root))` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 204: `assert!(!shares_first_generation(&mirror, sibling_root))` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 221: `assert_eq!( recovered .read_truth() .read_snapshot(&snapshot, execution) .expect("read request remains live") .unwrap() .entities() .len(), expected )` - semantic boundary still requires direct review; no move authorized.

#### `divergent_mirror_partition_rebuilds_cross_partition_adjacency_after_reuse`

Source: `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:235`.

- **UNCLASSIFIED** 284: `assert!(!shares_partition_generation( &mirror, root, PartitionId::main() ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 289: `assert!(shares_partition_generation(&mirror, root, PartitionId(41)))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 292: `assert!(std::ptr::eq( mirror_relation.relation_arena.generations.get(0).unwrap(), root_relation.relation_arena.generations.get(0).unwrap(), ))` - semantic boundary still requires direct review; no move authorized.

#### `recovered_main_and_sibling_catalogs_reuse_exact_canonical_payloads`

Source: `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:299`.

- **C2** 333: `assert_eq!(readmitted.identity(), original.identity())` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 334: `assert_eq!(readmitted.parentage(), original.parentage())` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 335: `assert_eq!(readmitted.roots(), original.roots())` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 336: `assert_eq!( readmitted .canonical_payload_digest() .expect("resident fixture has a canonical digest"), original .canonical_payload_digest() .expect("resident fixture has a canonical digest") )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.

#### `equal_image_cannot_bypass_global_contract_readmission`

Source: `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:348`.

- **UNCLASSIFIED** 355: `assert_eq!( checkpoint.branch_roots[0].partition_images, checkpoint.partition_images )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 365: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 366: `assert!(error.detail.contains("aspect readmission denied"))` - semantic boundary still requires direct review; no move authorized.

### `durability/canonical_commit_artifact/retained_export/skip_tests.rs`

#### `seeking_past_an_acknowledged_index_prefix_never_demands_its_journal_bodies`

Source: `crates/worth-relational/src/durability/canonical_commit_artifact/retained_export/skip_tests.rs:35`.

- **UNCLASSIFIED** 86: `assert_eq!(selected.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 87: `assert_eq!(selected[0].as_bytes(), artifacts[3].as_bytes())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 88: `assert_eq!(source.reads.load(Ordering::Relaxed), 1)` - semantic boundary still requires direct review; no move authorized.

### `durability/data/checkpoint_images.rs`

#### `streamed_branch_root_image_digest_matches_buffered_wire_for_divergent_roots`

Source: `crates/worth-relational/src/durability/data/checkpoint_images.rs:319`.

- **UNCLASSIFIED** 333: `assert_eq!(checkpoint.branch_roots.len(), 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 336: `assert_eq!( branch_root_partition_image_digest(&root.partition_images).unwrap(), buffered_digest(&root.partition_images) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 344: `assert_eq!( branch_root_partition_image_digest(&altered).unwrap(), buffered_digest(&altered) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 348: `assert_ne!( branch_root_partition_image_digest(&altered).unwrap(), original )` - semantic boundary still requires direct review; no move authorized.

### `durability/data/native_checkpoint/component_custody/tests.rs`

#### `native_custody_correction_delayed_canonical_selection_retains_actual_root_and_interrupted_ack`

Source: `crates/worth-relational/src/durability/data/native_checkpoint/component_custody/tests.rs:101`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.

#### `native_custody_correction_delayed_empty_selection_retains_actual_schema_and_interrupted_ack`

Source: `crates/worth-relational/src/durability/data/native_checkpoint/component_custody/tests.rs:105`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.

#### `native_custody_correction_last_actual_capture_abandonment_releases_shared_root_custody`

Source: `crates/worth-relational/src/durability/data/native_checkpoint/component_custody/tests.rs:109`.

- **UNCLASSIFIED** 136: `assert!(original.upgrade().is_some())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 138: `assert!( original.upgrade().is_none(), "last actual unselected capture abandonment releases its original observation" )` - semantic boundary still requires direct review; no move authorized.

### `durability/data/native_checkpoint/tests/selected_ack.rs`

#### `selected_ack_partial_transfer_preserves_later_cells_and_resumes_same_capture`

Source: `crates/worth-relational/src/durability/data/native_checkpoint/tests/selected_ack.rs:24`.

- **UNCLASSIFIED** 76: `assert_eq!(observed, Some(true))` - semantic boundary still requires direct review; no move authorized.
- **C2** 77: `assert!(cells[0].selected_directory().is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 78: `assert!( cells[1..] .iter() .all(|cell| cell.selected_directory().is_none()), "a real stop leaves every later covered cell untouched" )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 84: `assert!(later_cell.selected_directory().is_none())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 85: `assert!(later_cell .source_placement() .unwrap() .0 .same_owner(&later_entry))` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 90: `assert!( runtime .durability .checkpoint_selection() .snapshot() .is_none(), "partial cell transfer is not directory placement or completion" )` - semantic boundary still requires direct review; no move authorized.
- **C2** 103: `assert!(Arc::ptr_eq( &original, checkpoint.publication.as_ref().unwrap() ))` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 108: `assert!(cell.selected_directory().is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 109: `assert!(cell.source_placement().unwrap().0.same_owner(entry))` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 111: `assert!(later_cell.selected_directory().is_none())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 112: `assert!(later_cell .source_placement() .unwrap() .0 .same_owner(&later_entry))` - actual residency, identity, pin, allocation, dictionary or cost observation.

#### `selected_ack_exposes_backed_data_before_retirement_and_releases_holds_only_on_completion`

Source: `crates/worth-relational/src/durability/data/native_checkpoint/tests/selected_ack.rs:119`.

- **UNCLASSIFIED** 192: `assert!(retained_basis.upgrade().is_some())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 203: `assert!(refused)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 209: `assert_eq!(selected.manifest.covered_position, covered)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 210: `assert!(selected.manifest.index_definition_frontier > old_frontier)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 211: `assert_eq!( runtime .durability .checkpoint_selection() .acknowledged_index_definition_frontier(), old_frontier, "DATA availability is not completed catalog acceptance" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 219: `assert!( prior.upgrade().is_some(), "displaced read is retained before retirement progress" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 223: `assert!( retained_basis.upgrade().is_some(), "DATA pointer placement cannot release capture holds or complete ACK" )` - semantic boundary still requires direct review; no move authorized.
- **C2** 230: `assert!(Arc::ptr_eq(&selected, &directory))` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 232: `assert_eq!(frame.commit_id(), artifact.commit_id())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 239: `assert_eq!( canonical.index_definition_delta.definitions(), &[definition] )` - semantic boundary still requires direct review; no move authorized.
- **C2** 255: `assert!(Arc::ptr_eq( &original, checkpoint.publication.as_ref().unwrap() ))` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 259: `assert!(later_cell.selected_directory().is_none())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 260: `assert!(later_cell .source_placement() .unwrap() .0 .same_owner(&later_entry))` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 265: `assert_eq!( runtime .durability .checkpoint_selection() .acknowledged_index_definition_frontier(), selected.manifest.index_definition_frontier )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 272: `assert!(prior.upgrade().is_none())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 273: `assert!( retained_basis.upgrade().is_none(), "only actual completed ACK releases original component holds" )` - semantic boundary still requires direct review; no move authorized.

### `durability/log/native_checkpoint_codec/inspection/tests.rs`

#### `inspection_compares_every_native_model_value_as_original_bytes`

Source: `crates/worth-relational/src/durability/log/native_checkpoint_codec/inspection/tests.rs:130`.

- **UNCLASSIFIED** 136: `assert!( decoded.live_records.contains(&identity), "the census uses the actual committed identity: {identity:?}" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 154: `assert_ne!(decoded.model, after.model, "ignored model field {key}")` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 155: `assert!( equivalent.is_none(), "equivalent {key} encoding refused: {error:?}" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 172: `assert_ne!( decoded.model, after.model, "ignored allocator model field {key}" )` - semantic boundary still requires direct review; no move authorized.

#### `inspection_refuses_unknown_root_allocator_and_row_fields`

Source: `crates/worth-relational/src/durability/log/native_checkpoint_codec/inspection/tests.rs:179`.

- **UNCLASSIFIED** 187: `assert!(RelationalNativeCheckpoint::inspect_bytes(&execution, &after).is_err())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 192: `assert!(!rows.is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 196: `assert!( RelationalNativeCheckpoint::inspect_bytes(&execution, &after).is_err(), "unknown field in {key} row" )` - semantic boundary still requires direct review; no move authorized.

#### `restore_refuses_unknown_fields_before_publishing_any_model_state`

Source: `crates/worth-relational/src/durability/log/native_checkpoint_codec/inspection/tests.rs:204`.

- **UNCLASSIFIED** 213: `assert!(control.history().immutable_commit_count() > 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 249: `assert_eq!( error.class, RecoveryFailureClass::CorruptCheckpoint, "{owner}" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 254: `assert!( error.detail.contains("unknown field") && error.detail.contains("future_field"), "{owner}: {error:?}" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 262: `assert_eq!( before.bytes(), after.bytes(), "restore changed state for {owner}" )` - semantic boundary still requires direct review; no move authorized.

#### `the_live_census_includes_divergent_retained_branch_roots`

Source: `crates/worth-relational/src/durability/log/native_checkpoint_codec/inspection/tests.rs:271`.

- **UNCLASSIFIED** 291: `assert!( inspected.live_records.contains(&CheckpointRecordIdentity { class: RecordAllocationClass::Entity, partition_id: entity.partition_value(), slot: entity.local_slot_value(), generation: entity.generation_value(), }), "missing live entity from a retained branch: {entity:?}" )` - semantic boundary still requires direct review; no move authorized.

#### `burned_symbol_frontier_does_not_hide_published_symbol_damage`

Source: `crates/worth-relational/src/durability/log/native_checkpoint_codec/inspection/tests.rs:304`.

- **UNCLASSIFIED** 317: `assert_eq!(after.symbol_identity_frontier, next)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 318: `assert_eq!(after.model, inspected.model)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 319: `assert_eq!(after.live_records, inspected.live_records)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 326: `assert!(RelationalNativeCheckpoint::inspect_bytes(&execution, &invalid).is_err())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 349: `assert!( RelationalNativeCheckpoint::inspect_bytes(&execution, &tampered).is_err(), "symbol payloads still undergo owner readmission and digest validation" )` - semantic boundary still requires direct review; no move authorized.

### `durability/log/native_checkpoint_codec/kind_authentication_tests.rs`

#### `checkpoint_readmission_refuses_retired_integer_array_digest_before_restore`

Source: `crates/worth-relational/src/durability/log/native_checkpoint_codec/kind_authentication_tests.rs:13`.

- **UNCLASSIFIED** 30: `assert_ne!(current, retired)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 46: `assert!(replacements > 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 48: `assert_eq!( expect_durability_failure(stop).class, RecoveryFailureClass::CorruptCheckpoint )` - semantic boundary still requires direct review; no move authorized.

#### `forged_cross_region_kind_commitment_is_refused_before_member_reads`

Source: `crates/worth-relational/src/durability/log/native_checkpoint_codec/kind_authentication_tests.rs:55`.

- **UNCLASSIFIED** 155: `assert_eq!(source.reads.load(Ordering::SeqCst), 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 156: `assert!(matches!(failure, RelationalRecoveryStop::Durability(error) if error.class == RecoveryFailureClass::CorruptCheckpoint && error.detail.contains("StorageContentMismatch")))` - semantic boundary still requires direct review; no move authorized.

### `durability/log/native_checkpoint_codec/manifest_integrity_tests.rs`

#### `shared_members_with_inconsistent_leaf_facts_refuse_the_manifest`

Source: `crates/worth-relational/src/durability/log/native_checkpoint_codec/manifest_integrity_tests.rs:5`.

- **UNCLASSIFIED** 39: `assert_eq!(manifest.roots.len(), 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 67: `assert_eq!( failure.class, crate::durability::data::RecoveryFailureClass::CorruptCheckpoint )` - semantic boundary still requires direct review; no move authorized.

#### `sparse_page_membership_accepts_late_holes_and_refuses_duplicate_or_forged_ids`

Source: `crates/worth-relational/src/durability/log/native_checkpoint_codec/manifest_integrity_tests.rs:75`.

- **UNCLASSIFIED** 82: `assert_eq!(Page::find(&pages, 2), Some(&even))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 83: `assert_eq!(Page::find(&pages, 3), Some(&odd))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 84: `assert!(Page::find(&pages, 4).is_none())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 86: `assert!(validate_pages(&[odd, even, duplicate]).is_err())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 89: `assert!( validate_pages(&[forged]).is_err(), "the declared endpoints must cover the exact bits" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 95: `assert!( validate_pages(&[overflow]).is_err(), "a hostile high bit must refuse without arithmetic overflow" )` - semantic boundary still requires direct review; no move authorized.

### `durability/log/native_checkpoint_codec/manifest_validation/components/tests.rs`

#### `recorded_component_checkpoint_header_tamper_refuses_without_defaults_or_foreign_binding`

Source: `crates/worth-relational/src/durability/log/native_checkpoint_codec/manifest_validation/components/tests.rs:59`.

- **UNCLASSIFIED** 133: `assert_eq!( error.class, crate::durability::data::RecoveryFailureClass::CorruptCheckpoint, "header mutation {change}: {error:?}" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 138: `assert!( error.detail.contains("component") || error.detail.contains("recorded_components"), "header mutation {change}: {error:?}" )` - semantic boundary still requires direct review; no move authorized.

### `durability/log/native_checkpoint_codec/tests.rs`

#### `member_directory_preserves_divergent_sibling_roots_and_pinned_reader`

Source: `crates/worth-relational/src/durability/log/native_checkpoint_codec/tests.rs:10`.

- **UNCLASSIFIED** 46: `assert_eq!(checkpoint.branch_roots.len(), 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 78: `assert_eq!(sibling_shared, main_shared)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 79: `assert_ne!(sibling_root.regions[0].head, main_root.regions[0].head)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 88: `assert!( region_keys.len() < (sibling_root.regions.len() + main_root.regions.len() + wire.manifest.mirror.len()) * 2 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 94: `assert_eq!( wire.members .iter() .filter(|(locator, _)| *locator == key) .count(), 1, "equal regions must have exactly one member body across roots and mirror" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 104: `assert_eq!(restored.checkpoint.branch_roots, checkpoint.branch_roots)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 105: `assert_eq!( restored.checkpoint.partition_images, checkpoint.partition_images )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 109: `assert_eq!( runtime .read_truth() .read_snapshot(&pinned, execution) .expect("read request remains live") .unwrap() .entities() .len(), 2 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 134: `assert_eq!( recovered .read_truth() .read_snapshot(&snapshot, execution) .expect("read request remains live") .unwrap() .entities() .len(), 3 )` - semantic boundary still requires direct review; no move authorized.

#### `current_manifest_deduplicates_equal_region_members_and_round_trips`

Source: `crates/worth-relational/src/durability/log/native_checkpoint_codec/tests.rs:148`.

- **UNCLASSIFIED** 162: `assert_eq!(wire.manifest.roots[0].regions, wire.manifest.mirror)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 164: `assert_eq!( wire.members .iter() .filter(|(key, _)| *key == leaf.head) .count(), 1 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 171: `assert_eq!( wire.members .iter() .filter(|(key, _)| *key == leaf.history) .count(), 1 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 178: `assert_eq!(wire.manifest.commits.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 182: `assert_eq!(decoded.envelopes.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 183: `assert_eq!( decoded.branch_roots[0].partition_images, decoded.partition_images )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 187: `assert!(decoded.envelopes[0].derived_index_artifacts.is_empty())` - semantic boundary still requires direct review; no move authorized.

#### `current_directory_rejects_damaged_duplicate_and_missing_members`

Source: `crates/worth-relational/src/durability/log/native_checkpoint_codec/tests.rs:191`.

- **UNCLASSIFIED** 228: `assert!( decode_checkpoint(execution, &bytes).is_err(), "alteration {alteration}" )` - semantic boundary still requires direct review; no move authorized.

#### `native_capture_sections_count_current_directory_values`

Source: `crates/worth-relational/src/durability/log/native_checkpoint_codec/tests.rs:236`.

- **UNCLASSIFIED** 251: `assert_eq!(sections.total, native.bytes().len())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 266: `assert!(canonical_bytes > 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 267: `assert_eq!(sections.envelopes, canonical_bytes)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 268: `assert_eq!( sections.branch_roots, rmp_serde::to_vec_named(&wire.manifest.roots).unwrap().len() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 272: `assert_eq!( sections.branch_cells, rmp_serde::to_vec_named(&wire.manifest.branch_cells) .unwrap() .len() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 278: `assert_eq!( sections.partition_mirror, rmp_serde::to_vec_named(&wire.manifest.mirror) .unwrap() .len() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 284: `assert_eq!( sections.derived_indexes, rmp_serde::to_vec_named(&wire.manifest.index_definitions) .unwrap() .len() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 290: `assert_eq!( sections.total, sections.envelopes + sections.branch_roots + sections.branch_cells + sections.partition_mirror + sections.derived_indexes + sections.framing_and_metadata )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 306: `assert_eq!(native, repeated)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 307: `assert_eq!(native.clone().captured_sections(), Some(sections))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 308: `assert!( crate::durability::data::RelationalNativeCheckpoint::from_untrusted_bytes(native.bytes()) .captured_sections() .is_none() )` - semantic boundary still requires direct review; no move authorized.

#### `chk6_and_retired_inner_partition_grammars_are_refused`

Source: `crates/worth-relational/src/durability/log/native_checkpoint_codec/tests.rs:316`.

- **UNCLASSIFIED** 335: `assert_eq!( expect_durability_failure(stop).class, RecoveryFailureClass::UnsupportedLegacySemantics )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 350: `assert_eq!( expect_durability_failure(stop).class, RecoveryFailureClass::CorruptCheckpoint )` - semantic boundary still requires direct review; no move authorized.

### `durability/medium.rs`

No direct test declarations. Executing helper roots and claims:


### `durability/medium/binding.rs`

No direct test declarations. Executing helper roots and claims:


### `history/commit/artifact_tests.rs`

#### `fresh_scalar_commit_seals_exact_native_lifecycle_and_field_touches`

Source: `crates/worth-relational/src/history/commit/artifact_tests.rs:14`.

- **UNCLASSIFIED** 42: `assert!(touches.iter().any(|touch| matches!( touch, RelationalDescriptiveTouch::EntityLifecycle { kind: KindId(1), .. } )))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 49: `assert!(touches.iter().any(|touch| matches!(touch, RelationalDescriptiveTouch::AspectRevision { aspect, .. } if aspect.as_str() == "name" )))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 52: `assert!(touches.iter().any(|touch| matches!(touch, RelationalDescriptiveTouch::FieldRevision { kind: KindId(1), aspect, path, presence, .. } if aspect.as_str() == "name" && path.fields()[0].as_str() == "name" && *presence == crate::storage::data::RelationalFieldPresence::Present )))` - semantic boundary still requires direct review; no move authorized.

#### `installed_field_index_commit_touch_carries_native_old_and_new_keys`

Source: `crates/worth-relational/src/history/commit/artifact_tests.rs:62`.

- **UNCLASSIFIED** 96: `assert!(touches.iter().any(|touch| matches!(touch, RelationalDescriptiveTouch::IndexMembership { index: id, old_key: None, new_key: Some(key), .. } if *id == index.index_id && key == &expected )))` - semantic boundary still requires direct review; no move authorized.

#### `recovery_relinks_an_already_sealed_payload_without_reencoding_or_weakening_root_checks`

Source: `crates/worth-relational/src/history/commit/artifact_tests.rs:108`.

- **C2** 130: `assert!(Arc::ptr_eq( &preencoded.envelope_residency(), &relinked.envelope_residency() ))` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 134: `assert_eq!(relinked.identity(), legacy.identity())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 135: `assert_eq!(relinked.parentage(), legacy.parentage())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 136: `assert_eq!(relinked.roots(), legacy.roots())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 137: `assert_eq!( relinked .canonical_payload_digest() .expect("resident fixture has a canonical digest"), legacy .canonical_payload_digest() .expect("resident fixture has a canonical digest") )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 147: `assert!(matches!( preencoded.relink_preencoded_recovery(&equal_but_unsealed, Some(&root), None), Err(RelationalCommitArtifactDenial::RootLinkage) ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 155: `assert!(matches!( preencoded.relink_preencoded_recovery(&envelope, None, Some(&mismatched_schema)), Err(RelationalCommitArtifactDenial::RootLinkage) ))` - semantic boundary still requires direct review; no move authorized.

#### `canonical_artifact_accounting_excludes_nested_diagnostic_perturbation`

Source: `crates/worth-relational/src/history/commit/artifact_tests.rs:162`.

- C4 action 193: `.body_mut_for_test()`.
- C4 action 204: `.body_mut_for_test()`.
- **C2** 195: `assert!( envelope.allocation_inventory().authoritative_nested_bytes > omitted_checkpoint .allocation_inventory() .authoritative_nested_bytes, "the populated branch-cell checkpoint owns canonical nested bytes", )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 229: `assert_eq!( artifact.canonical_payload_bytes(), baseline_payload_bytes, "diagnostic richness cannot perturb canonical authority payload bytes" )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 242: `assert_eq!( allocation(RelationalCommitAuthoritativeAllocationKind::ArtifactObject), std::mem::size_of::<RelationalCommitArtifact>() as u64, )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 246: `assert_eq!( allocation(RelationalCommitAuthoritativeAllocationKind::CanonicalPayload), 0, "unpositioned scalar observations retain no byte allocation", )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 251: `assert_eq!( allocation(RelationalCommitAuthoritativeAllocationKind::EnvelopeObject), std::mem::size_of::<crate::history::data::CanonicalCommitEnvelope>() as u64, )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 255: `assert_eq!( allocation(RelationalCommitAuthoritativeAllocationKind::EnvelopeNestedOwnerStorage), baseline_authoritative_nested, "diagnostic allocation cannot enter authoritative nested-envelope accounting" )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 260: `assert!( artifact.excluded_allocation_inventory().diagnostic_bytes > baseline_diagnostic_bytes.saturating_add(2_048), "nested diagnostic strings and canonical bytes remain visible in their own lane" )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.

#### `populated_derived_index_artifacts_only_change_the_optional_cache_lane`

Source: `crates/worth-relational/src/history/commit/artifact_tests.rs:268`.

- C4 action 280: `envelope.body_mut_for_test().derived_index_artifacts = DerivedIndexArtifacts::default();`.
- C4 action 291: `envelope.body_mut_for_test().derived_index_artifacts =`.
- **C2** 308: `assert_eq!(perturbed.canonical_payload_bytes(), baseline_payload_bytes)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 309: `assert_eq!( perturbed.authoritative_allocation_observations(), baseline_allocations, "optional derived indexes cannot perturb any authoritative allocation lane", )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 314: `assert!( perturbed .excluded_allocation_inventory() .optional_cache_bytes > baseline_optional_cache_bytes, "the populated artifact remains observable in the optional-cache lane", )` - actual residency, identity, pin, allocation, dictionary or cost observation.

### `history/commit/artifact_tests/shared_readmission.rs`

#### `shared_readmitted_envelope_clones_keep_the_original_allocation_and_grant_until_last_drop`

Source: `crates/worth-relational/src/history/commit/artifact_tests/shared_readmission.rs:10`.

- **C2** 22: `assert!(std::ptr::eq(shared.as_ref(), clone.as_ref()))` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 24: `assert!(held > fixed_allocation_bill())` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 26: `assert_eq!(budget.charged_memory_bytes(), held)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 27: `assert_eq!(clone.commit.commit_id, source.commit_id())` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 29: `assert_eq!(budget.charged_memory_bytes(), 0)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.

#### `actual_weak_position_retains_only_the_fixed_original_control_bill_until_last_drop`

Source: `crates/worth-relational/src/history/commit/artifact_tests/shared_readmission.rs:33`.

- **C2** 45: `assert!(budget.charged_memory_bytes() > fixed)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 47: `assert!(retained.resident().is_none())` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 48: `assert_eq!( budget.charged_memory_bytes(), fixed, "payload heaps and original raw bytes are gone" )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 54: `assert_eq!( budget.charged_memory_bytes(), fixed, "the remaining actual Weak owner shares one bill" )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 60: `assert_eq!(budget.charged_memory_bytes(), 0)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.

#### `embedded_checkpoint_preserves_ignored_unknown_nested_fields_without_retaining_their_heaps`

Source: `crates/worth-relational/src/history/commit/artifact_tests/shared_readmission.rs:117`.

- **C2** 132: `assert!(original.derived_index_artifacts.is_empty())` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 147: `assert_eq!(&baseline, original)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 149: `assert!(held > 0)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 151: `assert_eq!(budget.charged_memory_bytes(), 0)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 154: `assert_eq!(&accepted, original)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 155: `assert_eq!( budget.charged_memory_bytes(), held, "ignored/absent input is not retained decoded storage" )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 161: `assert_eq!(budget.charged_memory_bytes(), 0)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 186: `assert_eq!(rmp_serde::to_vec_named(&extension).unwrap()[0], marker)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 197: `assert_eq!(omitted[0], 0xde, "the actual producer emits a map16")` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 218: `assert_eq!(matches.len(), 1)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.

### `history/commit/artifact_tests/strategy_readmission.rs`

#### `strategy_input_clone_retains_only_its_original_decoded_bytes_grant`

Source: `crates/worth-relational/src/history/commit/artifact_tests/strategy_readmission.rs:10`.

- **C2** 24: `assert!(std::ptr::eq( original.canonical_bytes(), retained.canonical_bytes() ))` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 28: `assert!( source.as_bytes().len() > retained.canonical_bytes().len() * 100, "the small input must not retain the huge independently owned index key" )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 34: `assert!(expected > 0)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 36: `assert_eq!(budget.charged_memory_bytes(), expected)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 37: `assert!(!retained.canonical_bytes().is_empty())` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 39: `assert_eq!(budget.charged_memory_bytes(), 0)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.

#### `strategy_descriptor_clone_retains_only_its_original_decoded_locator_grant`

Source: `crates/worth-relational/src/history/commit/artifact_tests/strategy_readmission.rs:44`.

- **C2** 58: `assert!(std::ptr::eq( original.intent_scope_targets(), retained.intent_scope_targets() ))` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 67: `assert!( nested > 0, "the real nested locator strings/path own storage" )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 75: `assert!( actual .envelope() .strategy_artifacts .as_ref() .unwrap() .canonical_input() .canonical_bytes() .len() as u64 > expected * 100, "the small locator must not retain the huge input sibling" )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 87: `assert!(expected > 0)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 89: `assert_eq!(budget.charged_memory_bytes(), expected)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 90: `assert_eq!(retained.intent_scope_targets().len(), 1)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 92: `assert_eq!(budget.charged_memory_bytes(), 0)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.

### `history/commit/envelope_residency/memory_publication_tests.rs`

#### `memory_original_frame_survives_export_checkpoint_and_last_alias`

Source: `crates/worth-relational/src/history/commit/envelope_residency/memory_publication_tests.rs:18`.

- **C2** 41: `assert_eq!(original.charged_payload_bytes(), Some(bill))` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 49: `assert_eq!(reference.as_bytes(), original.bytes())` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 57: `assert_eq!(exported.as_bytes().as_ptr(), original.bytes().as_ptr())` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 58: `assert_eq!( budget.charged_memory_bytes(), before, "export verifies/shares; it allocates no replacement frame" )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 67: `assert!(native .bytes() .starts_with(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC))` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 76: `assert_eq!(again.as_bytes().as_ptr(), original.bytes().as_ptr())` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 86: `assert_eq!( budget.charged_memory_bytes(), bill, "only the escaped original frame remains" )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 93: `assert_eq!(budget.charged_memory_bytes(), bill)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 95: `assert_eq!(budget.charged_memory_bytes(), 0)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.

#### `memory_frame_quota_and_cancel_refuse_before_position_or_head`

Source: `crates/worth-relational/src/history/commit/envelope_residency/memory_publication_tests.rs:98`.

- **C4** 137: `assert!(bill > controller)` - reviewed exact injected owner rendezvous or request-fault campaign; preserve its original control.
- **C4** 156: `assert_eq!( requested, bill, "independent reference frame bytes, not the controller or preparation" )` - reviewed exact injected owner rendezvous or request-fault campaign; preserve its original control.
- **C2** 160: `assert_eq!( runtime.patch_position_reservation_counters().assignments, counters.assignments )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C4** 164: `assert_eq!( runtime.history.latest_recorded_patch_position(), before_head )` - reviewed exact injected owner rendezvous or request-fault campaign; preserve its original control.
- **C2** 169: `assert_eq!(limited.charged_memory_bytes(), 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C4** 188: `assert!(matches!( runtime .publication_port() .compare_and_publish(&original, candidate), crate::mvcc::RelationalPublicationOutcome::RequestStopped( worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled) ) ))` - reviewed exact injected owner rendezvous or request-fault campaign; preserve its original control.
- **C2** 196: `assert_eq!( runtime.patch_position_reservation_counters().assignments, counters.assignments )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C4** 200: `assert_eq!( runtime.history.latest_recorded_patch_position(), before_head )` - reviewed exact injected owner rendezvous or request-fault campaign; preserve its original control.
- **C2** 204: `assert_eq!(cancelled.charged_memory_bytes(), 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 206: `assert_eq!( runtime.patch_position_reservation_counters().assignments, counters.assignments + 1 )` - actual residency, identity, pin, allocation, dictionary or cost observation.

#### `memory_frame_preserves_original_payload_and_scalar_tuple_observations`

Source: `crates/worth-relational/src/history/commit/envelope_residency/memory_publication_tests.rs:234`.

- **C2** 245: `assert_eq!(artifact.canonical_payload_bytes(), bytes.len() as u64)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 246: `assert_eq!( artifact.canonical_payload_digest().unwrap(), <[u8; 32]>::from(Sha256::digest(&bytes)) )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 250: `assert!(artifact.envelope_residency().payload_storage_bytes() > bytes.len() as u64)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 253: `assert_ne!( original.payload_digest().as_bytes(), &artifact.canonical_payload_digest().unwrap() )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 298: `assert_eq!( std::mem::size_of::<EnvelopeBacking>(), std::mem::size_of::<OldBacking>() + std::mem::size_of::<usize>() )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 302: `assert_eq!( std::mem::size_of::<RelationalCommitEnvelopeResidency>(), std::mem::size_of::<OldCell>() + std::mem::size_of::<usize>() )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.

#### `journal_publication_keeps_journal_placement_without_memory_frame`

Source: `crates/worth-relational/src/history/commit/envelope_residency/memory_publication_tests.rs:309`.

- **C2** 335: `assert!(entry.same_owner(&cell.journal_placement().unwrap().0))` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 336: `assert!(cell.resident_payload_bytes().is_none())` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 337: `assert_eq!(cell.payload_storage_bytes(), 0)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 342: `assert_eq!(exported.as_bytes(), original.as_ref())` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 343: `assert!(Arc::ptr_eq( &source, &runtime.durability.cold_source().unwrap() ))` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.

### `history/commit/envelope_residency_tests.rs`

#### `an_indexed_envelope_remains_cold_until_explicit_readmission_and_loads_once`

Source: `crates/worth-relational/src/history/commit/envelope_residency_tests.rs:80`.

- **UNCLASSIFIED** 91: `assert!(cell.get().is_none())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 92: `assert_eq!(source.reads.load(Ordering::Relaxed), 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 94: `assert_eq!(first.commit, expected)` - semantic boundary still requires direct review; no move authorized.
- **C2** 95: `assert!(Arc::ptr_eq( &first, &cell.demand(&execution, 1 << 30).unwrap() ))` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 99: `assert_eq!(source.reads.load(Ordering::Relaxed), 1)` - semantic boundary still requires direct review; no move authorized.

#### `an_indexed_envelope_storage_failure_is_typed_and_sticky`

Source: `crates/worth-relational/src/history/commit/envelope_residency_tests.rs:103`.

- **UNCLASSIFIED** 122: `assert_eq!( failure, crate::storage::data::RelationalStorageUnavailable { subject: crate::storage::data::RelationalStorageSubject::Envelope(position), cause: Cause::Damaged, } )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 130: `assert_eq!(source.reads.load(Ordering::Relaxed), 1)` - semantic boundary still requires direct review; no move authorized.

#### `relinking_other_envelopes_preserves_an_index_even_while_its_body_is_in_flight`

Source: `crates/worth-relational/src/history/commit/envelope_residency_tests.rs:134`.

- **UNCLASSIFIED** 153: `assert_eq!(retained.get(id).unwrap().receipt(), body.commit)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 156: `assert!(weak.upgrade().is_none())` - semantic boundary still requires direct review; no move authorized.
- **C2** 157: `assert!(retained.get(id).unwrap().resident_envelope().is_none())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 158: `assert!( catalog .retained_entries_except(&std::collections::BTreeSet::from([id])) .get(id) .is_none(), "only explicitly replaced identities leave the projection" )` - semantic boundary still requires direct review; no move authorized.

#### `settlement_releases_a_journal_flight_but_preserves_memory_history_and_position`

Source: `crates/worth-relational/src/history/commit/envelope_residency_tests.rs:168`.

- **C2** 197: `assert_eq!(cell.positioned_if_resident().unwrap().position(), position)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 198: `assert!( weak.upgrade().is_some(), "unsettled publication retains its exact body" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 202: `assert_eq!( source.reads.load(Ordering::Relaxed), reads, "binding never loads" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 208: `assert_eq!(weak.upgrade().is_none(), journal_backed)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 209: `assert_eq!( source.reads.load(Ordering::Relaxed), reads, "settlement never loads" )` - semantic boundary still requires direct review; no move authorized.
- **C2** 214: `assert_eq!( cell.journal_index().as_deref(), journal_backed.then_some(&entry) )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 219: `assert_eq!(flight.commit, entry.commit)` - semantic boundary still requires direct review; no move authorized.
- **C2** 220: `assert_eq!(cell.positioned_if_resident().unwrap().position(), position)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 222: `assert!(source.reads.load(Ordering::Relaxed) > reads)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 224: `assert_eq!(source.reads.load(Ordering::Relaxed), reads)` - semantic boundary still requires direct review; no move authorized.

#### `positioned_demand_owns_its_result_across_journal_settlement`

Source: `crates/worth-relational/src/history/commit/envelope_residency_tests.rs:230`.

- **UNCLASSIFIED** 242: `assert_eq!(first.position(), entry.position)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 243: `assert_eq!(first.commit, entry.commit)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 244: `assert_eq!(source.reads.load(Ordering::Relaxed), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 245: `assert_eq!(cell.demand_positioned(&execution, 1 << 30).unwrap(), first)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 246: `assert_eq!(source.reads.load(Ordering::Relaxed), 1)` - semantic boundary still requires direct review; no move authorized.

#### `unpositioned_memory_envelopes_and_cold_digests_refuse_explicitly`

Source: `crates/worth-relational/src/history/commit/envelope_residency_tests.rs:250`.

- C4 action 272: `conflicting.body_mut_for_test().commit.version_id = crate::identity::data::VersionId(u64::MAX);`.
- **C4** 264: `assert_eq!( artifact.canonical_payload_digest(), Err(super::RelationalCommitArtifactDenial::CanonicalPayloadEncoding) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 269: `assert!(artifact.canonical_payload_digest().is_ok())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 270: `assert_eq!(artifact.matches_envelope(&envelope), Ok(true))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 273: `assert_eq!(artifact.matches_envelope(&conflicting), Ok(false))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 281: `assert_eq!( failure.class, crate::durability::data::RecoveryFailureClass::CorruptCheckpoint )` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `history/retention/component_cut_tests.rs`

#### `native_custody_correction_cross_lane_acquire_release_has_one_actual_capture_cut`

Source: `crates/worth-relational/src/history/retention/component_cut_tests.rs:20`.

- **C2** 48: `assert!( owner.external_cut.try_lock().is_err(), "the actual capture owns the complete cut while paused between lanes" )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 67: `assert!(selected.iter().any( |basis| basis.identity().branch_id() == runtime.main_branch_identity().branch_id() ))` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 70: `assert!( !selected .iter() .any(|basis| basis.identity().branch_id() == &other), "A and B were never simultaneous external obligations" )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 77: `assert!(!after.iter().any( |basis| basis.identity().branch_id() == runtime.main_branch_identity().branch_id() ))` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 80: `assert!(after .iter() .any(|basis| basis.identity().branch_id() == &other))` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.

#### `native_custody_correction_current_lane_inventory_is_collected_inside_actual_cut`

Source: `crates/worth-relational/src/history/retention/component_cut_tests.rs:93`.

- **C2** 103: `assert!( retention.inner.external_cut.try_lock().is_err(), "current lane discovery must share the external retention cut" )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 116: `assert_eq!(selected.len(), 1)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 117: `assert_eq!(selected[0].descriptor(), basis.descriptor())` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.

### `indexes/access/definition_lookup.rs`

#### `exact_lookup_preserves_restored_numeric_identity_and_disambiguates_name`

Source: `crates/worth-relational/src/indexes/access/definition_lookup.rs:61`.

- **pureowner** 80: `assert_eq!(lookup.definition_count(), 4)` - detached matching values; runtime/publication is irrelevant.
- **pureowner** 81: `assert_eq!(lookup.candidate_count_for_name("same-name"), 4)` - detached matching values; runtime/publication is irrelevant.
- **pureowner** 82: `assert_eq!(lookup.candidate_count_for_name("missing"), 0)` - detached matching values; runtime/publication is irrelevant.
- **pureowner** 84: `assert_eq!( lookup .matching_definition(&installed(0, kind.clone(), false)) .unwrap() .index_id, DerivedIndexId(13) )` - detached matching values; runtime/publication is irrelevant.
- **pureowner** 91: `assert_eq!( lookup .matching_definition(&installed(0, kind.clone(), true)) .unwrap() .index_id, DerivedIndexId(12) )` - detached matching values; runtime/publication is irrelevant.
- **pureowner** 98: `assert!(lookup .matching_definition(&installed(0, wrong_kind, true)) .is_none())` - detached matching values; runtime/publication is irrelevant.
- **pureowner** 101: `assert!(lookup .matching_definition(&DerivedIndexDefinition { index_id: DerivedIndexId(0), name: "missing".to_owned(), kind, branch_scoped: false, }) .is_none())` - detached matching values; runtime/publication is irrelevant.

#### `lookup_snapshot_is_detached_from_subsequent_owner_registration`

Source: `crates/worth-relational/src/indexes/access/definition_lookup.rs:112`.

- **HAPPY** 135: `assert_eq!(snapshot.definition_count(), 1)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 136: `assert_eq!(snapshot.candidate_count_for_name("snapshot-index"), 1)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 137: `assert_eq!(snapshot.candidate_count_for_name("later-index"), 0)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 138: `assert_eq!( snapshot.matching_definition(&expected).unwrap().index_id, installed.index_id )` - reviewed public owner result over explicit durable provider.

### `publication/cdc/planning/checkpoint_resolution/tests.rs`

#### `subscriber_stream_recovers_from_durable_canonical_envelopes_when_patch_stream_index_outlives_envelope`

Source: `crates/worth-relational/src/publication/cdc/planning/checkpoint_resolution/tests.rs:9`.

- **UNCLASSIFIED** 19: `assert!(runtime .history_authority() .evict_commit_envelope_preserving_patch_position_for_durable_recovery_test( first.commit.commit_id, ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 31: `assert_eq!(resumed.patches.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 32: `assert_eq!(resumed.patches[0].position.0, 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 33: `assert_eq!( resumed.recovery_decision.source, crate::facade::publication::SubscriberRecoverySource::DurableCanonicalRecovery )` - semantic boundary still requires direct review; no move authorized.

### `runtime/state/subsystems/services/symbol_table/reservations/tests.rs`

#### `publication_excludes_abandoned_preview_meanings_adopted_for_reading`

Source: `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs:4`.

- **UNCLASSIFIED** 21: `assert_eq!( owner.with_read(|symbols| symbols.resolve(unused).map(str::to_owned)), None )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 25: `assert_eq!( owner.with_read(|symbols| symbols.resolve(used).map(str::to_owned)), Some("actual-field".to_owned()) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 29: `assert!(owner.reservations.lock().unwrap().pending.is_empty())` - semantic boundary still requires direct review; no move authorized.

#### `concurrent_pending_meanings_deduplicate_and_publish_only_the_accepted_candidate`

Source: `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs:33`.

- **UNCLASSIFIED** 38: `assert_eq!(accepted.reserve("shared").unwrap(), shared)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 40: `assert!(owner.configuration_snapshot().entries.is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 41: `assert_eq!(owner.with_read(|symbols| symbols.symbol("shared")), None)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 42: `assert_eq!( accepted .binding() .with_read(|symbols| symbols.resolve(shared).map(str::to_owned)), Some("shared".to_owned()) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 55: `assert_eq!( owner.with_read(|symbols| symbols.symbol("shared")), Some(shared) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 59: `assert_eq!( owner.with_read(|symbols| symbols.resolve(unused).map(str::to_owned)), None )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 63: `assert!(owner.reservations.lock().unwrap().pending.is_empty())` - semantic boundary still requires direct review; no move authorized.

#### `abandoning_the_last_custodian_spends_the_identity_and_removes_pending_text`

Source: `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs:67`.

- **UNCLASSIFIED** 72: `assert!(owner.reservations.lock().unwrap().pending.is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 75: `assert!(fresh.0 > abandoned.0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 82: `assert_eq!( owner.with_read(|symbols| symbols.symbol("retry")), Some(fresh) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 86: `assert_eq!( owner.with_read(|symbols| symbols.resolve(abandoned).map(str::to_owned)), None )` - semantic boundary still requires direct review; no move authorized.

#### `independent_admission_uses_the_pending_identity`

Source: `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs:93`.

- **UNCLASSIFIED** 102: `assert_eq!( owner.admit_field_text(&execution, &bytes).unwrap().0, symbol )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 112: `assert_eq!( owner.with_read(|symbols| symbols.symbol("independent")), Some(symbol) )` - semantic boundary still requires direct review; no move authorized.

#### `parallel_candidates_share_pending_meaning_without_publishing_it`

Source: `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs:119`.

- **UNCLASSIFIED** 138: `assert!(candidates .iter() .all(|(_, symbol)| *symbol == candidates[0].1))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 141: `assert!(owner.snapshot().entries.is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 143: `assert!(owner.reservations.lock().unwrap().pending.is_empty())` - semantic boundary still requires direct review; no move authorized.

#### `checkpoint_recovery_preserves_abandoned_symbol_identity_frontiers`

Source: `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs:147`.

- **UNCLASSIFIED** 174: `assert!(fresh.0 > symbol.0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 175: `assert_eq!( recovered .services .symbols .with_read(|symbols| symbols.resolve(symbol).map(str::to_owned)), None )` - semantic boundary still requires direct review; no move authorized.

#### `canonical_symbol_sealing_and_cross_owner_adoption_refuse_before_publication`

Source: `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs:191`.

- **UNCLASSIFIED** 198: `assert!(candidate.seal_for_publication(&false_meaning).is_err())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 199: `assert!(owner.with_read(|interner| interner.resolve(symbol).is_none()))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 203: `assert!(matches!( first.adopt(&second), Err(super::SymbolReservationDenial::InvalidDictionary) ))` - semantic boundary still requires direct review; no move authorized.

### `runtime/state/subsystems/visibility/snapshot_handles/published_registry/original_cleanup/tests.rs`

#### `result_snapshot_cleanup_registry_contention_preserves_exact_cell_and_competing_task`

Source: `crates/worth-relational/src/runtime/state/subsystems/visibility/snapshot_handles/published_registry/original_cleanup/tests.rs:10`.

- **UNCLASSIFIED** 26: `assert_eq!(capacity.occupied_handles(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 52: `assert_eq!(locked.by_id.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 53: `assert!(!result.snapshot_release_observation().was_released())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 62: `assert_eq!(registry.count(), 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 63: `assert_eq!(capacity.occupied_handles(), 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 74: `assert!(matches!(*locked, PublishedSnapshotRetirement::Removed(_)))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 77: `assert!(matches!( runtime.delete_branch(execution, &target).unwrap(), crate::branch::RelationalBranchDeletionOutcome::Deleted(_) ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 82: `assert!(original_root.upgrade().is_some(), "the exact original issued root survives head movement, branch removal and real reclamation while its binding awaits retirement")` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 86: `assert!(matches!( task.advance(execution, &runtime), Progress::Complete { .. } ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 90: `assert!(matches!( competing.advance(execution, &runtime), Progress::Complete { .. } ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 94: `assert_eq!(registry.count(), 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 95: `assert_eq!(capacity.occupied_handles(), 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 97: `assert!(original_root.upgrade().is_none(), "the SAME original root is reclaimable only after actual binding retirement; no hidden test read/root lease pins it")` - semantic boundary still requires direct review; no move authorized.

### `storage/residency/cross_region_forgery_tests.rs`

#### `rehashed_cross_region_kind_forgery_is_refused_by_the_canonical_root`

Source: `crates/worth-relational/src/storage/residency/cross_region_forgery_tests.rs:184`.

- **UNCLASSIFIED** 229: `assert_eq!(head.kind_projection.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 230: `assert_eq!( head.kind_projection[0].relation.partition_id, PartitionId(3) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 312: `assert!(matches!(failure, RelationalRecoveryStop::Durability(error) if error.class == RecoveryFailureClass::CorruptCheckpoint && error.detail.contains("StorageContentMismatch")))` - semantic boundary still requires direct review; no move authorized.

### `tests/complexity/contracts/commit_budgets/schema_continuity_boundaries.rs`

#### `complexity_budget_schema_transition_classification_is_changed_atom_bounded`

Source: `crates/worth-relational/src/tests/complexity/contracts/commit_budgets/schema_continuity_boundaries.rs:5`.

- C4 action 12: `runtime.set_schema_registry_for_test(`.
- **C2** 48: `assert_eq!(counters.schema_transition_atoms_inspected, 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 49: `assert_eq!(counters.schema_changed_subtrees_inspected, 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 50: `assert_eq!(counters.schema_unchanged_subtrees_reused_by_fingerprint, 0)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 51: `assert_eq!(counters.schema_bridge_descriptors_built, 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 52: `assert_eq!(counters.schema_transition_continue_visible_bridge_count, 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 53: `assert_eq!(counters.schema_reconciliation_preserve_information_count, 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.

#### `complexity_budget_subscriber_resume_continuity_is_boundary_local`

Source: `crates/worth-relational/src/tests/complexity/contracts/commit_budgets/schema_continuity_boundaries.rs:57`.

- C4 action 64: `runtime.set_schema_registry_for_test(`.
- **C2** 104: `assert_eq!(counters.subscriber_resume_evaluations, 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 105: `assert_eq!(counters.subscriber_continue_visible_bridge_count, 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 106: `assert_eq!(counters.schema_normalized_descriptor_compositions, 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.

#### `complexity_budget_milestone5_closeout_keeps_schema_cdc_and_recovery_boundary_local`

Source: `crates/worth-relational/src/tests/complexity/contracts/commit_budgets/schema_continuity_boundaries.rs:110`.

- C4 action 119: `runtime.set_schema_registry_for_test(`.
- **C2** 155: `assert_eq!(schema_counters.schema_transition_atoms_inspected, 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 156: `assert_eq!(schema_counters.schema_changed_subtrees_inspected, 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 157: `assert_eq!(schema_counters.schema_bridge_descriptors_built, 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 158: `assert_eq!( schema_counters.schema_transition_continue_visible_bridge_count, 1 )` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 162: `assert_eq!(schema_counters.replay_digest_parity_checks, 0)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 163: `assert_eq!(schema_counters.replay_deep_artifact_parity_checks, 0)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 175: `assert_eq!(cdc_counters.schema_transition_atoms_inspected, 0)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 176: `assert_eq!(cdc_counters.subscriber_resume_evaluations, 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 177: `assert_eq!(cdc_counters.subscriber_continue_visible_bridge_count, 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 178: `assert_eq!(cdc_counters.schema_normalized_descriptor_compositions, 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 179: `assert_eq!(cdc_counters.replay_digest_parity_checks, 0)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 180: `assert_eq!(cdc_counters.replay_deep_artifact_parity_checks, 0)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 191: `assert!(plan_counters.replay_digest_parity_checks >= 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 192: `assert_eq!(plan_counters.replay_deep_artifact_parity_checks, 0)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 193: `assert_eq!( plan.authority_continuity.verification_outcome, crate::durability::data::RecoveryVerificationOutcome::VerifiedAtLayer( crate::replay::data::ReplayVerificationLayer::DigestParity ) )` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 219: `assert!(recovered_counters.replay_digest_parity_checks >= 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 220: `assert_eq!(recovered_counters.replay_deep_artifact_parity_checks, 0)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 221: `assert!(recovered .replay() .canonical_commit_envelope(execution, transitioned.commit.commit_id) .unwrap() .is_some())` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.

### `tests/complexity/contracts/merge_budgets.rs`

#### `complexity_budget_merge_planning_reports_request_shaped_work`

Source: `crates/worth-relational/src/tests/complexity/contracts/merge_budgets.rs:10`.

- **C2** 40: `assert!(runtime .performance_access() .contracts() .iter() .any(|contract| contract.id == "runtime.merge.planning"))` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 45: `assert_eq!(counters.merge_planning_requests, 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 46: `assert!(counters.merge_planning_schema_kinds_snapshotted >= 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 47: `assert!(counters.merge_planning_target_commits_scoped >= 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 48: `assert!(counters.merge_planning_source_commits_scoped >= 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 49: `assert!(counters.merge_planning_target_records_scoped >= 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 50: `assert!(counters.merge_planning_source_records_scoped >= 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 51: `assert_eq!( counters.merge_identity_candidates_discovered, artifact.identity_discovery.candidate_count )` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 55: `assert!(counters.merge_identity_target_records_scanned >= 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 56: `assert!(counters.merge_identity_target_records_indexed >= 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 57: `assert_eq!( counters.merge_conflict_records_classified, artifact.conflict_classification.classified_record_count )` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 61: `assert_eq!( counters.merge_causal_records_annotated, artifact.causal_annotation.classified_record_count )` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 65: `assert_eq!( counters.merge_policy_records_resolved, artifact.policy_resolution.resolved_record_count )` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 69: `assert_eq!( counters.merge_lowered_records_emitted, artifact.lowered_plan.record_count )` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 73: `assert_eq!( counters.merge_decision_log_width, artifact.decision_log.decisions.len() )` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 77: `assert!(counters.merge_planning_elapsed_nanos > 0)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.

#### `complexity_budget_merge_execution_reports_admitted_records_and_emitted_mutations`

Source: `crates/worth-relational/src/tests/complexity/contracts/merge_budgets.rs:81`.

- **C2** 130: `assert!(runtime .performance_access() .contracts() .iter() .any(|contract| contract.id == "runtime.merge.execution_commit"))` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 135: `assert_eq!(counters.merge_execution_attempts, 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 136: `assert_eq!( counters.merge_execution_requests, outcome.structural_summary.executed_record_count )` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 140: `assert_eq!( counters.merge_execution_records_admitted, outcome.structural_summary.executed_record_count )` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 144: `assert_eq!( counters.merge_execution_mutation_intents_emitted, outcome.structural_summary.emitted_mutation_intent_count )` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 148: `assert_eq!(outcome.structural_summary.adopted_source_record_count, 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 149: `assert_eq!(outcome.structural_summary.emitted_entity_create_count, 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 150: `assert_eq!(changed_entities(&outcome.commit).len(), 1)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 151: `assert_ne!(changed_entities(&outcome.commit)[0], feature_only)` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.

### `tests/domains/cad/topology_identity_survival.rs`

#### `topology_identity_survival_preserves_reidentification_truth_across_recovery`

Source: `crates/worth-relational/src/tests/domains/cad/topology_identity_survival.rs:10`.

- **UNCLASSIFIED** 44: `assert_eq!( resolution.digest_basis().digest_mode(), HistoricalResolutionDigestMode::ExactDigestCanonicalOrder )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 48: `assert!(!resolution.resolved.is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 49: `assert_eq!(resolution.resolved[0], replacement_lineage)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 50: `assert_eq!(resolution.metrics.resolved_lineage_count, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 91: `assert_eq!( recovered_resolution.digest_basis(), resolution.digest_basis() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 95: `assert_eq!(recovered_resolution.resolved, resolution.resolved)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 96: `assert_eq!( recovered_resolution.traversed_event_ids, resolution.traversed_event_ids )` - semantic boundary still requires direct review; no move authorized.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/support/truth_comparison.rs:22 assert_recovered_commit_truth_matches`, `crates/worth-relational/src/tests/support.rs:361 assert_lineage_history_origin_invariants`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

### `tests/domains/fintech/actions/recovery.rs`

No direct test declarations. Executing helper roots and claims:

- `checkpoint_world` at 8: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/domains/fintech/workflows/persistence.rs:9 fintech_persisted_workflow_recovers_checkpoint_tail_and_keeps_queryable_portfolio_state`, `crates/worth-relational/src/tests/domains/fintech/workflows/persistence.rs:158 fintech_failure_injection_helpers_cover_savepoints_replay_and_checkpoint_corruption`, `crates/worth-relational/src/tests/domains/fintech/workflows/persistence.rs:216 fintech_persisted_workflow_recovers_from_its_filed_checkpoint`, `crates/worth-relational/src/tests/domains/fintech/workflows/persistence.rs:281 fintech_index_and_lineage_authority_survive_hostile_workflow_recovery`, `crates/worth-relational/src/tests/domains/fintech/workflows/persistence/checkpoint_fallback.rs:6 fintech_recovery_refuses_a_damaged_checkpoint_instead_of_losing_truth`.
- `recover_persisted_world` at 19: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/domains/fintech/workflows/persistence.rs:9 fintech_persisted_workflow_recovers_checkpoint_tail_and_keeps_queryable_portfolio_state`, `crates/worth-relational/src/tests/domains/fintech/workflows/persistence.rs:216 fintech_persisted_workflow_recovers_from_its_filed_checkpoint`, `crates/worth-relational/src/tests/domains/fintech/workflows/persistence.rs:281 fintech_index_and_lineage_authority_survive_hostile_workflow_recovery`.

### `tests/domains/fintech/failure_injection/durability.rs`

No direct test declarations. Executing helper roots and claims:

- `damage_filed_checkpoint` at 5: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/domains/fintech/workflows/persistence.rs:158 fintech_failure_injection_helpers_cover_savepoints_replay_and_checkpoint_corruption`, `crates/worth-relational/src/tests/domains/fintech/workflows/persistence/checkpoint_fallback.rs:6 fintech_recovery_refuses_a_damaged_checkpoint_instead_of_losing_truth`.

### `tests/domains/fintech/fixture/mod.rs`

No direct test declarations. Executing helper roots and claims:

- `build` at 281: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): not established by scoped free-function call index; inspect qualified/dynamic callers.

### `tests/domains/fintech/workflows/persistence/checkpoint_fallback.rs`

#### `fintech_recovery_refuses_a_damaged_checkpoint_instead_of_losing_truth`

Source: `crates/worth-relational/src/tests/domains/fintech/workflows/persistence/checkpoint_fallback.rs:6`.

- **UNCLASSIFIED** 16: `assert!(damage_filed_checkpoint(&world.runtime))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 29: `assert_eq!( expect_durability_failure(refused).class, RecoveryFailureClass::CorruptCheckpoint )` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/adjacency_checkpoint.rs`

#### `checkpoint_rebuilds_current_and_historical_adjacency_kind_buckets`

Source: `crates/worth-relational/src/tests/durability/contracts/adjacency_checkpoint.rs:4`.

- **HAPPY** 44: `assert_eq!(historical.into_records()[0].relation_id, relation)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 45: `assert!(current.into_records().is_empty())` - reviewed public owner result over explicit durable provider.

### `tests/durability/contracts/associated_effects.rs`

#### `associated_effects_refuse_memory_and_absent_settlement_evidence`

Source: `crates/worth-relational/src/tests/durability/contracts/associated_effects.rs:41`.

- C4 action 68: `.remove_canonical_publication_route_for_test(committed.commit.commit_id);`.
- **C4** 49: `assert!(matches!( memory .durability_authority() .discover_associated_effects(execution, &association), Err(Refusal::AssociatedMemory { .. }) ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 69: `assert!(matches!( owner.readmit_associated_effect(execution, &subject), Err(Refusal::MissingSettlementEvidence) ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `associated_effects_discover_cold_subject_without_reading_unrelated_history`

Source: `crates/worth-relational/src/tests/durability/contracts/associated_effects.rs:77`.

- C4 action 116: `attached_test_medium(&recovered).drop_artifact(unrelated.position());`.
- **C2** 114: `assert!(selected.positioned_if_resident().is_none())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 115: `assert!(unrelated_cell.positioned_if_resident().is_none())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C4** 121: `assert_eq!(subjects.len(), 1)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C2** 122: `assert!( selected.positioned_if_resident().is_none(), "discovery reads only metadata" )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C4** 129: `assert_eq!(effect.settlement(), Settlement::CheckpointReadmitted)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 130: `assert_eq!(effect.semantic_result(), semantic)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C2** 131: `assert!(unrelated_cell.positioned_if_resident().is_none())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C4** 139: `assert_eq!(selected_frame.envelope().semantic_result(), semantic)` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `tests/durability/contracts/associated_effects_discovery.rs`

#### `native_custody_correction_enumerates_known_orphan_and_unrelated_association_metadata`

Source: `crates/worth-relational/src/tests/durability/contracts/associated_effects_discovery.rs:11`.

- C4 action 52: `attached_test_medium(&runtime).drop_artifact(artifact.position());`.
- **C4** 24: `assert!( source .durability_authority() .discover_all_associated_effects(execution) .unwrap() .iter() .any(|subject| subject.association() == Some(association)), "actual pending route is discoverable before catalog settlement" )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 58: `assert_eq!(subjects.len(), 3)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 60: `assert_eq!(subject.association(), Some(expected))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C2** 61: `assert!(runtime .history .commit_artifact(subject.receipt().commit_id) .unwrap() .envelope_residency() .positioned_if_resident() .is_none())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C4** 69: `assert_eq!( runtime .durability_authority() .discover_associated_effects(execution, &known) .unwrap() .len(), 1 )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 81: `assert!(matches!( memory .durability_authority() .discover_all_associated_effects(execution), Err(RelationalAssociatedEffectRefusal::AssociatedMemory { .. }) ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `tests/durability/contracts/branch_and_aspect_history.rs`

#### `durability_contract_recovery_rebuilds_branch_heads_and_latest_commit`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:5`.

- **HAPPY** 21: `assert_eq!(outcome.recovered_commits, 2)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 22: `assert_eq!(outcome.latest_commit, Some(feature.commit.clone()))` - reviewed public owner result over explicit durable provider.
- **HAPPY** 23: `assert_eq!( recovered .history() .branch_head(&BranchId("feature".to_string())), Some(feature.commit.clone()) )` - reviewed public owner result over explicit durable provider.
- **HAPPY** 29: `assert_eq!( recovered .history() .branch_head(&BranchId("main".to_string())), Some(main.commit.clone()) )` - reviewed public owner result over explicit durable provider.

#### `durability_contract_recovery_preserves_aspect_bearing_patch_truth_and_history`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:38`.

- **HAPPY** 80: `assert_eq!(outcome.latest_commit, Some(updated.commit.clone()))` - reviewed public owner result over explicit durable provider.
- **HAPPY** 81: `assert_eq!(expected_history, recovered_history)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 82: `assert_eq!(expected_digest, recovered_digest)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 83: `assert_eq!( expected_envelope.patch.authoritative_record_patches, recovered_envelope.patch.authoritative_record_patches )` - reviewed public owner result over explicit durable provider.
- **HAPPY** 87: `assert_eq!( recovered_envelope.patch.authoritative_record_patches[0].authoritative_changed_aspects(), ordered_aspect_keys([aspect_key("name")]) )` - reviewed public owner result over explicit durable provider.
- **HAPPY** 91: `assert!(!recovered_envelope.patch.authoritative_record_patches[0].contains_opaque_aspect)` - reviewed public owner result over explicit durable provider.

#### `durability_contract_recovery_preserves_relation_aspect_history_for_retained_audit_relations`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:95`.

- **HAPPY** 139: `assert_eq!(outcome.latest_commit, Some(deleted.commit.clone()))` - reviewed public owner result over explicit durable provider.
- **HAPPY** 140: `assert_eq!(expected_history, recovered_history)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 141: `assert_eq!(expected_digest, recovered_digest)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 142: `assert_eq!(recovered_history.len(), 2)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 144: `assert_eq!( recovered_history[0].origin.changed_aspects, ordered_aspect_keys([ aspect_key("label"), aspect_key("lifecycle"), aspect_key("source"), aspect_key("target"), ]) )` - reviewed public owner result over explicit durable provider.
- **HAPPY** 153: `assert_eq!( recovered_history[1].origin.changed_aspects, ordered_aspect_keys([aspect_key("lifecycle")]) )` - reviewed public owner result over explicit durable provider.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/support.rs:345 assert_direct_history_origin_invariants`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `durability_contract_recovery_preserves_branch_local_endpoint_deletion_retirement_histories`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:160`.

- **HAPPY** 324: `assert_eq!(outcome.latest_commit, runtime.history().latest_commit())` - reviewed public owner result over explicit durable provider.
- **HAPPY** 325: `assert_eq!(expected_main_digest, recovered_main_digest)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 326: `assert_eq!(expected_feature_digest, recovered_feature_digest)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 327: `assert_eq!(expected_main_inspection, recovered_main_inspection)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 328: `assert_eq!(expected_feature_inspection, recovered_feature_inspection)` - reviewed public owner result over explicit durable provider.

### `tests/durability/contracts/branch_root_checkpoint.rs`

#### `checkpoint_recovery_rejects_missing_exact_branch_root_artifact`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:4`.

- **UNCLASSIFIED** 31: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 32: `assert!( error.detail.contains("missing branch-root image"), "{error:?}" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 36: `assert_eq!(recovered.history().immutable_commit_count(), 0)` - semantic boundary still requires direct review; no move authorized.

#### `checkpoint_recovery_rejects_root_artifact_without_commit_envelope`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:40`.

- **UNCLASSIFIED** 64: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 65: `assert!(error.detail.contains("names missing commit envelope"))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 66: `assert_eq!(recovered.history().immutable_commit_count(), 0)` - semantic boundary still requires direct review; no move authorized.

#### `checkpoint_recovery_rejects_duplicate_commit_envelopes_before_root_readmission`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:70`.

- **UNCLASSIFIED** 93: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 94: `assert!( error .detail .contains("reused by multiple canonical artifacts") || error .detail .contains("duplicate checkpoint commit envelope"), "{error:?}" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 103: `assert_eq!(recovered.history().immutable_commit_count(), 0)` - semantic boundary still requires direct review; no move authorized.

#### `checkpoint_recovery_readmits_shared_root_once_for_sibling_heads`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:107`.

- **UNCLASSIFIED** 121: `assert_eq!(plan.checkpoint.as_ref().unwrap().branch_roots.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 129: `assert_eq!( current_branch_entity_count(execution, &recovered, &BranchId("main".to_owned())), 1 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 133: `assert_eq!( current_branch_entity_count(execution, &recovered, &sibling), 1 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 141: `assert_eq!(main.observation().target(), sibling.observation().target())` - semantic boundary still requires direct review; no move authorized.
- **C2** 154: `assert!(std::sync::Arc::ptr_eq(&main_root, &sibling_root))` - actual residency, identity, pin, allocation, dictionary or cost observation.

#### `tail_recovery_resolves_the_checkpoint_target_root_not_the_fork_source_head`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:158`.

- **UNCLASSIFIED** 186: `assert_eq!(main_count, 2, "main contains seed plus its own tail write")` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 187: `assert_eq!( storm_count, 2, "storm excludes the source's later tail write" )` - semantic boundary still requires direct review; no move authorized.

#### `checkpoint_recovery_rejects_duplicate_global_partition_images`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:194`.

- **UNCLASSIFIED** 219: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 220: `assert!(error.detail.contains("duplicate partition image"))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 221: `assert_eq!(recovered.history().immutable_commit_count(), 0)` - semantic boundary still requires direct review; no move authorized.

#### `checkpoint_recovery_rejects_duplicate_branch_root_partition_images`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:225`.

- **UNCLASSIFIED** 248: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 249: `assert!(error.detail.contains("duplicate partition image"))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 250: `assert_eq!(recovered.history().immutable_commit_count(), 0)` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/branch_root_checkpoint/foreign_targets.rs`

#### `checkpoint_recovery_rejects_foreign_branch_target_substitution`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint/foreign_targets.rs:4`.

- **UNCLASSIFIED** 43: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 44: `assert!(error.detail.contains("foreign branch stream"))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 45: `assert_eq!(recovered.history().immutable_commit_count(), 0)` - semantic boundary still requires direct review; no move authorized.

#### `tail_recovery_rejects_foreign_branch_target_substitution`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint/foreign_targets.rs:49`.

- C4 action 84: `.map(|commit| commit.envelope_mut_for_test(execution))`.
- **C4** 99: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 100: `assert!(error.detail.contains("foreign branch stream"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 101: `assert_eq!(recovered.history().immutable_commit_count(), 0)` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `tests/durability/contracts/branch_root_content_binding.rs`

#### `checkpoint_rejects_previous_branch_root_format_before_readmission`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:5`.

- **UNCLASSIFIED** 11: `assert_eq!(root.format_version, 4)` - semantic boundary still requires direct review; no move authorized.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:295 assert_corrupt_recovery_leaves_no_recovered_state`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `checkpoint_rejects_altered_root_content_with_recomputed_image_digest`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:22`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:295 assert_corrupt_recovery_leaves_no_recovered_state`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `checkpoint_rejects_omitted_root_partition_with_recomputed_image_digest`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:40`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:295 assert_corrupt_recovery_leaves_no_recovered_state`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `checkpoint_rejects_same_commit_truth_root_relabeling`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:59`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:295 assert_corrupt_recovery_leaves_no_recovered_state`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `checkpoint_rejects_same_commit_schema_root_relabeling`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:80`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:295 assert_corrupt_recovery_leaves_no_recovered_state`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `tail_rejects_same_commit_root_descriptor_substitution`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:101`.

- C4 action 119: `.map(|commit| commit.envelope_mut_for_test(execution))`.
- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:295 assert_corrupt_recovery_leaves_no_recovered_state`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `tail_replay_reconstructs_the_owner_content_root`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:138`.

- **UNCLASSIFIED** 158: `assert_eq!(current_main_descriptor(&recovered), expected)` - semantic boundary still requires direct review; no move authorized.

#### `checkpoint_owner_root_is_stable_across_fingerprint_symbol_ids`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:162`.

- **UNCLASSIFIED** 175: `assert_eq!(left_error.detail, right_error.detail)` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/branch_root_schema_binding.rs`

#### `branch_roots_with_one_schema_deduplicate_the_durable_carrier`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_schema_binding.rs:4`.

- **UNCLASSIFIED** 23: `assert!(checkpoint.branch_roots.len() >= 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 24: `assert_eq!(checkpoint.branch_root_schema_images.len(), 1)` - semantic boundary still requires direct review; no move authorized.

#### `recovery_rejects_a_missing_branch_root_schema_carrier`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_schema_binding.rs:28`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/durability/contracts/branch_root_schema_binding.rs:168 assert_corrupt_schema_recovery`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `recovery_rejects_a_tampered_branch_root_schema_carrier`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_schema_binding.rs:43`.

- C4 action 52: `.corrupt_schema_root_for_test();`.
- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/durability/contracts/branch_root_schema_binding.rs:168 assert_corrupt_schema_recovery`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `recovery_rejects_a_schema_carrier_swapped_between_exact_roots`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_schema_binding.rs:58`.

- C4 action 66: `runtime.set_schema_registry_for_test(`.
- **C4** 110: `assert_eq!(checkpoint.branch_root_schema_images.len(), 2)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 113: `assert_ne!(root.schema_carrier_digest, replacement)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 140: `assert_eq!( error.class, RecoveryFailureClass::CorruptCheckpoint, "unexpected denial: {error:?}" )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 145: `assert!( error.detail.contains("schema carrier linkage mismatch"), "unexpected denial: {error:?}" )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 149: `assert_eq!(recovered.history().immutable_commit_count(), 0)` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `tests/durability/contracts/branch_root_schema_binding/exact_contracts.rs`

#### `recovered_exact_roots_interpret_records_with_their_own_schema_contracts`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_schema_binding/exact_contracts.rs:4`.

- C4 action 32: `runtime.set_schema_registry_for_test(v2_registry.clone());`.
- **C4** 103: `assert_eq!( read_entity_name(old_record), Some("legacy-v1-before-recovery".to_owned()) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 107: `assert_eq!( read_entity_aspect_field(old_record, aspect_key("display"), field_key("display")), None )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 128: `assert_eq!( read_entity_name(new_record), Some("new-contract".to_owned()) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 145: `assert_eq!( legacy_projection .entity_record_with_projection_scope(execution, old_entity, name_scope.clone(), |_| { Some(()) },) .expect("read request remains live"), Some(()) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 153: `assert_eq!( current_projection .entity_record_with_projection_scope( execution, new_entity, display_scope.clone(), |_| Some(()), ) .expect("read request remains live"), Some(()) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 164: `assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { legacy_projection .entity_record_with_projection_scope(execution, old_entity, display_scope, |_| Some(())) .expect("read request remains live") })) .is_err())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 170: `assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { current_projection .entity_record_with_projection_scope(execution, new_entity, name_scope, |_| Some(())) .expect("read request remains live") })) .is_err())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 206: `assert!(matches!( denial, TransactionCommitError::Conflict { error, .. } if matches!( error.class, crate::transactions::data::ConflictClass::RecordAspectPatchDenied { denial: crate::transactions::data::RecordAspectPatchDenial::FieldAuthoringDenied { reason: crate::transactions::data::AspectFieldTargetRejectionReason::UndeclaredAspect, .. }, .. } ) ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `tests/durability/contracts/branch_root_shared_target_binding.rs`

#### `checkpoint_rejects_later_shared_root_cell_with_relabeled_schema_root`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_shared_target_binding.rs:10`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:295 assert_corrupt_recovery_leaves_no_recovered_state`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `checkpoint_rejects_later_shared_root_cell_with_relabeled_truth_root`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_root_shared_target_binding.rs:35`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:295 assert_corrupt_recovery_leaves_no_recovered_state`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

### `tests/durability/contracts/branch_transition/checkpoint_state.rs`

#### `checkpoint_size_tracks_branch_state_not_covered_transitions`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_transition/checkpoint_state.rs:6`.

- **UNCLASSIFIED** 33: `assert_eq!(artifacts(&runtime).len(), count)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 38: `assert_eq!( checkpoint.covered_branch_transitions(), Some(Sequence(count as u64)) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 42: `assert!( artifacts(&runtime).is_empty(), "covered transition log is released" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 49: `assert!(artifacts(&recovered).is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 51: `assert_eq!( attached_test_medium(&recovered).transition_head(), Sequence(count as u64 + 1) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 61: `assert!( through_archive_and_pending_deletion <= direct + 32, "covered history grew the manifest: {direct} -> {through_archive_and_pending_deletion}" )` - semantic boundary still requires direct review; no move authorized.

#### `a_deleting_checkpoint_cell_requires_its_retired_name`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_transition/checkpoint_state.rs:68`.

- **UNCLASSIFIED** 107: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 108: `assert!( error .detail .contains("deleting branch 'pending' has no retired name"), "{error:?}" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 114: `assert_eq!(recovered.history().immutable_commit_count(), 0)` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/branch_transition/fates.rs`

#### `every_transition_fate_preserves_the_durable_before_live_boundary`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_transition/fates.rs:134`.

- C4 action 161: `medium.script_next_transition(answer);`.
- **C4** 167: `assert_ne!(footprint(&runtime), before, "{action:?}")` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 168: `assert_eq!(medium.transition_head(), Sequence(sequence))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 174: `assert_eq!(artifact.sequence(), Sequence(sequence))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 175: `assert_eq!(artifact.follows(), latest.patch_position())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 181: `assert!(denial.publication_refused(), "{denial:?}")` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 182: `assert_eq!(footprint(&runtime), before, "{action:?}")` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 183: `assert_eq!(medium.transition_head(), Sequence(sequence - 1))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 185: `assert_eq!(medium.transition_head(), Sequence(sequence))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 198: `assert_eq!(found, Sequence(sequence))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 199: `assert_eq!(follows, latest.patch_position())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 200: `assert_eq!(digest, offered.payload_digest())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 201: `assert_eq!(footprint(&runtime), before, "{action:?}")` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 204: `assert_eq!( (medium.offered().len(), medium.offered_transitions().len()), offers )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 208: `assert_eq!(footprint(&runtime), before, "{action:?}")` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 211: `assert_eq!( medium.transition_sequences(), (1..=medium.transition_head().0) .map(Sequence) .collect::<Vec<_>>() )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 224: `assert_eq!(cell.is_some(), took_effect)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 225: `assert_eq!( cell.unwrap().lifecycle_posture(), if took_effect { RelationalBranchLifecyclePosture::Archived } else { RelationalBranchLifecyclePosture::Live } )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 233: `assert_eq!( cell.unwrap().lifecycle_posture(), if took_effect { RelationalBranchLifecyclePosture::Deleting } else { RelationalBranchLifecyclePosture::Live } )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 241: `assert_eq!(cell.is_none(), took_effect)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/durability/contracts/branch_transition/fates.rs:110 assert_blocked`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `an_unresolved_commit_also_blocks_branch_transitions_with_its_original_fate`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_transition/fates.rs:249`.

- C4 action 255: `medium.script_next_publication(ScriptedPublication::IndeterminateAfterEffect);`.
- **C4** 270: `assert!(medium.offered_transitions().is_empty())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/durability/contracts/branch_transition/fates.rs:110 assert_blocked`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

### `tests/durability/contracts/branch_transition/mod.rs`

No direct test declarations. Executing helper roots and claims:

- `reopen` at 36: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/durability/contracts/branch_transition/checkpoint_state.rs:6 checkpoint_size_tracks_branch_state_not_covered_transitions`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/fates.rs:134 every_transition_fate_preserves_the_durable_before_live_boundary`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/recovery.rs:6 a_durable_fork_without_a_subsequent_commit_reopens_and_commits`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/recovery.rs:27 a_fork_covered_by_a_checkpoint_after_its_source_moves_accepts_a_target_tail`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/recovery.rs:49 a_fork_in_the_transition_tail_is_merged_before_the_target_commit`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/recovery.rs:72 lifecycle_postures_and_retired_names_survive_checkpoints_and_crashes`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/replay.rs:5 partial_replay_includes_an_intermediate_fork_without_its_own_commit`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/tampering.rs:36 a_sequence_gap_and_invalid_follows_positions_are_refused`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/tampering.rs:72 a_commit_carrying_an_unknown_branch_cell_cannot_replace_its_missing_fork`.

### `tests/durability/contracts/branch_transition/reconstruction.rs`

#### `memory_checkpoint_keeps_replay_reconstructible`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_transition/reconstruction.rs:31`.

- **UNCLASSIFIED** 37: `assert_eq!( runtime.config().durability.policy.log.retention_mode, crate::config::data::DurableLogRetentionMode::CompactAfterCheckpoint )` - semantic boundary still requires direct review; no move authorized.

#### `replay_reads_only_the_gap_after_its_checkpoint`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_transition/reconstruction.rs:91`.

- **UNCLASSIFIED** 129: `assert!(reads.len() <= 2, "covered history was reread: {reads:?}")` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 130: `assert!( reads .iter() .all(|sequence| sequence.0 > covered && sequence.0 <= covered + 2), "read outside the missing gap: {reads:?}" )` - semantic boundary still requires direct review; no move authorized.

#### `recovery_plan_after_native_capture_keeps_covered_forks`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_transition/reconstruction.rs:140`.

- **UNCLASSIFIED** 165: `assert_eq!(protection.bounds().canonical.0, covered.0 + 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 187: `assert_eq!( recovered .history() .branch_head(&BranchId("A".into())) .unwrap() .commit_id, target.commit.commit_id )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 195: `assert_eq!( recovered .history() .branch_head(&BranchId("main".into())) .unwrap() .commit_id, later.commit.commit_id )` - semantic boundary still requires direct review; no move authorized.

#### `a_memory_fork_can_replay_its_compacted_durable_source`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_transition/reconstruction.rs:206`.

- **UNCLASSIFIED** 218: `assert!( speculative.durability.medium().is_none(), "a runtime fork cannot publish to its source's medium" )` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/branch_transition/recovery.rs`

#### `a_durable_fork_without_a_subsequent_commit_reopens_and_commits`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_transition/recovery.rs:6`.

- **UNCLASSIFIED** 20: `assert_eq!( again.history().branch_head(&BranchId("target".into())), recovered.history().branch_head(&BranchId("target".into())) )` - semantic boundary still requires direct review; no move authorized.

#### `a_fork_covered_by_a_checkpoint_after_its_source_moves_accepts_a_target_tail`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_transition/recovery.rs:27`.

- **UNCLASSIFIED** 42: `assert_eq!( recovered.history().branch_head(&BranchId("target".into())), runtime.history().branch_head(&BranchId("target".into())) )` - semantic boundary still requires direct review; no move authorized.

#### `a_fork_in_the_transition_tail_is_merged_before_the_target_commit`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_transition/recovery.rs:49`.

- **UNCLASSIFIED** 64: `assert_eq!( recovered.history().branch_head(&BranchId("target".into())), runtime.history().branch_head(&BranchId("target".into())) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 68: `assert_eq!(artifacts(&recovered), artifacts(&runtime))` - semantic boundary still requires direct review; no move authorized.

#### `lifecycle_postures_and_retired_names_survive_checkpoints_and_crashes`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_transition/recovery.rs:72`.

- **UNCLASSIFIED** 111: `assert_eq!( recovered .history .branch_cell(&BranchId(name.into())) .unwrap() .lifecycle_posture(), posture )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 120: `assert!(recovered .branch_identity(&BranchId("deleted".into())) .is_err())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 127: `assert!(matches!( recovered.fork_branch(execution, BranchId(name.into()), basis), Err(Stop::Denied(RelationalForkDenial::RetiredTarget)) ))` - semantic boundary still requires direct review; no move authorized.

#### `independent_runtimes_encode_byte_identical_transition_histories`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_transition/recovery.rs:136`.

- **UNCLASSIFIED** 163: `assert_eq!(first.len(), 5)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 164: `assert_eq!(first, history())` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/branch_transition/replay.rs`

#### `partial_replay_includes_an_intermediate_fork_without_its_own_commit`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_transition/replay.rs:5`.

- **HAPPY** 30: `assert!(replay.mismatches.is_empty(), "{replay:?}")` - reviewed public owner result over explicit durable provider.
- **HAPPY** 34: `assert!(artifacts(&runtime).is_empty())` - reviewed public owner result over explicit durable provider.

### `tests/durability/contracts/branch_transition/tampering.rs`

#### `a_damaged_transition_frame_is_refused_at_the_medium_read`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_transition/tampering.rs:19`.

- C4 action 27: `medium.flip_transition_byte(Sequence(1), bytes.len() - 1);`.
- **C4** 32: `assert!(matches!(refusal, crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifactFrameRefusal::DigestMismatch { .. }))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `a_sequence_gap_and_invalid_follows_positions_are_refused`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_transition/tampering.rs:36`.

- C4 action 49: `medium.remove_transition(Sequence(1));`.
- C4 action 62: `medium.replace_transition(Sequence(2), artifact.as_bytes().into());`.
- **C4** 46: `assert_eq!(artifacts(&pristine), artifacts(&runtime))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 64: `assert_eq!( refusal(execution, &runtime).class, RecoveryFailureClass::NonContiguousBranchTransitionTail )` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `a_commit_carrying_an_unknown_branch_cell_cannot_replace_its_missing_fork`

Source: `crates/worth-relational/src/tests/durability/contracts/branch_transition/tampering.rs:72`.

- C4 action 81: `attached_test_medium(&runtime).remove_transition(Sequence(1));`.
- **C4** 80: `assert_eq!(artifacts(&pristine), artifacts(&runtime))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 83: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 84: `assert!( error .detail .contains("which no checkpoint or durable branch transition created"), "{error:?}" )` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `tests/durability/contracts/caller_association.rs`

#### `caller_association_bounds_and_rebinding_refuse_without_retaining_publication`

Source: `crates/worth-relational/src/tests/durability/contracts/caller_association.rs:69`.

- **UNCLASSIFIED** 73: `assert_eq!(Association::from_bytes(&[]), Err(Refusal::Empty))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 75: `assert_eq!(full.as_bytes().len(), Association::MAX_LEN)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 77: `assert_eq!( rmp_serde::from_slice::<Association>(&encoded).unwrap(), full )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 81: `assert!(rmp_serde::from_slice::<Association>(&[0xc4, 0]).is_err())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 84: `assert!(rmp_serde::from_slice::<Association>(&oversized_wire).is_err())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 85: `assert_eq!( Association::from_bytes(&[0; Association::MAX_LEN + 1]), Err(Refusal::TooLong { actual: Association::MAX_LEN + 1 }) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 99: `assert!(proposal .bind_caller_association(association.clone()) .is_err())` - semantic boundary still requires direct review; no move authorized.
- **C2** 102: `assert!(runtime.record_identity.pending_snapshot().is_empty())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 109: `assert_eq!(candidate.caller_association().as_ref(), Some(&association))` - semantic boundary still requires direct review; no move authorized.
- **C2** 110: `assert_eq!( runtime.history.pending_canonical_publication_route_count(), routes + 1 )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 115: `assert_eq!( runtime.history.pending_canonical_publication_route_count(), routes )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 119: `assert!(runtime.record_identity.pending_snapshot().is_empty())` - actual residency, identity, pin, allocation, dictionary or cost observation.

#### `caller_association_survives_embedded_checkpoint_and_refuses_old_semantics`

Source: `crates/worth-relational/src/tests/durability/contracts/caller_association.rs:123`.

- **UNCLASSIFIED** 144: `assert_eq!(commit.envelope().caller_association(), Some(&association))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 146: `assert_eq!(reencoded.as_bytes(), artifact.as_bytes())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 153: `assert_eq!( expect_durability_failure(stop).class, crate::durability::data::RecoveryFailureClass::UnsupportedLegacySemantics )` - semantic boundary still requires direct review; no move authorized.

#### `caller_association_survives_cold_checkpoint_history_and_journal_tail`

Source: `crates/worth-relational/src/tests/durability/contracts/caller_association.rs:161`.

- **UNCLASSIFIED** 182: `assert_eq!( outcome.coverage.checkpoint_covered_position, Some(head.position()) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 186: `assert_eq!(outcome.coverage.replayed_tail_commits, 1)` - semantic boundary still requires direct review; no move authorized.
- **C2** 192: `assert!(cold.selected_directory().is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 193: `assert!(cold.journal_placement().is_none())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 194: `assert!( cold.positioned_if_resident().is_none(), "historical body begins cold" )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 203: `assert_eq!(actual.len(), 3)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 205: `assert_eq!(recovered.position(), original.position())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 206: `assert_eq!(recovered.commit_id(), original.commit_id())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 207: `assert_eq!(recovered.payload_digest(), original.payload_digest())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 208: `assert_eq!(recovered.as_bytes(), original.as_bytes())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 211: `assert_eq!(readmitted.envelope().commit, original.envelope().commit)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 212: `assert_eq!( readmitted.envelope().semantic_result(), original.envelope().semantic_result() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 216: `assert_eq!( readmitted.envelope().caller_association(), Some(&association) )` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/canonical_artifact_neutrality.rs`

#### `independent_runtimes_publish_byte_identical_canonical_artifacts`

Source: `crates/worth-relational/src/tests/durability/contracts/canonical_artifact_neutrality.rs:162`.

- **UNCLASSIFIED** 167: `assert_ne!(first.runtime_instance_id(), second.runtime_instance_id())` - semantic boundary still requires direct review; no move authorized.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/durability/contracts/canonical_artifact_neutrality.rs:201 assert_same_artifacts`, `crates/worth-relational/src/tests/durability/contracts/canonical_artifact_neutrality.rs:16 run_script`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `a_recovered_runtime_continues_with_the_reference_artifacts`

Source: `crates/worth-relational/src/tests/durability/contracts/canonical_artifact_neutrality.rs:177`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/durability/contracts/canonical_artifact_neutrality.rs:201 assert_same_artifacts`, `crates/worth-relational/src/tests/durability/contracts/canonical_artifact_neutrality.rs:16 run_script`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

### `tests/durability/contracts/checkpoint_recovery.rs`

#### `durability_contract_checkpoint_tail_recovery_preserves_post_checkpoint_commits`

Source: `crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs:6`.

- **UNCLASSIFIED** 27: `assert_eq!(outcome.recovered_commits, 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 28: `assert_eq!(outcome.coverage.checkpoint_commits, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 29: `assert_eq!(outcome.coverage.replayed_tail_commits, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 30: `assert_eq!(outcome.latest_commit, Some(later.commit.clone()))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 31: `assert_eq!( recovered .history() .branch_head(&BranchId("main".to_string())), Some(later.commit.clone()) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 37: `assert_eq!( recovered .replay() .canonical_commit_envelope(execution, main.commit.commit_id) .unwrap() .unwrap() .commit .commit_id, main.commit.commit_id )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 48: `assert_eq!(retained.envelopes.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 49: `assert_eq!(retained.partition_images.as_ptr(), checkpoint_images)` - semantic boundary still requires direct review; no move authorized.

#### `durability_tail_replay_releases_each_commit_snapshot_beyond_live_capacity`

Source: `crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs:53`.

- **UNCLASSIFIED** 84: `assert_eq!(outcome.recovered_commits, 71)` - semantic boundary still requires direct review; no move authorized.
- **C2** 85: `assert_eq!( recovered .storage_access() .storage_stats(execution) .unwrap() .published_snapshot_handle_count, 0, "replay closeout must not consume the live published-snapshot budget" )` - actual residency, identity, pin, allocation, dictionary or cost observation.

#### `durability_contract_recovery_fails_closed_without_exact_branch_cells`

Source: `crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs:97`.

- **UNCLASSIFIED** 122: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 123: `assert!(error .detail .contains("durable checkpoint omitted exact branch-cell state"))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 126: `assert_eq!(recovered.history().immutable_commit_count(), 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 127: `assert!(recovered .history() .branch_head(&BranchId("main".to_owned())) .is_none())` - semantic boundary still requires direct review; no move authorized.

#### `durability_contract_recovery_rejects_branch_cell_target_without_catalog_artifact`

Source: `crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs:134`.

- **UNCLASSIFIED** 172: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 173: `assert!( error.detail.contains("references missing commit artifact"), "{error:?}" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 177: `assert_eq!(recovered.history().immutable_commit_count(), 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 178: `assert!(recovered .history() .branch_head(&BranchId("main".to_owned())) .is_none())` - semantic boundary still requires direct review; no move authorized.

#### `durability_contract_recovery_rejects_tail_checkpoint_drift_before_mutation`

Source: `crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs:185`.

- C4 action 202: `.map(|commit| commit.envelope_mut_for_test(execution))`.
- **C4** 219: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 220: `assert!(error .detail .contains("recovery branch-cell state conflicts"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 223: `assert!(recovered.history().immutable_commit_count() <= 1)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 224: `assert_ne!( recovered .history() .branch_head(&BranchId("main".to_owned())) .map(|head| head.commit_id), Some(second.commit.commit_id) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `durability_contract_checkpoint_recovers_index_metadata`

Source: `crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs:234`.

- **UNCLASSIFIED** 277: `assert_eq!(generation.generation_id, build.generations[0].generation_id)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 278: `assert_eq!(generation.source_commit_id, commit.commit.commit_id)` - semantic boundary still requires direct review; no move authorized.

#### `durability_contract_checkpoint_recovers_lineage_authority`

Source: `crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs:282`.

- **UNCLASSIFIED** 304: `assert_eq!( checkpoint .lineage .digest_basis() .published_lineage_commit_count, runtime .history() .commit_envelopes_snapshot() .iter() .filter(|envelope| envelope.has_lineage_authority()) .count() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 316: `assert_eq!( checkpoint .lineage .digest_basis() .published_lineage_event_count, runtime .history() .commit_envelopes_snapshot() .iter() .map(|envelope| envelope.lineage_digest_basis().lineage_event_count()) .sum::<usize>() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 328: `assert_eq!( checkpoint .lineage .digest_basis() .published_lineage_decision_count, runtime .history() .commit_envelopes_snapshot() .iter() .map(|envelope| envelope.lineage_digest_basis().lineage_decision_count()) .sum::<usize>() )` - semantic boundary still requires direct review; no move authorized.
- **C2** 340: `assert_eq!( checkpoint.lineage.counters().node_count, checkpoint.lineage.nodes().len() )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 364: `assert_eq!(graph.nodes.len(), 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 365: `assert_eq!(graph.events.len(), 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 366: `assert!(graph .events .iter() .all(|event| event.kind == LineageEventKind::Create))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 370: `assert_eq!( recovered .lineage_access() .for_record(execution, first_entity) .unwrap() .unwrap() .lineage_id(), first_lineage )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 379: `assert_eq!( recovered .lineage_access() .for_record(execution, second_entity) .unwrap() .unwrap() .lineage_id(), second_lineage )` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/checkpoint_recovery/checkpoint_fallback.rs`

#### `durability_contract_damaged_checkpoint_refuses_rather_than_shortening_history`

Source: `crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery/checkpoint_fallback.rs:5`.

- C4 action 14: `attached_test_medium(&runtime).damage_checkpoint_payload();`.
- **C4** 25: `assert_eq!( expect_durability_failure(refused).class, RecoveryFailureClass::CorruptCheckpoint )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 29: `assert!(recovered .history() .branch_head(&BranchId("main".to_string())) .is_none())` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `tests/durability/contracts/checkpoint_recovery/empty_head.rs`

#### `durability_contract_empty_checkpoint_restores_a_live_retained_head`

Source: `crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery/empty_head.rs:5`.

- **UNCLASSIFIED** 27: `assert_eq!(basis.observation().selected_root_identity(), 0)` - semantic boundary still requires direct review; no move authorized.
- **C2** 28: `assert_eq!(recovered.retention_cost_counters().head_installs, 1)` - actual residency, identity, pin, allocation, dictionary or cost observation.

### `tests/durability/contracts/declared_schema_binding.rs`

#### `another_label_with_identical_contracts_refuses_before_service`

Source: `crates/worth-relational/src/tests/durability/contracts/declared_schema_binding.rs:74`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/durability/contracts/declared_schema_binding.rs:45 assert_refused_as_declared_schema`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `after_a_durable_transition_only_its_target_is_served`

Source: `crates/worth-relational/src/tests/durability/contracts/declared_schema_binding.rs:92`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/durability/contracts/declared_schema_binding.rs:45 assert_refused_as_declared_schema`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

### `tests/durability/contracts/derived_index_checkpoint.rs`

#### `reclamation_and_checkpoint_keep_sibling_head_generation`

Source: `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs:61`.

- **C2** 89: `assert!(runtime.indexes.generation(sibling_generation).is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 90: `assert!(runtime.indexes.generation(main_generation).is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 103: `assert!(recovered.indexes.generation(sibling_generation).is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 104: `assert!(recovered.indexes.generation(main_generation).is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.

#### `live_fork_scoped_generation_readmits_from_shared_basis`

Source: `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs:108`.

- **UNCLASSIFIED** 127: `assert!(built.failed_indexes.is_empty())` - semantic boundary still requires direct review; no move authorized.
- **C2** 143: `assert!(recovered.indexes.generation(generation).is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.

#### `repeated_edits_checkpoint_only_the_live_index_payload`

Source: `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs:147`.

- **UNCLASSIFIED** 174: `assert!(retained_bytes * 4 < legacy_bytes)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 175: `assert!(runtime.index_access().generations_snapshot().len() < 24)` - semantic boundary still requires direct review; no move authorized.

#### `checkpoint_index_artifact_survives_tail_replay_without_envelope_caches`

Source: `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs:179`.

- **UNCLASSIFIED** 188: `assert!(checkpoint.derived_index_artifacts.is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 189: `assert_eq!(checkpoint.derived_index_checkpoint_format, 2)` - semantic boundary still requires direct review; no move authorized.
- **C2** 203: `assert!(recovered.indexes.generation(first_generation).is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 204: `assert!(recovered.indexes.generation(unjournaled_cache).is_none())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 206: `assert!(recovered.indexes.generation(rebuilt).is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.

#### `checkpoint_index_artifact_missing_payload_denies_before_recovery`

Source: `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs:210`.

- **UNCLASSIFIED** 230: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 248: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - semantic boundary still requires direct review; no move authorized.

#### `checkpoint_recovery_denies_a_digest_valid_foreign_generation_affinity`

Source: `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs:252`.

- **UNCLASSIFIED** 290: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 291: `assert!(error.detail.contains("branch affinity"))` - semantic boundary still requires direct review; no move authorized.

#### `reclamation_keeps_a_pinned_old_reader_until_release`

Source: `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs:295`.

- **C2** 310: `assert!(runtime.indexes.generation(middle_generation).is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 312: `assert!(runtime.indexes.generation(first_generation).is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 313: `assert!(runtime.indexes.generation(latest_generation).is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 314: `assert!(runtime.indexes.generation(middle_generation).is_none())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 327: `assert!(recovered.indexes.generation(first_generation).is_none())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 328: `assert!(recovered.indexes.generation(latest_generation).is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 332: `assert!(runtime.indexes.generation(first_generation).is_none())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 333: `assert!(runtime.indexes.generation(latest_generation).is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.

#### `repeated_closed_branches_leave_no_latest_generation_residue`

Source: `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs:337`.

- **UNCLASSIFIED** 360: `assert!(built.failed_indexes.is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 363: `assert!(runtime .delete_branch(execution, &identity) .unwrap() .deleted() .is_some())` - semantic boundary still requires direct review; no move authorized.
- **C2** 368: `assert!(runtime.indexes.generation(generation).is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 369: `assert_eq!(runtime.run_index_generation_reclamation_pass(), Some(1))` - semantic boundary still requires direct review; no move authorized.
- **C2** 370: `assert!(runtime.indexes.generation(generation).is_none())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 371: `assert!(runtime .index_access() .read_latest_generation(execution, index.index_id, &branch) .unwrap() .is_none())` - semantic boundary still requires direct review; no move authorized.
- **C2** 376: `assert_eq!(runtime.history.retired_branch_binding_count(), 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 378: `assert!(runtime.index_access().generations_snapshot().is_empty())` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/derived_index_checkpoint/artifact_affinity.rs`

#### `legacy_index_affinity_is_checked_before_replacing_a_live_recovery_target`

Source: `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/artifact_affinity.rs:5`.

- **UNCLASSIFIED** 44: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 45: `assert!(error.detail.contains("branch affinity"), "{error:?}")` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 46: `assert_eq!(target.observe_branch(&identity).unwrap().0, before)` - semantic boundary still requires direct review; no move authorized.

#### `envelope_index_affinity_requires_admission_before_installation`

Source: `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/artifact_affinity.rs:55`.

- C4 action 64: `envelope.body_mut_for_test().derived_index_artifacts =`.
- C4 action 70: `envelope.body_mut_for_test().derived_index_artifacts =`.
- **C4** 77: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 78: `assert!(error.detail.contains("branch affinity"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C2** 79: `assert_eq!( runtime.indexes.generation(id).unwrap().as_ref(), actual.as_ref() )` - actual residency, identity, pin, allocation, dictionary or cost observation.

### `tests/durability/contracts/derived_index_checkpoint/deleted_branch_reader.rs`

#### `deleted_branch_reader_retains_only_its_exact_scoped_generation`

Source: `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/deleted_branch_reader.rs:5`.

- **UNCLASSIFIED** 27: `assert!(built.failed_indexes.is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 29: `assert!(runtime .delete_branch(execution, &identity) .unwrap() .deleted() .is_some())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 34: `assert_eq!( runtime .run_index_generation_reclamation_after_publication(&anchor) .unwrap(), Some(0) )` - semantic boundary still requires direct review; no move authorized.
- **C2** 40: `assert!(runtime.indexes.generation(generation).is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 53: `assert!(recovered.indexes.generation(generation).is_none())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 55: `assert_eq!( runtime .run_index_generation_reclamation_after_publication(&anchor) .unwrap(), Some(0) )` - semantic boundary still requires direct review; no move authorized.
- **C2** 61: `assert!(runtime.indexes.generation(generation).is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 63: `assert_eq!( runtime .run_index_generation_reclamation_after_publication(&anchor) .unwrap(), Some(1) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 69: `assert_eq!(runtime.run_index_generation_reclamation_pass(), Some(0))` - semantic boundary still requires direct review; no move authorized.
- **C2** 70: `assert!(runtime.indexes.generation(generation).is_none())` - actual residency, identity, pin, allocation, dictionary or cost observation.

### `tests/durability/contracts/derived_index_checkpoint/historical_reader.rs`

#### `historical_reader_from_live_head_survives_head_retirement`

Source: `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/historical_reader.rs:4`.

- **UNCLASSIFIED** 30: `assert_eq!(retained.branch_id(), &branch)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 39: `assert_eq!( runtime .run_index_generation_reclamation_after_publication(&next) .unwrap(), Some(0) )` - semantic boundary still requires direct review; no move authorized.
- **C2** 45: `assert!(runtime.indexes.generation(first_generation).is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 46: `assert!(runtime.indexes.generation(next_generation).is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 48: `assert_eq!( runtime .run_index_generation_reclamation_after_publication(&next) .unwrap(), Some(1) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 54: `assert_eq!(runtime.run_index_generation_reclamation_pass(), Some(0))` - semantic boundary still requires direct review; no move authorized.
- **C2** 55: `assert!(runtime.indexes.generation(first_generation).is_none())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 56: `assert!(runtime.indexes.generation(next_generation).is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.

### `tests/durability/contracts/derived_index_checkpoint/lagging_scoped.rs`

#### `lagging_inherited_and_own_stream_scoped_generations_survive_checkpoint_reopen`

Source: `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/lagging_scoped.rs:7`.

- **UNCLASSIFIED** 27: `assert!(inherited.failed_indexes.is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 28: `assert_eq!(inherited.generations.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 30: `assert_ne!(anchor.commit.branch_id, branch)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 47: `assert_eq!(retained.index_id, index.index_id)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 48: `assert_eq!(retained.source_commit_id, source.commit_id)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 49: `assert_eq!(retained.source_branch_id, branch)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 50: `assert_eq!(retained.applicability.branch_id, branch)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 51: `assert_eq!(retained.applicability.version_id, source.version_id)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 52: `assert_ne!(source.commit_id, current.commit_id)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 55: `assert_eq!(basis.observation().commit_receipt(), Some(current))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 56: `assert!( reopened .index_access() .read_published_generation_for_observation( execution, index.index_id, &basis.observation() ) .unwrap() .is_none(), "retained lagging cache is not current index authority" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 73: `assert_eq!( read.base().and_then(read_entity_name), Some(expected_name.to_owned()) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 86: `assert_eq!(first.commit.branch_id, branch)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 113: `assert_eq!(second.commit.branch_id, branch)` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/derived_index_checkpoint/scoped_affinity.rs`

#### `real_shared_parent_root_scoped_checkpoint_restores_lookup_and_content`

Source: `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/scoped_affinity.rs:71`.

- **UNCLASSIFIED** 88: `assert_ne!(artifact.identity().authoring_branch(), &branch)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 94: `assert_eq!(admitted.generation_id, generation)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 95: `assert_eq!(admitted.source_branch_id, branch)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 116: `assert_eq!(actual.generation_id(), generation)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 117: `assert_eq!(actual.candidate_entity_ids(), &[entity])` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 126: `assert_eq!(parity.candidate_entity_ids(), &[entity])` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 127: `assert_eq!( name_on_branch(execution, &restored, &branch, entity), Some(NAME.into()) )` - semantic boundary still requires direct review; no move authorized.

#### `digest_valid_unissued_scoped_branch_denies_without_replacing_target_state`

Source: `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/scoped_affinity.rs:135`.

- **UNCLASSIFIED** 147: `assert_eq!(generations.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 149: `assert!(checkpoint .branch_cells .iter() .all(|cell| cell.branch_id != foreign))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 163: `assert_eq!(decoded[0].source_branch_id, foreign)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 164: `assert_eq!(decoded[0].applicability.branch_id, foreign)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 177: `assert_eq!( name_on_branch(execution, &target, &BranchId("main".into()), entity), Some("target-must-remain".into()) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 186: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 187: `assert_eq!(target.observe_branch(&identity).unwrap().0, head)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 188: `assert_eq!(target.index_access().definitions_snapshot(), definitions)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 189: `assert_eq!(target.index_access().generations_snapshot(), generations)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 190: `assert_eq!( name_on_branch(execution, &target, &BranchId("main".into()), entity), Some("target-must-remain".into()) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 194: `assert!(target.branch_identity(&foreign).is_err())` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/descriptor_continuity.rs`

#### `durability_contract_failure_schema_mismatch_is_explicit`

Source: `crates/worth-relational/src/tests/durability/contracts/descriptor_continuity.rs:5`.

- **UNCLASSIFIED** 36: `assert_eq!(error.class, RecoveryFailureClass::SchemaMismatch)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 37: `assert!(matches!( error.authority_continuity_mismatch, Some(RecoveryAuthorityContinuityMismatch::SchemaRegistryShape { expected_primary_schema_version: SchemaVersionId(1), found_primary_schema_version: SchemaVersionId(2), .. }) ))` - semantic boundary still requires direct review; no move authorized.

#### `durability_contract_failure_descriptor_semantics_version_mismatch_is_explicit`

Source: `crates/worth-relational/src/tests/durability/contracts/descriptor_continuity.rs:48`.

- **UNCLASSIFIED** 69: `assert_eq!(error.class, RecoveryFailureClass::SchemaMismatch)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 70: `assert!(matches!( error.authority_continuity_mismatch, Some( RecoveryAuthorityContinuityMismatch::DescriptorSemanticsVersion { expected: DescriptorSemanticsVersion(99), found: DescriptorSemanticsVersion(1), } ) ))` - semantic boundary still requires direct review; no move authorized.

#### `durability_recovery_plan_preserves_explicit_verification_mode`

Source: `crates/worth-relational/src/tests/durability/contracts/descriptor_continuity.rs:82`.

- **UNCLASSIFIED** 102: `assert_eq!( normal.verification_mode(), RecoveryVerificationMode::NormalRecoveryVerification )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 106: `assert_eq!( audit.verification_mode(), RecoveryVerificationMode::AuditRecoveryVerification )` - semantic boundary still requires direct review; no move authorized.

#### `contradictory_descriptor_version_refuses_before_recovery_or_owner_effects`

Source: `crates/worth-relational/src/tests/durability/contracts/descriptor_continuity.rs:113`.

- C4 action 120: `attached_test_medium(&runtime).rewrite_artifact_envelope(`.
- **C4** 135: `assert!( matches!(refused, crate::durability::data::RelationalRecoveryStop::Durability(error) if error.class == RecoveryFailureClass::CorruptCanonicalArtifact && error.detail.contains("PositionAdmission(SemanticResult(SchemaMismatch))")) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 140: `assert_eq!(runtime.runtime_instance_id(), owner)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 141: `assert_eq!( runtime.history().branch_head(&BranchId("main".into())), head )` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `durability_contract_failure_descriptor_canonical_basis_version_mismatch_is_explicit`

Source: `crates/worth-relational/src/tests/durability/contracts/descriptor_continuity.rs:148`.

- C4 action 155: `runtime.set_schema_registry_for_test(`.
- C4 action 188: `attached_test_medium(&runtime).rewrite_artifact_envelope(`.
- **C4** 207: `assert_eq!( plan.authority_continuity.descriptor_version_parity, RecoveryAuthorityParity::Drift )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 211: `assert!(matches!( plan.authority_continuity.first_mismatch, Some( RecoveryAuthorityContinuityMismatch::DescriptorCanonicalBasisVersion { expected: DescriptorCanonicalBasisVersion(1), found: DescriptorCanonicalBasisVersion(99), } ) ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 220: `assert_eq!( plan.authority_continuity.verification_outcome, RecoveryVerificationOutcome::Rejected { layer: ReplayVerificationLayer::DigestParity, detail: "descriptor canonical basis version mismatch".to_string(), } )` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `durability_recovery_emits_authority_continuity_diagnostic_before_execution`

Source: `crates/worth-relational/src/tests/durability/contracts/descriptor_continuity.rs:230`.

- **UNCLASSIFIED** 256: `assert_eq!( diagnostic_field(authority_continuity_entry, "verification_rejected"), &RelationalDiagnosticValue::Bool(false) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 260: `assert_eq!( diagnostic_field(authority_continuity_entry, "verification_layer"), &RelationalDiagnosticValue::string("DigestParity") )` - semantic boundary still requires direct review; no move authorized.

#### `durability_certification_recovery_authority_continuity_is_explained_and_counted`

Source: `crates/worth-relational/src/tests/durability/contracts/descriptor_continuity.rs:267`.

- **UNCLASSIFIED** 294: `assert_eq!( diagnostic_field(authority_continuity_entry, "verification_mode"), &RelationalDiagnosticValue::string("NormalRecoveryVerification") )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 298: `assert_eq!( diagnostic_field(authority_continuity_entry, "verification_rejected"), &RelationalDiagnosticValue::Bool(false) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 302: `assert_eq!( diagnostic_field(authority_continuity_entry, "verification_layer"), &RelationalDiagnosticValue::string("DigestParity") )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 306: `assert_eq!( diagnostic_field(authority_continuity_entry, "descriptor_version_parity"), &verified_at_digest_parity_value() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 310: `assert_eq!( diagnostic_field(authority_continuity_entry, "schema_transition_parity"), &verified_at_digest_parity_value() )` - semantic boundary still requires direct review; no move authorized.
- **C2** 315: `assert!(counters.replay_digest_parity_checks >= 1)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 316: `assert_eq!(counters.descriptor_version_mismatches_encountered, 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.

### `tests/durability/contracts/fork_provenance_checkpoint.rs`

#### `recovery_rejects_one_sided_fork_provenance_in_checkpoint`

Source: `crates/worth-relational/src/tests/durability/contracts/fork_provenance_checkpoint.rs:125`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/durability/contracts/fork_provenance_checkpoint.rs:77 assert_recovery_rejects_fork_checkpoint`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `recovery_rejects_mismatched_fork_provenance_identity_in_checkpoint`

Source: `crates/worth-relational/src/tests/durability/contracts/fork_provenance_checkpoint.rs:140`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/durability/contracts/fork_provenance_checkpoint.rs:77 assert_recovery_rejects_fork_checkpoint`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `recovery_rejects_one_sided_fork_provenance_in_tail`

Source: `crates/worth-relational/src/tests/durability/contracts/fork_provenance_checkpoint.rs:151`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/durability/contracts/fork_provenance_checkpoint.rs:77 assert_recovery_rejects_fork_checkpoint`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

### `tests/durability/contracts/fork_tail_after_source_moved.rs`

#### `a_fork_commits_after_a_checkpoint_taken_once_its_source_moved_on`

Source: `crates/worth-relational/src/tests/durability/contracts/fork_tail_after_source_moved.rs:10`.

- **HAPPY** 28: `assert_eq!(outcome.recovered_commits, 4)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 29: `assert!(recovered.branch_identity(&storm).is_ok())` - reviewed public owner result over explicit durable provider.
- **HAPPY** 30: `assert_eq!( recovered.history().latest_commit(), runtime.history().latest_commit() )` - reviewed public owner result over explicit durable provider.

### `tests/durability/contracts/global_generation_reclamation.rs`

#### `repeated_global_same_basis_builds_reclaim_duplicates_but_keep_pinned_root`

Source: `crates/worth-relational/src/tests/durability/contracts/global_generation_reclamation.rs:5`.

- **UNCLASSIFIED** 30: `assert!(built.failed_indexes.is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 57: `assert!(next_built.failed_indexes.is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 60: `assert_eq!( runtime .run_index_generation_reclamation_after_publication(&next) .unwrap(), Some(16) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 66: `assert_eq!(runtime.run_index_generation_reclamation_pass(), Some(0))` - semantic boundary still requires direct review; no move authorized.
- **C2** 67: `assert!(runtime.indexes.generation(original).is_none())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 68: `assert!(duplicates[..duplicates.len() - 1] .iter() .all(|id| runtime.indexes.generation(*id).is_none()))` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 71: `assert!(runtime.indexes.generation(selected_pinned).is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 72: `assert!(runtime.indexes.generation(current).is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 80: `assert_eq!(saved.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 81: `assert_eq!(saved[0].generation_id, current)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 83: `assert_eq!( runtime .run_index_generation_reclamation_after_publication(&next) .unwrap(), Some(1) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 89: `assert_eq!(runtime.run_index_generation_reclamation_pass(), Some(0))` - semantic boundary still requires direct review; no move authorized.
- **C2** 90: `assert!(runtime.indexes.generation(selected_pinned).is_none())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 91: `assert!(runtime.indexes.generation(current).is_some())` - actual residency, identity, pin, allocation, dictionary or cost observation.

### `tests/durability/contracts/lineage_allocator_denials.rs`

#### `tail_created_lineage_allocator_exhaustion_denies_before_replay`

Source: `crates/worth-relational/src/tests/durability/contracts/lineage_allocator_denials.rs:5`.

- C4 action 20: `.envelope_mut_for_test(execution)`.
- C4 action 21: `.published_lineage_mut_for_test()`.
- C4 action 25: `.envelope_mut_for_test(execution)`.
- **C4** 35: `assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 36: `assert!(error.detail.contains("tail lineage id exhausted"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 37: `assert_eq!(recovered.history().immutable_commit_count(), 0)` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `live_lineage_allocator_exhaustion_denies_before_public_effects`

Source: `crates/worth-relational/src/tests/durability/contracts/lineage_allocator_denials.rs:41`.

- **UNCLASSIFIED** 68: `assert!(format!("{error:?}").contains("lineage id allocator exhausted"))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 69: `assert_eq!( runtime.lineage.identity_frontiers(), (u64::MAX - 1, event_frontier) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 73: `assert_eq!( runtime .history() .branch_head(&BranchId("main".to_owned())) .expect("baseline head"), baseline.commit )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 80: `assert_eq!( runtime .storage_access() .storage_stats(execution) .unwrap() .live_entities, baseline_entities )` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/lineage_artifact_recovery.rs`

#### `checkpoint_duplicate_lineage_event_ids_deny`

Source: `crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs:5`.

- C4 action 25: `.map(|commit| commit.envelope_mut_for_test(execution))`.
- C4 action 28: `.published_lineage_mut_for_test()`.
- C4 action 37: `.envelope_mut_for_test(execution)`.
- **C4** 42: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 43: `assert!(error.detail.contains("reused"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `tail_duplicate_lineage_event_ids_deny`

Source: `crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs:47`.

- C4 action 65: `.map(|commit| commit.envelope_mut_for_test(execution))`.
- C4 action 68: `.published_lineage_mut_for_test()`.
- C4 action 77: `.envelope_mut_for_test(execution)`.
- **C4** 82: `assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 83: `assert!(error.detail.contains("reused"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `checkpoint_and_tail_lineage_event_id_collision_denies`

Source: `crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs:87`.

- C4 action 112: `.envelope_mut_for_test(execution)`.
- C4 action 113: `.published_lineage_mut_for_test()`.
- C4 action 117: `.envelope_mut_for_test(execution)`.
- **C4** 121: `assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 122: `assert!(error.detail.contains("reused"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `tail_lineage_commit_and_branch_cross_splices_deny`

Source: `crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs:126`.

- C4 action 141: `.envelope_mut_for_test(execution)`.
- C4 action 142: `.published_lineage_mut_for_test()`.
- C4 action 149: `.envelope_mut_for_test(execution)`.
- **C4** 153: `assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 154: `assert!(error.detail.contains("cross-spliced"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `checkpoint_lineage_commit_cross_splice_denies`

Source: `crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs:158`.

- C4 action 172: `.envelope_mut_for_test(execution)`.
- C4 action 173: `.published_lineage_mut_for_test()`.
- C4 action 179: `.envelope_mut_for_test(execution)`.
- **C4** 183: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 184: `assert!(error.detail.contains("cross-spliced"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `replay_and_durable_lineage_payload_conflict_denies`

Source: `crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs:188`.

- C4 action 212: `.map(|commit| commit.envelope_mut_for_test(execution))`.
- C4 action 215: `.published_lineage_mut_for_test()`.
- C4 action 223: `.map(|commit| commit.envelope_mut_for_test(execution))`.
- **C4** 229: `assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 230: `assert!(error.detail.contains("create lineage id"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `checkpoint_lineage_node_deletion_denies_before_installation`

Source: `crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs:234`.

- **UNCLASSIFIED** 255: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 256: `assert!(error.detail.contains("lineage counters"))` - semantic boundary still requires direct review; no move authorized.

#### `checkpoint_lineage_node_remap_denies_before_installation`

Source: `crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs:260`.

- **UNCLASSIFIED** 277: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 278: `assert!(error.detail.contains("does not name its exact entity"))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 289: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 290: `assert!(error.detail.contains("does not name its exact entity"))` - semantic boundary still requires direct review; no move authorized.

#### `checkpoint_lineage_node_swap_denies_before_installation`

Source: `crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs:294`.

- **UNCLASSIFIED** 328: `assert_eq!(nodes.len(), 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 334: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 335: `assert!(error.detail.contains("does not name its exact entity"))` - semantic boundary still requires direct review; no move authorized.

#### `checkpoint_duplicate_lineage_entity_mapping_denies_before_installation`

Source: `crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs:339`.

- **UNCLASSIFIED** 373: `assert_eq!(nodes.len(), 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 377: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 378: `assert!(error.detail.contains("does not name its exact entity"))` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/lineage_artifact_recovery/lineage_regression.rs`

#### `tail_created_lineage_regression_below_checkpoint_high_water_denies`

Source: `crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery/lineage_regression.rs:5`.

- C4 action 20: `.envelope_mut_for_test(execution)`.
- C4 action 21: `.published_lineage_mut_for_test()`.
- C4 action 25: `.envelope_mut_for_test(execution)`.
- **C4** 29: `assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 30: `assert!(error .detail .contains("create lineage id 0 does not advance"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `tail_lineage_event_regression_below_checkpoint_high_water_denies`

Source: `crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery/lineage_regression.rs:36`.

- C4 action 51: `.envelope_mut_for_test(execution)`.
- C4 action 52: `.published_lineage_mut_for_test()`.
- C4 action 56: `.envelope_mut_for_test(execution)`.
- **C4** 60: `assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 61: `assert!(error.detail.contains("lineage event id 0 does not advance"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `tests/durability/contracts/lineage_event_allocator_recovery.rs`

#### `tail_lineage_event_allocator_exhaustion_denies_before_replay`

Source: `crates/worth-relational/src/tests/durability/contracts/lineage_event_allocator_recovery.rs:6`.

- C4 action 24: `.map(|commit| commit.envelope_mut_for_test(execution))`.
- C4 action 37: `.map(|commit| commit.envelope_mut_for_test(execution))`.
- C4 action 40: `.published_lineage_mut_for_test()`.
- C4 action 48: `.map(|commit| commit.envelope_mut_for_test(execution))`.
- **C4** 60: `assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 61: `assert!(error.detail.contains("tail lineage event id exhausted"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 62: `assert_eq!(recovered.history().immutable_commit_count(), 0)` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `unresolved_durable_publication_blocks_descendants_and_recovers_last_checkpoint`

Source: `crates/worth-relational/src/tests/durability/contracts/lineage_event_allocator_recovery.rs:66`.

- C4 action 77: `medium.script_next_publication(ScriptedPublication::IndeterminateWithoutEffect);`.
- **C4** 98: `assert_eq!(unresolved_position, PatchStreamPosition(2))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 99: `assert_eq!(runtime.lineage.node_count(), published_node_count)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 100: `assert_eq!(runtime.lineage.event_count(), published_event_count)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 101: `assert_eq!( runtime.history().branch_head(&BranchId("main".to_owned())), Some(checkpointed.commit.clone()) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 116: `assert_eq!(child_refusal.durability_unresolved_fate(), Some(&fate))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 119: `assert_eq!(recovery.recovered_commits, 1)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 120: `assert_eq!(recovery.coverage.replayed_tail_commits, 0)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 121: `assert_eq!( recovered .history() .branch_head(&BranchId("main".to_owned())), Some(checkpointed.commit.clone()) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 127: `assert_eq!( recovered.lineage.identity_frontiers(), frontiers_at_checkpoint )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 132: `assert_eq!(resumed.patch_position(), unresolved_position)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 133: `assert_eq!(resumed.commit.parents, vec![checkpointed.commit.commit_id])` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `multi_event_reservation_exhaustion_denies_before_public_effects`

Source: `crates/worth-relational/src/tests/durability/contracts/lineage_event_allocator_recovery.rs:138`.

- **UNCLASSIFIED** 172: `assert!(format!("{error:?}").contains("lineage event id allocator exhausted"))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 173: `assert_eq!( runtime.lineage.identity_frontiers(), (lineage_frontier, u64::MAX - 1) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 177: `assert_eq!( runtime .history() .branch_head(&BranchId("main".to_owned())) .expect("baseline head"), baseline_head )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 184: `assert_eq!( runtime .storage_access() .storage_stats(execution) .unwrap() .live_entities, baseline_entities )` - semantic boundary still requires direct review; no move authorized.

#### `branch_local_sparse_slot_create_publishes_and_recovers_exact_lineage`

Source: `crates/worth-relational/src/tests/durability/contracts/lineage_event_allocator_recovery.rs:195`.

- **UNCLASSIFIED** 210: `assert!(feature_entity.local_slot.0 > 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 226: `assert_eq!(event_targets, vec![lineage_id])` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 239: `assert_eq!( recovered .lineage .nodes_snapshot() .into_iter() .find(|node| node.entity_id() == feature_entity) .expect("recovered sparse lineage") .lineage_id(), lineage_id )` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/managed_field_text.rs`

#### `native_field_text_checkpoint_then_tail_recovers_exact_json_after_original_owners_drop`

Source: `crates/worth-relational/src/tests/durability/contracts/managed_field_text.rs:84`.

- **UNCLASSIFIED** 115: `assert_eq!(plan.checkpoint.as_ref().unwrap().envelopes.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 116: `assert_eq!(plan.tail_log.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 121: `assert_eq!(outcome.coverage.replayed_tail_commits, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 124: `assert_eq!(pinned.symbols().resolve(key), Some("native-managed-text"))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 125: `assert_eq!( pinned .symbols() .retained_text(key) .unwrap() .charged_payload_bytes(), None, "ordinary client-key custody stays System" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 138: `assert_eq!(symbols.resolve(key), Some("native-managed-text"))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 140: `assert_eq!( pinned .symbols() .symbol(JSON) .and_then(|symbol| pinned.symbols().resolve(symbol)), Some(JSON) )` - semantic boundary still requires direct review; no move authorized.

#### `native_field_text_checkpoint_readmission_denial_keeps_bytes_and_target_root_intact`

Source: `crates/worth-relational/src/tests/durability/contracts/managed_field_text.rs:150`.

- **UNCLASSIFIED** 187: `assert_eq!( error, RelationalRecoveryStop::Request( crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted { requested: FIRST_SCOPE_BYTES, admitted: 1, ancestor: 0, } ) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 197: `assert_eq!(native.bytes(), before)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 198: `assert_eq!(target.observe_branch(&identity).unwrap().0, original)` - semantic boundary still requires direct review; no move authorized.

#### `native_field_text_configuration_requires_current_explicit_readmission`

Source: `crates/worth-relational/src/tests/durability/contracts/managed_field_text.rs:212`.

- **UNCLASSIFIED** 223: `assert!(rmp_serde::from_slice::<crate::symbols::data::SymbolTableSnapshot>(&wire).is_err())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 232: `assert_eq!(managed.1.charged_payload_bytes(), Some(JSON.len() as u64))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 238: `assert_eq!(symbols.resolve(symbols.symbol(JSON).unwrap()), Some(JSON))` - semantic boundary still requires direct review; no move authorized.

#### `native_field_text_prior_checkpoint_format_refuses_before_managed_text`

Source: `crates/worth-relational/src/tests/durability/contracts/managed_field_text.rs:243`.

- **UNCLASSIFIED** 260: `assert!( matches!(error,RelationalRecoveryStop::Durability(ref error) if error.class == RecoveryFailureClass::UnsupportedLegacySemantics) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 263: `assert!( crate::durability::log::native_checkpoint_codec::decode_checkpoint( execution, native.bytes() ) .is_ok() )` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/managed_field_text/missing_dictionary.rs`

#### `native_field_text_missing_tail_dictionary_refuses_without_target_or_source_effect`

Source: `crates/worth-relational/src/tests/durability/contracts/managed_field_text/missing_dictionary.rs:6`.

- C4 action 18: `medium.rewrite_artifact_envelope(&execution, tail_position, |tail| {`.
- **C4** 19: `assert!( crate::symbols::data::StringInterner::default() .commit_texts(&tail.merged_plan, &tail.patch, []) .is_err(), "producer must refuse unresolved references, never omit their rows" )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 34: `assert!(required.contains(&managed))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 38: `assert_eq!( crate::symbols::data::required_commit_symbols(&tail.merged_plan, &tail.patch), required )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 58: `assert!( matches!(error, RelationalRecoveryStop::Durability(ref error) if error.class == RecoveryFailureClass::CorruptCanonicalArtifact) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 62: `assert_eq!(target.observe_branch(&identity).unwrap().0, before)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 63: `assert_eq!(target.services.symbols.snapshot(), symbols)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 64: `assert_eq!(medium.artifact_bytes(tail_position).unwrap(), malformed)` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `tests/durability/contracts/merge_replay_continuity.rs`

#### `durability_contract_recovery_preserves_merge_parent_order`

Source: `crates/worth-relational/src/tests/durability/contracts/merge_replay_continuity.rs:5`.

- **UNCLASSIFIED** 48: `assert_eq!( recovered_merge.commit.parents, vec![main.commit.commit_id, feature.commit.commit_id] )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 52: `assert_eq!( recovered_merge.merge_parent_branches, vec![BranchId("feature".to_string())] )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 56: `assert_eq!( recovered_merge.merge_base_commits, vec![main.commit.commit_id] )` - semantic boundary still requires direct review; no move authorized.

#### `durability_contract_replays_empty_intent_merge_currentness_once`

Source: `crates/worth-relational/src/tests/durability/contracts/merge_replay_continuity.rs:63`.

- **UNCLASSIFIED** 111: `assert_eq!( expected.observation().generation(), actual.observation().generation(), "recovery must apply the merge currentness transition exactly once" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 116: `assert_eq!(expected.truth_version(), actual.truth_version())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 117: `assert_eq!( expected_target.selected_commit_id(), actual_target.selected_commit_id() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 121: `assert_eq!(expected_target.version_id(), actual_target.version_id())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 122: `assert_eq!( merge.commit.commit_id.0, expected_target.selected_commit_id() )` - semantic boundary still requires direct review; no move authorized.

#### `durability_contract_replays_merge_from_typed_authority_when_diagnostics_are_absent`

Source: `crates/worth-relational/src/tests/durability/contracts/merge_replay_continuity.rs:129`.

- C4 action 156: `crate::tests::support::attached_test_medium(&runtime).rewrite_commit_envelope(`.
- **C4** 160: `assert!(merge_entry.merge_execution_authority.is_some())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 169: `assert!(populated_nested_bytes > omitted_nested_bytes)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 170: `assert!( merge_entry.diagnostics_summary.entries.is_empty(), "diagnostics never reach the durable artifact" )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 205: `assert!( source.envelope().diagnostics_summary.entries.is_empty(), "the actual recovery source supplies no diagnostics" )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 214: `assert!(recovered_merge.merge_execution_authority.is_some())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 215: `assert_eq!( recovered .history() .branch_head(&BranchId("main".to_string())) .expect("main head after recovery") .commit_id, merge.commit.commit.commit_id )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/support/truth_comparison.rs:60 assert_durable_commit_meaning_eq`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `durability_contract_rejects_parent_order_tampering_at_source_admission`

Source: `crates/worth-relational/src/tests/durability/contracts/merge_replay_continuity.rs:226`.

- C4 action 246: `crate::tests::support::attached_test_medium(&runtime).rewrite_commit_envelope(`.
- **C4** 250: `assert_eq!( merge_entry.commit.parents, vec![main.commit.commit_id, feature.commit.commit_id] )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 266: `assert_eq!(error.class, RecoveryFailureClass::CorruptCanonicalArtifact)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 267: `assert_eq!(error.history_drift_class, None)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 268: `assert!(error .detail .contains("PositionAdmission(SemanticResult(ReceiptMismatch))"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `durability_contract_does_not_reconstruct_missing_merge_authority_from_diagnostics`

Source: `crates/worth-relational/src/tests/durability/contracts/merge_replay_continuity.rs:274`.

- C4 action 301: `crate::tests::support::attached_test_medium(&runtime).rewrite_commit_envelope(`.
- **C4** 305: `assert!(merge_entry.merge_execution_authority.is_some())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 323: `assert_eq!(error.class, RecoveryFailureClass::ReplayFailure)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 324: `assert_eq!(error.history_drift_class, None)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 325: `assert!(error .detail .contains("failed to reconstruct merge execution summary"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `tests/durability/contracts/native_checkpoint.rs`

#### `native_checkpoint_round_trip_restores_a_live_editable_world`

Source: `crates/worth-relational/src/tests/durability/contracts/native_checkpoint.rs:56`.

- **UNCLASSIFIED** 78: `assert_ne!(recovered.runtime_instance_id(), pre_recovery_identity)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 79: `assert_eq!(outcome.latest_commit, Some(committed.commit.clone()))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 80: `assert_eq!( recovered .history() .branch_head(&BranchId("main".to_owned())), Some(committed.commit.clone()) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 87: `assert_eq!( recovered .history() .branch_head(&BranchId("main".to_owned())), Some(later.commit.clone()) )` - semantic boundary still requires direct review; no move authorized.

#### `recovery_authority_only_admits_a_basis_from_the_recovered_image`

Source: `crates/worth-relational/src/tests/durability/contracts/native_checkpoint.rs:96`.

- **UNCLASSIFIED** 122: `assert!(exact_authority.admit_basis(exact_basis).is_ok())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 137: `assert!(moved_authority.admit_basis(moved_basis).is_err())` - semantic boundary still requires direct review; no move authorized.

#### `native_checkpoint_round_trip_restores_current_relation_adjacency`

Source: `crates/worth-relational/src/tests/durability/contracts/native_checkpoint.rs:141`.

- **UNCLASSIFIED** 213: `assert_eq!(rows.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 214: `assert_eq!(rows[0].relation_id, relation)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 215: `assert_eq!(rows[0].target, target)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 230: `assert_eq!(projected.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 231: `assert_eq!(projected[0].relation_id, relation)` - semantic boundary still requires direct review; no move authorized.

#### `repeated_native_captures_do_not_retain_checkpoint_images`

Source: `crates/worth-relational/src/tests/durability/contracts/native_checkpoint.rs:235`.

- **UNCLASSIFIED** 261: `assert_eq!(runtime.durability.checkpoints().len(), retained_before)` - semantic boundary still requires direct review; no move authorized.

#### `corrupt_native_checkpoint_is_denied_without_publishing_recovered_state`

Source: `crates/worth-relational/src/tests/durability/contracts/native_checkpoint.rs:272`.

- **UNCLASSIFIED** 288: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 289: `assert_eq!(recovered.history().immutable_commit_count(), 0)` - semantic boundary still requires direct review; no move authorized.

#### `native_checkpoint_rejects_a_foreign_runtime_name_without_publishing_state`

Source: `crates/worth-relational/src/tests/durability/contracts/native_checkpoint.rs:293`.

- **UNCLASSIFIED** 318: `assert_eq!(error.class, RecoveryFailureClass::RuntimeNameMismatch)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 319: `assert_eq!(recovered.history().immutable_commit_count(), 0)` - semantic boundary still requires direct review; no move authorized.

#### `native_checkpoint_preserves_unsealed_builder_invariant_authority`

Source: `crates/worth-relational/src/tests/durability/contracts/native_checkpoint.rs:323`.

- **UNCLASSIFIED** 344: `assert_eq!( recovered .schema_contract_runtime .custom_invariant_registries .iter() .next() .unwrap() .rule_id() .as_str(), "native.checkpoint.rule" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 355: `assert!( !recovered .schema_contract_runtime .initial_custom_invariants_sealed )` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/native_checkpoint/allocation.rs`

#### `native_checkpoint_retains_all_admitted_capture_owners_and_restores_editable_truth`

Source: `crates/worth-relational/src/tests/durability/contracts/native_checkpoint/allocation.rs:25`.

- **UNCLASSIFIED** 50: `assert!( matches!(error, Denial::Request(crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted { requested, admitted: 0, ancestor: 0 }) if requested > 0), "{error:?}" )` - semantic boundary still requires direct review; no move authorized.
- **C2** 54: `assert_eq!(parent.charged_memory_bytes(), 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 55: `assert_eq!( runtime.history().branch_head(&BranchId("main".into())), head )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 74: `assert_eq!( error.classify(), ExecutionAllocationDenialClass::Request(worth_execution::WorkCeilingDenial::Stopped( worth_execution::MapKernelStop::Cancelled )) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 80: `assert_eq!(error.requested_payload_bytes(), None)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 94: `assert_eq!(captured.bytes(), system.bytes())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 95: `assert_eq!(captured.captured_sections(), system.captured_sections())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 97: `assert_eq!(clone.bytes().as_ptr(), captured.bytes().as_ptr())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 102: `assert!( retained_bytes >= quote, "the checkpoint retains its frame and all original capture owners" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 107: `assert_eq!( parent.reserve_memory(1).unwrap_err(), MemoryLimitDenial::Capacity { requested: 1, admitted: 0, level: MemoryLimitLevel::Policy { ancestor: 0 }, } )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 122: `assert_eq!(restored.latest_commit.as_ref(), Some(&committed.commit))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 123: `assert_ne!(recovered.runtime_instance_id(), prior_owner)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 125: `assert_eq!( recovered .history() .branch_head(&BranchId("main".into())) .as_ref(), Some(&later.commit) )` - semantic boundary still requires direct review; no move authorized.
- **C2** 136: `assert_eq!(parent.charged_memory_bytes(), FUNDED - retained_bytes)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 143: `assert_eq!(parent.charged_memory_bytes(), 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.

### `tests/durability/contracts/native_checkpoint/speculative_symbols.rs`

#### `unpublished_client_keys_leave_checkpoint_dictionary_equal_and_published_keys_restore`

Source: `crates/worth-relational/src/tests/durability/contracts/native_checkpoint/speculative_symbols.rs:4`.

- **UNCLASSIFIED** 30: `assert!(!runtime .services .symbols .with_read(|symbols| symbols.contains("unpublished-client-key")))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 48: `assert!(after.symbol_identity_frontier > before.symbol_identity_frontier)` - semantic boundary still requires direct review; no move authorized.
- **C2** 49: `assert_eq!(before.symbol_table, after.symbol_table)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 50: `assert_eq!(before.envelopes, after.envelopes)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 51: `assert_eq!(before.partition_images, after.partition_images)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 52: `assert_eq!(before.aspect_contracts, after.aspect_contracts)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 53: `assert_eq!( before.branch_root_schema_images, after.branch_root_schema_images )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 57: `assert_eq!(before.index_definitions, after.index_definitions)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 58: `assert_eq!(before.lineage, after.lineage)` - semantic boundary still requires direct review; no move authorized.
- **C2** 63: `assert_eq!( directory.manifest.record_identity, checkpoint.record_identity )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 67: `assert_eq!( directory.manifest.symbol_identity_frontier, checkpoint.symbol_identity_frontier )` - semantic boundary still requires direct review; no move authorized.
- **C2** 73: `assert_eq!( before.record_identity.schema_version, after.record_identity.schema_version )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 93: `assert_eq!(before, after)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 103: `assert_eq!(restored.latest_commit, Some(committed.commit.clone()))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 114: `assert!(recovered .services .symbols .with_read(|symbols| symbols.contains("published-after-abandonment")))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 118: `assert!(!recovered .services .symbols .with_read(|symbols| symbols.contains("unpublished-client-key")))` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/native_checkpoint_restore_benchmark.rs`

#### `synthetic_native_restore_and_image_digest_benchmark` - ignored

Source: `crates/worth-relational/src/tests/durability/contracts/native_checkpoint_restore_benchmark.rs:13`.

- **UNCLASSIFIED** 71: `assert_eq!(streamed, buffered)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 88: `assert_eq!( recovered .read_truth() .read_snapshot(&snapshot, execution) .expect("read request remains live") .unwrap() .entities() .len(), RECORDS )` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/record_allocation_recovery.rs`

#### `tail_replay_consumes_canonical_reused_record_allocation`

Source: `crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs:5`.

- **UNCLASSIFIED** 14: `assert!(runtime .visibility_authority() .release_snapshot(&created.snapshot) .is_ok())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 19: `assert!(runtime .visibility_authority() .release_snapshot(&deleted.snapshot) .is_ok())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 23: `assert!( runtime .retention() .run_pass(execution) .unwrap() .entity_reclaimed <= 1 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 31: `assert_eq!( runtime .storage_access() .storage_stats(execution) .unwrap() .reusable_entity_slots, 1 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 57: `assert_eq!(replacement.local_slot, original.local_slot)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 58: `assert!(replacement.generation.0 > original.generation.0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 63: `assert_eq!( read_entity_name(current.get_entity(replacement).unwrap()), Some("allocation-after".into()) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 71: `assert_eq!( read_entity_name(historical.get_entity(original).unwrap()), Some("allocation-before".into()) )` - semantic boundary still requires direct review; no move authorized.

#### `checkpoint_restores_burned_append_frontier_beyond_all_retained_roots`

Source: `crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs:78`.

- **UNCLASSIFIED** 91: `assert_eq!(reservation.slot as u64, original.local_slot.0 + 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 109: `assert_eq!(after_recovery.local_slot.0, original.local_slot.0 + 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 110: `assert_eq!(after_recovery.generation.0, 1)` - semantic boundary still requires direct review; no move authorized.

#### `recovery_releases_orphaned_append_reservation_without_reusing_its_gap`

Source: `crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs:114`.

- **UNCLASSIFIED** 127: `assert_eq!(reservation.slot as u64, original.local_slot.0 + 1)` - semantic boundary still requires direct review; no move authorized.
- **C2** 144: `assert!(recovered.record_identity.pending_snapshot().is_empty())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 146: `assert_eq!(after_recovery.local_slot.0, original.local_slot.0 + 2)` - semantic boundary still requires direct review; no move authorized.

#### `recovery_returns_orphaned_reclaimed_reservation_to_reuse`

Source: `crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs:150`.

- **UNCLASSIFIED** 159: `assert!(runtime .visibility_authority() .release_snapshot(&created.snapshot) .is_ok())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 164: `assert!(runtime .visibility_authority() .release_snapshot(&deleted.snapshot) .is_ok())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 176: `assert_eq!(reservation.slot as u64, original.local_slot.0)` - semantic boundary still requires direct review; no move authorized.
- **C2** 195: `assert!(recovered.record_identity.pending_snapshot().is_empty())` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 197: `assert_eq!(replacement.local_slot, original.local_slot)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 198: `assert!(replacement.generation.0 > original.generation.0)` - semantic boundary still requires direct review; no move authorized.

#### `tail_replay_rejects_canonical_allocation_targeting_a_live_slot`

Source: `crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs:202`.

- C4 action 220: `.envelope_mut_for_test(execution);`.
- **C4** 244: `assert_eq!(error.class, RecoveryFailureClass::ReplayFailure)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 245: `assert!(error.detail.contains("selects unavailable Entity"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 246: `assert_eq!(recovered.history().immutable_commit_count(), 0)` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `tail_replay_rejects_missing_canonical_allocation_evidence`

Source: `crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs:250`.

- C4 action 267: `.envelope_mut_for_test(execution)`.
- **C4** 278: `assert_eq!(error.class, RecoveryFailureClass::ReplayFailure)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 279: `assert!(error.detail.contains("missing ordinal 0"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 280: `assert_eq!(recovered.history().immutable_commit_count(), 0)` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `tail_replay_gap_admission_cannot_replace_a_different_canonical_effect`

Source: `crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs:284`.

- C4 action 306: `.envelope_mut_for_test(execution)`.
- **C4** 323: `assert_eq!(error.class, RecoveryFailureClass::ReplayFailure)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 324: `assert!(error.detail.contains("canonical effect"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 325: `assert_eq!(recovered.history().immutable_commit_count(), 0)` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `metadata_only_merge_rejects_unconsumable_allocation_evidence`

Source: `crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs:329`.

- C4 action 365: `.envelope_mut_for_test(execution);`.
- **C4** 379: `assert_eq!(error.class, RecoveryFailureClass::ReplayFailure)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 380: `assert!(error.detail.contains("without a mutation replay path"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 381: `assert_eq!(recovered.history().immutable_commit_count(), 0)` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `tests/durability/contracts/recorded_component_checkpoint.rs`

#### `recorded_component_checkpoint_readmits_cold_canonical_predecessor_and_successor`

Source: `crates/worth-relational/src/tests/durability/contracts/recorded_component_checkpoint.rs:239`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.

#### `recorded_component_checkpoint_readmits_original_empty_predecessor`

Source: `crates/worth-relational/src/tests/durability/contracts/recorded_component_checkpoint.rs:243`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.

#### `recorded_component_checkpoint_omits_released_components_and_plain_observations`

Source: `crates/worth-relational/src/tests/durability/contracts/recorded_component_checkpoint.rs:247`.

- **UNCLASSIFIED** 261: `assert_eq!(directory.manifest.roots.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 262: `assert!(directory.manifest.retained_empty_components.is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 263: `assert!(!directory.manifest.roots.iter().any(|root| root .recorded_components .contains(original.before.descriptor()) || root .recorded_components .contains(original.after.descriptor())))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 288: `assert_eq!( decoded.checkpoint.directory.unwrap().manifest.roots.len(), 1 )` - semantic boundary still requires direct review; no move authorized.

#### `recorded_component_checkpoint_refuses_chk15_before_source_admission`

Source: `crates/worth-relational/src/tests/durability/contracts/recorded_component_checkpoint.rs:295`.

- **UNCLASSIFIED** 306: `assert_eq!( expect_durability_failure(stop).class, crate::durability::data::RecoveryFailureClass::UnsupportedLegacySemantics )` - semantic boundary still requires direct review; no move authorized.

#### `recorded_original_retirement_preserves_both_leases_across_busy_and_stop`

Source: `crates/worth-relational/src/tests/durability/contracts/recorded_component_checkpoint.rs:313`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.

### `tests/durability/contracts/rejected_and_parent_closure.rs`

#### `durability_contract_recovery_ignores_rejected_relation_integrity_attempts`

Source: `crates/worth-relational/src/tests/durability/contracts/rejected_and_parent_closure.rs:5`.

- **UNCLASSIFIED** 69: `assert_eq!(error.code(), DiagnosticCode::RelationCardinalityViolation)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 93: `assert_eq!(runtime.history().latest_commit(), latest_commit_before)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 94: `assert_eq!( runtime.publication().latest_patch().unwrap().position, latest_patch_before )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 98: `assert_eq!( relation_aspect_history_digest_on_branch( execution, &runtime, &BranchId("main".to_string()), relation, None, ), main_digest_before )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 125: `assert_eq!(outcome.latest_commit, latest_commit_before.clone())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 126: `assert_eq!( relation_aspect_history_digest_on_branch( execution, &recovered, &BranchId("main".to_string()), relation, None, ), main_digest_before )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 136: `assert!(recovered .replay() .canonical_commit_envelope(execution, latest_commit_before.unwrap().commit_id) .unwrap() .is_some())` - semantic boundary still requires direct review; no move authorized.

#### `durability_contract_failure_missing_authoritative_parent_closure_is_explicit`

Source: `crates/worth-relational/src/tests/durability/contracts/rejected_and_parent_closure.rs:144`.

- **UNCLASSIFIED** 181: `assert_eq!(parent.commit.commit_id.0, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 182: `assert_eq!( error.class, RecoveryFailureClass::MissingAuthoritativeParentClosure )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 186: `assert_eq!( error.history_drift_class, Some(crate::facade::history::HistoryDriftClass::CanonicalHistoryDrift) )` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/retention_inspection_store.rs`

#### `durability_contract_recovery_rebuilds_branch_head_root_obligations`

Source: `crates/worth-relational/src/tests/durability/contracts/retention_inspection_store.rs:5`.

- **C2** 39: `assert_eq!(retention.active_snapshot_count, 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 40: `assert_eq!(retention.branch_pinned_entities, 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 41: `assert_eq!(retention.branch_pinned_relations, 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 51: `assert!(feature_truth.get_entity(source_entity).is_some())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 52: `assert!(feature_truth .relations() .iter() .any(|relation| { relation.source == source_entity && relation.target == target_entity }))` - semantic boundary still requires direct review; no move authorized.

#### `durability_contract_recovery_preserves_inspection_truth_bundle`

Source: `crates/worth-relational/src/tests/durability/contracts/retention_inspection_store.rs:59`.

- **HAPPY** 124: `assert_eq!(expected, actual)` - reviewed public owner result over explicit durable provider.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/support.rs:239 capture_inspection_truth_bundle`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `durability_contract_branch_heads_do_not_mutate_legacy_record_pins`

Source: `crates/worth-relational/src/tests/durability/contracts/retention_inspection_store.rs:128`.

- **C2** 136: `assert_eq!( inspection .inspect_record_retention(execution, RecordRef::Entity(entity)) .unwrap() .expect("entity retention after create") .pins .branch_pins, 0 )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 155: `assert_eq!( inspection .inspect_record_retention(execution, RecordRef::Entity(entity)) .unwrap() .expect("entity retention after branch create") .pins .branch_pins, 0 )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 167: `assert_eq!( inspection .inspect_record_retention(execution, RecordRef::Entity(entity)) .unwrap() .expect("entity retention after main update") .pins .branch_pins, 0 )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 185: `assert_eq!( inspection .inspect_record_retention(execution, RecordRef::Entity(entity)) .unwrap() .expect("entity retention after feature update") .pins .branch_pins, 0 )` - actual residency, identity, pin, allocation, dictionary or cost observation.

### `tests/durability/contracts/schema_transition_continuity.rs`

#### `durable_recovery_and_schema_mismatch_test`

Source: `crates/worth-relational/src/tests/durability/contracts/schema_transition_continuity.rs:5`.

- C4 action 12: `runtime.set_schema_registry_for_test(`.
- **C4** 92: `assert_eq!( live_schema_transition, recovered_envelope.schema_transition.clone() )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 96: `assert_eq!( live_schema_continuation_descriptor, recovered_envelope.schema_continuation_descriptor.clone() )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 100: `assert_eq!( live_schema_reconciliation_descriptor, recovered_envelope.schema_reconciliation_descriptor.clone() )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 104: `assert_eq!( live_recovery_authority_continuity, plan.authority_continuity.clone() )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 108: `assert!(recovered_diagnostics .by_scope(DiagnosticsScope::History) .into_iter() .flat_map(|artifact| artifact.entries.iter()) .any(|entry| { entry.code == DiagnosticCode::DurableRecoveryAuthorityContinuityEvaluated && diagnostic_field_optional(entry, "verification_layer") == Some(&RelationalDiagnosticValue::string("DigestParity")) }))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C2** 118: `assert!(recovered_counters.replay_digest_parity_checks >= 1)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C4** 139: `assert_eq!(error.class, RecoveryFailureClass::SchemaMismatch)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 140: `assert!(matches!( error.authority_continuity_mismatch, Some(RecoveryAuthorityContinuityMismatch::SchemaRegistryShape { .. }) ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 144: `assert!(matches!( error.authority_continuity_mismatch, Some(RecoveryAuthorityContinuityMismatch::SchemaRegistryShape { expected_primary_schema_version: SchemaVersionId(2), .. }) ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 151: `assert_eq!( diagnostic_field( recovery_authority_continuity_diagnostic, "verification_layer" ), &RelationalDiagnosticValue::string("DigestParity") )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 158: `assert!(!error.detail.is_empty())` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `durability_contract_failure_aspect_plan_mismatch_is_explicit`

Source: `crates/worth-relational/src/tests/durability/contracts/schema_transition_continuity.rs:162`.

- **UNCLASSIFIED** 217: `assert_ne!(expected_revision, mismatched_revision)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 218: `assert_eq!(error.class, RecoveryFailureClass::SchemaMismatch)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 219: `assert!(matches!( error.authority_continuity_mismatch, Some(RecoveryAuthorityContinuityMismatch::EntityAspectPlanRevision { kind_id: KindId(1), expected_revision: expected, found_revision: found, .. }) if expected == expected_revision.0 && found == mismatched_revision.0 ))` - semantic boundary still requires direct review; no move authorized.

#### `durability_contract_failure_relation_integrity_plan_mismatch_is_explicit`

Source: `crates/worth-relational/src/tests/durability/contracts/schema_transition_continuity.rs:231`.

- **UNCLASSIFIED** 336: `assert_eq!(error.class, RecoveryFailureClass::SchemaMismatch)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 337: `assert!(matches!( error.authority_continuity_mismatch, Some(RecoveryAuthorityContinuityMismatch::RelationIntegrityPlanRevision { kind_id: KindId(2), contract_family: RelationIntegrityContractFamily::Cardinality, ref expected_contract_ids, ref found_contract_ids, .. }) if expected_contract_ids == &vec![crate::schema::data::ContractId::from("source_max_one")] && found_contract_ids == &vec![crate::schema::data::ContractId::from("source_max_two")] ))` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/semantic_result.rs`

#### `semantic_result_discovery_must_be_present_in_each_actual_index_entry`

Source: `crates/worth-relational/src/tests/durability/contracts/semantic_result.rs:13`.

- **UNCLASSIFIED** 26: `assert_eq!( entry.association_discovery, crate::history::data::RelationalAssociationDiscovery::Unassociated )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 31: `assert_eq!( serde_json::from_value::<RelationalCommitIndexEntry>(wire.clone()).unwrap(), *entry )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 40: `assert!( failure.to_string().contains("association_discovery"), "{failure}" )` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/semantic_result/bulk.rs`

#### `semantic_result_bulk_entities_and_relations_seal_and_readmit_original_bindings`

Source: `crates/worth-relational/src/tests/durability/contracts/semantic_result/bulk.rs:63`.

- **UNCLASSIFIED** 68: `assert_eq!(semantic.created_entities().len(), 32)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 69: `assert_eq!(semantic.created_relations().len(), 31)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 79: `assert_eq!(semantic.created_entities()[index].key.client_key, key)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 80: `assert_eq!(semantic.created_entities()[index].assigned, assigned)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 83: `assert_eq!(relation.key.client_key, format!("bulk-edge-{index:02}"))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 90: `assert_eq!(source.client_key, format!("bulk-input-{index:02}"))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 91: `assert_eq!(target.client_key, format!("bulk-input-{:02}", index + 1))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 95: `assert_eq!(admitted.envelope().semantic_result(), semantic)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 96: `assert!( admitted .envelope() .symbol_texts .entries .iter() .any(|(_, text)| text.as_str() == "bulk-input-00"), "the actual symbol dictionary participates in this bulk source" )` - semantic boundary still requires direct review; no move authorized.

#### `semantic_result_bulk_malformed_correspondence_refuses_after_frame_reseal`

Source: `crates/worth-relational/src/tests/durability/contracts/semantic_result/bulk.rs:109`.

- **UNCLASSIFIED** 127: `assert_eq!(offsets.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 130: `assert!(Artifact::from_untrusted_bytes(bytes.clone()).is_ok())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 131: `assert!(matches!( Artifact::readmit(&bytes, execution), Err( crate::facade::durability::RelationalCanonicalCommitArtifactRefusal::PositionAdmission( crate::runtime::RelationalCanonicalPositionAdmissionDenial::SemanticResult( SemanticRefusal::ConflictingCreateCorrespondence ) ) ) ))` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/semantic_result/custom_versions.rs`

#### `semantic_result_preserves_custom_enforcement_version_distinct_from_aggregate_request`

Source: `crates/worth-relational/src/tests/durability/contracts/semantic_result/custom_versions.rs:60`.

- **UNCLASSIFIED** 78: `assert!( !callback.is_empty(), "the original ordinary producer actually ran the installed custom rule" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 100: `assert_eq!(execution_meaning.metadata.version, predecessor)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 101: `assert_eq!(custom.version, committed.commit.version_id)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 102: `assert_ne!(custom.version, execution_meaning.metadata.version, "the aggregate request describes the predecessor; the real custom callback checked the proposed effect")` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 103: `assert!(callback.contains(&( custom.observation_kind, custom.version, custom.current_version )))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 108: `assert_eq!( custom.observation_kind, execution_meaning.metadata.observation_kind )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 112: `assert_eq!( custom.current_version, execution_meaning.metadata.current_version )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 118: `assert_eq!(readmitted.envelope().semantic_result(), semantic)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 133: `assert_eq!(offsets.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 136: `assert!(Artifact::from_untrusted_bytes(bytes.clone()).is_ok())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 137: `assert!(matches!(Artifact::readmit(&bytes, execution), Err(crate::facade::durability::RelationalCanonicalCommitArtifactRefusal::PositionAdmission(crate::runtime::RelationalCanonicalPositionAdmissionDenial::SemanticResult(crate::transactions::data::RelationalCommitSemanticResultRefusal::ValidationMismatch)))))` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/contracts/tail_checkpoint_validation.rs`

#### `durability_contract_recovery_rejects_tail_target_without_immutable_artifact`

Source: `crates/worth-relational/src/tests/durability/contracts/tail_checkpoint_validation.rs:6`.

- C4 action 23: `.map(|commit| commit.envelope_mut_for_test(execution))`.
- **C4** 49: `assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 50: `assert!(error.detail.contains("recovery root"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 51: `assert!(error.detail.contains("is unavailable"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 52: `assert!(recovered.history().immutable_commit_count() <= 1)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 53: `assert_ne!( recovered .history() .branch_head(&BranchId("main".to_owned())) .map(|head| head.commit_id), Some(second.commit.commit_id) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `tests/durability/medium/branch_transition_races.rs`

#### `durable_fork_and_source_commit_complete_without_deferral`

Source: `crates/worth-relational/src/tests/durability/medium/branch_transition_races.rs:67`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/durability/medium/branch_transition_races.rs:47 assert_streams`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `durable_fork_and_source_archive_complete_in_lock_order`

Source: `crates/worth-relational/src/tests/durability/medium/branch_transition_races.rs:106`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/durability/medium/branch_transition_races.rs:47 assert_streams`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `durable_fork_waits_for_another_branch_commit`

Source: `crates/worth-relational/src/tests/durability/medium/branch_transition_races.rs:137`.

- **UNCLASSIFIED** 161: `assert_eq!( contents.offered_transitions().last().unwrap().follows().0, 2 )` - semantic boundary still requires direct review; no move authorized.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/durability/medium/branch_transition_races.rs:47 assert_streams`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `memory_commit_waits_behind_a_transition_without_consuming_its_candidate`

Source: `crates/worth-relational/src/tests/durability/medium/branch_transition_races.rs:168`.

- **UNCLASSIFIED** 198: `assert_eq!(commit.join().unwrap().patch_position().0, 2)` - semantic boundary still requires direct review; no move authorized.
- **C2** 200: `assert_eq!(runtime.patch_position_reservation_counters().deferrals, 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.

### `tests/durability/medium/checkpoint_member_storage.rs`

#### `filed_external_members_survive_checkpoint_replacement_and_obey_read_limits`

Source: `crates/worth-relational/src/tests/durability/medium/checkpoint_member_storage.rs:7`.

- **UNCLASSIFIED** 45: `assert_eq!( source.read_member(locator, 0), Err(Cause::LimitExceeded { length: expected.len() as u64, limit: 0 }) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 52: `assert_eq!( source.read_member(locator, locator.length).unwrap(), expected.as_ref() )` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/medium/cold_merge_history_fixture.rs`

Source is now removed by the concurrent cold-family migration. MOVED C4 context; this report does not recertify its accepted successor.

### `tests/durability/medium/cold_merge_history_matrix/exhaustive.rs`

Source is now removed by the concurrent cold-family migration. MOVED C4 context; this report does not recertify its accepted successor.

### `tests/durability/medium/cold_recovery_fixture.rs`

Source is now removed by the concurrent cold-family migration. MOVED C4 context; this report does not recertify its accepted successor.

### `tests/durability/medium/durable_before_live.rs`

#### `a_refused_publication_was_offered_to_the_medium_and_moved_nothing_live`

Source: `crates/worth-relational/src/tests/durability/medium/durable_before_live.rs:63`.

- C4 action 83: `medium.script_next_publication(ScriptedPublication::ProvenNoEffect);`.
- **C4** 86: `assert!(matches!( refused, TransactionCommitError::PublicationDenied { denial: RelationalPublicationDenial::DurablePublicationRefused( RelationalDurablePublicationRefusal::ProvenNoEffect { .. } ), .. } ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 96: `assert_eq!(offered.position(), PatchStreamPosition(2))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 97: `assert_eq!(live_footprint(execution, &runtime), before)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 98: `assert_eq!(medium.head(), PatchStreamPosition(1))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 99: `assert_eq!( runtime .preparation_runtime_snapshot() .history .index_definition_frontier(), 1 )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 107: `assert_eq!( accepted .envelope() .index_definition_delta .base_next_index_id, 1 )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 114: `assert_eq!( accepted.envelope().index_definition_delta.definitions(), &[definition] )` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `a_refused_position_is_not_consumed`

Source: `crates/worth-relational/src/tests/durability/medium/durable_before_live.rs:122`.

- C4 action 130: `medium.script_next_publication(ScriptedPublication::ProvenNoEffect);`.
- **C4** 135: `assert_eq!(accepted.patch_position(), PatchStreamPosition(2))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 136: `assert_ne!( accepted.commit.commit_id, refused.commit_id(), "an identity a refused candidate held is burned, never reissued" )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 141: `assert_eq!(accepted.commit.parents, vec![anchor.commit.commit_id])` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 142: `assert_eq!(medium.head(), PatchStreamPosition(2))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `every_acknowledged_commit_is_the_artifact_the_medium_holds_at_its_position`

Source: `crates/worth-relational/src/tests/durability/medium/durable_before_live.rs:147`.

- **HAPPY** 160: `assert_eq!(readmitted.commit_id(), committed.commit.commit_id)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 161: `assert_eq!(readmitted.position(), committed.patch_position())` - reviewed public owner result over explicit durable provider.
- **HAPPY** 164: `assert_eq!( medium.positions(), vec![ PatchStreamPosition(1), PatchStreamPosition(2), PatchStreamPosition(3) ] )` - reviewed public owner result over explicit durable provider.

#### `a_memory_only_runtime_publishes_without_a_medium`

Source: `crates/worth-relational/src/tests/durability/medium/durable_before_live.rs:175`.

- **UNCLASSIFIED** 182: `assert!(attached_durable_medium(&runtime).is_none())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 183: `assert_eq!(runtime.durability().durable_commit_count(), 1)` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/medium/embedded_cost.rs`

#### `embedded_restore_allocation_grows_linearly_in_relation_bearing_regions`

Source: `crates/worth-relational/src/tests/durability/medium/embedded_cost.rs:8`.

- **UNCLASSIFIED** 24: `assert!( output.status.success(), "{}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 30: `assert!( String::from_utf8_lossy(&output.stdout).contains("embedded restore allocations:"), "child must execute the probe" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 39: `assert!( wide <= narrow.saturating_mul(7) + 256 * 1024, "four times the regions must not cause quadratic allocation: {narrow} -> {wide}" )` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/medium/lifecycle_lock_order.rs`

#### `fork_against_checkpoint_capture`

Source: `crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs:87`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.

#### `fork_against_runtime_fork`

Source: `crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs:91`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.

#### `fork_against_reclamation`

Source: `crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs:95`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.

#### `archive_against_checkpoint_capture`

Source: `crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs:99`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.

#### `archive_against_runtime_fork`

Source: `crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs:103`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.

#### `archive_against_reclamation`

Source: `crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs:107`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.

#### `checkpoint_registry_is_readable_while_a_fork_is_durable_not_yet_live`

Source: `crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs:112`.

- **UNCLASSIFIED** 151: `assert_eq!(cells.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 152: `assert_eq!(lifecycle.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 162: `assert_eq!(contents.transition_sequences().len(), 1)` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/medium/lineage_frontiers.rs`

#### `checkpoint_restore_protects_published_lineage_identities_in_each_lane`

Source: `crates/worth-relational/src/tests/durability/medium/lineage_frontiers.rs:45`.

- **UNCLASSIFIED** 96: `assert!(checkpoint.directory.is_some())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 97: `assert!( checkpoint.lineage.nodes().is_empty(), "cold reopen loads no node payloads" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 106: `assert_eq!( receiver.lineage.identity_frontiers(), durable_floor, "{lane:?}: D7 resumes one past durable values" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 117: `assert!(!envelope.lineage_events().is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 119: `assert!( event.event_id() >= durable_floor.1, "{lane:?}: a published event id must never return" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 123: `assert!( event.targets().iter().all(|id| id.0 >= durable_floor.0), "{lane:?}: a published node id must never return" )` - semantic boundary still requires direct review; no move authorized.

#### `inflight_tail_replay_resumes_after_the_largest_durable_lineage_values`

Source: `crates/worth-relational/src/tests/durability/medium/lineage_frontiers.rs:132`.

- **UNCLASSIFIED** 200: `assert_eq!( plan.tail_log.len(), 1, "the inflight publication belongs to the tail" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 209: `assert_eq!( receiver.lineage.identity_frontiers(), (node_floor, event_floor) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 219: `assert!(envelope .lineage_events() .iter() .all(|event| event.event_id() >= event_floor && event.targets().iter().all(|id| id.0 >= node_floor)))` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/medium/recovered_publication.rs`

#### `a_recovered_runtime_publishes_its_next_commit_at_the_next_position`

Source: `crates/worth-relational/src/tests/durability/medium/recovered_publication.rs:9`.

- **UNCLASSIFIED** 31: `assert_eq!( cold.read_commit(PatchStreamPosition(1), 1 << 24).unwrap(), medium .artifact_bytes(PatchStreamPosition(1)) .unwrap() .into_vec() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 38: `assert!(matches!( cold.read_commit(PatchStreamPosition(1), 1), Err( crate::storage::data::RelationalStorageUnavailableCause::LimitExceeded { limit: 1, .. } ) ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 45: `assert_eq!(medium.offered().len(), offered_before_recovery)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 46: `assert_eq!( recovery.coverage.checkpoint_covered_position, Some(PatchStreamPosition(2)) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 50: `assert_eq!(recovery.coverage.replayed_tail_commits, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 52: `assert_eq!(next.patch_position(), PatchStreamPosition(4))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 53: `assert_eq!(next.commit.parents, vec![tail.commit.commit_id])` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 54: `assert!(!commit_ids.contains(&next.commit.commit_id))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 55: `assert_eq!(medium.head(), PatchStreamPosition(4))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 60: `assert_eq!(second.recovered_commits, 4)` - semantic boundary still requires direct review; no move authorized.
- **C2** 62: `assert!(std::sync::Arc::ptr_eq(&cold, &reopened_cold))` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 63: `assert_eq!( reopened_again .history() .branch_head(&BranchId("main".to_owned())), Some(next.commit.clone()) )` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/medium/recovery_refusals.rs`

#### `an_artifact_whose_frame_is_damaged_is_refused_at_the_read`

Source: `crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs:48`.

- C4 action 55: `medium.flip_artifact_byte(PatchStreamPosition(2), last_byte);`.
- **C4** 61: `assert!(matches!(refusal, crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactFrameRefusal::DigestMismatch { .. }))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `an_artifact_that_does_not_readmit_refuses_as_corrupt`

Source: `crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs:65`.

- **UNCLASSIFIED** 73: `assert_eq!( refusal_class(execution, &runtime), RecoveryFailureClass::CorruptCanonicalArtifact )` - semantic boundary still requires direct review; no move authorized.

#### `a_gap_in_the_journal_refuses_as_non_contiguous`

Source: `crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs:80`.

- C4 action 85: `attached_test_medium(&runtime).drop_artifact(PatchStreamPosition(2));`.
- **C4** 87: `assert_eq!( refusal_class(execution, &runtime), RecoveryFailureClass::NonContiguousCanonicalTail )` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `a_repeated_entry_refuses_as_non_contiguous`

Source: `crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs:94`.

- C4 action 100: `.duplicate_artifact(PatchStreamPosition(2), PatchStreamPosition(3));`.
- **C4** 102: `assert_eq!( refusal_class(execution, &runtime), RecoveryFailureClass::NonContiguousCanonicalTail )` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `a_tail_that_does_not_continue_from_the_checkpoint_refuses`

Source: `crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs:109`.

- C4 action 119: `attached_test_medium(&runtime).drop_artifact(PatchStreamPosition(3));`.
- **C4** 121: `assert_eq!( refusal_class(execution, &runtime), RecoveryFailureClass::NonContiguousCanonicalTail )` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `a_checkpoint_filed_under_the_wrong_position_refuses_as_corrupt`

Source: `crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs:128`.

- **UNCLASSIFIED** 136: `assert_eq!(native.covered_position(), Some(PatchStreamPosition(2)))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 147: `assert_eq!( refusal_class(execution, &runtime), RecoveryFailureClass::CorruptCheckpoint )` - semantic boundary still requires direct review; no move authorized.

#### `a_damaged_checkpoint_refuses_as_corrupt`

Source: `crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs:154`.

- C4 action 160: `attached_test_medium(&runtime).damage_checkpoint_payload();`.
- **C4** 162: `assert_eq!( refusal_class(execution, &runtime), RecoveryFailureClass::CorruptCheckpoint )` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `tests/durability/medium/reservation_wait.rs`

#### `a_durable_publisher_waits_for_the_reservation_instead_of_deferring`

Source: `crates/worth-relational/src/tests/durability/medium/reservation_wait.rs:61`.

- **UNCLASSIFIED** 86: `assert_eq!(first.patch_position(), PatchStreamPosition(2))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 87: `assert_eq!(second.patch_position(), PatchStreamPosition(3))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 88: `assert_eq!( contents.positions(), vec![ PatchStreamPosition(1), PatchStreamPosition(2), PatchStreamPosition(3) ] )` - semantic boundary still requires direct review; no move authorized.
- **C2** 97: `assert_eq!(counters.deferrals, 0, "nobody deferred")` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 98: `assert_eq!(counters.assignments, 3)` - actual residency, identity, pin, allocation, dictionary or cost observation.

#### `a_cancelled_waiter_stops_before_any_effect`

Source: `crates/worth-relational/src/tests/durability/medium/reservation_wait.rs:104`.

- **UNCLASSIFIED** 145: `assert!( matches!( cancelled, Err(crate::transactions::data::TransactionCommitError::Execution { .. }) ), "a cancelled waiter returns its request stop: {cancelled:?}" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 152: `assert_eq!(first.patch_position(), PatchStreamPosition(2))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 153: `assert_eq!( contents.positions(), vec![PatchStreamPosition(1), PatchStreamPosition(2)] )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 157: `assert_eq!(contents.offered().len(), 2, "the waiter offered nothing")` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 158: `assert_eq!( runtime .history() .branch_head(&BranchId("feature".to_owned())), feature_before, "the waiter installed nothing" )` - semantic boundary still requires direct review; no move authorized.
- **C2** 166: `assert_eq!((counters.deferrals, counters.assignments), (0, 2))` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 177: `assert_eq!(retried.patch_position(), PatchStreamPosition(3))` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/medium/single_owner.rs`

#### `a_forked_runtime_never_inherits_the_medium`

Source: `crates/worth-relational/src/tests/durability/medium/single_owner.rs:14`.

- **UNCLASSIFIED** 23: `assert!(attached_durable_medium(&forked).is_none())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 27: `assert_eq!(contents.positions(), vec![PatchStreamPosition(1)])` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 28: `assert_eq!(contents.offered().len(), 1, "the fork offered nothing")` - semantic boundary still requires direct review; no move authorized.

#### `recovery_moves_the_medium_into_the_rebuilt_runtime`

Source: `crates/worth-relational/src/tests/durability/medium/single_owner.rs:32`.

- **HAPPY** 52: `assert_ne!(runtime.runtime_instance_id(), replaced)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 54: `assert_eq!(next.patch_position(), PatchStreamPosition(2))` - reviewed public owner result over explicit durable provider.
- **HAPPY** 55: `assert_eq!(contents.head(), PatchStreamPosition(2))` - reviewed public owner result over explicit durable provider.

#### `an_attachment_the_medium_moved_out_of_publishes_nothing`

Source: `crates/worth-relational/src/tests/durability/medium/single_owner.rs:60`.

- **UNCLASSIFIED** 72: `assert!(binding.detach().is_some(), "the medium moves out once")` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 73: `assert!(binding.detach().is_none(), "and only once")` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 80: `assert!(matches!( refused, Err(RelationalDurableStepStop::Refused( RelationalDurablePublicationRefusal::MediumMoved )) ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 86: `assert!(contents.offered().is_empty())` - semantic boundary still requires direct review; no move authorized.

### `tests/durability/medium/unresolved_fate.rs`

#### `an_unresolved_publication_installs_nothing_and_refuses_every_later_commit_alike`

Source: `crates/worth-relational/src/tests/durability/medium/unresolved_fate.rs:31`.

- C4 action 51: `medium.script_next_publication(ScriptedPublication::IndeterminateWithoutEffect);`.
- **C4** 66: `assert_eq!(position, PatchStreamPosition(2))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 67: `assert_eq!(position, offered.position())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 68: `assert_eq!(commit_id, offered.commit_id())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 69: `assert_eq!(digest, offered.payload_digest())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 70: `assert_eq!(main_head(&runtime), Some(anchor.commit.clone()))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 71: `assert_eq!(runtime.durability().durable_commit_count(), 1)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 72: `assert_eq!( runtime .preparation_runtime_snapshot() .history .index_definition_frontier(), 1 )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C2** 79: `assert_eq!( runtime.visibility.published_snapshot_handle_count(), published_handles )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C4** 86: `assert!(matches!( later, TransactionCommitError::DurabilityUnresolved { .. } ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 90: `assert_eq!(later.durability_unresolved_fate(), Some(&fate))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 91: `assert_eq!( medium.offered().len(), offered_before_retry, "a later position is never offered over an unresolved one" )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 96: `assert_eq!(main_head(&runtime), Some(anchor.commit.clone()))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `reopening_resolves_an_absent_unresolved_commit_as_never_committed`

Source: `crates/worth-relational/src/tests/durability/medium/unresolved_fate.rs:100`.

- C4 action 107: `medium.script_next_publication(ScriptedPublication::IndeterminateWithoutEffect);`.
- **C4** 117: `assert!(medium.artifact_bytes(position).is_none())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 121: `assert_eq!(recovery.recovered_commits, 1)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 122: `assert_eq!(main_head(&reopened), Some(anchor.commit.clone()))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 124: `assert_eq!(resumed.patch_position(), position)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 125: `assert_eq!(resumed.commit.parents, vec![anchor.commit.commit_id])` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 126: `assert_eq!(medium.head(), position)` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `reopening_resolves_a_present_unresolved_commit_as_committed_exactly_once`

Source: `crates/worth-relational/src/tests/durability/medium/unresolved_fate.rs:131`.

- C4 action 138: `medium.script_next_publication(ScriptedPublication::IndeterminateAfterEffect);`.
- **C4** 152: `assert_eq!(medium.head(), position)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 153: `assert_eq!(main_head(&runtime), Some(anchor.commit.clone()))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 157: `assert_eq!(recovery.recovered_commits, 2)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 159: `assert_eq!(head.commit_id, commit_id)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 160: `assert_eq!( reopened .durability() .durable_log() .iter() .filter(|entry| entry.envelope().commit.commit_id == commit_id) .count(), 1 )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 170: `assert_eq!(next.patch_position(), PatchStreamPosition(position.0 + 1))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 171: `assert_eq!(next.commit.parents, vec![commit_id])` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 172: `assert_ne!(next.commit.commit_id, commit_id)` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `tests/harness/fixtures/runtime.rs`

No direct test declarations. Executing helper roots and claims:

- `build_runtime` at 11: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/publication/cdc/certification/durable_recovery.rs:87 cdc_certification_persisted_seeded_matrix_survives_checkpoint_compaction_and_recovery`, `crates/worth-relational/src/tests/publication/cdc/certification/durable_recovery.rs:128 cdc_certification_retention_truncation_recovers_exact_suffix_from_old_checkpoint`, `crates/worth-relational/src/tests/publication/cdc/certification/hostile_resume_pressure.rs:11 cdc_certification_subscriber_api_fuzz_matrix_stays_consistent_under_hostile_history_pressure`, `crates/worth-relational/src/tests/publication/cdc/certification/hostile_resume_pressure.rs:42 cdc_certification_concurrent_branch_merge_pressure_keeps_subscriber_order_stable`, `crates/worth-relational/src/tests/publication/cdc/certification/seeded_resume.rs:18 cdc_certification_seeded_matrix_is_deterministic_and_resume_exact`, `crates/worth-relational/src/tests/publication/cdc/certification/seeded_resume.rs:75 cdc_certification_thousand_step_random_resume_matrix_converges` (ignored), `crates/worth-relational/src/tests/publication/cdc/certification/seeded_resume.rs:219 cdc_certification_restart_loops_do_not_leak_resume_state_across_sessions`, `crates/worth-relational/src/tests/publication/cdc/deterministic_matrix.rs:5 seeded_cdc_matrix_is_deterministic_and_window_stable`.

### `tests/harness/scenario/runner/property_runtime.rs`

No direct test declarations. Executing helper roots and claims:

- `build_property_runtime` at 9: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/publication/cdc/certification/durable_recovery.rs:87 cdc_certification_persisted_seeded_matrix_survives_checkpoint_compaction_and_recovery`, `crates/worth-relational/src/tests/publication/cdc/certification/property_matrices.rs:86 cdc_property_random_operation_matrix_converges`, `crates/worth-relational/src/tests/publication/cdc/certification/property_matrices.rs:126 cdc_property_persisted_random_operation_matrix_recovers`, `crates/worth-relational/src/tests/publication/cdc/certification/property_matrices.rs:162 cdc_property_branch_local_update_and_merge_matrix_converges`.

### `tests/history/milestone_7a/certification_bundle.rs`

No direct test declarations. Executing helper roots and claims:

- `run_merge_ready_history_shape_certification` at 69: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/history/milestone_7a/ancestry_query_matrix.rs:4 merge_ready_history_ancestry_queries_resolve_branch_reasoning`, `crates/worth-relational/src/tests/history/milestone_7a/canonical_parent_lists.rs:4 merge_ready_history_parent_lists_survive_publication_replay_and_recovery`, `crates/worth-relational/src/tests/history/milestone_7a/replay_durability_diagnostics.rs:4 merge_ready_history_replay_durability_and_diagnostics_are_authoritative`.

### `tests/history/milestone_7a/counter_breadth.rs`

#### `merge_ready_history_shape_reports_counter_breadth_explicitly`

Source: `crates/worth-relational/src/tests/history/milestone_7a/counter_breadth.rs:7`.

- **UNCLASSIFIED** 45: `assert!(replay.failure.is_none(), "{:?}", replay)` - semantic boundary still requires direct review; no move authorized.
- **C2** 48: `assert!(runtime_counters.merge_history_ancestry_traversals >= 1)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 49: `assert!(runtime_counters.merge_history_ancestry_nodes_visited >= 4)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 50: `assert!(runtime_counters.merge_history_parent_comparisons >= 2)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 51: `assert!(runtime_counters.merge_history_replay_planning_nodes_visited >= 4)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 52: `assert!(runtime_counters.merge_history_replay_parent_checks >= 4)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 66: `assert!(recovered_counters.merge_history_durability_validation_nodes_visited >= 4)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 67: `assert!(recovered_counters.merge_history_durability_parent_checks >= 4)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 68: `assert!(recovered_counters.merge_history_parent_comparisons >= 2)` - actual residency, identity, pin, allocation, dictionary or cost observation.

### `tests/history/milestone_7b.rs`

#### `merge_planning_artifact_certification_is_stable_across_recovery`

Source: `crates/worth-relational/src/tests/history/milestone_7b.rs:63`.

- **UNCLASSIFIED** 69: `assert_eq!(artifact.digest_basis.schema, artifact.schema_snapshot)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 70: `assert_eq!( artifact.digest_basis.decision_log, artifact.decision_log_digest_basis )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 74: `assert_eq!( artifact.digest_basis.lowered_plan.records.len(), artifact.lowered_plan.records.len() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 78: `assert_eq!( artifact.decision_log_digest_basis.canonical_records.len(), artifact.decision_log.decisions.len() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 82: `assert!(!artifact.schema_snapshot.touched_kinds.is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 83: `assert!(artifact.lowered_plan.record_count > 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 84: `assert!(artifact.summary.request_summary.contains("main"))` - semantic boundary still requires direct review; no move authorized.

#### `merge_planning_schema_snapshot_changes_when_schema_semantics_change`

Source: `crates/worth-relational/src/tests/history/milestone_7b.rs:88`.

- **UNCLASSIFIED** 168: `assert_ne!(prefer_richer_digest, fail_on_conflict_digest)` - semantic boundary still requires direct review; no move authorized.

### `tests/history/milestone_7c_phase_a/compilation_denials.rs`

#### `compile_execution_ready_merge_plan_rejects_missing_source_record`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_a/compilation_denials.rs:12`.

- C4 action 36: `prepared.execution_ready_plan_mut_for_test().source_records = Arc::from([]);`.
- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.

#### `verify_prepared_merge_execution_rejects_corrupted_compiled_plan`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_a/compilation_denials.rs:48`.

- C4 action 72: `prepared.bound_executable_plan_mut_for_test().record_plans = Arc::from([]);`.
- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.

### `tests/history/milestone_7c_phase_a/prepared_merge_verification.rs`

#### `verify_prepared_merge_execution_accepts_fresh_prepared_merge`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_a/prepared_merge_verification.rs:14`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.

#### `verify_prepared_merge_execution_rejects_runtime_instance_mismatch`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_a/prepared_merge_verification.rs:46`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.

#### `verify_prepared_merge_execution_rejects_target_head_drift`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_a/prepared_merge_verification.rs:82`.

- **UNCLASSIFIED** 114: `assert_eq!(branch, BranchId("main".to_string()))` - semantic boundary still requires direct review; no move authorized.

#### `verify_prepared_merge_execution_rejects_source_head_drift`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_a/prepared_merge_verification.rs:121`.

- **UNCLASSIFIED** 158: `assert_eq!(branch, BranchId("feature".to_string()))` - semantic boundary still requires direct review; no move authorized.

#### `verify_prepared_merge_execution_rejects_schema_semantic_drift`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_a/prepared_merge_verification.rs:165`.

- C4 action 220: `runtime.set_schema_registry_for_test(drifted_registry);`.
- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.

#### `verify_prepared_merge_execution_rejects_merge_base_drift`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_a/prepared_merge_verification.rs:232`.

- C4 action 257: `.authority_binding_mut_for_test()`.
- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.

### `tests/history/milestone_7c_phase_a/readiness_admission.rs`

#### `prepare_merge_execution_admits_fully_ready_source_only_addition`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_a/readiness_admission.rs:20`.

- **UNCLASSIFIED** 49: `assert_eq!(prepared.request(), &normalized_request)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 50: `assert!(prepared.artifact().lowered_plan.fully_execution_ready)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 51: `assert_eq!(prepared.artifact().lowered_plan.blocked_count, 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 52: `assert_eq!(prepared.artifact().lowered_plan.rejected_count, 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 53: `assert!(prepared .execution_ready_plan() .lowered_records .iter() .all(|record| matches!( record.record_decision, crate::merge::data::LoweredRecordDecision::Execute(_) )))` - semantic boundary still requires direct review; no move authorized.

#### `prepare_merge_execution_rejects_blocked_merge_plans`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_a/readiness_admission.rs:64`.

- **UNCLASSIFIED** 94: `assert_eq!(report.blocked_count, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 95: `assert_eq!(report.rejected_count, 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 96: `assert_eq!(report.denied_records.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 97: `assert_eq!( report.denied_records[0].decision, crate::merge::data::LoweredRecordDecisionKind::Block )` - semantic boundary still requires direct review; no move authorized.

#### `prepare_merge_execution_rejects_rejected_merge_plans`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_a/readiness_admission.rs:107`.

- **UNCLASSIFIED** 170: `assert_eq!(report.blocked_count, 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 171: `assert_eq!(report.rejected_count, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 172: `assert_eq!(report.denied_records.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 173: `assert_eq!( report.denied_records[0].decision, crate::merge::data::LoweredRecordDecisionKind::Reject )` - semantic boundary still requires direct review; no move authorized.

#### `runtime_prepare_merge_execution_matches_merge_access_surface`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_a/readiness_admission.rs:183`.

- **UNCLASSIFIED** 214: `assert_eq!(via_runtime.request(), &normalized_request)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 215: `assert_eq!(via_runtime.artifact(), via_access.artifact())` - semantic boundary still requires direct review; no move authorized.

### `tests/history/milestone_7c_phase_a/record_plan_compilation.rs`

#### `prepare_merge_execution_compiles_source_addition_record_plan`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_a/record_plan_compilation.rs:25`.

- **UNCLASSIFIED** 50: `assert_eq!(prepared.bound_executable_plan().record_plans.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 54: `assert_eq!( read_entity_field(entity, field_key("name")), Some("feature-only".into()) )` - semantic boundary still requires direct review; no move authorized.

#### `prepare_merge_execution_compiles_exact_shared_record_plan`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_a/record_plan_compilation.rs:66`.

- **UNCLASSIFIED** 122: `assert_eq!(prepared.bound_executable_plan().record_plans.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 125: `assert!(!plan.equality_witness.witness_digest.is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 126: `assert_eq!( plan.provenance.classification, crate::merge::data::MergeConflictClass::ExactSharedTruth )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 130: `assert!(!plan.aspect_plan.is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 133: `assert_eq!( shared_value.policy, MergeValueMaterialization::EqualityWitnessDigest )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 139: `assert!(!digest.is_empty())` - semantic boundary still requires direct review; no move authorized.

#### `prepare_merge_execution_compiles_reconcile_record_plan`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_a/record_plan_compilation.rs:152`.

- **UNCLASSIFIED** 218: `assert_eq!(prepared.bound_executable_plan().record_plans.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 221: `assert_eq!(plan.identity_basis.source_record, plan.source_record)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 222: `assert_eq!(plan.identity_basis.target_record, plan.target_record)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 223: `assert!(!plan.aspect_plan.is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 234: `assert_eq!( source_value.policy, MergeValueMaterialization::SnapshotPinnedRead )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 238: `assert_eq!( target_value.policy, MergeValueMaterialization::SnapshotPinnedRead )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 242: `assert_eq!( base_value.policy, MergeValueMaterialization::SnapshotPinnedRead )` - semantic boundary still requires direct review; no move authorized.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/history/milestone_7c_phase_a/record_plan_compilation.rs:256 assert_pinned_visible_aspect_evidence`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

### `tests/history/milestone_7c_phase_a/verification_counters.rs`

#### `verify_prepared_merge_execution_does_not_increment_planning_counters`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_a/verification_counters.rs:9`.

- **UNCLASSIFIED** 41: `assert_eq!( before.merge_planning_requests, after.merge_planning_requests )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 45: `assert_eq!( before.merge_planning_schema_kinds_snapshotted, after.merge_planning_schema_kinds_snapshotted )` - semantic boundary still requires direct review; no move authorized.
- **C2** 49: `assert_eq!( before.merge_planning_elapsed_nanos, after.merge_planning_elapsed_nanos )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 53: `assert_eq!( after.merge_execution_verification_requests, before.merge_execution_verification_requests + 1 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 57: `assert_eq!( after.merge_execution_branch_head_checks, before.merge_execution_branch_head_checks + 2 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 61: `assert_eq!( after.merge_execution_merge_base_checks, before.merge_execution_merge_base_checks + 1 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 65: `assert_eq!( after.merge_execution_compiled_plan_digest_checks, before.merge_execution_compiled_plan_digest_checks + 1 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 69: `assert!( after.merge_execution_schema_kinds_snapshotted >= before.merge_execution_schema_kinds_snapshotted )` - semantic boundary still requires direct review; no move authorized.

#### `verify_prepared_merge_execution_reports_verification_counters_without_planning_work`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_a/verification_counters.rs:76`.

- **C2** 108: `assert_eq!(counters.merge_planning_requests, 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 109: `assert_eq!(counters.merge_planning_schema_kinds_snapshotted, 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 110: `assert_eq!(counters.merge_planning_elapsed_nanos, 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 111: `assert_eq!(counters.merge_execution_verification_requests, 1)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 112: `assert_eq!(counters.merge_execution_branch_head_checks, 2)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 113: `assert_eq!(counters.merge_execution_merge_base_checks, 1)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 114: `assert_eq!(counters.merge_execution_compiled_plan_digest_checks, 1)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 115: `assert!(counters.merge_execution_schema_kinds_snapshotted > 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.

### `tests/history/milestone_7c_phase_d.rs`

#### `derive_merge_commit_mutation_plan_emits_source_authorized_create_intent`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_d.rs:32`.

- **UNCLASSIFIED** 59: `assert_eq!(plan.transaction_id, TransactionId(77))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 60: `assert_eq!(plan.structural_summary.executed_record_count, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 61: `assert_eq!(plan.structural_summary.adopted_source_record_count, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 62: `assert_eq!(plan.structural_summary.emitted_entity_create_count, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 63: `assert_eq!(plan.merged_plan.merged_intents.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 66: `assert_eq!(spec.kind_id, KindId(1))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 67: `assert_eq!( spec.fields.get(&test_patch_target("name", "name")), Some(&AspectValue::String(InternedString::Raw( "feature-only".to_string() ))) )` - semantic boundary still requires direct review; no move authorized.

#### `derive_merge_commit_mutation_plan_preserves_exact_shared_truth_without_mutation`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_d.rs:79`.

- **UNCLASSIFIED** 137: `assert_eq!(plan.structural_summary.preserved_shared_record_count, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 138: `assert_eq!(plan.structural_summary.emitted_mutation_intent_count, 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 139: `assert!(plan.merged_plan.merged_intents.is_empty())` - semantic boundary still requires direct review; no move authorized.

#### `derive_merge_commit_mutation_plan_reconciles_target_with_source_authorized_aspects`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_d.rs:143`.

- **UNCLASSIFIED** 280: `assert_eq!(plan.structural_summary.reconciled_record_count, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 281: `assert_eq!(plan.structural_summary.emitted_entity_update_count, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 282: `assert_eq!(plan.merged_plan.merged_intents.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 285: `assert_eq!( intent.fields.get(&test_patch_target("status", "status")), Some(&AspectValue::String("active".into())) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 289: `assert_eq!(intent.fields.get(&test_patch_target("name", "name")), None)` - semantic boundary still requires direct review; no move authorized.

#### `derive_merge_commit_mutation_plan_does_not_rely_on_raw_lowered_record_arrays`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_d.rs:296`.

- C4 action 320: `prepared.execution_ready_plan_mut_for_test().lowered_records = Arc::from([]);`.
- **C4** 324: `assert_eq!(plan.structural_summary.executed_record_count, 1)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 325: `assert_eq!(plan.merged_plan.merged_intents.len(), 1)` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `tests/history/milestone_7c_phase_e/canonical_envelope_publication.rs`

#### `execute_prepared_merge_publishes_ordered_multi_parent_commit_through_canonical_envelope`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_e/canonical_envelope_publication.rs:11`.

- **UNCLASSIFIED** 19: `assert_eq!(merge.commit.merge_parent_count(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 20: `assert_eq!( merge.commit.commit.parents, vec![main_head_commit_id, feature_head_commit_id] )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 24: `assert_eq!( merge.execution_summary.target_head_commit_id, main_head_commit_id )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 28: `assert_eq!( merge.execution_summary.source_head_commit_id, feature_head_commit_id )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 32: `assert_eq!(merge.execution_summary.executed_record_count, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 38: `assert_eq!(envelope.commit.parents, merge.commit.commit.parents)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 39: `assert_eq!( envelope.merge_parent_branches, vec![BranchId("feature".to_string())] )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 43: `assert_eq!( runtime .history() .branch_head(&BranchId("main".to_string())) .expect("main branch head") .commit_id, merge.commit.commit.commit_id )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 51: `assert_eq!( runtime .history() .branch_head(&BranchId("feature".to_string())) .expect("feature branch head") .commit_id, feature_head_commit_id )` - semantic boundary still requires direct review; no move authorized.

#### `execute_prepared_merge_preserves_reserved_summary_when_optional_diagnostics_budget_is_zero`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_e/canonical_envelope_publication.rs:62`.

- **UNCLASSIFIED** 97: `assert_eq!(envelope.diagnostics_summary.entries.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 98: `assert_eq!( envelope.diagnostics_summary.entries[0].code, DiagnosticCode::MergeExecutionPublished )` - semantic boundary still requires direct review; no move authorized.

### `tests/history/milestone_7c_phase_e/commit_context_admission.rs`

#### `merge_commit_context_rejects_mismatched_parent_branch_metadata`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_e/commit_context_admission.rs:9`.

- **UNCLASSIFIED** 50: `assert_eq!(branch_id, BranchId("wrong".to_string()))` - semantic boundary still requires direct review; no move authorized.

### `tests/history/milestone_7c_phase_e/durability_recovery_diagnostics.rs`

#### `execute_prepared_merge_survives_durability_append_and_recovery`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_e/durability_recovery_diagnostics.rs:11`.

- **UNCLASSIFIED** 33: `assert_eq!(before_bundle.visible_truth, recovered_bundle.visible_truth)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 39: `assert!(merge_envelope .diagnostics_summary .entries .iter() .any(|entry| entry.code == DiagnosticCode::MergeExecutionPublished))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 50: `assert_eq!( diagnostic_field(merge_execution_entry, "commit_id"), &RelationalDiagnosticValue::CommitId(merge.commit.commit.commit_id) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 54: `assert_eq!( diagnostic_field(merge_execution_entry, "execution_digest"), &RelationalDiagnosticValue::String(merge.execution_summary.execution_digest.clone()) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 58: `assert_eq!( diagnostic_field(merge_execution_entry, "diagnostics_digest"), &RelationalDiagnosticValue::String(merge.execution_summary.diagnostics_digest.clone()) )` - semantic boundary still requires direct review; no move authorized.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/support.rs:138 capture_aspect_truth_bundle`, `crates/worth-relational/src/tests/support/truth_comparison.rs:60 assert_durable_commit_meaning_eq`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

### `tests/history/milestone_7c_phase_e/execution_counters_diagnostics.rs`

#### `execute_prepared_merge_reports_execution_counters_and_structural_summary`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_e/execution_counters_diagnostics.rs:10`.

- **UNCLASSIFIED** 41: `assert_eq!(merge.structural_summary.executed_record_count, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 42: `assert_eq!(merge.structural_summary.adopted_source_record_count, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 43: `assert_eq!(merge.structural_summary.emitted_mutation_intent_count, 1)` - semantic boundary still requires direct review; no move authorized.
- **C2** 44: `assert_eq!(counters.merge_execution_attempts, 1)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 45: `assert_eq!(counters.merge_execution_requests, 1)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 46: `assert_eq!(counters.merge_execution_records_admitted, 1)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 47: `assert_eq!(counters.merge_execution_mutation_intents_emitted, 1)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 48: `assert_eq!( merge .commit .execution() .complexity_delta .merge_execution_attempts, 1 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 56: `assert_eq!( merge .commit .execution() .complexity_delta .merge_execution_requests, 1 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 64: `assert_eq!( merge .commit .execution() .complexity_delta .merge_execution_records_admitted, 1 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 72: `assert_eq!( merge .commit .execution() .complexity_delta .merge_execution_mutation_intents_emitted, 1 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 80: `assert!(merge.commit.diagnostics().iter().any(|artifact| { artifact.kind == DiagnosticsArtifactKind::MinimalSummary && artifact .entries .iter() .any(|entry| entry.code == DiagnosticCode::MergeExecutionPublished) }))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 99: `assert!(execution_artifact.entries.iter().all(|entry| { entry.code == DiagnosticCode::MergeExecutionPublished && diagnostic_field_optional(entry, "blocked_count").is_none() && diagnostic_field_optional(entry, "rejected_count").is_none() }))` - semantic boundary still requires direct review; no move authorized.

### `tests/history/milestone_7c_phase_e/failure_attempt_diagnostics.rs`

#### `execute_prepared_merge_records_attempt_without_success_on_failure`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_e/failure_attempt_diagnostics.rs:10`.

- **UNCLASSIFIED** 42: `assert!(matches!( error, crate::facade::merge::MergeExecutionError::StaleBranchHead { .. } ))` - semantic boundary still requires direct review; no move authorized.
- **C2** 46: `assert_eq!(counters.merge_execution_attempts, 1)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 47: `assert_eq!(counters.merge_execution_requests, 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 48: `assert_eq!(counters.merge_execution_records_admitted, 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 49: `assert_eq!(counters.merge_execution_mutation_intents_emitted, 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 50: `assert!(runtime .publication() .diagnostics() .artifacts() .iter() .any(|artifact| artifact.scope == DiagnosticsScope::History && artifact.kind == DiagnosticsArtifactKind::Failure && artifact .entries .iter() .any(|entry| entry.code == DiagnosticCode::DeterministicMergeViolation)))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 61: `assert!(!runtime .publication() .diagnostics() .artifacts() .iter() .any(|artifact| artifact.scope == DiagnosticsScope::History && artifact.kind == DiagnosticsArtifactKind::DetailedTrace && artifact .entries .iter() .any(|entry| entry.code == DiagnosticCode::MergeExecutionPublished)))` - semantic boundary still requires direct review; no move authorized.

### `tests/history/milestone_7c_phase_e/fixtures.rs`

No direct test declarations. Executing helper roots and claims:

- `execute_feature_into_main_merge` at 8: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/history/milestone_7c_phase_e/canonical_envelope_publication.rs:11 execute_prepared_merge_publishes_ordered_multi_parent_commit_through_canonical_envelope`, `crates/worth-relational/src/tests/history/milestone_7c_phase_e/durability_recovery_diagnostics.rs:11 execute_prepared_merge_survives_durability_append_and_recovery`, `crates/worth-relational/src/tests/history/milestone_7c_phase_e/merge_ready_history_shape.rs:7 execute_prepared_merge_produces_merge_ready_history_shape`.

### `tests/history/milestone_7c_phase_g/certification_cases.rs`

No direct test declarations. Executing helper roots and claims:

- `certify_exact_shared_merge_execution` at 15: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/history/milestone_7c_phase_g/mod.rs:16 authoritative_merge_execution_certification_emits_machine_checkable_artifacts`.
  - UNCLASSIFIED assertion 44: `assert_eq!(merge.structural_summary.preserved_shared_record_count, 1)`.
  - UNCLASSIFIED assertion 45: `assert_eq!(merge.structural_summary.emitted_mutation_intent_count, 0)`.
- `certify_source_only_addition_merge_execution` at 55: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/history/milestone_7c_phase_g/mod.rs:16 authoritative_merge_execution_certification_emits_machine_checkable_artifacts`.
  - UNCLASSIFIED assertion 82: `assert_eq!(merge.structural_summary.adopted_source_record_count, 1)`.
  - UNCLASSIFIED assertion 83: `assert_eq!(merge.structural_summary.emitted_entity_create_count, 1)`.
- `certify_prefer_richer_merge_execution` at 93: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/history/milestone_7c_phase_g/mod.rs:16 authoritative_merge_execution_certification_emits_machine_checkable_artifacts`.
  - UNCLASSIFIED assertion 149: `assert_eq!(merge.structural_summary.reconciled_record_count, 1)`.
  - UNCLASSIFIED assertion 150: `assert_eq!(merge.structural_summary.emitted_entity_update_count, 1)`.
  - UNCLASSIFIED assertion 159: `assert_eq!(read_entity_name(current_record), Some("shared-name".into()))`.
  - UNCLASSIFIED assertion 160: `assert_eq!( read_entity_field(current_record, crate::tests::support::field_key("status")), Some("active".into()) )`.

### `tests/history/milestone_7c_phase_g/schema_fixtures.rs`

No direct test declarations. Executing helper roots and claims:


### `tests/history/milestone_7c_phase_g/stale_prepared_merge_denials.rs`

#### `authoritative_merge_execution_certification_rejects_stale_prepared_merge_after_target_advances`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_g/stale_prepared_merge_denials.rs:11`.

- **UNCLASSIFIED** 40: `assert_eq!(branch, BranchId("main".to_string()))` - semantic boundary still requires direct review; no move authorized.

#### `authoritative_merge_execution_certification_rejects_stale_prepared_merge_after_source_advances`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_g/stale_prepared_merge_denials.rs:47`.

- **UNCLASSIFIED** 81: `assert_eq!(branch, BranchId("feature".to_string()))` - semantic boundary still requires direct review; no move authorized.

#### `authoritative_merge_execution_certification_rejects_metadata_reference_movement`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_g/stale_prepared_merge_denials.rs:88`.

- **UNCLASSIFIED** 130: `assert_eq!( runtime .history() .branch_head(&target_branch) .expect("metadata movement does not change the immutable head") .commit_id, target_head.commit_id )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 138: `assert_eq!( runtime .history .branch_cell(&target_branch) .expect("main reference remains live") .observation() .generation() .get(), target_generation + 1 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 154: `assert_eq!(branch, target_branch)` - semantic boundary still requires direct review; no move authorized.

#### `authoritative_merge_execution_certification_rejects_schema_semantic_drift_after_planning`

Source: `crates/worth-relational/src/tests/history/milestone_7c_phase_g/stale_prepared_merge_denials.rs:161`.

- C4 action 185: `runtime.set_schema_registry_for_test(drifted_schema_registry());`.
- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.

### `tests/history/milestone_7d_phase_a.rs`

#### `merge_planning_conflict_classification_carries_target_view_visibility_evidence_for_exact_shared_truth`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_a.rs:13`.

- **UNCLASSIFIED** 50: `assert_eq!(classification.class, MergeConflictClass::ExactSharedTruth)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 51: `assert_eq!( classification.target_visibility_evidence.kind, crate::facade::merge::MergeVisibilityEvidenceKind::TargetCandidateViewLookup )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 55: `assert_eq!( classification.target_visibility_evidence.state, crate::facade::merge::MergeVisibilityState::Visible )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 59: `assert_eq!( classification .target_visibility_evidence .embedded_surface_state, Some(crate::facade::merge::MergeVisibilityState::Visible) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 65: `assert_eq!( classification.target_visibility_evidence.lifecycle, Some(crate::storage::data::RecordLifecycleState::Live) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 69: `assert!(classification .target_visibility_evidence .created_at_version .is_some())` - semantic boundary still requires direct review; no move authorized.

#### `merge_planning_conflict_classification_carries_base_window_evidence_for_target_deleted_record`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_a.rs:76`.

- **UNCLASSIFIED** 112: `assert!(matches!( classification.class, MergeConflictClass::Deletion(_) ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 116: `assert_eq!( classification.base_visibility_evidence.kind, crate::facade::merge::MergeVisibilityEvidenceKind::BaseResolvedViewLookup )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 120: `assert_eq!( classification.base_visibility_evidence.state, crate::facade::merge::MergeVisibilityState::Visible )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 124: `assert_eq!( classification.base_visibility_evidence.lifecycle, Some(crate::storage::data::RecordLifecycleState::Live) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 128: `assert_eq!( classification.target_visibility_evidence.kind, crate::facade::merge::MergeVisibilityEvidenceKind::TargetEmbeddedSurface )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 132: `assert_eq!( classification.target_visibility_evidence.state, crate::facade::merge::MergeVisibilityState::NotVisible )` - semantic boundary still requires direct review; no move authorized.

#### `merge_planning_conflict_classification_carries_target_lookup_evidence_for_source_deleted_record`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_a.rs:139`.

- **UNCLASSIFIED** 169: `assert_eq!( classification.class, MergeConflictClass::Deletion( crate::facade::merge::DeletionMergeClass::SourceDeletedTargetLive ) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 175: `assert_eq!( classification.source_visibility_evidence.kind, crate::facade::merge::MergeVisibilityEvidenceKind::SourceEmbeddedSurface )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 179: `assert_eq!( classification.source_visibility_evidence.state, crate::facade::merge::MergeVisibilityState::NotVisible )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 183: `assert_eq!( classification.target_visibility_evidence.kind, crate::facade::merge::MergeVisibilityEvidenceKind::TargetCandidateViewLookup )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 187: `assert_eq!( classification.target_visibility_evidence.state, crate::facade::merge::MergeVisibilityState::Visible )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 191: `assert_eq!( classification .target_visibility_evidence .embedded_surface_state, Some(crate::facade::merge::MergeVisibilityState::Visible) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 197: `assert_eq!( classification.base_visibility_evidence.kind, crate::facade::merge::MergeVisibilityEvidenceKind::BaseResolvedViewLookup )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 201: `assert_eq!( classification.base_visibility_evidence.state, crate::facade::merge::MergeVisibilityState::Visible )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 205: `assert_eq!( classification.base_visibility_evidence.lifecycle, Some(crate::storage::data::RecordLifecycleState::Live) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 222: `assert_eq!( target_record.lifecycle, crate::storage::data::RecordLifecycleState::Live )` - semantic boundary still requires direct review; no move authorized.

#### `merge_planning_digest_basis_carries_visibility_evidence_rows`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_a.rs:229`.

- **UNCLASSIFIED** 256: `assert_eq!( artifact.digest_basis.conflict.records.len(), artifact .digest_basis .conflict .source_visibility_evidence .len() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 264: `assert_eq!( artifact.digest_basis.conflict.records.len(), artifact .digest_basis .conflict .target_visibility_evidence .len() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 272: `assert_eq!( artifact.digest_basis.conflict.records.len(), artifact .digest_basis .conflict .base_visibility_evidence .len() )` - semantic boundary still requires direct review; no move authorized.

#### `merge_planning_policy_surface_is_explicitly_runtime_owned_before_lowering`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_a.rs:283`.

- **UNCLASSIFIED** 312: `assert_eq!( artifact.policy_resolution.runtime_only_record_count, artifact.policy_resolution.resolved_record_count )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 316: `assert_eq!(artifact.policy_resolution.custom_policy_record_count, 0)` - semantic boundary still requires direct review; no move authorized.

#### `merge_planning_digest_basis_carries_policy_ownership_surface_rows`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_a.rs:320`.

- **UNCLASSIFIED** 347: `assert_eq!( artifact.digest_basis.policy.records.len(), artifact.digest_basis.policy.proof_boundaries.len() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 351: `assert!(artifact .digest_basis .policy .proof_boundaries .iter() .all(|boundary| boundary.ownership_surface == MergePolicyOwnershipSurface::RuntimeOnly))` - semantic boundary still requires direct review; no move authorized.

### `tests/history/milestone_7d_phase_b/deletion_block_reasons.rs`

#### `lowered_plan_preserves_source_deleted_target_live_block_reason`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_b/deletion_block_reasons.rs:13`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/history/milestone_7d_phase_b/deletion_block_reasons.rs:80 assert_blocked_deletion_lowering`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `lowered_plan_preserves_source_live_target_deleted_block_reason`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_b/deletion_block_reasons.rs:33`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/history/milestone_7d_phase_b/deletion_block_reasons.rs:80 assert_blocked_deletion_lowering`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `lowered_plan_preserves_deleted_vs_modified_block_reason`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_b/deletion_block_reasons.rs:60`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/history/milestone_7d_phase_b/deletion_block_reasons.rs:80 assert_blocked_deletion_lowering`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

### `tests/history/milestone_7d_phase_b/executable_admission.rs`

#### `admitted_source_addition_carries_executable_class`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_b/executable_admission.rs:13`.

- **UNCLASSIFIED** 48: `assert_eq!( lowered.resolution_class, MergeResolutionClass::SourceOnlyAddition )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 52: `assert_eq!( lowered.executable_class, Some(MergeExecutableClass::AdoptSourceRecord) )` - semantic boundary still requires direct review; no move authorized.

#### `compile_rejects_corrupted_non_executable_resolution_class`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_b/executable_admission.rs:59`.

- C4 action 84: `let execution_ready = prepared.execution_ready_plan_mut_for_test();`.
- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.

### `tests/history/milestone_7d_phase_b/policy_boundaries.rs`

#### `lowered_plan_carries_explicit_manual_resolution_policy_boundary_for_generic_denial`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_b/policy_boundaries.rs:15`.

- **UNCLASSIFIED** 45: `assert_eq!( lowered.policy_proof_boundary.ownership_surface, MergePolicyOwnershipSurface::RuntimeOnly )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 49: `assert_eq!( lowered.policy_proof_boundary.decision_boundary, MergePolicyDecisionBoundary::RequiresManualResolution { class: MergeManualResolutionClass::GenericRuntimeConflict, } )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 55: `assert_eq!( artifact.digest_basis.policy.proof_boundaries[lowered_index], lowered.policy_proof_boundary )` - semantic boundary still requires direct review; no move authorized.

#### `lowered_plan_carries_explicit_hard_reject_policy_boundary_for_fail_on_conflict`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_b/policy_boundaries.rs:62`.

- **UNCLASSIFIED** 99: `assert_eq!( lowered.policy_proof_boundary.ownership_surface, MergePolicyOwnershipSurface::RuntimeOnly )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 103: `assert_eq!( lowered.policy_proof_boundary.decision_boundary, MergePolicyDecisionBoundary::Reject { class: MergePolicyRejectClass::BuiltInFailOnConflict, } )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 109: `assert_eq!( artifact.digest_basis.policy.proof_boundaries[lowered_index], lowered.policy_proof_boundary )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 113: `assert!(matches!( lowered.record_decision, crate::facade::merge::LoweredRecordDecision::Reject(_) ))` - semantic boundary still requires direct review; no move authorized.

### `tests/history/milestone_7d_phase_c.rs`

#### `promoted_deleted_on_both_sides_compiles_to_an_explicit_executable_record_variant`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_c.rs:85`.

- **UNCLASSIFIED** 93: `assert_eq!(plan.record_plans.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 96: `assert_eq!( plan.provenance.resolution_class, MergeResolutionClass::Deletion(DeletionExecutionClass::DeletedOnBothSides) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 100: `assert_eq!( plan.provenance.executable_class, MergeExecutableClass::ConvergeDeletedOnBothSides )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 104: `assert_eq!( plan.semantics, crate::merge::data::DeletedOnBothSidesSemantics::AuthoritativeMutualDeletionConvergence )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 108: `assert_eq!( plan.lineage_continuity, crate::merge::data::MergeLineageContinuityVerdict::Unchanged )` - semantic boundary still requires direct review; no move authorized.

#### `promoted_deleted_on_both_sides_derives_zero_mutation_intent_execution_plan`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_c.rs:118`.

- **UNCLASSIFIED** 127: `assert_eq!(plan.structural_summary.executed_record_count, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 128: `assert_eq!( plan.structural_summary .converged_deleted_on_both_sides_count, 1 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 133: `assert_eq!( plan.structural_summary .deleted_on_both_sides_lineage_unchanged_count, 1 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 138: `assert_eq!(plan.structural_summary.emitted_mutation_intent_count, 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 139: `assert!(plan.merged_plan.merged_intents.is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 140: `assert_eq!( plan.merge_execution_summary .converged_deleted_on_both_sides_count, 1 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 145: `assert_eq!( plan.merge_execution_summary .deleted_on_both_sides_lineage_unchanged_count, 1 )` - semantic boundary still requires direct review; no move authorized.

#### `promoted_deleted_on_both_sides_executes_through_authoritative_merge_publication`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_c.rs:153`.

- **UNCLASSIFIED** 169: `assert_eq!(merge.structural_summary.executed_record_count, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 170: `assert_eq!( merge .structural_summary .converged_deleted_on_both_sides_count, 1 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 176: `assert_eq!( merge .structural_summary .deleted_on_both_sides_lineage_unchanged_count, 1 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 182: `assert_eq!(merge.structural_summary.emitted_mutation_intent_count, 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 183: `assert!(envelope.merged_plan.merged_intents.is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 191: `assert_eq!( diagnostic_field(summary_entry, "converged_deleted_on_both_sides_count"), &RelationalDiagnosticValue::Unsigned(1) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 195: `assert_eq!( diagnostic_field( summary_entry, "deleted_on_both_sides_lineage_unchanged_count" ), &RelationalDiagnosticValue::Unsigned(1) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 226: `assert!(matches!( diagnostic_field(record_entry, "equality_witness_digest"), RelationalDiagnosticValue::String(_) ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 230: `assert_eq!( diagnostic_field(record_entry, "deletion_semantics"), &RelationalDiagnosticValue::String("AuthoritativeMutualDeletionConvergence".to_string()) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 234: `assert_eq!( diagnostic_field(record_entry, "lineage_continuity"), &RelationalDiagnosticValue::String("Unchanged".to_string()) )` - semantic boundary still requires direct review; no move authorized.

#### `real_feature_branch_delete_after_main_delete_is_authorable_and_classifies_as_deleted_on_both_sides`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_c.rs:241`.

- **UNCLASSIFIED** 275: `assert_eq!( lowered.resolution_class, MergeResolutionClass::Deletion(DeletionExecutionClass::DeletedOnBothSides) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 279: `assert_eq!( lowered.executable_class, Some(MergeExecutableClass::ConvergeDeletedOnBothSides) )` - semantic boundary still requires direct review; no move authorized.

#### `branch_local_delete_allowance_does_not_make_same_branch_stale_delete_legal`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_c.rs:286`.

- **UNCLASSIFIED** 304: `assert_eq!( error.code(), crate::facade::diagnostics::DiagnosticCode::StaleHandle )` - semantic boundary still requires direct review; no move authorized.

### `tests/history/milestone_7d_phase_d/deleted_convergence.rs`

#### `deleted_on_both_sides_merge_commit_has_replay_and_recovery_parity`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_d/deleted_convergence.rs:4`.

- **UNCLASSIFIED** 28: `assert_eq!(merge.structural_summary.executed_record_count, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 29: `assert_eq!( merge .structural_summary .converged_deleted_on_both_sides_count, 1 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 35: `assert_eq!( merge .structural_summary .deleted_on_both_sides_lineage_unchanged_count, 1 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 41: `assert_eq!(merge.structural_summary.emitted_mutation_intent_count, 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 63: `assert!( replay.failure.is_none(), "replay certification failure: {replay:?}" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 82: `assert_eq!(live_truth.visible_truth, recovered_truth.visible_truth)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 83: `assert_eq!( live_truth.entity_history_digests, recovered_truth.entity_history_digests )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 94: `assert_eq!( diagnostic_field(summary_entry, "converged_deleted_on_both_sides_count"), &RelationalDiagnosticValue::Unsigned(1) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 98: `assert_eq!( diagnostic_field( summary_entry, "deleted_on_both_sides_lineage_unchanged_count" ), &RelationalDiagnosticValue::Unsigned(1) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 105: `assert_eq!( diagnostic_field(summary_entry, "execution_digest"), &RelationalDiagnosticValue::String(merge.execution_summary.execution_digest.clone()) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 109: `assert_eq!( diagnostic_field(summary_entry, "diagnostics_digest"), &RelationalDiagnosticValue::String(merge.execution_summary.diagnostics_digest.clone()) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 139: `assert_eq!( diagnostic_field(record_entry, "lineage_continuity"), &RelationalDiagnosticValue::String("Unchanged".to_string()) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 143: `assert!(matches!( diagnostic_field(record_entry, "equality_witness_digest"), RelationalDiagnosticValue::String(_) ))` - semantic boundary still requires direct review; no move authorized.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/support.rs:138 capture_aspect_truth_bundle`, `crates/worth-relational/src/tests/support/truth_comparison.rs:60 assert_durable_commit_meaning_eq`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

### `tests/history/milestone_7d_phase_d/mod.rs`

No direct test declarations. Executing helper roots and claims:


### `tests/history/milestone_7d_phase_d/prepared_merge_denials.rs`

#### `deleted_on_both_sides_prepared_merge_rejects_target_head_drift`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_d/prepared_merge_denials.rs:4`.

- **UNCLASSIFIED** 28: `assert_eq!(branch, BranchId("main".to_string()))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 45: `assert!(failure_artifact.entries.iter().any(|entry| { entry.code == DiagnosticCode::DeterministicMergeViolation && diagnostic_field(entry, "target_branch") == &RelationalDiagnosticValue::BranchId(BranchId("main".to_string())) && diagnostic_field(entry, "source_branch") == &RelationalDiagnosticValue::BranchId(BranchId("feature".to_string())) }))` - semantic boundary still requires direct review; no move authorized.

#### `deleted_on_both_sides_prepared_merge_rejects_schema_semantic_drift`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_d/prepared_merge_denials.rs:55`.

- C4 action 75: `runtime.set_schema_registry_for_test(drifted_schema_registry());`.
- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.

#### `non_executable_deletion_denial_is_stable_across_recovery`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_d/prepared_merge_denials.rs:84`.

- **UNCLASSIFIED** 146: `assert_eq!( live_record.resolution_class, MergeResolutionClass::Deletion(DeletionExecutionClass::DeletedVsModified) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 150: `assert_eq!( live_record.blocked_reason, Some(LoweredMergeBlockedReason::DeletedVsModified) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 154: `assert_eq!(live_record.executable_class, None)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 155: `assert_eq!(live_record, recovered_record)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 156: `assert_eq!( live_artifact.digest_basis.lowered_plan.denial_bundle_kinds[live_index], Some(LoweredRecordDenialKind::BlockedDeletedVsModified) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 160: `assert_eq!( recovered_artifact .digest_basis .lowered_plan .denial_bundle_kinds[recovered_index], Some(LoweredRecordDenialKind::BlockedDeletedVsModified) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 167: `assert_eq!( live_artifact.digest_basis.lowered_plan, recovered_artifact.digest_basis.lowered_plan )` - semantic boundary still requires direct review; no move authorized.

### `tests/history/milestone_7d_phase_d/topology_local_rewire.rs`

#### `topology_endpoint_divergence_denial_is_stable_across_recovery`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_d/topology_local_rewire.rs:4`.

- **UNCLASSIFIED** 125: `assert_eq!( live_record.blocked_reason, Some(LoweredMergeBlockedReason::RelationEndpointRewiredEscalated) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 129: `assert_eq!( live_classification .relation_evidence .as_ref() .expect("relation evidence") .propagation, RelationConflictPropagation::RelationLocalRewireCandidate )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 137: `assert_eq!( live_classification .relation_evidence .as_ref() .expect("relation evidence") .topology_neighborhood_records .len(), 2 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 146: `assert_eq!( live_classification .relation_evidence .as_ref() .expect("relation evidence") .topology_neighborhood_rewired_records .len(), 1 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 155: `assert_eq!( live_classification .relation_evidence .as_ref() .expect("relation evidence") .topology_region_conflict_reason, None )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 163: `assert_eq!(live_record.executable_class, None)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 164: `assert_eq!(live_record, recovered_record)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 165: `assert_eq!( live_artifact.digest_basis.lowered_plan.denial_bundle_kinds[live_index], Some(LoweredRecordDenialKind::BlockedRelationEndpointRewiredEscalated) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 169: `assert_eq!( recovered_artifact .digest_basis .lowered_plan .denial_bundle_kinds[recovered_index], Some(LoweredRecordDenialKind::BlockedRelationEndpointRewiredEscalated) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 176: `assert_eq!( live_artifact.digest_basis.lowered_plan, recovered_artifact.digest_basis.lowered_plan )` - semantic boundary still requires direct review; no move authorized.

#### `disjoint_rewire_neighborhoods_do_not_escalate_to_topology_region_conflict`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_d/topology_local_rewire.rs:183`.

- **UNCLASSIFIED** 271: `assert_eq!(local_topology_records.len(), 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 272: `assert_eq!( artifact .lowered_plan .records .iter() .filter(|record| { matches!( record.resolution_class, MergeResolutionClass::Topology(TopologyExecutionClass::TopologyRegionConflict) ) }) .count(), 0 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 288: `assert_eq!( record.blocked_reason, Some(LoweredMergeBlockedReason::RelationEndpointRewiredEscalated) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 302: `assert_eq!( evidence.propagation, RelationConflictPropagation::RelationLocalRewireCandidate )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 306: `assert_eq!(evidence.topology_region_conflict_reason, None)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 307: `assert_eq!(evidence.topology_neighborhood_rewired_records.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 308: `assert_eq!(evidence.topology_neighborhood_records.len(), 2)` - semantic boundary still requires direct review; no move authorized.
- **C2** 312: `assert_eq!(counters.merge_topology_region_conflicts_detected, 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 313: `assert_eq!(counters.merge_topology_region_records_escalated, 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.

### `tests/history/milestone_7d_phase_d/topology_local_rewire/neighborhood_detection.rs`

#### `unrelated_relation_additions_do_not_inflate_topology_region_detection_counters`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_d/topology_local_rewire/neighborhood_detection.rs:4`.

- **UNCLASSIFIED** 89: `assert_eq!(local_topology_records.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **C2** 92: `assert_eq!(counters.merge_topology_relation_candidates_scoped, 2)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 93: `assert_eq!(counters.merge_topology_endpoint_incidences_scoped, 5)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 94: `assert_eq!(counters.merge_topology_region_conflicts_detected, 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 95: `assert_eq!(counters.merge_topology_region_records_escalated, 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.

### `tests/history/milestone_7d_phase_d/topology_region.rs`

#### `topology_region_conflict_detection_reports_bounded_neighborhood_counters`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_d/topology_region.rs:4`.

- **UNCLASSIFIED** 79: `assert_eq!(topology_records.len(), 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 81: `assert_eq!( record.blocked_reason, Some(LoweredMergeBlockedReason::TopologyRegionConflict) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 97: `assert_eq!(topology_classifications.len(), 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 103: `assert_eq!( evidence.propagation, RelationConflictPropagation::EscalatesToTopologyRegionConflict )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 107: `assert_eq!( evidence.topology_region_conflict_reason, Some(TopologyRegionConflictReason::ConnectedRewireNeighborhood) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 121: `assert_eq!(neighborhood.len(), 4)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 122: `assert!(topology_record_refs.is_subset(&neighborhood))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 123: `assert_eq!(rewired, topology_record_refs)` - semantic boundary still requires direct review; no move authorized.
- **C2** 127: `assert_eq!(counters.merge_topology_relation_candidates_scoped, 4)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 128: `assert!(counters.merge_topology_endpoint_incidences_scoped >= 8)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 129: `assert_eq!(counters.merge_topology_region_conflicts_detected, 1)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 130: `assert_eq!(counters.merge_topology_region_records_escalated, 2)` - actual residency, identity, pin, allocation, dictionary or cost observation.

#### `topology_region_conflict_denial_is_stable_across_recovery`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_d/topology_region.rs:134`.

- **UNCLASSIFIED** 206: `assert_eq!(live_topology_records.len(), 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 217: `assert_eq!(live_classifications.len(), 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 223: `assert_eq!( evidence.propagation, RelationConflictPropagation::EscalatesToTopologyRegionConflict )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 227: `assert_eq!( evidence.topology_region_conflict_reason, Some(TopologyRegionConflictReason::ConnectedRewireNeighborhood) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 231: `assert_eq!(evidence.topology_neighborhood_records.len(), 4)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 232: `assert_eq!(evidence.topology_neighborhood_rewired_records.len(), 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 251: `assert_eq!( live_artifact.digest_basis.conflict, recovered_artifact.digest_basis.conflict )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 255: `assert_eq!( live_artifact.digest_basis.lowered_plan, recovered_artifact.digest_basis.lowered_plan )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 259: `assert_eq!( live_artifact.conflict_classification.classifications, recovered_artifact.conflict_classification.classifications )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 263: `assert_eq!( live_artifact.lowered_plan.records, recovered_artifact.lowered_plan.records )` - semantic boundary still requires direct review; no move authorized.

### `tests/history/milestone_7d_phase_e.rs`

#### `inspection_input_round_trips_deleted_vs_modified_without_host_projection`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_e.rs:22`.

- **UNCLASSIFIED** 52: `assert_eq!(input.request(), &normalized_request)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 53: `assert_eq!(input.lowered_plan(), &planning.lowered_plan)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 54: `assert_eq!(from_input, from_access)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 55: `assert_eq!(from_input.request(), &normalized_request)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 56: `assert_eq!(from_input.rows().len(), planning.lowered_plan.records.len())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 63: `assert_eq!( denied.admission(), RelationalMergeInspectionAdmission::ExecutionDenied )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 67: `assert_eq!( denied.blocked_reason(), Some(LoweredMergeBlockedReason::DeletedVsModified) )` - semantic boundary still requires direct review; no move authorized.

#### `execution_surface_preserves_topology_region_conflict_authority_rows`

Source: `crates/worth-relational/src/tests/history/milestone_7d_phase_e.rs:74`.

- **UNCLASSIFIED** 145: `assert_eq!(topology_rows.len(), 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 146: `assert!(topology_rows .iter() .all(|row| { row.admission() == RelationalMergeInspectionAdmission::ExecutionDenied }))` - semantic boundary still requires direct review; no move authorized.

### `tests/history/milestone_7e_phase_a.rs`

#### `merge_branch_basis_matches_planning_artifact_and_history_scope_authority`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_a.rs:16`.

- **UNCLASSIFIED** 50: `assert_eq!( artifact.ancestry.merge_base_commit_id, artifact.branch_basis.merge_base().commit().commit_id )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 54: `assert_eq!( artifact.branch_basis.merge_base().commit().commit_id, artifact.merge_base().commit().commit_id )` - semantic boundary still requires direct review; no move authorized.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/history/milestone_7e_phase_a.rs:187 assert_exact_branch_basis`, `crates/worth-relational/src/tests/history/milestone_7e_phase_b.rs:276 assert_exact_branch_basis`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `merge_branch_basis_denial_happens_before_planning_for_missing_or_disconnected_branches`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_a.rs:61`.

- **UNCLASSIFIED** 74: `assert!(matches!( missing_source, crate::history::data::RelationalMergeBranchBasisDenial::MissingSourceHead { branch_id } if branch_id == missing_branch ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 79: `assert!(matches!( runtime .merge() .inspect_planning_scope(execution, MergePlanningRequest::new( main_branch.clone(), missing_branch.clone(), MergeIntent::ReconcileIntoTarget, )), Err(crate::merge::data::MergePlanningError::MissingSourceHead { branch_id }) if branch_id == missing_branch ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 97: `assert!(matches!( missing_merge_base, crate::history::data::RelationalMergeBranchBasisDenial::MissingMergeBase { source_branch, target_branch, } if source_branch == orphan_branch && target_branch == main_branch ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 104: `assert_eq!( runtime.merge().inspect_planning_scope( execution, MergePlanningRequest::new( main_branch.clone(), orphan_branch, MergeIntent::ReconcileIntoTarget, ) ), Err( crate::merge::data::MergePlanningError::RequestNormalization( crate::merge::data::RelationalMergeRequestNormalizationDenial::OwnerBinding( crate::merge::data::RelationalMergeRequestBindingDenial::IdentityMismatch, ), ) ), "a descriptive orphan with no coherent exact root is denied before ancestry planning", )` - semantic boundary still requires direct review; no move authorized.

#### `merge_branch_basis_survives_published_merge_outcome_and_durability_recovery`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_a.rs:125`.

- **UNCLASSIFIED** 184: `assert_eq!(live_authority, recovered_authority)` - semantic boundary still requires direct review; no move authorized.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/history/milestone_7e_phase_a.rs:187 assert_exact_branch_basis`, `crates/worth-relational/src/tests/history/milestone_7e_phase_b.rs:276 assert_exact_branch_basis`, `crates/worth-relational/src/tests/history/milestone_7e_phase_a.rs:213 assert_summary_projects_exact_basis`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

### `tests/history/milestone_7e_phase_b.rs`

#### `foundational_current_basis_lowering_preserves_exact_relational_basis_truth`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_b.rs:26`.

- **UNCLASSIFIED** 49: `assert_eq!( lowered.strong_basis().payload().domain(), CanonicalBasisDomain::BoundaryArtifact )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 53: `assert_eq!( lowered.materialized().category(), FoundationalBoundaryArtifactCategory::Artifact )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 57: `assert_eq!( lowered.materialized().role(), FoundationalBoundaryArtifactRole::AuthoritativeCurrent )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 61: `assert_eq!( lowered.materialized().source(), FoundationalBoundaryMaterializationSource::NativeAuthority )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 65: `assert_eq!( lowered.materialized().seam(), FoundationalBoundaryMaterializationSeam::BoundaryExchange )` - semantic boundary still requires direct review; no move authorized.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/history/milestone_7e_phase_a.rs:187 assert_exact_branch_basis`, `crates/worth-relational/src/tests/history/milestone_7e_phase_b.rs:276 assert_exact_branch_basis`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `foundational_current_basis_bridge_and_readmission_preserve_exact_basis_truth`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_b.rs:73`.

- **UNCLASSIFIED** 97: `assert_eq!( bridged.materialized().source(), FoundationalBoundaryMaterializationSource::NativeAuthority )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 107: `assert_eq!( readmitted.strong_basis().payload().domain(), CanonicalBasisDomain::BoundaryArtifact )` - semantic boundary still requires direct review; no move authorized.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/history/milestone_7e_phase_a.rs:187 assert_exact_branch_basis`, `crates/worth-relational/src/tests/history/milestone_7e_phase_b.rs:276 assert_exact_branch_basis`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `foundational_current_basis_lowering_rejects_published_historical_basis_in_live_and_recovered_lanes`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_b.rs:115`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/history/milestone_7e_phase_b.rs:237 assert_historical_summary_basis_denies_as_current`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `foundational_current_basis_lowering_rejects_stale_relational_basis_before_publication`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_b.rs:147`.

- **UNCLASSIFIED** 178: `assert_eq!(retained_digest, stale_basis.basis_digest())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 179: `assert_ne!(current_digest, retained_digest)` - semantic boundary still requires direct review; no move authorized.

### `tests/history/milestone_7e_phase_c.rs`

#### `normalized_merge_request_equivalence_is_exact_across_authoring_lanes`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_c.rs:15`.

- **UNCLASSIFIED** 41: `assert_eq!(normalized_from_planning, normalized_from_execution)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 42: `assert_eq!(normalized_from_planning, explicit_specialist)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 43: `assert_eq!( normalized_from_planning.request_digest(), normalized_from_execution.request_digest() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 47: `assert_eq!( normalized_from_planning.request_digest(), explicit_specialist.request_digest() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 51: `assert_eq!( normalized_from_planning.family(), RelationalMergeRequestFamily::FullBranchReconciliation )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 55: `assert_eq!( normalized_from_planning.scope(), RelationalMergeScope::FullBranch )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 59: `assert_eq!( normalized_from_planning.correspondence_posture(), RelationalMergeCorrespondencePosture::Advisory )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 63: `assert_eq!( normalized_from_planning.schema_reconciliation_posture(), RelationalMergeSchemaReconciliationPosture::Participate )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 67: `assert_eq!( normalized_from_planning.topology_intent(), RelationalMergeTopologyIntent::PreserveTopologySemantics )` - semantic boundary still requires direct review; no move authorized.

#### `unsupported_request_posture_denies_before_history_or_planning_work`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_c.rs:74`.

- **UNCLASSIFIED** 75: `assert_eq!( NormalizedRelationalMergeRequest::admit_full_branch( BranchId("main".to_string()), BranchId("feature".to_string()), MergeIntent::ReconcileIntoTarget, RelationalMergeCorrespondencePosture::Strict, RelationalMergeSchemaReconciliationPosture::Participate, RelationalMergeTopologyIntent::PreserveTopologySemantics, ), Err( RelationalMergeRequestNormalizationDenial::UnsupportedCorrespondencePosture { posture: RelationalMergeCorrespondencePosture::Strict, }, ) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 90: `assert_eq!( NormalizedRelationalMergeRequest::admit_full_branch( BranchId("main".to_string()), BranchId("feature".to_string()), MergeIntent::ReconcileIntoTarget, RelationalMergeCorrespondencePosture::Advisory, RelationalMergeSchemaReconciliationPosture::RequireCompatibility, RelationalMergeTopologyIntent::PreserveTopologySemantics, ), Err( RelationalMergeRequestNormalizationDenial::UnsupportedSchemaReconciliationPosture { posture: RelationalMergeSchemaReconciliationPosture::RequireCompatibility, }, ) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 105: `assert_eq!( NormalizedRelationalMergeRequest::admit_full_branch( BranchId("main".to_string()), BranchId("feature".to_string()), MergeIntent::ReconcileIntoTarget, RelationalMergeCorrespondencePosture::Advisory, RelationalMergeSchemaReconciliationPosture::Participate, RelationalMergeTopologyIntent::RequireStrictTopologyStability, ), Err( RelationalMergeRequestNormalizationDenial::UnsupportedTopologyIntent { intent: RelationalMergeTopologyIntent::RequireStrictTopologyStability, } ) )` - semantic boundary still requires direct review; no move authorized.

#### `planning_execution_and_publication_consume_normalized_request_authority`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_c.rs:123`.

- **UNCLASSIFIED** 139: `assert_eq!(artifact.request, normalized)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 140: `assert_eq!( artifact.request.request_digest(), normalized.request_digest() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 144: `assert_eq!( artifact.digest_basis.request.target_branch, normalized.target_branch().clone() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 148: `assert_eq!( artifact.digest_basis.request.source_branch, normalized.source_branch().clone() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 152: `assert_eq!( artifact.digest_basis.request.correspondence_posture, normalized.correspondence_posture() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 156: `assert_eq!( artifact.digest_basis.request.schema_reconciliation_posture, normalized.schema_reconciliation_posture() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 160: `assert_eq!( artifact.digest_basis.request.topology_intent, normalized.topology_intent() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 164: `assert_eq!(artifact.inspection_input().request(), &normalized)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 169: `assert_eq!(prepared.request(), &normalized)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 170: `assert_eq!(prepared.artifact().request, normalized)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 171: `assert_eq!( prepared.execution_ready_plan().request.request_digest(), normalized.request_digest() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 175: `assert_eq!( prepared.bound_executable_plan().authority_binding.request, normalized )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 183: `assert_eq!(outcome.execution_summary.request, normalized)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 184: `assert_eq!( outcome.execution_summary.request.request_digest(), normalized.request_digest() )` - semantic boundary still requires direct review; no move authorized.

#### `normalized_request_deserialization_revalidates_admitted_truth`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_c.rs:191`.

- **UNCLASSIFIED** 204: `assert_eq!(decoded, normalized)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 219: `assert!(forged_result.is_err())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 235: `assert!(unsupported_result.is_err())` - semantic boundary still requires direct review; no move authorized.

### `tests/history/milestone_7e_phase_d.rs`

#### `foundational_lowering_preserves_exact_relational_request_truth`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_d.rs:14`.

- **UNCLASSIFIED** 25: `assert_eq!(lowered.normalized_request(), &normalized)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 26: `assert_eq!( lowered.normalized_request().request_digest(), normalized.request_digest() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 30: `assert_eq!( lowered.normalized_request().family(), RelationalMergeRequestFamily::FullBranchReconciliation )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 34: `assert_eq!( lowered.normalized_request().scope(), RelationalMergeScope::FullBranch )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 38: `assert_eq!( lowered.normalized_request().correspondence_posture(), RelationalMergeCorrespondencePosture::Advisory )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 42: `assert_eq!( lowered.normalized_request().schema_reconciliation_posture(), RelationalMergeSchemaReconciliationPosture::Participate )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 46: `assert_eq!( lowered.normalized_request().topology_intent(), RelationalMergeTopologyIntent::PreserveTopologySemantics )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 50: `assert_eq!( lowered.foundational_scope_family(), FoundationalMergeScopeFamily::FullBranch )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 54: `assert_eq!( lowered.foundational_intent(), FoundationalMergeIntent::ReconcileIntoTarget )` - semantic boundary still requires direct review; no move authorized.

#### `foundational_lowering_does_not_collapse_distinct_relational_meanings`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_d.rs:61`.

- **UNCLASSIFIED** 79: `assert_ne!(lowered_main_to_feature, lowered_release_to_hotfix)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 80: `assert_ne!( lowered_main_to_feature.normalized_request(), lowered_release_to_hotfix.normalized_request() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 84: `assert_eq!( lowered_main_to_feature.foundational_scope_family(), lowered_release_to_hotfix.foundational_scope_family() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 88: `assert_eq!( lowered_main_to_feature.foundational_intent(), lowered_release_to_hotfix.foundational_intent() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 92: `assert_ne!( lowered_main_to_feature.lowering_digest(), lowered_release_to_hotfix.lowering_digest() )` - semantic boundary still requires direct review; no move authorized.

#### `execution_authoring_and_explicit_normalization_lower_identically`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_d.rs:99`.

- **UNCLASSIFIED** 137: `assert_eq!(lowered_from_execution, lowered_from_explicit)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 138: `assert_eq!( lowered_from_execution.lowering_digest(), lowered_from_explicit.lowering_digest() )` - semantic boundary still requires direct review; no move authorized.

#### `foundational_lowering_uses_foundational_admission_vocabulary`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_d.rs:160`.

- **UNCLASSIFIED** 170: `assert_eq!( lowered.foundational_scope_family(), FoundationalMergeScopeFamily::FullBranch )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 174: `assert_eq!( lowered.foundational_intent(), FoundationalMergeIntent::ReconcileIntoTarget )` - semantic boundary still requires direct review; no move authorized.

### `tests/history/milestone_7e_phase_e.rs`

#### `retained_merge_proof_packet_preserves_exact_authority_truth`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_e.rs:12`.

- **UNCLASSIFIED** 25: `assert_eq!(proof_packet.request(), prepared.request())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 26: `assert_eq!( proof_packet.branch_basis(), &prepared.artifact().branch_basis )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 30: `assert_eq!( proof_packet.execution_digest(), prepared .bound_executable_plan() .authority_binding .executable_plan_digest )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 40: `assert_eq!( proof_packet.correspondence_witness_digest(), correspondence_witness.witness_digest() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 47: `assert_eq!( proof_packet.schema_reconciliation_witness_digest(), schema_witness.witness_digest() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 62: `assert_eq!( proof_packet.admitted_merge_surface(), expected_surface.as_slice() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 66: `assert_eq!( proof_packet.admission_posture(), crate::facade::merge::RelationalMergeProofPacketAdmissionPosture::ExecutionAdmitted )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 70: `assert!(!proof_packet.planning_digest().is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 71: `assert!(!proof_packet .foundational_request_lowering_digest() .is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 74: `assert!(!proof_packet.admitted_merge_surface_digest().is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 75: `assert!(!proof_packet.packet_digest().is_empty())` - semantic boundary still requires direct review; no move authorized.

#### `published_merge_outcome_retains_proof_packet_as_authority`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_e.rs:79`.

- **UNCLASSIFIED** 89: `assert_eq!( summary_packet, outcome.execution_summary.proof_packet.clone() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 93: `assert_eq!( summary_packet, live_authority.execution_summary.proof_packet.clone() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 97: `assert_eq!( outcome .execution_summary .correspondence_witness .witness_digest(), summary_packet.correspondence_witness_digest() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 104: `assert_eq!(summary_packet.request(), &outcome.execution_summary.request)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 105: `assert_eq!( summary_packet.branch_basis(), &outcome.execution_summary.branch_basis )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 109: `assert_eq!( summary_packet.execution_digest(), outcome.execution_summary.execution_digest )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 113: `assert_eq!( outcome.execution_summary.target_head_commit_id, summary_packet.branch_basis().target_head().commit_id )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 117: `assert_eq!( outcome.execution_summary.source_head_commit_id, summary_packet.branch_basis().source_head().commit_id )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 121: `assert_eq!( outcome.execution_summary.merge_base_commit_id, summary_packet .branch_basis() .merge_base() .commit() .commit_id )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 129: `assert!(outcome .execution_summary .retains_consistent_proof_packet_authority())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 132: `assert!(live_authority.retains_consistent_proof_packet_authority())` - semantic boundary still requires direct review; no move authorized.

#### `proof_packet_deserialization_rejects_forged_retained_truth`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_e.rs:136`.

- **UNCLASSIFIED** 152: `assert_eq!(decoded, proof_packet)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 162: `assert!(forged_digest_result.is_err())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 172: `assert!(forged_surface_result.is_err())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 192: `assert!(forged_surface_payload_result.is_err())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 217: `assert!(malformed_execution_digest_result.is_err())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 242: `assert!(malformed_planning_digest_result.is_err())` - semantic boundary still requires direct review; no move authorized.

#### `merge_proof_packet_survives_publication_and_recovery_without_drift`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_e.rs:246`.

- **UNCLASSIFIED** 260: `assert_eq!(live_packet, live_authority.execution_summary.proof_packet)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 261: `assert_eq!( live_packet, recovered_authority.execution_summary.proof_packet )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 265: `assert_eq!( live_packet.packet_digest(), recovered_authority .execution_summary .proof_packet .packet_digest() )` - semantic boundary still requires direct review; no move authorized.

### `tests/history/milestone_7e_phase_f.rs`

#### `merge_proof_packet_canonical_basis_parity_survives_publication_and_recovery`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_f.rs:17`.

- **UNCLASSIFIED** 40: `assert_eq!(live_basis, published_basis)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 41: `assert_eq!(live_basis, recovered_basis)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 42: `assert_eq!(entry_map(&live_basis), entry_map(&published_basis))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 43: `assert_eq!(entry_map(&live_basis), entry_map(&recovered_basis))` - semantic boundary still requires direct review; no move authorized.

#### `merge_proof_packet_canonical_basis_localizes_single_field_drift`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_f.rs:47`.

- **UNCLASSIFIED** 95: `assert_eq!( request_drift, vec![ "merge.packet.digest", "merge.request.digest", "merge.request_lowering.digest", ] )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 124: `assert_eq!( branch_basis_drift, vec!["merge.branch_basis.digest", "merge.packet.digest"] )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 145: `assert_eq!( planning_drift, vec!["merge.packet.digest", "merge.planning.digest"] )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 166: `assert_eq!( execution_drift, vec!["merge.execution.digest", "merge.packet.digest"] )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 196: `assert_eq!( surface_drift, vec![ "merge.admitted_surface.digest", "merge.admitted_surface.row.0.digest", "merge.packet.digest", ] )` - semantic boundary still requires direct review; no move authorized.

### `tests/history/milestone_7e_phase_g/mod.rs`

#### `retained_merge_correspondence_witness_preserves_exact_admitted_truth`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_g/mod.rs:25`.

- **UNCLASSIFIED** 38: `assert_eq!( witness.request_digest(), prepared.request().request_digest() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 42: `assert_eq!( witness.branch_basis_digest(), prepared.execution_ready_plan().basis.basis_digest() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 46: `assert_eq!( witness.rows().len(), prepared.artifact().identity_discovery.candidates.len() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 50: `assert_eq!(witness.admitted_rows().count(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 51: `assert!(witness.rows().iter().any(|row| row.posture() == RelationalMergeCorrespondenceWitnessPosture::Admitted && row.target_record().is_some() && matches!( row.reason(), crate::facade::merge::IdentityResolutionReason::ExactStorageIdentity )))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 58: `assert!(witness.rows().iter().any(|row| { row.posture() == RelationalMergeCorrespondenceWitnessPosture::UnavailableMissingTarget && row.target_record().is_none() }))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 62: `assert!(!witness.witness_digest().is_empty())` - semantic boundary still requires direct review; no move authorized.

#### `retained_merge_correspondence_witness_rejects_forged_or_incomplete_truth`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_g/mod.rs:66`.

- **UNCLASSIFIED** 82: `assert_eq!(decoded, witness)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 97: `assert!(forged_digest_result.is_err())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 107: `assert!(forged_witness_result.is_err())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 123: `assert!(forged_row_result.is_err())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 131: `assert!(forged_row_with_recomputed_witness_result.is_err())` - semantic boundary still requires direct review; no move authorized.

#### `retained_merge_correspondence_witness_preserves_authority_basis_distinctions`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_g/mod.rs:135`.

- **UNCLASSIFIED** 155: `assert_eq!( identical_witness, recomputed_witness(&witness, identical_rows) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 165: `assert_ne!(lineage_shifted_witness, identical_witness)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 166: `assert_ne!( lineage_shifted_witness.witness_digest(), identical_witness.witness_digest() )` - semantic boundary still requires direct review; no move authorized.

#### `retained_merge_correspondence_witness_preserves_runtime_declared_key_ambiguity_as_denial`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_g/mod.rs:173`.

- **UNCLASSIFIED** 208: `assert!(matches!( denied_row.reason(), crate::facade::merge::IdentityResolutionReason::DeclaredBasisAmbiguousVisibleTargetMatch ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 212: `assert!(matches!( denied_row.authority_basis(), crate::facade::merge::IdentityBasisKind::DeclaredKeySet(_) ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 216: `assert!(denied_row.target_record().is_none())` - semantic boundary still requires direct review; no move authorized.

#### `retained_merge_correspondence_witness_survives_publication_and_recovery`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_g/mod.rs:220`.

- **UNCLASSIFIED** 234: `assert_eq!( live_witness, live_authority.execution_summary.correspondence_witness )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 238: `assert_eq!( live_witness, recovered_authority.execution_summary.correspondence_witness )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 242: `assert_eq!( live_witness.witness_digest(), outcome .execution_summary .proof_packet .correspondence_witness_digest() )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 249: `assert!(outcome .execution_summary .retains_consistent_proof_packet_authority())` - semantic boundary still requires direct review; no move authorized.

### `tests/history/milestone_7e_phase_h/fixtures.rs`

No direct test declarations. Executing helper roots and claims:

- `durable_runtime_with_schema_declared_entity_policy` at 129: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/history/milestone_7e_phase_h/mod.rs:131 retained_schema_reconciliation_witness_preserves_runtime_additive_truth_across_authority_lanes`, `crates/worth-relational/src/tests/history/milestone_7e_phase_i/mod.rs:207 retained_merge_strategy_witness_survives_prepared_execution_publication_and_recovery`.

### `tests/history/milestone_7e_phase_h/mod.rs`

#### `retained_schema_reconciliation_witness_preserves_exact_category_and_linkage_truth`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_h/mod.rs:24`.

- **UNCLASSIFIED** 33: `assert_eq!( rows[0].classification(), SchemaReconciliationClassification::Additive )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 37: `assert_eq!( rows[0].policy(), Some(SchemaReconciliationPolicy::PreserveInformation) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 41: `assert_eq!(rows[0].denial(), None)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 42: `assert!(rows[0].correspondence_linkage().is_some())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 44: `assert_eq!( rows[1].classification(), SchemaReconciliationClassification::Narrowing )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 48: `assert_eq!(rows[1].policy(), None)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 49: `assert_eq!( rows[1].denial(), Some(RelationalSchemaReconciliationWitnessDenial::UnvalidatedSchemaCorrespondence) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 53: `assert_eq!( rows[1].posture(), RelationalSchemaReconciliationWitnessPosture::Denied )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 58: `assert_eq!( rows[2].classification(), SchemaReconciliationClassification::TypeContinuityDenied )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 62: `assert_eq!( rows[2].denial(), Some(RelationalSchemaReconciliationWitnessDenial::PolicyRejected) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 67: `assert_eq!( rows[3].classification(), SchemaReconciliationClassification::StructuralContinuityDenied )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 71: `assert_eq!( rows[3].denial(), Some(RelationalSchemaReconciliationWitnessDenial::StructuralIncompatible) )` - semantic boundary still requires direct review; no move authorized.

#### `retained_schema_reconciliation_witness_rejects_forged_or_rehashed_truth`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_h/mod.rs:78`.

- **UNCLASSIFIED** 83: `assert_eq!(decoded, witness)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 99: `assert!(forged_truth_result.is_err())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 115: `assert!(forged_posture_result.is_err())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 127: `assert!(forged_witness_result.is_err())` - semantic boundary still requires direct review; no move authorized.

#### `retained_schema_reconciliation_witness_preserves_runtime_additive_truth_across_authority_lanes`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_h/mod.rs:131`.

- **UNCLASSIFIED** 177: `assert_eq!( additive_row.policy(), Some(SchemaReconciliationPolicy::PreserveInformation) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 181: `assert_eq!(additive_row.denial(), None)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 182: `assert_eq!( additive_row.posture(), RelationalSchemaReconciliationWitnessPosture::Reconciled )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 186: `assert!(additive_row.correspondence_linkage().is_some())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 187: `assert!(!planning_witness.rows().is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 188: `assert_eq!( planning_witness, outcome.execution_summary.schema_reconciliation_witness )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 192: `assert_eq!( planning_witness, live_authority .execution_summary .schema_reconciliation_witness )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 198: `assert_eq!( planning_witness, recovered_authority .execution_summary .schema_reconciliation_witness )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 204: `assert_eq!( planning_witness.witness_digest(), outcome .execution_summary .proof_packet .schema_reconciliation_witness_digest() )` - semantic boundary still requires direct review; no move authorized.

#### `retained_schema_reconciliation_witness_preserves_runtime_narrowing_truth`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_h/mod.rs:214`.

- **UNCLASSIFIED** 249: `assert_eq!( row.policy(), Some(SchemaReconciliationPolicy::PreserveTargetContract) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 253: `assert_eq!(row.denial(), None)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 254: `assert_eq!( row.posture(), RelationalSchemaReconciliationWitnessPosture::Reconciled )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 258: `assert!(row.correspondence_linkage().is_some())` - semantic boundary still requires direct review; no move authorized.

#### `retained_schema_reconciliation_witness_keeps_runtime_type_denial_typed`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_h/mod.rs:262`.

- **UNCLASSIFIED** 298: `assert_eq!( row.denial(), Some(RelationalSchemaReconciliationWitnessDenial::PolicyRejected) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 302: `assert_eq!(row.policy(), None)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 303: `assert_eq!( row.posture(), RelationalSchemaReconciliationWitnessPosture::Denied )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 307: `assert!(row.correspondence_linkage().is_some())` - semantic boundary still requires direct review; no move authorized.

#### `retained_schema_reconciliation_witness_keeps_runtime_structural_denial_typed`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_h/mod.rs:311`.

- **UNCLASSIFIED** 363: `assert_eq!( row.denial(), Some(RelationalSchemaReconciliationWitnessDenial::StructuralIncompatible) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 367: `assert_eq!(row.policy(), None)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 368: `assert_eq!( row.posture(), RelationalSchemaReconciliationWitnessPosture::Denied )` - semantic boundary still requires direct review; no move authorized.

### `tests/history/milestone_7e_phase_i/fixtures.rs`

No direct test declarations. Executing helper roots and claims:

- `durable_runtime_with_schema_declared_entity_policy` at 28: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/history/milestone_7e_phase_h/mod.rs:131 retained_schema_reconciliation_witness_preserves_runtime_additive_truth_across_authority_lanes`, `crates/worth-relational/src/tests/history/milestone_7e_phase_i/mod.rs:207 retained_merge_strategy_witness_survives_prepared_execution_publication_and_recovery`.

### `tests/history/milestone_7e_phase_l/fixtures.rs`

No direct test declarations. Executing helper roots and claims:

- `runtime_with_schema_declared_entity_policy` at 27: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/history/milestone_7e_phase_h/mod.rs:214 retained_schema_reconciliation_witness_preserves_runtime_narrowing_truth`, `crates/worth-relational/src/tests/history/milestone_7e_phase_h/mod.rs:262 retained_schema_reconciliation_witness_keeps_runtime_type_denial_typed`, `crates/worth-relational/src/tests/history/milestone_7e_phase_i/mod.rs:18 retained_merge_strategy_witness_preserves_exact_strategy_truth`, `crates/worth-relational/src/tests/history/milestone_7e_phase_i/mod.rs:258 retained_merge_strategy_witness_live_authority_denies_internal_forged_strategy_truth`, `crates/worth-relational/src/tests/history/milestone_7e_phase_l/mod.rs:16 merge_support_inspection_witness_parity_survives_live_replay_recovery_and_compatibility_lane`, `crates/worth-relational/src/tests/history/milestone_7e_phase_l/mod.rs:67 merge_support_inspection_witness_denies_inconsistent_retained_proof_authority`, `crates/worth-relational/src/tests/history/milestone_7e_phase_l/mod.rs:109 merge_support_inspection_witness_denies_missing_required_proof_family`.

### `tests/history/milestone_7e_phase_m/fixtures.rs`

No direct test declarations. Executing helper roots and claims:

- `runtime_with_collaboration_merge_history` at 28: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/history/milestone_7e_phase_m/mod.rs:23 merge_collaboration_truth_replay_equivalence_survives_live_publication_and_recovery`, `crates/worth-relational/src/tests/history/milestone_7e_phase_m/mod.rs:97 merge_collaboration_support_surface_reflects_correspondence_posture_over_real_history`, `crates/worth-relational/src/tests/history/milestone_7e_phase_m/mod.rs:161 merge_collaboration_truth_localizes_single_family_drift`, `crates/worth-relational/src/tests/history/milestone_7e_phase_m/mod.rs:195 merge_collaboration_truth_denies_cross_family_witness_mismatch`, `crates/worth-relational/src/tests/history/milestone_7e_phase_m/mod.rs:253 merge_collaboration_truth_denies_summary_only_and_planner_shortcuts`.

### `tests/history/milestone_7e_phase_m/mod.rs`

#### `merge_collaboration_truth_replay_equivalence_survives_live_publication_and_recovery`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_m/mod.rs:23`.

- **UNCLASSIFIED** 44: `assert_eq!( live.authority.execution_summary, published.authority.execution_summary )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 48: `assert_eq!( live.authority.execution_summary, recovered_snapshot.authority.execution_summary )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 52: `assert_eq!( live.authority.execution_summary.request, published.authority.execution_summary.request )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 56: `assert_eq!( live.authority.execution_summary.branch_basis, published.authority.execution_summary.branch_basis )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 60: `assert_eq!( live.authority.execution_summary.correspondence_witness, published.authority.execution_summary.correspondence_witness )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 64: `assert_eq!( live.authority .execution_summary .schema_reconciliation_witness, published .authority .execution_summary .schema_reconciliation_witness )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 73: `assert_eq!( live.authority.execution_summary.strategy_witness, published.authority.execution_summary.strategy_witness )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 77: `assert_eq!(live.canonical_basis, published.canonical_basis)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 78: `assert_eq!( published.canonical_basis, recovered_snapshot.canonical_basis )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 82: `assert_eq!(live.support, published.support)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 83: `assert_eq!(published.support, recovered_snapshot.support)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 84: `assert!(published.support.rows().iter().any(|row| matches!( row, RelationalMergeSupportInspectionRow::Compatibility { posture: RelationalMergeSupportInspectionCompatibilityPosture::UnavailablePhaseDependency, absence: Some( RelationalMergeSupportInspectionAbsenceKind::MissingCompatibilityWitnessPhaseDependency ), .. } )))` - semantic boundary still requires direct review; no move authorized.

#### `merge_collaboration_support_surface_reflects_correspondence_posture_over_real_history`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_m/mod.rs:97`.

- **UNCLASSIFIED** 153: `assert_eq!(row.0, Some(witness.witness_digest()))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 154: `assert_eq!(row.1, witness.admitted_rows().count())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 155: `assert_eq!(row.2, denied_count)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 156: `assert_eq!(row.3, unavailable_count)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 157: `assert_eq!(row.4, witness.rows().first().map(|entry| entry.posture()))` - semantic boundary still requires direct review; no move authorized.

#### `merge_collaboration_truth_localizes_single_family_drift`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_m/mod.rs:161`.

- **UNCLASSIFIED** 169: `assert_eq!( collaboration_family_drift( &baseline, &branch_basis_drifted_authority(&baseline, alternate_branch_basis(&runtime)), ), vec!["branch_basis"] )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 176: `assert_eq!( collaboration_family_drift(&baseline, &proof_packet_drifted_authority(&baseline)), vec!["proof_packet"] )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 180: `assert_eq!( collaboration_family_drift(&baseline, &correspondence_drifted_authority(&baseline)), vec!["correspondence_witness"] )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 184: `assert_eq!( collaboration_family_drift(&baseline, &schema_drifted_authority(&baseline)), vec!["schema_reconciliation_witness"] )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 188: `assert_eq!( collaboration_family_drift(&baseline, &strategy_drifted_authority(&baseline)), vec!["strategy_witness"] )` - semantic boundary still requires direct review; no move authorized.

#### `merge_collaboration_truth_denies_cross_family_witness_mismatch`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_m/mod.rs:195`.

- C4 action 217: `crate::tests::support::attached_test_medium(&runtime).rewrite_commit_envelope(`.
- **C4** 211: `assert!(matches!( denied, Err(RelationalMergeSupportInspectionDenial::InconsistentRetainedProofAuthority) ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 245: `assert_eq!(error.class, RecoveryFailureClass::ReplayFailure)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 246: `assert_eq!(error.history_drift_class, None)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 247: `assert!(error .detail .contains("failed to reconstruct merge execution summary"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `merge_collaboration_truth_denies_summary_only_and_planner_shortcuts`

Source: `crates/worth-relational/src/tests/history/milestone_7e_phase_m/mod.rs:253`.

- C4 action 259: `crate::tests::support::attached_test_medium(&runtime).rewrite_commit_envelope(`.
- **C4** 285: `assert_eq!(error.class, RecoveryFailureClass::ReplayFailure)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 286: `assert_eq!(error.history_drift_class, None)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 287: `assert!(error .detail .contains("failed to reconstruct merge execution summary"))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `tests/history/milestone_8_5/replacement_lineage.rs`

No direct test declarations. Executing helper roots and claims:

- `run_replacement_strategy_certification` at 3: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/history/milestone_8_5/mod.rs:47 milestone_8_5_strategy_certification_preserves_merge_replay_and_recovery_truth`.
  - UNCLASSIFIED assertion 46: `assert_ne!(replacement_start_lineage, replacement_end_lineage)`.
  - UNCLASSIFIED assertion 52: `assert!(replacement_envelope.lineage_decision_log().iter().any( |decision| decision.kind == crate::lineage::data::LineageDecisionKind::ReplaceAccepted ))`.
  - UNCLASSIFIED assertion 67: `assert!( replacement_replay.failure.is_none(), "replacement replay failed: {replacement_replay:?}" )`.
  - UNCLASSIFIED assertion 71: `assert!(replacement_replay .compared_surfaces .contains(&ReplayObservableSurface::Strategy))`.
  - UNCLASSIFIED assertion 116: `assert!( recovered_replacement_replay.failure.is_none(), "recovered replacement replay failed: {recovered_replacement_replay:?}" )`.
  - UNCLASSIFIED assertion 153: `assert_eq!(recovered_bundle, live_bundle)`.

### `tests/history/milestone_8_5/replay_recovery.rs`

No direct test declarations. Executing helper roots and claims:


### `tests/history/milestone_8_5/replay_recovery/certification/executor_faults.rs`

No direct test declarations. Executing helper roots and claims:

- `certify` at 3: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): not established by scoped free-function call index; inspect qualified/dynamic callers.
  - UNCLASSIFIED assertion 29: `assert!(missing_executor_replay.mismatches.iter().any(|mismatch| { mismatch.class == ReplayMismatchClass::StrategyExecutorUnavailable && mismatch.surface == ReplayObservableSurface::Strategy }))`.
  - UNCLASSIFIED assertion 61: `assert!(failing_executor_replay.mismatches.iter().any(|mismatch| { mismatch.class == ReplayMismatchClass::StrategyExecutionFailure && mismatch.surface == ReplayObservableSurface::Strategy }))`.

### `tests/history/milestone_8_5/runtime_fixtures.rs`

No direct test declarations. Executing helper roots and claims:


### `tests/history/milestone_8_5/strategy_certification.rs`

No direct test declarations. Executing helper roots and claims:

- `run_strategy_merge_certification` at 3: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/history/milestone_8_5/mod.rs:47 milestone_8_5_strategy_certification_preserves_merge_replay_and_recovery_truth`.

### `tests/history/milestone_8_5_kubernetes/final_recovery.rs`

No direct test declarations. Executing helper roots and claims:

- `certify_final_recovery` at 22: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/history/milestone_8_5_kubernetes/mod.rs:46 milestone_8_5_kubernetes_style_intent_commit_certification_proves_staged_controller_outcomes`.
  - UNCLASSIFIED assertion 64: `assert_eq!( planning_evidence(&recovered_overlap_planning), live_bundle.overlap_conflict )`.
  - UNCLASSIFIED assertion 86: `assert_eq!( planning_evidence(&recovered_narrowed_planning), live_bundle.narrowed_non_conflict )`.
  - UNCLASSIFIED assertion 108: `assert_eq!( planning_evidence(&recovered_rebroadened_planning), live_bundle.rebroadened_conflict )`.
  - UNCLASSIFIED assertion 154: `assert_eq!( planning_evidence(&recovered_revalidated_planning), live_bundle.revalidated_converged_overlap )`.
  - UNCLASSIFIED assertion 163: `assert_eq!( KubernetesNoopEvidence { strategy_artifacts: recovered_revalidation_envelope .strategy_artifacts .as_ref() .expect("recovered revalidation strategy artifacts") .clone(), changed_record_count: recovered_revalidation_envelope .patch .authoritative_record_patches .len(), patch_record_count: recovered_revalidation_envelope .patch .authoritative_record_patches .len(), }, live_bundle.revalidation_noop )`.
  - UNCLASSIFIED assertion 181: `assert_eq!( recovered_broad_intent_replay, live_bundle.broad_intent_replay )`.
  - UNCLASSIFIED assertion 185: `assert_eq!( recovered_first_converge_replay, live_bundle.first_converge_replay )`.
  - UNCLASSIFIED assertion 189: `assert_eq!( recovered_rebroadened_intent_replay, live_bundle.rebroadened_intent_replay )`.
  - UNCLASSIFIED assertion 193: `assert_eq!( recovered_revalidation_replay, live_bundle.revalidation_noop_replay )`.
  - UNCLASSIFIED assertion 201: `assert_eq!( KubernetesBranchHeadEvidence { main: recovered.history().branch_head(&main_branch), controller: recovered.history().branch_head(&controller_branch), }, live_bundle.branch_heads )`.
  - UNCLASSIFIED assertion 208: `assert_eq!( KubernetesBranchVisibleTruthEvidence { main: visible_truth_for_branch(execution, &recovered, &main_branch, entity), controller: visible_truth_for_branch(execution, &recovered, &controller_branch, entity), }, live_bundle.visible_truth )`.

### `tests/history/milestone_8_5_kubernetes/replay_recovery.rs`

No direct test declarations. Executing helper roots and claims:

- `assert_strategy_replay_clean` at 23: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/history/milestone_8_5_kubernetes/mod.rs:46 milestone_8_5_kubernetes_style_intent_commit_certification_proves_staged_controller_outcomes`.
  - UNCLASSIFIED assertion 27: `assert!( replay .compared_surfaces .contains(&ReplayObservableSurface::Strategy), "expected strategy replay surface during {stage}: {replay:?}" )`.
  - UNCLASSIFIED assertion 33: `assert!( replay .mismatches .iter() .all(|mismatch| mismatch.surface != ReplayObservableSurface::Strategy), "unexpected strategy replay mismatch during {stage}: {replay:?}" )`.

### `tests/history/milestone_8_5_kubernetes/stage_sequence.rs`

No direct test declarations. Executing helper roots and claims:

- `run_kubernetes_style_certification` at 3: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/history/milestone_8_5_kubernetes/mod.rs:46 milestone_8_5_kubernetes_style_intent_commit_certification_proves_staged_controller_outcomes`.
  - UNCLASSIFIED assertion 67: `assert_eq!( planning_evidence(&recovered_overlap_planning), overlap_conflict )`.
  - UNCLASSIFIED assertion 112: `assert_eq!( idempotent_converge_commit .change_summary() .expect("idempotent change summary") .changed_record_count, 0 )`.
  - UNCLASSIFIED assertion 143: `assert_eq!( planning_evidence(&recovered_narrowed_planning), narrowed_non_conflict )`.
  - UNCLASSIFIED assertion 185: `assert!(rebroadened_intent_replay.failure.is_none())`.
  - UNCLASSIFIED assertion 186: `assert!(rebroadened_intent_replay .compared_surfaces .contains(&ReplayObservableSurface::Strategy))`.
  - UNCLASSIFIED assertion 201: `assert_eq!( planning_evidence(&recovered_rebroadened_planning), rebroadened_conflict )`.
  - UNCLASSIFIED assertion 211: `assert_eq!( recovered_rebroadened_intent_replay, rebroadened_intent_replay )`.
  - UNCLASSIFIED assertion 231: `assert_eq!( revalidation_commit .change_summary() .expect("revalidation change summary") .changed_record_count, 1 )`.

### `tests/history/milestone_8_5_kubernetes/strategy_runtime.rs`

No direct test declarations. Executing helper roots and claims:


### `tests/history/queries/aspect_history.rs`

#### `record_local_aspect_history_reads_committed_patch_truth`

Source: `crates/worth-relational/src/tests/history/queries/aspect_history.rs:5`.

- **UNCLASSIFIED** 28: `assert_eq!(history.len(), 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 29: `assert_eq!(traced.entries, history)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 30: `assert_eq!( traced.trace.requested_target, HistoryAspectQueryTarget::Entity(entity) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 34: `assert_eq!(traced.trace.returned_entries, 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 35: `assert_eq!(traced.trace.resolved_aspects, resolved_aspects)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 36: `assert_eq!( traced.trace.searched_commit_span, Some(AspectHistoryCommitSpan { first_commit_id: created.commit.commit_id, last_commit_id: updated.commit.commit_id, }) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 43: `assert_eq!(traced.trace.searched_lineage_event_span, None)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 44: `assert_eq!(traced.trace.traversed_lineage_events, 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 45: `assert_eq!(artifact.scope, DiagnosticsScope::History)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 46: `assert_eq!(artifact.kind, DiagnosticsArtifactKind::DetailedTrace)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 47: `assert_eq!(artifact.entries.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 48: `assert_eq!( artifact.entries[0].code, DiagnosticCode::AspectHistoryResolved )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 52: `assert_eq!( digest.requested_target, HistoryAspectQueryTarget::Entity(entity) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 56: `assert_eq!(digest.entry_count, 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 57: `assert_eq!(digest.resolved_aspects, resolved_aspects)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 59: `assert_eq!(history[0].origin.commit_id, created.commit.commit_id)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 60: `assert_eq!(history[0].origin.target, RecordRef::Entity(entity))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 61: `assert_eq!(history[0].origin.changed_aspects, resolved_aspects)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 62: `assert_eq!(history[1].origin.commit_id, updated.commit.commit_id)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 91: `assert_eq!(filtered.len(), 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 92: `assert_eq!(any_filtered.len(), 2)` - semantic boundary still requires direct review; no move authorized.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/support.rs:345 assert_direct_history_origin_invariants`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `aspect_history_projection_filter_matches_field_level_patch_locus`

Source: `crates/worth-relational/src/tests/history/queries/aspect_history.rs:96`.

- **UNCLASSIFIED** 178: `assert_eq!(status_history.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 179: `assert_eq!( status_history[0].origin.changed_aspects, ordered_aspect_keys([aspect_key("summary")]) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 183: `assert!(missing_sibling_history.is_empty())` - semantic boundary still requires direct review; no move authorized.

#### `bulk_like_aspect_history_filters_and_query_packets_stay_stable_after_recovery`

Source: `crates/worth-relational/src/tests/history/queries/aspect_history.rs:187`.

- **UNCLASSIFIED** 334: `assert_eq!(recovery.latest_commit, Some(delete_outcome.commit.clone()))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 335: `assert_eq!(planned_packet.target_count, leaves.len())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 336: `assert_eq!(before_recovery_reads.entities.len(), leaves.len())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 337: `assert_eq!(before_recovery_reads.entities, recovered_reads.entities)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 338: `assert_eq!(expected_entity_digests, recovered_entity_digests)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 339: `assert_eq!(expected_relation_digests, recovered_relation_digests)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 340: `assert_eq!( expected_lifecycle_relation_digests, recovered_lifecycle_relation_digests )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 344: `assert!(expected_entity_digests .iter() .all(|digest| digest.entry_count == 2))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 347: `assert!(expected_relation_digests .iter() .all(|digest| digest.entry_count == 1))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 350: `assert!(expected_lifecycle_relation_digests .iter() .all(|digest| digest.entry_count == 2))` - semantic boundary still requires direct review; no move authorized.

### `tests/history/replay/derived_index_and_workload.rs`

#### `replay_contract_reports_derived_index_drift_at_digest_layer_when_artifacts_are_tampered`

Source: `crates/worth-relational/src/tests/history/replay/derived_index_and_workload.rs:5`.

- C4 action 31: `assert!(runtime.history_authority().tamper_commit_envelope_for_test(`.
- C4 action 37: `.generations_mut_for_test()`.
- **C4** 31: `assert!(runtime.history_authority().tamper_commit_envelope_for_test( execution, commit.commit.commit_id, |envelope| { if let Some(generation) = envelope .derived_index_artifacts .generations_mut_for_test() .first_mut() { generation.status = crate::facade::indexes::DerivedIndexPublicationStatus::BuildFailed; } } ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 59: `assert_eq!(replay.failure, Some(ReplayFailureClass::ObservableMismatch))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 60: `assert!( replay.mismatches.iter().any(|mismatch| { mismatch.class == ReplayMismatchClass::DerivedIndexDrift && mismatch.surface == ReplayObservableSurface::DerivedIndexes && mismatch.verification_layer == crate::facade::replay::ReplayVerificationLayer::DigestParity }), "{:?}", replay.mismatches )` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `replay_and_recovery_preserve_aspect_bearing_truth_across_a_hostile_mixed_workload`

Source: `crates/worth-relational/src/tests/history/replay/derived_index_and_workload.rs:73`.

- **UNCLASSIFIED** 131: `assert!( runtime.replay().compare_outcome(&replay), "hostile replay mismatch: {replay:#?}" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 136: `assert!(replay_diagnostics .by_scope(DiagnosticsScope::Replay) .into_iter() .flat_map(|artifact| artifact.entries.iter()) .any(|entry| { entry.code == DiagnosticCode::CommitPublished && diagnostic_field_optional(entry, "verification_mode") == Some(&RelationalDiagnosticValue::string( "NormalRecoveryVerification", )) }))` - semantic boundary still requires direct review; no move authorized.
- **C2** 148: `assert!(replay_counters.replay_digest_parity_checks > 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 184: `assert!(recovered.replay().compare_outcome(&recovered_replay_check))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 192: `assert_eq!( recovered_replay_check.requested.commit_id, replace_outcome.commit.commit_id )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 196: `assert!(recovered_bundle.latest_patch.is_none())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 197: `assert!(recovered_bundle.latest_replay.is_none())` - semantic boundary still requires direct review; no move authorized.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/support/truth_comparison.rs:22 assert_recovered_commit_truth_matches`, `crates/worth-relational/src/tests/support.rs:138 capture_aspect_truth_bundle`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `hostile_commit_replay_equivalence_test`

Source: `crates/worth-relational/src/tests/history/replay/derived_index_and_workload.rs:201`.

- C4 action 222: `runtime.set_schema_registry_for_test(`.
- **C4** 299: `assert!( runtime.replay().compare_outcome(&replay), "hostile replay mismatch: {replay:#?}" )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 362: `assert!(recovered_replay_diagnostics .by_scope(DiagnosticsScope::Replay) .into_iter() .flat_map(|artifact| artifact.entries.iter()) .any(|entry| { entry.code == DiagnosticCode::CommitPublished && diagnostic_field_optional(entry, "verification_mode") == Some(&RelationalDiagnosticValue::string( "NormalRecoveryVerification", )) }))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 375: `assert_eq!(original_envelope, recovered_envelope)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 376: `assert_eq!(replay, recovered_replay)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 377: `assert_eq!( original_main_branch_head, recovered .history() .branch_head(&BranchId("main".to_string())) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 383: `assert_eq!( original_feature_branch_head, recovered .history() .branch_head(&BranchId("feature".to_string())) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 389: `assert_eq!(original_inspection, recovered_inspection)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C2** 391: `assert!(recovered_counters.replay_digest_parity_checks > 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/support/truth_comparison.rs:3 assert_stable_aspect_truth_bundle_eq`, `crates/worth-relational/src/tests/support.rs:138 capture_aspect_truth_bundle`, `crates/worth-relational/src/tests/support.rs:239 capture_inspection_truth_bundle`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

### `tests/inspection/mod.rs`

No direct test declarations. Executing helper roots and claims:


### `tests/inspection/structural_identity.rs`

#### `structural_identity_comparison_only_uses_fingerprint_truth`

Source: `crates/worth-relational/src/tests/inspection/structural_identity.rs:5`.

- **UNCLASSIFIED** 22: `assert_eq!( comparison.verdict, StructuralIdentityComparisonVerdict::IncomparableMissingFingerprint )` - semantic boundary still requires direct review; no move authorized.

#### `structural_identity_evidence_exposes_declared_fingerprint_and_lineage_for_entities_only`

Source: `crates/worth-relational/src/tests/inspection/structural_identity.rs:29`.

- C4 action 35: `assert!(runtime.set_entity_structural_identity_for_test(`.
- **C4** 35: `assert!(runtime.set_entity_structural_identity_for_test( entity, Some(StructuralFingerprint::new(Symbol(11), 101)), Some(LineageId(77)), ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 51: `assert_eq!( entity_evidence.structural_fingerprint, Some(StructuralFingerprint::new(Symbol(11), 101)) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 55: `assert_eq!(entity_evidence.lineage_id, Some(LineageId(77)))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 56: `assert!(entity_evidence.degradations.is_empty())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 69: `assert!(relation_evidence.structural_fingerprint.is_none())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 70: `assert!(relation_evidence.lineage_id.is_none())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 71: `assert!(relation_evidence .degradations .contains(&crate::facade::inspection::InspectionDegradation::MissingStructuralFingerprint))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 74: `assert!(relation_evidence .degradations .contains(&crate::facade::inspection::InspectionDegradation::MissingLineageIdentity))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `structural_identity_comparison_distinguishes_equal_mismatch_and_family_mismatch`

Source: `crates/worth-relational/src/tests/inspection/structural_identity.rs:80`.

- C4 action 89: `assert!(runtime.set_entity_structural_identity_for_test(`.
- C4 action 94: `assert!(runtime.set_entity_structural_identity_for_test(`.
- C4 action 99: `assert!(runtime.set_entity_structural_identity_for_test(`.
- C4 action 119: `assert!(runtime.set_entity_structural_identity_for_test(`.
- **C4** 89: `assert!(runtime.set_entity_structural_identity_for_test( left, Some(StructuralFingerprint::new(Symbol(21), 500)), Some(LineageId(1)), ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 94: `assert!(runtime.set_entity_structural_identity_for_test( right, Some(StructuralFingerprint::new(Symbol(21), 500)), Some(LineageId(2)), ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 99: `assert!(runtime.set_entity_structural_identity_for_test( other_family, Some(StructuralFingerprint::new(Symbol(22), 500)), Some(LineageId(3)), ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 114: `assert_eq!( equal.verdict, StructuralIdentityComparisonVerdict::EqualByFingerprint )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 119: `assert!(runtime.set_entity_structural_identity_for_test( right, Some(StructuralFingerprint::new(Symbol(21), 999)), Some(LineageId(2)), ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 133: `assert_eq!( mismatch.verdict, StructuralIdentityComparisonVerdict::NotEqualByFingerprint )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 147: `assert_eq!( family_mismatch.verdict, StructuralIdentityComparisonVerdict::IncomparableFingerprintFamilyMismatch )` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `structural_identity_query_is_family_scoped_and_entity_only`

Source: `crates/worth-relational/src/tests/inspection/structural_identity.rs:154`.

- C4 action 164: `assert!(runtime.set_entity_structural_identity_for_test(`.
- C4 action 169: `assert!(runtime.set_entity_structural_identity_for_test(`.
- C4 action 174: `assert!(runtime.set_entity_structural_identity_for_test(`.
- **C4** 164: `assert!(runtime.set_entity_structural_identity_for_test( left, Some(StructuralFingerprint::new(Symbol(31), 1)), Some(LineageId(10)), ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 169: `assert!(runtime.set_entity_structural_identity_for_test( right, Some(StructuralFingerprint::new(Symbol(31), 2)), Some(LineageId(11)), ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 174: `assert!(runtime.set_entity_structural_identity_for_test( ignored, Some(StructuralFingerprint::new(Symbol(32), 3)), Some(LineageId(12)), ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 192: `assert_eq!(queried.len(), 2)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 193: `assert!(queried.iter().all(|evidence| evidence.record_class == crate::facade::inspection::InspectionRecordClass::Entity))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 195: `assert!(queried.iter().all(|evidence| { evidence .structural_fingerprint .is_some_and(|fingerprint| fingerprint.family == Symbol(31)) }))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `structural_identity_historical_scope_does_not_leak_reused_slot_sidecars`

Source: `crates/worth-relational/src/tests/inspection/structural_identity.rs:203`.

- C4 action 210: `assert!(runtime.set_entity_structural_identity_for_test(`.
- **C4** 210: `assert!(runtime.set_entity_structural_identity_for_test( original_entity, Some(StructuralFingerprint::new(Symbol(41), 111)), Some(LineageId(41)), ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 222: `assert_eq!(original_entity.local_slot, replacement_entity.local_slot)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 243: `assert!(historical.structural_fingerprint.is_none())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 244: `assert!(historical.lineage_id.is_none())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 245: `assert!(historical .degradations .contains(&crate::facade::inspection::InspectionDegradation::MissingStructuralFingerprint))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 248: `assert!(historical .degradations .contains(&crate::facade::inspection::InspectionDegradation::MissingLineageIdentity))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 251: `assert_eq!( current.structural_fingerprint, Some(StructuralFingerprint::new(Symbol(42), 222)) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 255: `assert_eq!(current.lineage_id, Some(LineageId(42)))` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `structural_identity_recovery_preserves_current_evidence_and_queries`

Source: `crates/worth-relational/src/tests/inspection/structural_identity.rs:259`.

- C4 action 272: `assert!(runtime.set_entity_structural_identity_for_test(`.
- C4 action 277: `assert!(runtime.set_entity_structural_identity_for_test(`.
- **C4** 272: `assert!(runtime.set_entity_structural_identity_for_test( left, Some(StructuralFingerprint::new(family, 1001)), Some(LineageId(501)), ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 277: `assert!(runtime.set_entity_structural_identity_for_test( right, Some(StructuralFingerprint::new(family, 1002)), Some(LineageId(502)), ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 337: `assert_eq!(expected_left, actual_left)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 338: `assert_eq!(expected_query, actual_query)` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `inspection_truth_bundle_recovery_parity_holds_for_current_and_historical_surfaces`

Source: `crates/worth-relational/src/tests/inspection/structural_identity.rs:342`.

- **UNCLASSIFIED** 378: `assert_eq!(expected, actual)` - semantic boundary still requires direct review; no move authorized.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/support.rs:239 capture_inspection_truth_bundle`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

### `tests/lineage/resolution.rs`

#### `historical_lineage_resolution_follows_replace_events`

Source: `crates/worth-relational/src/tests/lineage/resolution.rs:18`.

- **UNCLASSIFIED** 65: `assert_eq!(resolution.start, start_lineage)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 66: `assert_eq!( resolution.boundedness_basis, HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 70: `assert_eq!(resolution.traversed_event_ids.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 71: `assert_eq!(resolution.resolved.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 72: `assert_ne!(resolution.resolved[0], start_lineage)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 73: `assert_eq!(resolution.metrics.traversed_event_count, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 74: `assert!(resolution.metrics.event_visit_count >= 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 75: `assert_eq!(resolution.metrics.resolved_lineage_count, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 76: `assert_eq!( resolution.digest_basis().digest_mode(), HistoricalResolutionDigestMode::ExactDigestCanonicalOrder )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 80: `assert_eq!(resolution.digest_basis().requested_start(), start_lineage)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 81: `assert_eq!( resolution.digest_basis().canonical_traversed_event_ids(), resolution.traversed_event_ids )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 85: `assert_eq!(resolution.trace.digest_basis(), resolution.digest_basis())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 86: `assert_eq!( runtime .lineage_access() .graph( execution, LineageGraphRequest { branch_id: BranchId("main".to_string()), traversal_basis: LineageGraphTraversalBasis::FullBranchGraphMaterialization, } ) .unwrap() .events .iter() .filter(|event| event.commit.commit_id == outcome.commit.commit_id) .count(), 2 )` - semantic boundary still requires direct review; no move authorized.

#### `refused_durable_publication_leaves_no_replacement_lineage_behind`

Source: `crates/worth-relational/src/tests/lineage/resolution.rs:106`.

- C4 action 134: `attached_test_medium(&runtime).script_next_publication(ScriptedPublication::ProvenNoEffect);`.
- **C4** 159: `assert!(matches!( refused, TransactionCommitError::PublicationDenied { denial: crate::mvcc::RelationalPublicationDenial::DurablePublicationRefused(_), .. } ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 168: `assert_eq!(after.nodes.len(), before.nodes.len())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 169: `assert_eq!(after.events.len(), before.events.len())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 170: `assert_eq!( runtime.history().branch_head(&BranchId("main".to_owned())), head_before )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 185: `assert_eq!(resolution.resolved, vec![start_lineage])` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 186: `assert!(resolution.traversed_event_ids.is_empty())` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `historical_lineage_resolution_does_not_scan_unrelated_branch_events`

Source: `crates/worth-relational/src/tests/lineage/resolution.rs:190`.

- **UNCLASSIFIED** 256: `assert_eq!(resolution.metrics.traversed_event_count, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 257: `assert_eq!(resolution.metrics.event_visit_count, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 258: `assert!(total_branch_events > resolution.metrics.event_visit_count)` - semantic boundary still requires direct review; no move authorized.

#### `lineage_aspect_history_keeps_origin_events_and_marks_resolution_context`

Source: `crates/worth-relational/src/tests/lineage/resolution.rs:262`.

- **UNCLASSIFIED** 325: `assert_eq!(history.start_lineage_id, start_lineage)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 326: `assert_eq!( traced.trace.requested_target, HistoryAspectQueryTarget::Lineage(start_lineage) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 330: `assert_eq!(traced.trace.returned_entries, 3)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 331: `assert_eq!(traced.trace.traversed_lineage_events, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 332: `assert_eq!( traced.trace.searched_lineage_event_span, Some(AspectHistoryLineageEventSpan { first_event_id: history.traversed_event_ids[0], last_event_id: history.traversed_event_ids[0], }) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 339: `assert_eq!(artifact.scope, DiagnosticsScope::Lineage)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 340: `assert_eq!(artifact.kind, DiagnosticsArtifactKind::DetailedTrace)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 341: `assert_eq!( artifact.entries[0].code, DiagnosticCode::LineageAspectHistoryResolved )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 345: `assert_eq!( digest.requested_target, HistoryAspectQueryTarget::Lineage(start_lineage) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 349: `assert_eq!(digest.entry_count, 3)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 350: `assert_eq!(digest.traversed_lineage_events, 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 351: `assert_eq!( digest.resolved_lineage_chain_len, history.resolved_lineage_chain.len() as u64 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 355: `assert_eq!(traced.history.as_ref(), Some(&history))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 356: `assert_eq!(history.traversed_event_ids.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 357: `assert_eq!(history.entries.len(), 3)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 359: `assert_eq!( history.entries[0].origin.commit_id, created.commit.commit_id )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 363: `assert_eq!( history.entries[1].origin.commit_id, replacement.commit.commit_id )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 367: `assert_eq!( history.entries[2].origin.commit_id, replacement.commit.commit_id )` - semantic boundary still requires direct review; no move authorized.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/support.rs:361 assert_lineage_history_origin_invariants`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

### `tests/performance_profiles/artifact_recoverability_matrix.rs`

#### `perf_artifact_recoverability_matrix` - ignored

Source: `crates/worth-relational/src/tests/performance_profiles/artifact_recoverability_matrix.rs:6`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- C2 transitive cost claims: `crates/worth-relational/src/tests/performance_support/mod.rs:51 assert_budget`, `crates/worth-relational/src/tests/performance_support/mod.rs:80 capture_perf_samples`. Preserve the actual campaign helpers, original durable fixture and ignored/cost root.

### `tests/performance_profiles/chip_simulator_matrix/checkpoint_recover_compile.rs`

No direct test declarations. Executing helper roots and claims:

- `certify_checkpoint_window_recover_compile_round_trip` at 4: C2 cost helper; keep original durable origin and ignored/cost root.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/performance_profiles/chip_simulator_matrix/mod.rs:13 perf_chip_simulator_matrix` (ignored).
  - C2 assertion 130: `assert!(checkpoint_recover_compile_samples .iter() .all(|sample| sample.elapsed_micros > 0))`.

### `tests/performance_profiles/commit_delta_matrix.rs`

#### `perf_commit_delta_matrix` - ignored

Source: `crates/worth-relational/src/tests/performance_profiles/commit_delta_matrix.rs:5`.

- **C2** 26: `assert!(narrow_samples .iter() .all(|sample| sample.elapsed_micros > 0))` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 90: `assert!(cross_partition_samples .iter() .all(|sample| sample.elapsed_micros > 0))` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 146: `assert!(persisted_single_create_samples .iter() .all(|sample| sample.elapsed_micros > 0))` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- C2 transitive cost claims: `crates/worth-relational/src/tests/performance_support/mod.rs:51 assert_budget`, `crates/worth-relational/src/tests/performance_support/mod.rs:80 capture_perf_samples`. Preserve the actual campaign helpers, original durable fixture and ignored/cost root.

### `tests/performance_profiles/durability_append_matrix.rs`

#### `perf_durability_append_matrix` - ignored

Source: `crates/worth-relational/src/tests/performance_profiles/durability_append_matrix.rs:5`.

- **C2** 26: `assert!(fresh_append_samples .iter() .all(|sample| sample.elapsed_micros > 0))` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 54: `assert!(warm_append_samples .iter() .all(|sample| sample.elapsed_micros > 0))` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- C2 transitive cost claims: `crates/worth-relational/src/tests/performance_support/mod.rs:51 assert_budget`, `crates/worth-relational/src/tests/performance_support/mod.rs:80 capture_perf_samples`. Preserve the actual campaign helpers, original durable fixture and ignored/cost root.

### `tests/performance_profiles/geometry_kernel_matrix/identity_survival.rs`

No direct test declarations. Executing helper roots and claims:

- `certify_topology_identity_survival_recovery_round_trip` at 4: C2 cost helper; keep original durable origin and ignored/cost root.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/performance_profiles/geometry_kernel_matrix/mod.rs:8 perf_geometry_kernel_matrix` (ignored).
  - C2 assertion 130: `assert!(topology_identity_samples .iter() .all(|sample| sample.elapsed_micros > 0))`.

### `tests/performance_profiles/hot_cold_path_matrix/chip_recovery_compile.rs`

No direct test declarations. Executing helper roots and claims:

- `certify_chip_hot_compile_vs_recovery_compile` at 4: C2 cost helper; keep original durable origin and ignored/cost root.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/performance_profiles/hot_cold_path_matrix/mod.rs:10 perf_hot_cold_path_matrix` (ignored).
  - C2 assertion 187: `assert!(chip_hot_vs_recovery_samples .iter() .all(|sample| sample.elapsed_micros > 0))`.

### `tests/performance_profiles/hot_cold_path_matrix/chip_rich_recovery_compile.rs`

No direct test declarations. Executing helper roots and claims:

- `certify_chip_rich_compile_hot_vs_recovery_compile` at 4: C2 cost helper; keep original durable origin and ignored/cost root.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/performance_profiles/hot_cold_path_matrix/mod.rs:10 perf_hot_cold_path_matrix` (ignored).
  - C2 assertion 205: `assert!(chip_rich_compile_samples .iter() .all(|sample| sample.elapsed_micros > 0))`.

### `tests/performance_profiles/hot_cold_path_matrix/geometry_publication_replay_truth.rs`

No direct test declarations. Executing helper roots and claims:

- `certify_geometry_rich_publication_hot_vs_replay_truth` at 4: C2 cost helper; keep original durable origin and ignored/cost root.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/performance_profiles/hot_cold_path_matrix/mod.rs:10 perf_hot_cold_path_matrix` (ignored).
  - C2 assertion 213: `assert!(geometry_rich_publication_samples .iter() .all(|sample| sample.elapsed_micros > 0))`.

### `tests/performance_profiles/hot_cold_path_matrix/geometry_replay_reconstruction.rs`

No direct test declarations. Executing helper roots and claims:

- `certify_geometry_hot_commit_vs_replay_reconstruction` at 4: C2 cost helper; keep original durable origin and ignored/cost root.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/performance_profiles/hot_cold_path_matrix/mod.rs:10 perf_hot_cold_path_matrix` (ignored).
  - C2 assertion 185: `assert!(geometry_hot_vs_replay_samples .iter() .all(|sample| sample.elapsed_micros > 0))`.

### `tests/performance_profiles/index_parity_matrix.rs`

#### `perf_index_parity_matrix` - ignored

Source: `crates/worth-relational/src/tests/performance_profiles/index_parity_matrix.rs:5`.

- **C2** 37: `assert!(build.failed_indexes.is_empty())` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 86: `assert!(warm_generation_samples .iter() .all(|sample| sample.elapsed_micros > 0))` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 133: `assert!(build.failed_indexes.is_empty())` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 185: `assert!(build_failed_samples .iter() .all(|sample| sample.elapsed_micros > 0))` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 232: `assert!(build.failed_indexes.is_empty())` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 313: `assert!(persisted_recovery_samples .iter() .all(|sample| sample.elapsed_micros > 0))` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- C2 transitive cost claims: `crates/worth-relational/src/tests/performance_support/mod.rs:51 assert_budget`, `crates/worth-relational/src/tests/performance_support/mod.rs:80 capture_perf_samples`. Preserve the actual campaign helpers, original durable fixture and ignored/cost root.

### `tests/performance_profiles/merge_lineage_matrix/feature_adoption_execution.rs`

No direct test declarations. Executing helper roots and claims:

- `certify_merge_execution_feature_adoption` at 3: C2 cost helper; keep original durable origin and ignored/cost root.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/performance_profiles/merge_lineage_matrix/mod.rs:13 perf_merge_lineage_matrix` (ignored).
  - C2 assertion 68: `assert!(merge_execution_samples .iter() .all(|sample| sample.elapsed_micros > 0))`.

### `tests/performance_profiles/merge_lineage_matrix/persisted_commit_floor.rs`

No direct test declarations. Executing helper roots and claims:

- `certify_merge_execution_vs_persisted_commit_floor` at 3: C2 cost helper; keep original durable origin and ignored/cost root.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/performance_profiles/merge_lineage_matrix/mod.rs:13 perf_merge_lineage_matrix` (ignored).
  - C2 assertion 92: `assert!(merge_vs_commit_floor_samples .iter() .all(|sample| sample.elapsed_micros > 0))`.

### `tests/performance_profiles/merge_lineage_matrix/phase_timing.rs`

No direct test declarations. Executing helper roots and claims:

- `certify_merge_execute_phase_timing_feature_adoption` at 3: C2 cost helper; keep original durable origin and ignored/cost root.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/performance_profiles/merge_lineage_matrix/mod.rs:13 perf_merge_lineage_matrix` (ignored).
  - C2 assertion 108: `assert!(merge_phase_timing_samples .iter() .all(|sample| sample.elapsed_micros > 0))`.

### `tests/performance_profiles/merge_lineage_matrix/planning_divergent_update.rs`

No direct test declarations. Executing helper roots and claims:

- `certify_merge_planning_divergent_update` at 3: C2 cost helper; keep original durable origin and ignored/cost root.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/performance_profiles/merge_lineage_matrix/mod.rs:13 perf_merge_lineage_matrix` (ignored).
  - C2 assertion 48: `assert!(merge_planning_samples .iter() .all(|sample| sample.elapsed_micros > 0))`.

### `tests/performance_profiles/merge_lineage_matrix/prepare_execute_split.rs`

No direct test declarations. Executing helper roots and claims:

- `certify_merge_prepare_vs_execute_feature_adoption` at 3: C2 cost helper; keep original durable origin and ignored/cost root.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/performance_profiles/merge_lineage_matrix/mod.rs:13 perf_merge_lineage_matrix` (ignored).
  - C2 assertion 72: `assert!(merge_prepare_execute_split_samples .iter() .all(|sample| sample.elapsed_micros > 0))`.

### `tests/performance_profiles/merge_lineage_matrix/verify_execute_split.rs`

No direct test declarations. Executing helper roots and claims:

- `certify_merge_verify_vs_execute_feature_adoption` at 3: C2 cost helper; keep original durable origin and ignored/cost root.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/performance_profiles/merge_lineage_matrix/mod.rs:13 perf_merge_lineage_matrix` (ignored).
  - C2 assertion 79: `assert!(merge_verify_execute_split_samples .iter() .all(|sample| sample.elapsed_micros > 0))`.

### `tests/performance_profiles/merge_lineage_matrix/zero_diagnostics_execution.rs`

No direct test declarations. Executing helper roots and claims:

- `certify_merge_execution_zero_diagnostics_budget` at 3: C2 cost helper; keep original durable origin and ignored/cost root.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/performance_profiles/merge_lineage_matrix/mod.rs:13 perf_merge_lineage_matrix` (ignored).
  - C2 assertion 84: `assert!(merge_execution_zero_diag_samples .iter() .all(|sample| sample.elapsed_micros > 0))`.

### `tests/performance_profiles/recoverability_policy_matrix.rs`

#### `perf_recoverability_policy_matrix` - ignored

Source: `crates/worth-relational/src/tests/performance_profiles/recoverability_policy_matrix.rs:6`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.
- C2 transitive cost claims: `crates/worth-relational/src/tests/performance_support/mod.rs:51 assert_budget`, `crates/worth-relational/src/tests/performance_support/mod.rs:80 capture_perf_samples`. Preserve the actual campaign helpers, original durable fixture and ignored/cost root.

### `tests/performance_profiles/replay_recovery_matrix.rs`

#### `perf_replay_recovery_matrix` - ignored

Source: `crates/worth-relational/src/tests/performance_profiles/replay_recovery_matrix.rs:6`.

- **C2** 65: `assert!(durable_replay_samples .iter() .all(|sample| sample.elapsed_micros > 0))` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- **C2** 149: `assert!(checkpoint_recovery_samples .iter() .all(|sample| sample.elapsed_micros > 0))` - actual cost/work/counter campaign; preserve ignored/cost caller and real durable fixture.
- C2 transitive cost claims: `crates/worth-relational/src/tests/performance_support/mod.rs:51 assert_budget`, `crates/worth-relational/src/tests/performance_support/mod.rs:80 capture_perf_samples`. Preserve the actual campaign helpers, original durable fixture and ignored/cost root.

### `tests/performance_profiles/sustained_load_matrix/replay_window_drift.rs`

No direct test declarations. Executing helper roots and claims:

- `certify_replay_window_drift_stability` at 3: C2 cost helper; keep original durable origin and ignored/cost root.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/performance_profiles/sustained_load_matrix/mod.rs:13 perf_sustained_load_matrix` (ignored).
  - C2 assertion 44: `assert!( outcome.failure.is_none(), "replay window drift sample should not fail: {:?}", outcome.failure )`.
  - C2 assertion 89: `assert!(replay_window_drift_samples .iter() .all(|sample| sample.elapsed_micros > 0))`.

### `tests/performance_profiles/workflow_matrix/persisted_recovery_replay.rs`

No direct test declarations. Executing helper roots and claims:

- `certify_persisted_recovery_replay_round_trip` at 4: C2 cost helper; keep original durable origin and ignored/cost root.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/performance_profiles/workflow_matrix/mod.rs:11 perf_workflow_matrix` (ignored).
  - C2 assertion 152: `assert!(replay_recovery_samples .iter() .all(|sample| sample.elapsed_micros > 0))`.

### `tests/performance_profiles/workflow_matrix/trade_correction_analysis.rs`

No direct test declarations. Executing helper roots and claims:

- `certify_trade_correction_analysis_round_trip` at 3: C2 cost helper; keep original durable origin and ignored/cost root.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/performance_profiles/workflow_matrix/mod.rs:11 perf_workflow_matrix` (ignored).
  - C2 assertion 234: `assert!(trade_correction_samples .iter() .all(|sample| sample.elapsed_micros > 0))`.

### `tests/publication/cdc/certification/durable_recovery.rs`

#### `cdc_certification_durable_recovery_matches_head_and_midstream_consumers`

Source: `crates/worth-relational/src/tests/publication/cdc/certification/durable_recovery.rs:20`.

- C4 action 38: `.evict_commit_envelope_for_durable_recovery_test(crate::history::data::CommitId(1)));`.
- C4 action 41: `.evict_commit_envelope_for_durable_recovery_test(crate::history::data::CommitId(2)));`.
- C4 action 44: `.evict_commit_envelope_for_durable_recovery_test(crate::history::data::CommitId(3)));`.
- **C4** 36: `assert!(runtime .history_authority() .evict_commit_envelope_for_durable_recovery_test(crate::history::data::CommitId(1)))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 39: `assert!(runtime .history_authority() .evict_commit_envelope_for_durable_recovery_test(crate::history::data::CommitId(2)))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 42: `assert!(runtime .history_authority() .evict_commit_envelope_for_durable_recovery_test(crate::history::data::CommitId(3)))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 50: `assert_eq!( durable_mid.recovery_decision.source, SubscriberRecoverySource::DurableCanonicalRecovery )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 60: `assert_eq!(durable_mid_stitched, expected_mid)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 83: `assert_eq!(durable_mid_stitched, recovered_patch_batch.patches)` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `cdc_certification_persisted_seeded_matrix_survives_checkpoint_compaction_and_recovery`

Source: `crates/worth-relational/src/tests/publication/cdc/certification/durable_recovery.rs:87`.

- **UNCLASSIFIED** 120: `assert_eq!( recovered_patch_stream, full_from_head, "persisted seed {seed} drifted after recovery" )` - semantic boundary still requires direct review; no move authorized.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/harness/scenario/runner/scenario_execution.rs:17 run_seeded_scenario`, `crates/worth-relational/src/tests/harness/certify/assertions.rs:20 assert_multi_subscriber_converges`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `cdc_certification_retention_truncation_recovers_exact_suffix_from_old_checkpoint`

Source: `crates/worth-relational/src/tests/publication/cdc/certification/durable_recovery.rs:128`.

- **UNCLASSIFIED** 151: `assert_eq!(resumed, expected)` - semantic boundary still requires direct review; no move authorized.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/harness/scenario/runner/scenario_execution.rs:17 run_seeded_scenario`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

### `tests/publication/cdc/certification/schema_boundary_parity.rs`

#### `cdc_certification_schema_boundary_continuation_is_explained_and_counted`

Source: `crates/worth-relational/src/tests/publication/cdc/certification/schema_boundary_parity.rs:12`.

- C4 action 19: `runtime.set_schema_registry_for_test(`.
- **C4** 59: `assert_eq!( batch.continuation.continuation_outcome(), crate::facade::schema::SchemaContinuationClassification::ContinueWithVisibleBridge )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 63: `assert!(batch .diagnostics .iter() .flat_map(|artifact| artifact.entries.iter()) .any(|entry| entry.code == DiagnosticCode::SubscriberContractEvaluated))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 68: `assert!(batch .diagnostics .iter() .flat_map(|artifact| artifact.entries.iter()) .any(|entry| entry.code == DiagnosticCode::SubscriberBoundaryEvaluated))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C2** 73: `assert_eq!(counters.subscriber_resume_evaluations, 1)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 74: `assert_eq!(counters.subscriber_continue_visible_bridge_count, 1)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 75: `assert_eq!(counters.schema_normalized_descriptor_compositions, 1)` - actual residency, identity, pin, allocation, dictionary or cost observation.

#### `diff_cdc_truth_parity_test`

Source: `crates/worth-relational/src/tests/publication/cdc/certification/schema_boundary_parity.rs:79`.

- C4 action 88: `runtime.set_schema_registry_for_test(`.
- C4 action 122: `runtime.set_schema_registry_for_test(`.
- **C4** 223: `assert_eq!( diff_digest, digest_patch_batch_surface(&recovered_patch_batch.patches) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 227: `assert_eq!( cdc_digest, digest_subscriber_boundary_cdc_surface( &recovered_batch.patches, recovered_batch.continuation.crossed_boundaries(), recovered_batch.continuation.continuation_summary(), &recovered_batch.recovery_decision, ) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 236: `assert_eq!( cdc_diagnostics_digest, digest_diagnostics_batch_surface(&recovered_batch.diagnostics) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C2** 240: `assert_eq!( continuation_counter_snapshot, digest_subscriber_continuation_counter_pair( recovered_counters.subscriber_continue_visible_bridge_count, recovered_counters.schema_normalized_descriptor_compositions, ) )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 247: `assert!(live_counters.replay_digest_parity_checks == 0)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 248: `assert!(live_counters.subscriber_continue_visible_bridge_count >= 1)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 249: `assert!(live_counters.schema_normalized_descriptor_compositions >= 1)` - actual residency, identity, pin, allocation, dictionary or cost observation.

### `tests/publication/cdc/recovery.rs`

#### `subscriber_stream_recovers_from_durable_canonical_envelopes_when_checkpoint_is_not_in_memory`

Source: `crates/worth-relational/src/tests/publication/cdc/recovery.rs:4`.

- C4 action 15: `.evict_commit_envelope_for_durable_recovery_test(first.commit.commit_id));`.
- **C4** 13: `assert!(runtime .history_authority() .evict_commit_envelope_for_durable_recovery_test(first.commit.commit_id))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 23: `assert_eq!(resumed.patches.len(), 1)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 24: `assert_eq!(resumed.patches[0].position.0, 2)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 25: `assert_eq!( resumed.recovery_decision.source, crate::facade::publication::SubscriberRecoverySource::DurableCanonicalRecovery )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 29: `assert_eq!( resumed .latest_available_checkpoint .as_ref() .map(|checkpoint| checkpoint.position().0), Some(2) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `subscriber_stream_reports_durable_latest_available_checkpoint_when_latest_head_is_not_in_memory`

Source: `crates/worth-relational/src/tests/publication/cdc/recovery.rs:39`.

- C4 action 52: `.evict_commit_envelope_for_durable_recovery_test(first.commit.commit_id));`.
- C4 action 55: `.evict_commit_envelope_for_durable_recovery_test(second.commit.commit_id));`.
- **C4** 50: `assert!(runtime .history_authority() .evict_commit_envelope_for_durable_recovery_test(first.commit.commit_id))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 53: `assert!(runtime .history_authority() .evict_commit_envelope_for_durable_recovery_test(second.commit.commit_id))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 63: `assert_eq!(resumed.patches.len(), 1)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 64: `assert_eq!(resumed.patches[0].position.0, 2)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 65: `assert_eq!( resumed.recovery_decision.source, crate::facade::publication::SubscriberRecoverySource::DurableCanonicalRecovery )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 69: `assert_eq!( resumed .latest_available_checkpoint .as_ref() .map(|checkpoint| checkpoint.position().0), Some(2) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `tests/publication/cdc/replay_parity.rs`

#### `subscriber_stream_matches_patch_stream_for_committed_history`

Source: `crates/worth-relational/src/tests/publication/cdc/replay_parity.rs:5`.

- **UNCLASSIFIED** 28: `assert_eq!(subscriber_batch.patches, patch_batch.patches)` - semantic boundary still requires direct review; no move authorized.

#### `durable_source_subscriber_stream_matches_recovered_runtime_patch_stream`

Source: `crates/worth-relational/src/tests/publication/cdc/replay_parity.rs:32`.

- C4 action 44: `.evict_commit_envelope_for_durable_recovery_test(first.commit.commit_id));`.
- **C4** 42: `assert!(runtime .history_authority() .evict_commit_envelope_for_durable_recovery_test(first.commit.commit_id))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 76: `assert_eq!(subscriber_batch.patches, recovered_patch_batch.patches)` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `subscriber_stream_keeps_patch_parity_when_later_retained_envelope_is_missing`

Source: `crates/worth-relational/src/tests/publication/cdc/replay_parity.rs:80`.

- **UNCLASSIFIED** 90: `assert!(runtime .history_authority() .evict_commit_envelope_preserving_patch_position_for_durable_recovery_test( second.commit.commit_id, ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 111: `assert_eq!(subscriber_batch.patches, patch_batch.patches)` - semantic boundary still requires direct review; no move authorized.

### `tests/publication/cdc/subscriber_failures/checkpoint_identity.rs`

#### `subscriber_stream_rejects_schema_unsupported_checkpoint`

Source: `crates/worth-relational/src/tests/publication/cdc/subscriber_failures/checkpoint_identity.rs:5`.

- **UNCLASSIFIED** 22: `assert_eq!(error.class, SubscriberStreamFailureClass::SchemaUnsupported)` - semantic boundary still requires direct review; no move authorized.

#### `subscriber_stream_rejects_checkpoint_without_history_or_durable_coverage`

Source: `crates/worth-relational/src/tests/publication/cdc/subscriber_failures/checkpoint_identity.rs:26`.

- **UNCLASSIFIED** 35: `assert_eq!( error.class, SubscriberStreamFailureClass::DurableCoverageGap )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 39: `assert!(error .diagnostics .iter() .all(|artifact| artifact.scope == DiagnosticsScope::Replay))` - semantic boundary still requires direct review; no move authorized.

#### `subscriber_stream_rejects_checkpoint_with_mismatched_contract_identity`

Source: `crates/worth-relational/src/tests/publication/cdc/subscriber_failures/checkpoint_identity.rs:46`.

- **UNCLASSIFIED** 71: `assert_eq!( error.class, SubscriberStreamFailureClass::SubscriberContractMismatch )` - semantic boundary still requires direct review; no move authorized.

### `tests/publication/cdc/subscriber_failures/durable_descriptor_version.rs`

#### `subscriber_stream_rejects_durable_only_checkpoint_with_descriptor_version_mismatch`

Source: `crates/worth-relational/src/tests/publication/cdc/subscriber_failures/durable_descriptor_version.rs:6`.

- C4 action 16: `.evict_commit_envelope_for_durable_recovery_test(first.commit.commit_id));`.
- **C4** 14: `assert!(runtime .history_authority() .evict_commit_envelope_for_durable_recovery_test(first.commit.commit_id))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 38: `assert_eq!( error.class, SubscriberStreamFailureClass::DescriptorVersionMismatch )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 42: `assert_eq!( error .latest_available_checkpoint .as_ref() .map(|checkpoint| checkpoint.position().0), Some(1) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `tests/publication/patch/batching.rs`

#### `patch_stream_resume_batches_commits_without_duplication`

Source: `crates/worth-relational/src/tests/publication/patch/batching.rs:5`.

- **UNCLASSIFIED** 35: `assert_eq!(first_batch.patches.len(), 2)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 36: `assert_eq!(first_batch.next_position, Some(PatchStreamPosition(2)))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 37: `assert_eq!(first_batch.latest_position, Some(PatchStreamPosition(3)))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 38: `assert_eq!(resumed.patches.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 39: `assert_eq!(resumed.resumed_after, Some(PatchStreamPosition(2)))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 40: `assert_eq!(resumed.patches[0].position, PatchStreamPosition(3))` - semantic boundary still requires direct review; no move authorized.

#### `patch_stream_rejects_unknown_resume_position`

Source: `crates/worth-relational/src/tests/publication/patch/batching.rs:44`.

- **UNCLASSIFIED** 62: `assert_eq!( error.class, PatchStreamReadErrorClass::UnknownResumePosition )` - semantic boundary still requires direct review; no move authorized.

#### `patch_stream_uses_durable_canonical_history_when_retained_envelope_is_missing`

Source: `crates/worth-relational/src/tests/publication/patch/batching.rs:69`.

- **UNCLASSIFIED** 79: `assert!(runtime .history_authority() .evict_commit_envelope_preserving_patch_position_for_durable_recovery_test( first.commit.commit_id, ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 96: `assert_eq!(batch.patches.len(), 3)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 97: `assert_eq!(batch.patches[0].position, PatchStreamPosition(1))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 98: `assert_eq!(batch.next_position, Some(PatchStreamPosition(3)))` - semantic boundary still requires direct review; no move authorized.

### `tests/query/indexes/discard_rebuild/checkpoint.rs`

#### `discarded_ordering_stays_cold_after_native_checkpoint_restore_then_rebuilds`

Source: `crates/worth-relational/src/tests/query/indexes/discard_rebuild/checkpoint.rs:4`.

- **HAPPY** 15: `assert!(built.failed_indexes.is_empty())` - reviewed public owner result over explicit durable provider.
- **HAPPY** 22: `assert!( envelope.derived_index_artifacts().is_empty(), "current commits keep rebuildable payloads in the native catalog" )` - reviewed public owner result over explicit durable provider.
- **HAPPY** 41: `assert!(discarded.removed_generation_count() > 0)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 58: `assert_eq!(recovered.latest_commit, Some(committed))` - reviewed public owner result over explicit durable provider.
- **HAPPY** 59: `assert_eq!( restored.index_access().matching_definition(&fixture.index), Some(fixture.index.clone()) )` - reviewed public owner result over explicit durable provider.
- **HAPPY** 63: `assert!(restored .index_access() .read_latest_generation(execution, fixture.index.index_id, &BranchId("main".into())) .unwrap() .is_none())` - reviewed public owner result over explicit durable provider.
- **HAPPY** 91: `assert_eq!( cold.kind(), BoundedRelatedEntityOrderedLookupDenialKind::ExactGenerationUnavailable )` - reviewed public owner result over explicit durable provider.
- **HAPPY** 100: `assert!(rebuilt.failed_indexes.is_empty())` - reviewed public owner result over explicit durable provider.
- **HAPPY** 101: `assert!(rebuilt.execution_denial.is_none())` - reviewed public owner result over explicit durable provider.
- **HAPPY** 102: `assert_eq!(rebuilt.generations[0].entries, generation.entries)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 119: `assert_eq!(page.child_entity_ids(), fixture.expected)` - reviewed public owner result over explicit durable provider.
- **HAPPY** 120: `assert!(!page.has_more())` - reviewed public owner result over explicit durable provider.
- **HAPPY** 122: `assert_eq!( restored .history() .branch_head(&BranchId("main".into())) .as_ref(), Some(&later.commit) )` - reviewed public owner result over explicit durable provider.

### `tests/query/indexes/mod.rs`

No direct test declarations. Executing helper roots and claims:

- `durable_runtime_with_index_field_aspects` at 49: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/query/concurrency.rs:11 concurrent_snapshot_and_version_reads_match_serial_truth`, `crates/worth-relational/src/tests/query/concurrency.rs:100 concurrent_read_pressure_keeps_cache_diagnostics_coherent`, `crates/worth-relational/src/tests/query/concurrency.rs:205 concurrent_pinned_traversal_reads_stay_snapshot_stable_under_hot_rewrite_pressure`, `crates/worth-relational/src/tests/query/concurrency/relation_index_parity.rs:4 concurrent_relation_index_certification_parity_stays_stable_under_scheduler_pressure`, `crates/worth-relational/src/tests/query/execution/aspect_filtered_scans.rs:129 planned_query_execution_aspect_filter_supports_field_mask_presence`, `crates/worth-relational/src/tests/query/execution/leased_explicit_targets.rs:46 leased_explicit_read_preserves_packet_identity_and_reports_stops`, `crates/worth-relational/src/tests/query/execution/leased_explicit_targets.rs:190 leased_field_filter_charges_captured_comparison_bytes`, `crates/worth-relational/src/tests/query/execution/leased_explicit_targets.rs:259 leased_kind_scan_stops_at_a_record_checkpoint`, `crates/worth-relational/src/tests/query/execution/leased_explicit_targets.rs:326 leased_kind_scan_rejects_one_record_larger_than_its_packet_ceiling`, `crates/worth-relational/src/tests/query/execution/leased_explicit_targets/traversal_and_values.rs:4 leased_traversal_stops_inside_high_fanout`, `crates/worth-relational/src/tests/query/execution/leased_explicit_targets/traversal_and_values.rs:84 leased_any_of_scan_matches_serial_with_reversed_values`, `crates/worth-relational/src/tests/query/execution/read_law_matrix.rs:19 runtime_read_law_matrix_keeps_rows_dependencies_revisions_and_work`, `crates/worth-relational/src/tests/query/execution/read_law_matrix/bounded_index.rs:12 runtime_bounded_index_law_keeps_cold_stop_limit_and_generation`, `crates/worth-relational/src/tests/query/execution/read_law_matrix/refusals.rs:32 runtime_read_law_plan_absence_and_stopped_request`, `crates/worth-relational/src/tests/query/execution/read_law_matrix/refusals.rs:98 runtime_read_law_bounded_index_refusal_values`, `crates/worth-relational/src/tests/query/execution/read_law_matrix/remaining_door_stops.rs:51 runtime_read_law_view_input_stops_and_refuses_by_value`, `crates/worth-relational/src/tests/query/execution/read_law_matrix/remaining_door_stops.rs:91 runtime_read_law_selected_view_stops_and_refuses_by_value`, `crates/worth-relational/src/tests/query/execution/read_law_matrix/remaining_door_stops.rs:136 runtime_read_law_relation_join_stops_and_refuses_by_value`, `crates/worth-relational/src/tests/query/execution/read_law_matrix/remaining_doors.rs:27 runtime_read_law_remaining_view_input_preparation`, `crates/worth-relational/src/tests/query/execution/read_law_matrix/remaining_doors.rs:75 runtime_read_law_remaining_selected_view`, `crates/worth-relational/src/tests/query/execution/read_law_matrix/remaining_doors.rs:112 runtime_read_law_remaining_relation_join`, `crates/worth-relational/src/tests/query/execution/read_request_custody.rs:67 production_kind_scan_spends_its_callers_work_ceiling`, `crates/worth-relational/src/tests/query/execution/read_request_custody.rs:93 production_plan_has_equal_bits_and_charged_work_at_every_backing`, `crates/worth-relational/src/tests/query/execution/read_request_custody.rs:125 production_plan_cancellation_has_one_typed_cause_at_every_backing`, `crates/worth-relational/src/tests/query/execution/read_request_custody.rs:155 serial_scope_admission_precedes_unavailable_snapshot`.

### `tests/query/indexes/related_entity_schema_recovery.rs`

#### `recovered_related_ordering_uses_each_exact_roots_schema_contract`

Source: `crates/worth-relational/src/tests/query/indexes/related_entity_schema_recovery.rs:14`.

- **UNCLASSIFIED** 35: `assert_eq!( recovered .read_truth() .snapshot_schema_version(&legacy_snapshot), Some(SchemaVersionId(1)) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 53: `assert_eq!(old.child_entity_ids(), &[alpha, beta])` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 60: `assert_eq!( recovered .read_truth() .snapshot_schema_version(&current_snapshot), Some(SchemaVersionId(2)) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 74: `assert!(current.child_entity_ids().is_empty())` - semantic boundary still requires direct review; no move authorized.

### `tests/schema/certification.rs`

#### `schema_evolution_cdc_contract_test`

Source: `crates/worth-relational/src/tests/schema/certification.rs:60`.

- C4 action 74: `runtime.set_schema_registry_for_test(schema_v2);`.
- **C4** 152: `assert_eq!( schema_transition_digest, digest_schema_transition_descriptor( recovered_envelope.schema_transition.as_ref().unwrap(), recovered_envelope.descriptor_semantics_version, ) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 159: `assert_eq!( schema_boundary_cdc_digest, digest_subscriber_boundary_cdc_surface( &recovered_batch.patches, recovered_batch.continuation.crossed_boundaries(), recovered_batch.continuation.continuation_summary(), &recovered_batch.recovery_decision, ) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 168: `assert_eq!( subscriber_contract_matrix, digest_subscriber_continuation_summary(recovered_batch.continuation.continuation_summary()) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 172: `assert_eq!( transition_decision_digest, digest_schema_transition_decision( recovered_envelope .schema_continuation_descriptor .as_ref() .unwrap(), recovered_envelope .schema_reconciliation_descriptor .as_ref() .unwrap(), recovered_envelope.descriptor_semantics_version, ) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 186: `assert_eq!( descriptor_semantics_version, recovered_envelope.descriptor_semantics_version )` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `tests/support/durability.rs`

No direct test declarations. Executing helper roots and claims:

- `checkpoint_into_medium` at 42: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/branch/owner_services/materialization_port_tests.rs:19 durability_replays_suspended_and_restored_materialization_roots`, `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:242 fresh_source_refuses_real_selected_empty_checkpoint`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:12 equal_exact_root_reuses_the_readmitted_partition_substrate`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:132 shared_restored_substrate_keeps_a_pinned_reader_exact_after_next_write`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:179 divergent_sibling_is_not_reused_as_the_storage_mirror`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:235 divergent_mirror_partition_rebuilds_cross_partition_adjacency_after_reuse`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:299 recovered_main_and_sibling_catalogs_reuse_exact_canonical_payloads`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:348 equal_image_cannot_bypass_global_contract_readmission`, `crates/worth-relational/src/durability/log/native_checkpoint_codec/tests.rs:10 member_directory_preserves_divergent_sibling_roots_and_pinned_reader`, `crates/worth-relational/src/publication/cdc/planning/checkpoint_resolution/tests.rs:9 subscriber_stream_recovers_from_durable_canonical_envelopes_when_patch_stream_index_outlives_envelope`, `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs:147 checkpoint_recovery_preserves_abandoned_symbol_identity_frontiers`, `crates/worth-relational/src/tests/domains/cad/topology_identity_survival.rs:10 topology_identity_survival_preserves_reidentification_truth_across_recovery`, `crates/worth-relational/src/tests/domains/fintech/workflows/persistence.rs:9 fintech_persisted_workflow_recovers_checkpoint_tail_and_keeps_queryable_portfolio_state`, `crates/worth-relational/src/tests/domains/fintech/workflows/persistence.rs:158 fintech_failure_injection_helpers_cover_savepoints_replay_and_checkpoint_corruption`, `crates/worth-relational/src/tests/domains/fintech/workflows/persistence.rs:216 fintech_persisted_workflow_recovers_from_its_filed_checkpoint`, `crates/worth-relational/src/tests/domains/fintech/workflows/persistence.rs:281 fintech_index_and_lineage_authority_survive_hostile_workflow_recovery`, `crates/worth-relational/src/tests/domains/fintech/workflows/persistence/checkpoint_fallback.rs:6 fintech_recovery_refuses_a_damaged_checkpoint_instead_of_losing_truth`, `crates/worth-relational/src/tests/durability/contracts/adjacency_checkpoint.rs:4 checkpoint_rebuilds_current_and_historical_adjacency_kind_buckets`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:5 durability_contract_recovery_rebuilds_branch_heads_and_latest_commit`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:38 durability_contract_recovery_preserves_aspect_bearing_patch_truth_and_history`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:95 durability_contract_recovery_preserves_relation_aspect_history_for_retained_audit_relations`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:160 durability_contract_recovery_preserves_branch_local_endpoint_deletion_retirement_histories`, `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:4 checkpoint_recovery_rejects_missing_exact_branch_root_artifact`, `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:40 checkpoint_recovery_rejects_root_artifact_without_commit_envelope`, `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:70 checkpoint_recovery_rejects_duplicate_commit_envelopes_before_root_readmission`.
- `durable_recovery_source` at 54: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/branch/owner_services/materialization_port_tests.rs:19 durability_replays_suspended_and_restored_materialization_roots`, `crates/worth-relational/src/commit_strategies/facade_tests/replay_recovery_failures.rs:5 replay_commit_reports_strategy_executor_unavailable_when_recovered_runtime_lacks_executor`, `crates/worth-relational/src/commit_strategies/facade_tests/replay_recovery_failures.rs:62 replay_commit_reports_strategy_execution_failure_when_recovered_executor_rejects`, `crates/worth-relational/src/durability/authority/checkpoint_concurrency_tests.rs:43 checkpoint_reconstruction_does_not_exclude_publication_after_capture`, `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:242 fresh_source_refuses_real_selected_empty_checkpoint`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:132 shared_restored_substrate_keeps_a_pinned_reader_exact_after_next_write`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:179 divergent_sibling_is_not_reused_as_the_storage_mirror`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:299 recovered_main_and_sibling_catalogs_reuse_exact_canonical_payloads`, `crates/worth-relational/src/durability/log/native_checkpoint_codec/tests.rs:10 member_directory_preserves_divergent_sibling_roots_and_pinned_reader`, `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs:147 checkpoint_recovery_preserves_abandoned_symbol_identity_frontiers`, `crates/worth-relational/src/tests/complexity/contracts/commit_budgets/schema_continuity_boundaries.rs:110 complexity_budget_milestone5_closeout_keeps_schema_cdc_and_recovery_boundary_local`, `crates/worth-relational/src/tests/domains/cad/topology_identity_survival.rs:10 topology_identity_survival_preserves_reidentification_truth_across_recovery`, `crates/worth-relational/src/tests/domains/fintech/workflows/persistence.rs:9 fintech_persisted_workflow_recovers_checkpoint_tail_and_keeps_queryable_portfolio_state`, `crates/worth-relational/src/tests/durability/contracts/adjacency_checkpoint.rs:4 checkpoint_rebuilds_current_and_historical_adjacency_kind_buckets`, `crates/worth-relational/src/tests/durability/contracts/associated_effects.rs:77 associated_effects_discover_cold_subject_without_reading_unrelated_history`, `crates/worth-relational/src/tests/durability/contracts/associated_effects_discovery.rs:11 native_custody_correction_enumerates_known_orphan_and_unrelated_association_metadata`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:5 durability_contract_recovery_rebuilds_branch_heads_and_latest_commit`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:38 durability_contract_recovery_preserves_aspect_bearing_patch_truth_and_history`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:95 durability_contract_recovery_preserves_relation_aspect_history_for_retained_audit_relations`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:160 durability_contract_recovery_preserves_branch_local_endpoint_deletion_retirement_histories`, `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:4 checkpoint_recovery_rejects_missing_exact_branch_root_artifact`, `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:40 checkpoint_recovery_rejects_root_artifact_without_commit_envelope`, `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:70 checkpoint_recovery_rejects_duplicate_commit_envelopes_before_root_readmission`, `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:107 checkpoint_recovery_readmits_shared_root_once_for_sibling_heads`, `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:158 tail_recovery_resolves_the_checkpoint_target_root_not_the_fork_source_head`.
- `medium_recovery_plan` at 64: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/branch/owner_services/materialization_port_tests.rs:19 durability_replays_suspended_and_restored_materialization_roots`, `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs:147 checkpoint_recovery_preserves_abandoned_symbol_identity_frontiers`, `crates/worth-relational/src/tests/durability/contracts/adjacency_checkpoint.rs:4 checkpoint_rebuilds_current_and_historical_adjacency_kind_buckets`, `crates/worth-relational/src/tests/durability/contracts/associated_effects.rs:77 associated_effects_discover_cold_subject_without_reading_unrelated_history`, `crates/worth-relational/src/tests/durability/contracts/associated_effects_discovery.rs:11 native_custody_correction_enumerates_known_orphan_and_unrelated_association_metadata`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:5 durability_contract_recovery_rebuilds_branch_heads_and_latest_commit`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:38 durability_contract_recovery_preserves_aspect_bearing_patch_truth_and_history`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:95 durability_contract_recovery_preserves_relation_aspect_history_for_retained_audit_relations`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/checkpoint_state.rs:6 checkpoint_size_tracks_branch_state_not_covered_transitions`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/fates.rs:134 every_transition_fate_preserves_the_durable_before_live_boundary`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/recovery.rs:6 a_durable_fork_without_a_subsequent_commit_reopens_and_commits`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/recovery.rs:27 a_fork_covered_by_a_checkpoint_after_its_source_moves_accepts_a_target_tail`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/recovery.rs:49 a_fork_in_the_transition_tail_is_merged_before_the_target_commit`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/recovery.rs:72 lifecycle_postures_and_retired_names_survive_checkpoints_and_crashes`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/replay.rs:5 partial_replay_includes_an_intermediate_fork_without_its_own_commit`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/tampering.rs:36 a_sequence_gap_and_invalid_follows_positions_are_refused`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/tampering.rs:72 a_commit_carrying_an_unknown_branch_cell_cannot_replace_its_missing_fork`, `crates/worth-relational/src/tests/durability/contracts/caller_association.rs:161 caller_association_survives_cold_checkpoint_history_and_journal_tail`, `crates/worth-relational/src/tests/durability/contracts/canonical_artifact_neutrality.rs:177 a_recovered_runtime_continues_with_the_reference_artifacts`, `crates/worth-relational/src/tests/durability/contracts/fork_tail_after_source_moved.rs:10 a_fork_commits_after_a_checkpoint_taken_once_its_source_moved_on`, `crates/worth-relational/src/tests/durability/contracts/lineage_event_allocator_recovery.rs:66 unresolved_durable_publication_blocks_descendants_and_recovers_last_checkpoint`, `crates/worth-relational/src/tests/durability/medium/recovered_publication.rs:9 a_recovered_runtime_publishes_its_next_commit_at_the_next_position`, `crates/worth-relational/src/tests/durability/medium/unresolved_fate.rs:100 reopening_resolves_an_absent_unresolved_commit_as_never_committed`, `crates/worth-relational/src/tests/durability/medium/unresolved_fate.rs:131 reopening_resolves_a_present_unresolved_commit_as_committed_exactly_once`, `crates/worth-relational/src/tests/history/milestone_7a/ancestry_query_matrix.rs:4 merge_ready_history_ancestry_queries_resolve_branch_reasoning`.
- `checkpoint_durably` at 80: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/branch/owner_services/materialization_port_tests.rs:19 durability_replays_suspended_and_restored_materialization_roots`, `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:242 fresh_source_refuses_real_selected_empty_checkpoint`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:12 equal_exact_root_reuses_the_readmitted_partition_substrate`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:132 shared_restored_substrate_keeps_a_pinned_reader_exact_after_next_write`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:179 divergent_sibling_is_not_reused_as_the_storage_mirror`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:235 divergent_mirror_partition_rebuilds_cross_partition_adjacency_after_reuse`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:299 recovered_main_and_sibling_catalogs_reuse_exact_canonical_payloads`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:348 equal_image_cannot_bypass_global_contract_readmission`, `crates/worth-relational/src/durability/log/native_checkpoint_codec/tests.rs:10 member_directory_preserves_divergent_sibling_roots_and_pinned_reader`, `crates/worth-relational/src/publication/cdc/planning/checkpoint_resolution/tests.rs:9 subscriber_stream_recovers_from_durable_canonical_envelopes_when_patch_stream_index_outlives_envelope`, `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs:147 checkpoint_recovery_preserves_abandoned_symbol_identity_frontiers`, `crates/worth-relational/src/tests/domains/cad/topology_identity_survival.rs:10 topology_identity_survival_preserves_reidentification_truth_across_recovery`, `crates/worth-relational/src/tests/domains/fintech/workflows/persistence.rs:9 fintech_persisted_workflow_recovers_checkpoint_tail_and_keeps_queryable_portfolio_state`, `crates/worth-relational/src/tests/domains/fintech/workflows/persistence.rs:158 fintech_failure_injection_helpers_cover_savepoints_replay_and_checkpoint_corruption`, `crates/worth-relational/src/tests/domains/fintech/workflows/persistence.rs:216 fintech_persisted_workflow_recovers_from_its_filed_checkpoint`, `crates/worth-relational/src/tests/domains/fintech/workflows/persistence.rs:281 fintech_index_and_lineage_authority_survive_hostile_workflow_recovery`, `crates/worth-relational/src/tests/domains/fintech/workflows/persistence/checkpoint_fallback.rs:6 fintech_recovery_refuses_a_damaged_checkpoint_instead_of_losing_truth`, `crates/worth-relational/src/tests/durability/contracts/adjacency_checkpoint.rs:4 checkpoint_rebuilds_current_and_historical_adjacency_kind_buckets`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:5 durability_contract_recovery_rebuilds_branch_heads_and_latest_commit`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:38 durability_contract_recovery_preserves_aspect_bearing_patch_truth_and_history`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:95 durability_contract_recovery_preserves_relation_aspect_history_for_retained_audit_relations`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:160 durability_contract_recovery_preserves_branch_local_endpoint_deletion_retirement_histories`, `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:4 checkpoint_recovery_rejects_missing_exact_branch_root_artifact`, `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:40 checkpoint_recovery_rejects_root_artifact_without_commit_envelope`, `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:70 checkpoint_recovery_rejects_duplicate_commit_envelopes_before_root_readmission`.
- `recovery_plan_for` at 96: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/commit_strategies/facade_tests/replay_recovery_failures.rs:5 replay_commit_reports_strategy_executor_unavailable_when_recovered_runtime_lacks_executor`, `crates/worth-relational/src/commit_strategies/facade_tests/replay_recovery_failures.rs:62 replay_commit_reports_strategy_execution_failure_when_recovered_executor_rejects`, `crates/worth-relational/src/durability/authority/checkpoint_concurrency_tests.rs:43 checkpoint_reconstruction_does_not_exclude_publication_after_capture`, `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:242 fresh_source_refuses_real_selected_empty_checkpoint`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:132 shared_restored_substrate_keeps_a_pinned_reader_exact_after_next_write`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:179 divergent_sibling_is_not_reused_as_the_storage_mirror`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:299 recovered_main_and_sibling_catalogs_reuse_exact_canonical_payloads`, `crates/worth-relational/src/durability/log/native_checkpoint_codec/tests.rs:10 member_directory_preserves_divergent_sibling_roots_and_pinned_reader`, `crates/worth-relational/src/tests/complexity/contracts/commit_budgets/schema_continuity_boundaries.rs:110 complexity_budget_milestone5_closeout_keeps_schema_cdc_and_recovery_boundary_local`, `crates/worth-relational/src/tests/domains/cad/topology_identity_survival.rs:10 topology_identity_survival_preserves_reidentification_truth_across_recovery`, `crates/worth-relational/src/tests/domains/fintech/workflows/persistence.rs:9 fintech_persisted_workflow_recovers_checkpoint_tail_and_keeps_queryable_portfolio_state`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:160 durability_contract_recovery_preserves_branch_local_endpoint_deletion_retirement_histories`, `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:4 checkpoint_recovery_rejects_missing_exact_branch_root_artifact`, `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:40 checkpoint_recovery_rejects_root_artifact_without_commit_envelope`, `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:70 checkpoint_recovery_rejects_duplicate_commit_envelopes_before_root_readmission`, `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:107 checkpoint_recovery_readmits_shared_root_once_for_sibling_heads`, `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:158 tail_recovery_resolves_the_checkpoint_target_root_not_the_fork_source_head`, `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:194 checkpoint_recovery_rejects_duplicate_global_partition_images`, `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:225 checkpoint_recovery_rejects_duplicate_branch_root_partition_images`, `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint/foreign_targets.rs:4 checkpoint_recovery_rejects_foreign_branch_target_substitution`, `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint/foreign_targets.rs:49 tail_recovery_rejects_foreign_branch_target_substitution`, `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:5 checkpoint_rejects_previous_branch_root_format_before_readmission`, `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:22 checkpoint_rejects_altered_root_content_with_recomputed_image_digest`, `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:40 checkpoint_rejects_omitted_root_partition_with_recomputed_image_digest`, `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:59 checkpoint_rejects_same_commit_truth_root_relabeling`.
- `recover_with` at 128: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/branch/owner_services/materialization_port_tests.rs:19 durability_replays_suspended_and_restored_materialization_roots`, `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs:147 checkpoint_recovery_preserves_abandoned_symbol_identity_frontiers`, `crates/worth-relational/src/tests/durability/contracts/adjacency_checkpoint.rs:4 checkpoint_rebuilds_current_and_historical_adjacency_kind_buckets`, `crates/worth-relational/src/tests/durability/contracts/associated_effects.rs:77 associated_effects_discover_cold_subject_without_reading_unrelated_history`, `crates/worth-relational/src/tests/durability/contracts/associated_effects_discovery.rs:11 native_custody_correction_enumerates_known_orphan_and_unrelated_association_metadata`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:5 durability_contract_recovery_rebuilds_branch_heads_and_latest_commit`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:38 durability_contract_recovery_preserves_aspect_bearing_patch_truth_and_history`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:95 durability_contract_recovery_preserves_relation_aspect_history_for_retained_audit_relations`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/checkpoint_state.rs:6 checkpoint_size_tracks_branch_state_not_covered_transitions`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/fates.rs:134 every_transition_fate_preserves_the_durable_before_live_boundary`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/recovery.rs:6 a_durable_fork_without_a_subsequent_commit_reopens_and_commits`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/recovery.rs:27 a_fork_covered_by_a_checkpoint_after_its_source_moves_accepts_a_target_tail`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/recovery.rs:49 a_fork_in_the_transition_tail_is_merged_before_the_target_commit`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/recovery.rs:72 lifecycle_postures_and_retired_names_survive_checkpoints_and_crashes`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/replay.rs:5 partial_replay_includes_an_intermediate_fork_without_its_own_commit`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/tampering.rs:36 a_sequence_gap_and_invalid_follows_positions_are_refused`, `crates/worth-relational/src/tests/durability/contracts/branch_transition/tampering.rs:72 a_commit_carrying_an_unknown_branch_cell_cannot_replace_its_missing_fork`, `crates/worth-relational/src/tests/durability/contracts/caller_association.rs:161 caller_association_survives_cold_checkpoint_history_and_journal_tail`, `crates/worth-relational/src/tests/durability/contracts/canonical_artifact_neutrality.rs:177 a_recovered_runtime_continues_with_the_reference_artifacts`, `crates/worth-relational/src/tests/durability/contracts/fork_tail_after_source_moved.rs:10 a_fork_commits_after_a_checkpoint_taken_once_its_source_moved_on`, `crates/worth-relational/src/tests/durability/contracts/lineage_event_allocator_recovery.rs:66 unresolved_durable_publication_blocks_descendants_and_recovers_last_checkpoint`, `crates/worth-relational/src/tests/durability/medium/recovered_publication.rs:9 a_recovered_runtime_publishes_its_next_commit_at_the_next_position`, `crates/worth-relational/src/tests/durability/medium/unresolved_fate.rs:100 reopening_resolves_an_absent_unresolved_commit_as_never_committed`, `crates/worth-relational/src/tests/durability/medium/unresolved_fate.rs:131 reopening_resolves_a_present_unresolved_commit_as_committed_exactly_once`, `crates/worth-relational/src/tests/history/milestone_7a/ancestry_query_matrix.rs:4 merge_ready_history_ancestry_queries_resolve_branch_reasoning`.

### `tests/support/durable_medium.rs`

No direct test declarations. Executing helper roots and claims:


### `tests/support/durable_medium/cold_source.rs`

No direct test declarations. Executing helper roots and claims:


### `tests/support/durable_medium/runtime_attachment.rs`

No direct test declarations. Executing helper roots and claims:

- `attached_test_medium` at 153: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/branch/coordination/managed_fork_cleanup.rs:92 managed_fork_refusal_and_success_drop_lookup_outside_native_coordination`, `crates/worth-relational/src/branch/owner_services/materialization_port_tests.rs:19 durability_replays_suspended_and_restored_materialization_roots`, `crates/worth-relational/src/commit_strategies/facade_tests/replay_recovery_failures.rs:5 replay_commit_reports_strategy_executor_unavailable_when_recovered_runtime_lacks_executor`, `crates/worth-relational/src/commit_strategies/facade_tests/replay_recovery_failures.rs:62 replay_commit_reports_strategy_execution_failure_when_recovered_executor_rejects`, `crates/worth-relational/src/durability/authority/checkpoint_concurrency_tests.rs:43 checkpoint_reconstruction_does_not_exclude_publication_after_capture`, `crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs:59 acknowledged_transition_admits_only_its_exact_successor_and_releases_snapshot`, `crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs:113 foreign_recovery_and_foreign_candidate_are_refused_before_effects`, `crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs:177 moved_source_and_stale_candidate_cannot_extend_recovery_lineage`, `crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs:212 refused_durable_transition_returns_its_authority_and_moves_nothing`, `crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs:254 unresolved_durable_transition_installs_nothing_and_names_its_fate`, `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:242 fresh_source_refuses_real_selected_empty_checkpoint`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:12 equal_exact_root_reuses_the_readmitted_partition_substrate`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:35 native_restore_work_counts_revised_history_sibling_roots_and_partition_images`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:132 shared_restored_substrate_keeps_a_pinned_reader_exact_after_next_write`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:179 divergent_sibling_is_not_reused_as_the_storage_mirror`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:235 divergent_mirror_partition_rebuilds_cross_partition_adjacency_after_reuse`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:299 recovered_main_and_sibling_catalogs_reuse_exact_canonical_payloads`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:348 equal_image_cannot_bypass_global_contract_readmission`, `crates/worth-relational/src/durability/data/native_checkpoint/component_custody/tests.rs:101 native_custody_correction_delayed_canonical_selection_retains_actual_root_and_interrupted_ack`, `crates/worth-relational/src/durability/data/native_checkpoint/component_custody/tests.rs:105 native_custody_correction_delayed_empty_selection_retains_actual_schema_and_interrupted_ack`, `crates/worth-relational/src/durability/data/native_checkpoint/tests/selected_ack.rs:24 selected_ack_partial_transfer_preserves_later_cells_and_resumes_same_capture`, `crates/worth-relational/src/durability/data/native_checkpoint/tests/selected_ack.rs:119 selected_ack_exposes_backed_data_before_retirement_and_releases_holds_only_on_completion`, `crates/worth-relational/src/durability/log/native_checkpoint_codec/inspection/tests.rs:130 inspection_compares_every_native_model_value_as_original_bytes`, `crates/worth-relational/src/durability/log/native_checkpoint_codec/inspection/tests.rs:179 inspection_refuses_unknown_root_allocator_and_row_fields`, `crates/worth-relational/src/durability/log/native_checkpoint_codec/inspection/tests.rs:204 restore_refuses_unknown_fields_before_publishing_any_model_state`.

### `tests/support/durable_medium/tamper.rs`

No direct test declarations. Executing helper roots and claims:


### `tests/support/durable_medium/transition_contents.rs`

No direct test declarations. Executing helper roots and claims:


### `tests/support/runtime.rs`

No direct test declarations. Executing helper roots and claims:

- `durable_runtime_with_test_schema` at 137: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/branch/coordination/managed_fork_cleanup.rs:92 managed_fork_refusal_and_success_drop_lookup_outside_native_coordination`, `crates/worth-relational/src/branch/owner_services/materialization_port_tests.rs:19 durability_replays_suspended_and_restored_materialization_roots`, `crates/worth-relational/src/branch/root_authoritative_payload_tests.rs:225 schema_transition_descriptors_are_each_bound_into_the_committed_root`, `crates/worth-relational/src/commit_strategies/facade_tests/replay_recovery_failures.rs:5 replay_commit_reports_strategy_executor_unavailable_when_recovered_runtime_lacks_executor`, `crates/worth-relational/src/commit_strategies/facade_tests/replay_recovery_failures.rs:62 replay_commit_reports_strategy_execution_failure_when_recovered_executor_rejects`, `crates/worth-relational/src/durability/authority/checkpoint_complete/tests/empty_schema.rs:130 complete_refuses_recovered_empty_without_original_birth_authority`, `crates/worth-relational/src/durability/authority/checkpoint_complete/tests/partial_aspect_update.rs:11 complete_preserves_partial_update_without_demanding_untouched_aspect_reference`, `crates/worth-relational/src/durability/authority/checkpoint_complete/tests/pinned_delete.rs:9 complete_fold_matches_original_relation_delete_with_snapshot_and_replay_pins`, `crates/worth-relational/src/durability/authority/checkpoint_concurrency_tests.rs:9 completing_an_older_capture_cannot_replace_the_compacted_transition_basis`, `crates/worth-relational/src/durability/authority/checkpoint_concurrency_tests.rs:43 checkpoint_reconstruction_does_not_exclude_publication_after_capture`, `crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs:59 acknowledged_transition_admits_only_its_exact_successor_and_releases_snapshot`, `crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs:113 foreign_recovery_and_foreign_candidate_are_refused_before_effects`, `crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs:177 moved_source_and_stale_candidate_cannot_extend_recovery_lineage`, `crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs:212 refused_durable_transition_returns_its_authority_and_moves_nothing`, `crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs:254 unresolved_durable_transition_installs_nothing_and_names_its_fate`, `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:23 fresh_source_keeps_actual_runtime_root_and_installed_schema_for_complete`, `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:168 fresh_source_refuses_foreign_and_nonempty_actual_sources_without_rebuild`, `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:196 fresh_receipt_refuses_actual_movement_and_preserves_original_cancellation`, `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:242 fresh_source_refuses_real_selected_empty_checkpoint`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:12 equal_exact_root_reuses_the_readmitted_partition_substrate`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:35 native_restore_work_counts_revised_history_sibling_roots_and_partition_images`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:132 shared_restored_substrate_keeps_a_pinned_reader_exact_after_next_write`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:179 divergent_sibling_is_not_reused_as_the_storage_mirror`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:235 divergent_mirror_partition_rebuilds_cross_partition_adjacency_after_reuse`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:299 recovered_main_and_sibling_catalogs_reuse_exact_canonical_payloads`.
- `durable_runtime_with_test_schema_profile` at 141: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/branch/coordination/managed_fork_cleanup.rs:92 managed_fork_refusal_and_success_drop_lookup_outside_native_coordination`, `crates/worth-relational/src/branch/owner_services/materialization_port_tests.rs:19 durability_replays_suspended_and_restored_materialization_roots`, `crates/worth-relational/src/branch/root_authoritative_payload_tests.rs:225 schema_transition_descriptors_are_each_bound_into_the_committed_root`, `crates/worth-relational/src/commit_strategies/facade_tests/replay_recovery_failures.rs:5 replay_commit_reports_strategy_executor_unavailable_when_recovered_runtime_lacks_executor`, `crates/worth-relational/src/commit_strategies/facade_tests/replay_recovery_failures.rs:62 replay_commit_reports_strategy_execution_failure_when_recovered_executor_rejects`, `crates/worth-relational/src/durability/authority/checkpoint_complete/tests/empty_schema.rs:130 complete_refuses_recovered_empty_without_original_birth_authority`, `crates/worth-relational/src/durability/authority/checkpoint_complete/tests/partial_aspect_update.rs:11 complete_preserves_partial_update_without_demanding_untouched_aspect_reference`, `crates/worth-relational/src/durability/authority/checkpoint_complete/tests/pinned_delete.rs:9 complete_fold_matches_original_relation_delete_with_snapshot_and_replay_pins`, `crates/worth-relational/src/durability/authority/checkpoint_concurrency_tests.rs:9 completing_an_older_capture_cannot_replace_the_compacted_transition_basis`, `crates/worth-relational/src/durability/authority/checkpoint_concurrency_tests.rs:43 checkpoint_reconstruction_does_not_exclude_publication_after_capture`, `crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs:59 acknowledged_transition_admits_only_its_exact_successor_and_releases_snapshot`, `crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs:113 foreign_recovery_and_foreign_candidate_are_refused_before_effects`, `crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs:177 moved_source_and_stale_candidate_cannot_extend_recovery_lineage`, `crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs:212 refused_durable_transition_returns_its_authority_and_moves_nothing`, `crates/worth-relational/src/durability/authority/checkpoint_transition/tests.rs:254 unresolved_durable_transition_installs_nothing_and_names_its_fate`, `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:23 fresh_source_keeps_actual_runtime_root_and_installed_schema_for_complete`, `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:168 fresh_source_refuses_foreign_and_nonempty_actual_sources_without_rebuild`, `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:196 fresh_receipt_refuses_actual_movement_and_preserves_original_cancellation`, `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:242 fresh_source_refuses_real_selected_empty_checkpoint`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:12 equal_exact_root_reuses_the_readmitted_partition_substrate`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:35 native_restore_work_counts_revised_history_sibling_roots_and_partition_images`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:132 shared_restored_substrate_keeps_a_pinned_reader_exact_after_next_write`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:179 divergent_sibling_is_not_reused_as_the_storage_mirror`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:235 divergent_mirror_partition_rebuilds_cross_partition_adjacency_after_reuse`, `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:299 recovered_main_and_sibling_catalogs_reuse_exact_canonical_payloads`.
- `durable_runtime_with_declared_aspect_schema` at 151: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:38 durability_contract_recovery_preserves_aspect_bearing_patch_truth_and_history`, `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:95 durability_contract_recovery_preserves_relation_aspect_history_for_retained_audit_relations`, `crates/worth-relational/src/tests/durability/contracts/branch_root_schema_binding.rs:58 recovery_rejects_a_schema_carrier_swapped_between_exact_roots`, `crates/worth-relational/src/tests/durability/contracts/schema_transition_continuity.rs:162 durability_contract_failure_aspect_plan_mismatch_is_explicit`, `crates/worth-relational/src/tests/history/queries/aspect_history.rs:187 bulk_like_aspect_history_filters_and_query_packets_stay_stable_after_recovery`, `crates/worth-relational/src/tests/history/replay/derived_index_and_workload.rs:73 replay_and_recovery_preserve_aspect_bearing_truth_across_a_hostile_mixed_workload`, `crates/worth-relational/src/tests/history/replay/derived_index_and_workload.rs:201 hostile_commit_replay_equivalence_test`, `crates/worth-relational/src/tests/query/concurrency.rs:11 concurrent_snapshot_and_version_reads_match_serial_truth`, `crates/worth-relational/src/tests/query/concurrency.rs:100 concurrent_read_pressure_keeps_cache_diagnostics_coherent`, `crates/worth-relational/src/tests/query/concurrency.rs:205 concurrent_pinned_traversal_reads_stay_snapshot_stable_under_hot_rewrite_pressure`, `crates/worth-relational/src/tests/query/concurrency/relation_index_parity.rs:4 concurrent_relation_index_certification_parity_stays_stable_under_scheduler_pressure`, `crates/worth-relational/src/tests/query/execution/aspect_filtered_scans.rs:129 planned_query_execution_aspect_filter_supports_field_mask_presence`, `crates/worth-relational/src/tests/query/execution/leased_explicit_targets.rs:46 leased_explicit_read_preserves_packet_identity_and_reports_stops`, `crates/worth-relational/src/tests/query/execution/leased_explicit_targets.rs:190 leased_field_filter_charges_captured_comparison_bytes`, `crates/worth-relational/src/tests/query/execution/leased_explicit_targets.rs:259 leased_kind_scan_stops_at_a_record_checkpoint`, `crates/worth-relational/src/tests/query/execution/leased_explicit_targets.rs:326 leased_kind_scan_rejects_one_record_larger_than_its_packet_ceiling`, `crates/worth-relational/src/tests/query/execution/leased_explicit_targets/traversal_and_values.rs:4 leased_traversal_stops_inside_high_fanout`, `crates/worth-relational/src/tests/query/execution/leased_explicit_targets/traversal_and_values.rs:84 leased_any_of_scan_matches_serial_with_reversed_values`, `crates/worth-relational/src/tests/query/execution/read_law_matrix.rs:19 runtime_read_law_matrix_keeps_rows_dependencies_revisions_and_work`, `crates/worth-relational/src/tests/query/execution/read_law_matrix/bounded_index.rs:12 runtime_bounded_index_law_keeps_cold_stop_limit_and_generation`, `crates/worth-relational/src/tests/query/execution/read_law_matrix/refusals.rs:32 runtime_read_law_plan_absence_and_stopped_request`, `crates/worth-relational/src/tests/query/execution/read_law_matrix/refusals.rs:98 runtime_read_law_bounded_index_refusal_values`, `crates/worth-relational/src/tests/query/execution/read_law_matrix/remaining_door_stops.rs:51 runtime_read_law_view_input_stops_and_refuses_by_value`, `crates/worth-relational/src/tests/query/execution/read_law_matrix/remaining_door_stops.rs:91 runtime_read_law_selected_view_stops_and_refuses_by_value`, `crates/worth-relational/src/tests/query/execution/read_law_matrix/remaining_door_stops.rs:136 runtime_read_law_relation_join_stops_and_refuses_by_value`.

### `tests/transactions/core/aggregate_identity/cold_recovery_fixture.rs`

No direct test declarations. Executing helper roots and claims:

- `reopen` at 132: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/transactions/core/aggregate_identity/cold_recovery.rs:34 cold_split_and_warm_split_publish_identical_aggregate_names_without_history_loads`, `crates/worth-relational/src/tests/transactions/core/aggregate_identity/cold_recovery.rs:59 cold_aggregate_source_refusal_publishes_nothing_and_can_retry`.
  - UNCLASSIFIED assertion 206: `assert!(matches!( attachment.publish(&artifact), DurablePublicationFate::Committed ))`.
  - UNCLASSIFIED assertion 227: `assert!( source.reads.lock().unwrap().is_empty(), "reopen keeps record/history members cold" )`.
  - UNCLASSIFIED assertion 231: `assert_eq!(source.journal_reads.load(Ordering::Relaxed), 0)`.
  - UNCLASSIFIED assertion 232: `assert!( (1..4).contains(&source.canonical_reads.lock().unwrap().len()), "only actual distinct head envelopes are read" )`.

### `tests/transactions/core/aggregate_identity/components/fixtures.rs`

No direct test declarations. Executing helper roots and claims:

- `persisted` at 72: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/transactions/core/aggregate_identity/cold_recovery.rs:34 cold_split_and_warm_split_publish_identical_aggregate_names_without_history_loads`, `crates/worth-relational/src/tests/transactions/core/aggregate_identity/cold_recovery.rs:59 cold_aggregate_source_refusal_publishes_nothing_and_can_retry`, `crates/worth-relational/src/tests/transactions/core/aggregate_identity/components.rs:156 checkpoint_tail_recovery_and_cert_replay_keep_component_memberships_and_split_merge_lineage`, `crates/worth-relational/src/tests/transactions/core/aggregate_identity/logical_components/checkpoint.rs:13 physical_membership_recipe_and_alternating_derived_catalog_restore_from_real_checkpoints`, `crates/worth-relational/src/tests/transactions/core/aggregate_identity/logical_components/checkpoint.rs:124 changed_configured_domain_cannot_replace_the_actual_recovered_root_or_its_handles`, `crates/worth-relational/src/tests/transactions/core/aggregate_identity/logical_components/seeded.rs:128 seeded_full_key_differential_fork_recovery_and_replay_keep_actual_names_and_lineage`, `crates/worth-relational/src/tests/transactions/core/aggregate_identity/logical_components/serialization.rs:20 required_content_authority_keeps_exact_shape_and_positional_empty_tails`, `crates/worth-relational/src/tests/transactions/core/aggregate_identity/recovery.rs:5 checkpoint_schema_readmission_and_tail_replay_rederive_identical_private_records`.

### `tests/transactions/core/aggregate_identity/fixtures.rs`

No direct test declarations. Executing helper roots and claims:

- `persisted` at 76: UNCLASSIFIED helper boundary; zero declarations do not authorize deletion.
  - Syntactic candidate caller roots (qualified calls/name collisions need review): `crates/worth-relational/src/tests/transactions/core/aggregate_identity/cold_recovery.rs:34 cold_split_and_warm_split_publish_identical_aggregate_names_without_history_loads`, `crates/worth-relational/src/tests/transactions/core/aggregate_identity/cold_recovery.rs:59 cold_aggregate_source_refusal_publishes_nothing_and_can_retry`, `crates/worth-relational/src/tests/transactions/core/aggregate_identity/components.rs:156 checkpoint_tail_recovery_and_cert_replay_keep_component_memberships_and_split_merge_lineage`, `crates/worth-relational/src/tests/transactions/core/aggregate_identity/logical_components/checkpoint.rs:13 physical_membership_recipe_and_alternating_derived_catalog_restore_from_real_checkpoints`, `crates/worth-relational/src/tests/transactions/core/aggregate_identity/logical_components/checkpoint.rs:124 changed_configured_domain_cannot_replace_the_actual_recovered_root_or_its_handles`, `crates/worth-relational/src/tests/transactions/core/aggregate_identity/logical_components/seeded.rs:128 seeded_full_key_differential_fork_recovery_and_replay_keep_actual_names_and_lineage`, `crates/worth-relational/src/tests/transactions/core/aggregate_identity/logical_components/serialization.rs:20 required_content_authority_keeps_exact_shape_and_positional_empty_tails`, `crates/worth-relational/src/tests/transactions/core/aggregate_identity/recovery.rs:5 checkpoint_schema_readmission_and_tail_replay_rederive_identical_private_records`.

### `tests/transactions/core/aggregate_identity/logical_components/checkpoint.rs`

#### `physical_membership_recipe_and_alternating_derived_catalog_restore_from_real_checkpoints`

Source: `crates/worth-relational/src/tests/transactions/core/aggregate_identity/logical_components/checkpoint.rs:13`.

- **UNCLASSIFIED** 43: `assert_eq!( extended.derived_relation(&physical_identity), Some(handle.clone()) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 58: `assert_eq!(expected.len(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 65: `assert_eq!( pin.derived_relations_from(execution, &handle, member, None) .unwrap() .data() .endpoints(), expected )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 72: `assert_eq!(super::laws::current_truth(execution, &recovered), truth)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 80: `assert_eq!( extended.derived_relation(&identity("Law")), Some(handle.clone()) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 101: `assert_eq!( pin.derived_relations_from(execution, &handle, carrier, None) .unwrap() .data() .endpoints() .len(), 2 )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 110: `assert_eq!( read(execution, &recovered, &snapshot, key(carrier, side)).identity(), name )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 117: `assert_eq!(original.len(), restored.len())` - semantic boundary still requires direct review; no move authorized.
- UNCLASSIFIED transitive claims: `crates/worth-relational/src/tests/support/truth_comparison.rs:60 assert_durable_commit_meaning_eq`. Preserve/classify those helper bodies; direct rows alone do not clear this test.

#### `changed_configured_domain_cannot_replace_the_actual_recovered_root_or_its_handles`

Source: `crates/worth-relational/src/tests/transactions/core/aggregate_identity/logical_components/checkpoint.rs:124`.

- **UNCLASSIFIED** 147: `assert_eq!( pin.schema().projected_member_domain(&domain()), Some(original_domain.clone()) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 151: `assert!(pin .derived_relations_from(execution, &definition, key(carrier, 0), None) .unwrap() .data() .endpoints() .is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 181: `assert_eq!( expect_durability_failure(error).class, crate::durability::data::RecoveryFailureClass::SchemaMismatch )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 185: `assert_eq!(restored.history().commit_envelopes_snapshot(), before)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 195: `assert_eq!( expect_durability_failure(denied).class, crate::durability::data::RecoveryFailureClass::DeclaredSchemaMismatch )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 199: `assert_eq!( restored .durability_authority() .native_checkpoint(execution, execution.allocation_policy()) .unwrap(), before )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 217: `assert_eq!(actual_domain.parts(), original_domain.parts())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 218: `assert_ne!(actual_domain, foreign_domain)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 219: `assert!(!pin.schema().admits_projected_member_domain(&foreign_domain))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 224: `assert_eq!( error, DerivedRelationReadDenial::Definition(DerivedRelationDenial::ForeignDefinition) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 232: `assert!(pin .derived_relations_from(execution, &actual_definition, key(carrier, 0), None) .unwrap() .data() .endpoints() .is_empty())` - semantic boundary still requires direct review; no move authorized.

### `tests/transactions/core/authored_write_history/adjacency/audit.rs`

#### `audit_lifecycle_coverage_survives_unrelated_history_discard_and_durable_recovery`

Source: `crates/worth-relational/src/tests/transactions/core/authored_write_history/adjacency/audit.rs:14`.

- **UNCLASSIFIED** 35: `assert!( before .compare_adjacency_support(&execution, &after, c, KindId(2), ReadDirection::Outgoing) .unwrap(), "unrelated audit retention preserves the exact live relation population" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 41: `assert_eq!(before.compare_adjacency_support(&execution, &after, a, KindId(2), ReadDirection::Outgoing), Err(RelationalAdjacencySupportDenial::UnsupportedVisibility), "complete authored history does not turn retained audit visibility into a readable live list")` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 50: `assert!(touches.contains(&Touch::RelationLifecycle { relation, kind: KindId(2) }))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 54: `assert!(touches.contains(&Touch::RelationMembership { relation, kind: KindId(2), source: a, target: b }))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 60: `assert!( !touches .iter() .any(|touch| matches!(touch, Touch::AdjacencyRevision { .. })), "retention must not fabricate a physical adjacency delta/revision" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 77: `assert!( before .compare_adjacency_support( &execution, &grown, c, KindId(2), ReadDirection::Outgoing ) .unwrap(), "unrelated relation population must not change this exact live support" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 93: `assert_eq!( scan(&runtime, execution, &source, &target, anchor, direction), [retired.commit.commit_id] )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 98: `assert!(scan( &runtime, execution, &source, &target, c, Direction::Outgoing ) .is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 113: `assert_eq!( scan( &runtime, execution, &source, &target, a, Direction::Outgoing ), [retired.commit.commit_id] )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 137: `assert_eq!( scan( &recovered, execution, &recovered_source, &recovered_target, anchor, direction ), [retired.commit.commit_id] )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 149: `assert!(scan( &recovered, execution, &recovered_source, &recovered_target, c, Direction::Outgoing ) .is_empty())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 163: `assert_eq!(cold.descriptive_touches(), canonical.descriptive_touches())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 164: `assert_eq!(cold.authoritative_payload_digest().unwrap(), digest)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 165: `assert_eq!(cold.encode_authoritative_payload().unwrap(), encoded)` - semantic boundary still requires direct review; no move authorized.

### `tests/transactions/core/authored_write_history/adjacency/recovery.rs`

#### `adjacency_occurrence_identities_rebuild_from_cold_canonical_history`

Source: `crates/worth-relational/src/tests/transactions/core/authored_write_history/adjacency/recovery.rs:5`.

- **UNCLASSIFIED** 27: `assert_eq!( scan( &runtime, execution, &source, &target, a, Direction::Outgoing ), expected )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 44: `assert_eq!( scan( &runtime, execution, &source, &target, a, Direction::Outgoing ), expected )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 68: `assert_eq!( scan( &recovered, execution, &recovered_source, &recovered_target, a, Direction::Outgoing ), expected )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 79: `assert_eq!( scan( &recovered, execution, &recovered_source, &recovered_target, b, Direction::Incoming ), [created.commit.commit_id, moved.commit.commit_id] )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 90: `assert_eq!( scan( &recovered, execution, &recovered_source, &recovered_target, c, Direction::Incoming ), [ moved.commit.commit_id, noop.commit.commit_id, removed.commit.commit_id ] )` - semantic boundary still requires direct review; no move authorized.

### `tests/transactions/core/authored_write_history/indexed/recovery.rs`

#### `index_rebuild_requires_live_verified_canonical_evidence`

Source: `crates/worth-relational/src/tests/transactions/core/authored_write_history/indexed/recovery.rs:5`.

- C4 action 101: `old_shape.body_mut_for_test().authored_index_writes = Default::default();`.
- **C4** 15: `assert!(matches!( runtime.authored_write_history_cursor( execution, &pre_install_source, selected(&runtime, &source), &[key(&definition, "before")], ), Err(RelationalAuthoredWriteDenial::IncompleteIndexHistory) ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 35: `assert!(matches!( runtime.authored_write_history_cursor( execution, &source, selected(&runtime, &target), &[fabricated], ), Err(RelationalAuthoredWriteDenial::IncompleteIndexHistory) ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 56: `assert!(runtime .rebuild_authored_write_generation( ExecutionRequest::serial(&empty), selected(&runtime, &target) ) .is_err())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 62: `assert_eq!( scan( &runtime, execution, &source, &target, key(&definition, "before") ), expected )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 78: `assert_eq!( scan( &runtime, execution, &source, &target, key(&definition, "before") ), expected )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 95: `assert_eq!(decoded, *envelope)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 96: `assert!(decoded.authored_index_writes.verified_entries().is_none())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 105: `assert!(legacy.authored_index_writes.is_unavailable())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 119: `assert_eq!( scan( &recovered, execution, &recovered_source, &recovered_target, RelationalAuthoredWriteSelector::Record { record: RecordRef::Entity(entity), scope: RelationalAuthoredWriteScope::Field( aspect_key("name"), CanonicalFieldPath::single(field_key("name")), ), }, ) .0, [equal.commit.commit_id], "cold field history retains the exact authored no-op despite an unchanged value" )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 137: `assert!(matches!( recovered.authored_write_history_cursor( execution, &recovered_source, selected(&recovered, &recovered_target), &[key(&definition, "before")] ), Err(RelationalAuthoredWriteDenial::IncompleteIndexHistory) ))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 149: `assert_eq!( scan( &recovered, execution, &fresh_source, &fresh_target, key(&definition, "before") ) .0, [fresh.commit.commit_id], "an earlier decoded gap outside this open interval does not poison live history" )` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `tests/transactions/core/owner_lifecycle_phase3b.rs`

#### `phase3b_owner_drop_during_off_thread_settlement_completes`

Source: `crates/worth-relational/src/tests/transactions/core/owner_lifecycle_phase3b.rs:22`.

- **UNCLASSIFIED** 58: `assert_eq!( closed_receiver.recv_timeout(NEGATIVE_BUDGET), Err(RecvTimeoutError::Timeout), "the owner waits for the settlement it already admitted", )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 65: `assert_eq!( join_completed_worker( settled_receiver.recv_timeout(COMPLETION_BUDGET), settling, COMPLETION_BUDGET, "an admitted settlement completes even though its owner is closing" ) .expect("the admitted settlement produces a real receipt"), commit_id, )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 82: `assert!(!port.retains_pending_settlement(commit_id))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 83: `assert!(matches!( port.repair_pending_publication_settlement(commit_id), Err(PublicationSettlementRepairError::OwnerUnavailable { runtime_instance_id: observed, }) if observed == runtime_instance_id ))` - semantic boundary still requires direct review; no move authorized.

#### `phase3b_settlement_admitted_after_owner_drop_is_denied`

Source: `crates/worth-relational/src/tests/transactions/core/owner_lifecycle_phase3b.rs:95`.

- **UNCLASSIFIED** 132: `assert!( matches!( port.settle_performed_publication(late), Err(crate::transactions::data::TransactionCommitError::PublicationDenied { denial: crate::mvcc::RelationalPublicationDenial::OwnerUnavailable { runtime_instance_id: observed, }, .. }) if observed == runtime_instance_id ), "a settlement not yet admitted when the owner began closing is denied", )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 144: `assert!(matches!( port.repair_pending_publication_settlement(late_commit_id), Err(PublicationSettlementRepairError::OwnerUnavailable { .. }) ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 150: `assert_eq!( join_completed_worker( settled_receiver.recv_timeout(COMPLETION_BUDGET), settling, COMPLETION_BUDGET, "the admitted settlement is not denied by the close it overlapped" ) .expect("the admitted settlement produces a real receipt"), paused_commit_id, )` - semantic boundary still requires direct review; no move authorized.

#### `phase3b_recovery_replacement_closes_the_prior_runtimes_ports`

Source: `crates/worth-relational/src/tests/transactions/core/owner_lifecycle_phase3b.rs:169`.

- **UNCLASSIFIED** 179: `assert_eq!(port.runtime_instance_id(), replaced_instance_id)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 192: `assert_ne!( runtime.runtime_instance_id(), replaced_instance_id, "recovery installs a different runtime instance", )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 197: `assert!(matches!( port.repair_pending_publication_settlement(seeded_commit_id), Err(PublicationSettlementRepairError::OwnerUnavailable { runtime_instance_id: observed, }) if observed == replaced_instance_id ))` - semantic boundary still requires direct review; no move authorized.

#### `phase3b_removed_lifecycle_premises_stay_removed`

Source: `crates/worth-relational/src/tests/transactions/core/owner_lifecycle_phase3b.rs:208`.

- **UNCLASSIFIED** 214: `assert!( !RUNTIME_STATE.contains("DerefMut"), "exclusive access to shared runtime state is not proved by an operator", )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 218: `assert!( !RUNTIME_STATE.contains("fn from_shared"), "a handle is minted from a tenure, never from a bare shared pointer", )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 222: `assert!( !RUNTIME_STATE.contains("impl Drop for RelationalRuntimeState"), "closing is owner work, not a consequence of the last pointer being released", )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 226: `assert!( RUNTIME_STATE.contains("impl Drop for RelationalRuntime {"), "the owner handle is what closes, and its own drop runs before its fields", )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 230: `assert!( !SETTLEMENT_PORT.contains("AdmittedSettlementOwner"), "an admitted operation carries a handle whose tenure is its admission", )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 234: `assert!( !PREPARATION_RUNTIME.contains("RelationalPreparationConfiguration"), "there is one configuration authority, not a preparation-side copy of it", )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 238: `assert!( !PREPARATION_RUNTIME.contains("synchronize_preparation_configuration"), "nothing has to be reconciled back into a second configuration", )` - semantic boundary still requires direct review; no move authorized.

### `tests/transactions/core/owner_service_phase3.rs`

#### `phase3_settlement_receivers_take_a_shared_borrow`

Source: `crates/worth-relational/src/tests/transactions/core/owner_service_phase3.rs:13`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.

#### `phase3b_owner_authority_and_owner_services_keep_their_receivers`

Source: `crates/worth-relational/src/tests/transactions/core/owner_service_phase3.rs:41`.

- UNCLASSIFIED: inspect match/panic outcomes and called assertion helpers; no direct assertion macro captured.

#### `phase3_settlement_port_is_a_cloneable_shared_borrow_service`

Source: `crates/worth-relational/src/tests/transactions/core/owner_service_phase3.rs:59`.

- **UNCLASSIFIED** 73: `assert_eq!( cloned.runtime_instance_id(), runtime_shared.runtime_instance_id(), )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 88: `assert!(!cloned.retains_pending_settlement(committed.commit.commit_id))` - semantic boundary still requires direct review; no move authorized.

#### `phase3_settlement_port_denies_after_its_runtime_owner_closes`

Source: `crates/worth-relational/src/tests/transactions/core/owner_service_phase3.rs:96`.

- **UNCLASSIFIED** 106: `assert!(settlement.retains_pending_settlement(commit_id))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 110: `assert!(!settlement.retains_pending_settlement(commit_id))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 111: `assert!(matches!( settlement.repair_pending_publication_settlement(commit_id), Err(PublicationSettlementRepairError::OwnerUnavailable { runtime_instance_id: observed, }) if observed == runtime_instance_id ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 117: `assert!(matches!( settlement.settle_performed_publication(performed), Err(TransactionCommitError::PublicationDenied { denial: crate::mvcc::RelationalPublicationDenial::OwnerUnavailable { runtime_instance_id: observed, }, .. }) if observed == runtime_instance_id ))` - semantic boundary still requires direct review; no move authorized.

#### `phase3_settlement_port_settles_a_performed_publication_from_another_thread`

Source: `crates/worth-relational/src/tests/transactions/core/owner_service_phase3.rs:132`.

- **UNCLASSIFIED** 148: `assert_eq!(settled.commit.commit_id, commit_id)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 149: `assert!(!settlement.retains_pending_settlement(commit_id))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 150: `assert_eq!( runtime .history() .immutable_commit_receipt(commit_id) .expect("off-thread settlement is visible to the owner") .commit_id, commit_id, )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 159: `assert_eq!(child.commit.parents, vec![commit_id])` - semantic boundary still requires direct review; no move authorized.

#### `phase3_settlement_port_repairs_a_lost_witness_by_identity`

Source: `crates/worth-relational/src/tests/transactions/core/owner_service_phase3.rs:168`.

- **UNCLASSIFIED** 179: `assert!(settlement.retains_pending_settlement(identity_commit_id))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 183: `assert_eq!(recovered.commit_id, identity_commit_id)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 184: `assert!(!settlement.retains_pending_settlement(identity_commit_id))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 187: `assert_eq!(child.commit.parents, vec![identity_commit_id])` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 188: `assert_eq!( runtime .durability() .durable_log() .iter() .filter(|entry| entry.envelope().commit.commit_id == identity_commit_id) .count(), 1, "the repaired settlement is recorded exactly once", )` - semantic boundary still requires direct review; no move authorized.

#### `phase3_foreign_settlement_denial_preserves_source_recovery`

Source: `crates/worth-relational/src/tests/transactions/core/owner_service_phase3.rs:205`.

- **UNCLASSIFIED** 232: `assert!(source_settlement.retains_pending_settlement(commit_id))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 246: `assert_eq!(expected_runtime_instance_id, foreign.runtime_instance_id())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 247: `assert_eq!(actual_runtime_instance_id, source.runtime_instance_id())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 252: `assert_eq!( foreign.durability().durable_log().len(), foreign_durable_before )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 256: `assert!(source_settlement.retains_pending_settlement(commit_id))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 260: `assert_eq!(recovered.commit_id, commit_id)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 261: `assert_eq!( source.durability().durable_log().len(), source_durable_before + 1, "the source settlement appends exactly once", )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 266: `assert!(!source_settlement.retains_pending_settlement(commit_id))` - semantic boundary still requires direct review; no move authorized.

### `tests/transactions/core/pending_settlement_recovery.rs`

#### `dropping_the_performed_witness_leaves_settlement_recoverable`

Source: `crates/worth-relational/src/tests/transactions/core/pending_settlement_recovery.rs:11`.

- **UNCLASSIFIED** 26: `assert_eq!( record.abandoned_capability_count(), 1, "abandoning the witness is observable" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 31: `assert_eq!( runtime.publication_binding().pending_settlement_count(), 1, "abandoning the witness may not release the obligation" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 43: `assert!(runtime .prepare_branch_transaction(execution, blocked) .unwrap_err() .detail() .contains("requires explicit owner settlement"))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 52: `assert_eq!(repaired.commit_id, commit_id)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 53: `assert_eq!(runtime.publication_binding().pending_settlement_count(), 0)` - semantic boundary still requires direct review; no move authorized.
- **C2** 55: `assert_eq!( runtime.visibility.published_snapshot_handle_count(), 0, "repair without a witness closes the published snapshot it opened" )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 62: `assert_eq!(child.commit.parents, vec![commit_id])` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 63: `assert_eq!(durable_appends(&runtime, commit_id), 1)` - semantic boundary still requires direct review; no move authorized.

#### `repeated_terminal_settlement_returns_one_receipt_and_one_durable_append`

Source: `crates/worth-relational/src/tests/transactions/core/pending_settlement_recovery.rs:70`.

- **UNCLASSIFIED** 81: `assert_eq!(settled.commit.commit_id, commit_id)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 86: `assert_eq!(repeated.commit_id, commit_id)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 87: `assert_eq!(repeated, settled.commit)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 88: `assert_eq!( durable_appends(&runtime, commit_id), 1, "asking again performs no second durable effect" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 93: `assert_eq!(runtime.publication_binding().pending_settlement_count(), 0)` - semantic boundary still requires direct review; no move authorized.

#### `repair_and_witness_settlement_execute_exactly_once`

Source: `crates/worth-relational/src/tests/transactions/core/pending_settlement_recovery.rs:100`.

- **UNCLASSIFIED** 112: `assert_eq!(repaired.commit_id, commit_id)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 113: `assert_eq!(runtime.publication_binding().pending_settlement_count(), 0)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 118: `assert_eq!(settled.commit.commit_id, commit_id)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 119: `assert_eq!(settled.commit, repaired)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 120: `assert_eq!( durable_appends(&runtime, commit_id), 1, "two settlement callers produce exactly one durable append" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 127: `assert_eq!(child.commit.parents, vec![commit_id])` - semantic boundary still requires direct review; no move authorized.

#### `owner_loss_retires_every_pending_settlement_exactly_once`

Source: `crates/worth-relational/src/tests/transactions/core/pending_settlement_recovery.rs:135`.

- **UNCLASSIFIED** 144: `assert!(!binding.settlement_admission_is_closed())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 145: `assert_eq!(binding.pending_settlement_count(), 1)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 149: `assert!( binding.settlement_admission_is_closed(), "a lost owner admits no further settlement" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 153: `assert_eq!( binding.pending_settlement_count(), 0, "owner loss drains its pending registry" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 158: `assert_eq!( binding.pending_settlement_owner_loss_count(), 1, "each retired pending settlement is accounted exactly once" )` - semantic boundary still requires direct review; no move authorized.

#### `interruption_after_movement_retains_its_pending_settlement`

Source: `crates/worth-relational/src/tests/transactions/core/pending_settlement_recovery.rs:168`.

- **UNCLASSIFIED** 213: `assert_ne!( runtime .branch_reference_state(&BranchId("main".to_owned())) .unwrap(), reference_before, )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 219: `assert_eq!( performed .late_interruption() .expect("late interruption is attached to the performed witness") .interruption(), interruption )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 226: `assert_eq!( runtime.publication_binding().pending_settlement_count(), 1, "movement that happened keeps its recoverable settlement record" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 234: `assert_eq!(runtime.publication_binding().pending_settlement_count(), 0)` - semantic boundary still requires direct review; no move authorized.

### `tests/transactions/core/publication_settlement.rs`

#### `direct_publication_settles_before_a_child_can_acknowledge`

Source: `crates/worth-relational/src/tests/transactions/core/publication_settlement.rs:6`.

- **UNCLASSIFIED** 34: `assert!(runtime .history() .recent_commit_ids(None, 8) .contains(&parent_id))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 38: `assert_eq!( runtime .publication() .read_patch_stream(execution, PatchStreamRequest::default()) .expect("direct parent is stream-visible") .latest_commit_id, Some(parent_id) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 50: `assert_eq!(subscriber.latest_commit_id, Some(parent_id))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 51: `assert!(subscriber .patches .iter() .any(|patch| patch.position == parent_position))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 59: `assert_eq!( fork_denial, crate::runtime::RelationalRuntimeForkDenial::PerformedPublicationRequiresSettlement { commit_id: parent_id, } )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 75: `assert!(format!("{child_denial:?}").contains("requires explicit owner settlement"))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 82: `assert_eq!(child.commit.parents, vec![parent_id])` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 86: `assert_eq!(tail.len(), 2, "checkpoint tail contains parent then child")` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 87: `assert_eq!(tail[0].envelope().commit.commit_id, parent_id)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 88: `assert_eq!(tail[0].position(), parent_position)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 89: `assert_eq!(tail[1].envelope().commit.commit_id, child.commit.commit_id)` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 90: `assert!(tail[0].position() < tail[1].position())` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 106: `assert_eq!( recovered .history() .immutable_commit_receipt(parent_id) .expect("settled direct parent recovers") .commit_id, parent_id )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 114: `assert_eq!( recovered .history() .branch_head(&BranchId("main".to_owned())) .expect("ordinary child remains the recovered head") .commit_id, child.commit.commit_id )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 127: `assert!(child_position < continued_position)` - semantic boundary still requires direct review; no move authorized.

### `tests/transactions/core/publication_stream_order.rs`

#### `branch_head_index_moves_at_cutover_before_reverse_settlement`

Source: `crates/worth-relational/src/tests/transactions/core/publication_stream_order.rs:5`.

- **C2** 56: `assert_eq!( runtime.history.oldest_branch_head_version(), Some(first_version), "all performed cutovers reach the head index before any settlement" )` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **C2** 74: `assert_eq!( runtime.history.oldest_branch_head_version(), Some(first_version), "reverse settlement cannot move branch-head truth backward" )` - actual residency, identity, pin, allocation, dictionary or cost observation.

#### `preparation_order_does_not_reserve_a_subscriber_stream_gap`

Source: `crates/worth-relational/src/tests/transactions/core/publication_stream_order.rs:82`.

- C4 action 94: `let routes = runtime.history.canonical_checkpoint_gate();`.
- **C4** 95: `assert_eq!(routes.has_unsettled_performed_publication(), None)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 105: `assert_eq!(routes.has_unsettled_performed_publication(), Some(commit_b))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 116: `assert!(before_a.patches.is_empty())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 125: `assert!( prepared_first.canonical_commit().commit.commit_id < published_first.canonical_commit().commit.commit_id )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 129: `assert!(position_b < prepared_first.patch_position())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 131: `assert_eq!(routes.has_unsettled_performed_publication(), Some(commit_a))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 143: `assert_eq!(after_a.patches.len(), 1)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 144: `assert_eq!(after_a.patches[0].position, prepared_first.patch_position())` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 145: `assert_eq!( after_a.latest_commit_id, Some(prepared_first.canonical_commit().commit.commit_id) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 153: `assert_eq!(routes.has_unsettled_performed_publication(), Some(commit_a))` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 159: `assert_eq!(routes.has_unsettled_performed_publication(), None)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 160: `assert_eq!(routes.settlement_route_visit_count() - before, 0)` - outcome follows concrete injected control listed for this test; C2 observations are separated.

#### `checkpoint_tail_recovery_follows_publication_order_with_exact_reserved_identities`

Source: `crates/worth-relational/src/tests/transactions/core/publication_stream_order.rs:164`.

- C4 action 228: `let routes = recovered.history.canonical_checkpoint_gate();`.
- **C4** 203: `assert!(prepared_first_id < published_first_id)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 204: `assert!(prepared_first_position > published_first_position)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 216: `assert_eq!(plan.tail_log.len(), 1)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 217: `assert_eq!(plan.tail_log[0].position(), prepared_first_position)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 218: `assert_eq!( plan.tail_log[0].envelope().commit.commit_id, prepared_first_id )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 230: `assert_eq!(routes.has_unsettled_performed_publication(), None)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 231: `assert_eq!(routes.settlement_route_visit_count() - before, 0)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 232: `assert_eq!( recovered .history() .branch_head(&BranchId("published-first".to_owned())) .expect("B branch head recovers from checkpoint") .commit_id, published_first_id )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 240: `assert_eq!( recovered .history() .branch_head(&BranchId("prepared-first".to_owned())) .expect("A branch head recovers from the position-selected tail") .commit_id, prepared_first_id )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 248: `assert_eq!( observed_entity_name( execution, &recovered, "prepared-first", prepared_first_entity ), Some("write-a".to_owned()) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 257: `assert_eq!( observed_entity_name( execution, &recovered, "prepared-first", published_first_entity ), None )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 266: `assert_eq!( observed_entity_name( execution, &recovered, "published-first", published_first_entity ), Some("write-b".to_owned()) )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 275: `assert_eq!( observed_entity_name( execution, &recovered, "published-first", prepared_first_entity ), None )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 284: `assert_eq!( observed_entity_name(execution, &recovered, "main", prepared_first_entity), None )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 288: `assert_eq!( observed_entity_name(execution, &recovered, "main", published_first_entity), None )` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 293: `assert!(continued.commit.commit_id > published_first_id)` - outcome follows concrete injected control listed for this test; C2 observations are separated.
- **C4** 294: `assert!(continued.patch_position() > prepared_first_position)` - outcome follows concrete injected control listed for this test; C2 observations are separated.

### `transactions/data/outcomes/commit_result/snapshot_cleanup_tests.rs`

#### `result_snapshot_cleanup_refuses_an_actual_admitted_runtime_copy`

Source: `crates/worth-relational/src/transactions/data/outcomes/commit_result/snapshot_cleanup_tests.rs:10`.

- **C2** 39: `assert!(!result.snapshot_release_observation().was_released())` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 49: `assert!(std::sync::Arc::ptr_eq(&returned, &result))` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 56: `assert!(completed)` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.

#### `native_custody_correction_result_cleanup_is_explicit_idempotent_and_keeps_external_read`

Source: `crates/worth-relational/src/transactions/data/outcomes/commit_result/snapshot_cleanup_tests.rs:61`.

- **UNCLASSIFIED** 76: `assert!(matches!( service.release_commit_result_snapshot(&result).unwrap(), Outcome::Released(_) ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 80: `assert!( matches!(service.release_commit_result_snapshot(&cloned).unwrap(), Outcome::AlreadyReleased { snapshot_id } if snapshot_id == result.snapshot.snapshot_id()) )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 83: `assert_eq!(runtime.visibility.handles.published_count(), 0)` - semantic boundary still requires direct review; no move authorized.
- **C2** 84: `assert_eq!(runtime.retention().active_snapshot_count(), 1)` - actual residency, identity, pin, allocation, dictionary or cost observation.
- **UNCLASSIFIED** 86: `assert_eq!( refused.reason(), crate::runtime::RelationalRuntimeCloseDenial::ReadCustodyRetained )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 93: `assert_eq!( cloned.commit, result.commit, "actual Arc results remain descriptive after explicit cleanup and close" )` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 97: `assert!(matches!( service.release_commit_result_snapshot(&result), Err(Denial::Owner( crate::branch::RelationalBranchBasisDenial::OwnerUnavailable )) ))` - semantic boundary still requires direct review; no move authorized.

#### `native_custody_correction_result_release_cell_tracks_public_release_closeout_and_transfer`

Source: `crates/worth-relational/src/transactions/data/outcomes/commit_result/snapshot_cleanup_tests.rs:106`.

- **C2** 116: `assert!(matches!( service.release_commit_result_snapshot(&normal).unwrap(), Outcome::AlreadyReleased { .. } ))` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 126: `assert!(matches!( service.release_commit_result_snapshot(&closed).unwrap(), Outcome::AlreadyReleased { .. } ))` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.
- **C2** 136: `assert!( matches!( service .release_commit_result_snapshot(&transferred) .unwrap(), Outcome::Released(_) ), "moving a real release obligation is not actual release" )` - reviewed original identity/allocation/residency/private capture proof; public consequences stay paired.

#### `native_custody_correction_result_cleanup_refuses_foreign_and_private_altered_pairing`

Source: `crates/worth-relational/src/transactions/data/outcomes/commit_result/snapshot_cleanup_tests.rs:149`.

- **UNCLASSIFIED** 155: `assert!(matches!( foreign .owner_component_services() .basis_port() .release_commit_result_snapshot(&result), Err(Denial::Snapshot( crate::visibility::RelationalSnapshotReleaseDenial::ForeignRuntime { .. } )) ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 177: `assert!(matches!( service.release_commit_result_snapshot(&altered), Err(Denial::RegistrationMismatch) ))` - semantic boundary still requires direct review; no move authorized.
- **UNCLASSIFIED** 181: `assert_eq!(runtime.visibility.handles.published_count(), 1)` - semantic boundary still requires direct review; no move authorized.
- **C2** 182: `assert_eq!(runtime.retention().active_snapshot_count(), 1)` - actual residency, identity, pin, allocation, dictionary or cost observation.

## Unclassified named-test queue

- `crates/worth-relational/src/branch/owner_services/materialization_port_tests.rs:durability_replays_suspended_and_restored_materialization_roots`
- `crates/worth-relational/src/branch/root_authoritative_payload_tests.rs:strategy_artifacts_are_bound_into_the_committed_root`
- `crates/worth-relational/src/branch/root_authoritative_payload_tests.rs:merge_execution_authority_is_bound_into_the_committed_root`
- `crates/worth-relational/src/branch/root_authoritative_payload_tests.rs:schema_transition_descriptors_are_each_bound_into_the_committed_root`
- `crates/worth-relational/src/commit_strategies/facade_tests/replay_recovery_failures.rs:replay_commit_reports_strategy_executor_unavailable_when_recovered_runtime_lacks_executor`
- `crates/worth-relational/src/commit_strategies/facade_tests/replay_recovery_failures.rs:replay_commit_reports_strategy_execution_failure_when_recovered_executor_rejects`
- `crates/worth-relational/src/durability/authority/checkpoint_complete/tests/empty_schema.rs:complete_empty_archive_and_initial_installation_preserve_unused_original_schema`
- `crates/worth-relational/src/durability/authority/checkpoint_complete/tests/empty_schema.rs:complete_refuses_recovered_empty_without_original_birth_authority`
- `crates/worth-relational/src/durability/authority/checkpoint_complete/tests/partial_aspect_update.rs:complete_preserves_partial_update_without_demanding_untouched_aspect_reference`
- `crates/worth-relational/src/durability/authority/checkpoint_complete/tests/pinned_delete.rs:complete_fold_matches_original_relation_delete_with_snapshot_and_replay_pins`
- `crates/worth-relational/src/durability/authority/checkpoint_concurrency_tests.rs:checkpoint_reconstruction_does_not_exclude_publication_after_capture`
- `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:fresh_source_refuses_foreign_and_nonempty_actual_sources_without_rebuild`
- `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:fresh_receipt_refuses_actual_movement_and_preserves_original_cancellation`
- `crates/worth-relational/src/durability/authority/fresh_source/tests.rs:fresh_source_refuses_real_selected_empty_checkpoint`
- `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:divergent_sibling_is_not_reused_as_the_storage_mirror`
- `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:divergent_mirror_partition_rebuilds_cross_partition_adjacency_after_reuse`
- `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore_tests.rs:equal_image_cannot_bypass_global_contract_readmission`
- `crates/worth-relational/src/durability/canonical_commit_artifact/retained_export/skip_tests.rs:seeking_past_an_acknowledged_index_prefix_never_demands_its_journal_bodies`
- `crates/worth-relational/src/durability/data/checkpoint_images.rs:streamed_branch_root_image_digest_matches_buffered_wire_for_divergent_roots`
- `crates/worth-relational/src/durability/data/native_checkpoint/component_custody/tests.rs:native_custody_correction_delayed_canonical_selection_retains_actual_root_and_interrupted_ack`
- `crates/worth-relational/src/durability/data/native_checkpoint/component_custody/tests.rs:native_custody_correction_delayed_empty_selection_retains_actual_schema_and_interrupted_ack`
- `crates/worth-relational/src/durability/data/native_checkpoint/component_custody/tests.rs:native_custody_correction_last_actual_capture_abandonment_releases_shared_root_custody`
- `crates/worth-relational/src/durability/data/native_checkpoint/tests/selected_ack.rs:selected_ack_partial_transfer_preserves_later_cells_and_resumes_same_capture`
- `crates/worth-relational/src/durability/data/native_checkpoint/tests/selected_ack.rs:selected_ack_exposes_backed_data_before_retirement_and_releases_holds_only_on_completion`
- `crates/worth-relational/src/durability/log/native_checkpoint_codec/inspection/tests.rs:inspection_compares_every_native_model_value_as_original_bytes`
- `crates/worth-relational/src/durability/log/native_checkpoint_codec/inspection/tests.rs:inspection_refuses_unknown_root_allocator_and_row_fields`
- `crates/worth-relational/src/durability/log/native_checkpoint_codec/inspection/tests.rs:restore_refuses_unknown_fields_before_publishing_any_model_state`
- `crates/worth-relational/src/durability/log/native_checkpoint_codec/inspection/tests.rs:the_live_census_includes_divergent_retained_branch_roots`
- `crates/worth-relational/src/durability/log/native_checkpoint_codec/inspection/tests.rs:burned_symbol_frontier_does_not_hide_published_symbol_damage`
- `crates/worth-relational/src/durability/log/native_checkpoint_codec/kind_authentication_tests.rs:checkpoint_readmission_refuses_retired_integer_array_digest_before_restore`
- `crates/worth-relational/src/durability/log/native_checkpoint_codec/kind_authentication_tests.rs:forged_cross_region_kind_commitment_is_refused_before_member_reads`
- `crates/worth-relational/src/durability/log/native_checkpoint_codec/manifest_integrity_tests.rs:shared_members_with_inconsistent_leaf_facts_refuse_the_manifest`
- `crates/worth-relational/src/durability/log/native_checkpoint_codec/manifest_integrity_tests.rs:sparse_page_membership_accepts_late_holes_and_refuses_duplicate_or_forged_ids`
- `crates/worth-relational/src/durability/log/native_checkpoint_codec/manifest_validation/components/tests.rs:recorded_component_checkpoint_header_tamper_refuses_without_defaults_or_foreign_binding`
- `crates/worth-relational/src/durability/log/native_checkpoint_codec/tests.rs:member_directory_preserves_divergent_sibling_roots_and_pinned_reader`
- `crates/worth-relational/src/durability/log/native_checkpoint_codec/tests.rs:current_manifest_deduplicates_equal_region_members_and_round_trips`
- `crates/worth-relational/src/durability/log/native_checkpoint_codec/tests.rs:current_directory_rejects_damaged_duplicate_and_missing_members`
- `crates/worth-relational/src/durability/log/native_checkpoint_codec/tests.rs:native_capture_sections_count_current_directory_values`
- `crates/worth-relational/src/durability/log/native_checkpoint_codec/tests.rs:chk6_and_retired_inner_partition_grammars_are_refused`
- `crates/worth-relational/src/history/commit/artifact_tests.rs:fresh_scalar_commit_seals_exact_native_lifecycle_and_field_touches`
- `crates/worth-relational/src/history/commit/artifact_tests.rs:installed_field_index_commit_touch_carries_native_old_and_new_keys`
- `crates/worth-relational/src/history/commit/artifact_tests.rs:recovery_relinks_an_already_sealed_payload_without_reencoding_or_weakening_root_checks`
- `crates/worth-relational/src/history/commit/envelope_residency_tests.rs:an_indexed_envelope_remains_cold_until_explicit_readmission_and_loads_once`
- `crates/worth-relational/src/history/commit/envelope_residency_tests.rs:an_indexed_envelope_storage_failure_is_typed_and_sticky`
- `crates/worth-relational/src/history/commit/envelope_residency_tests.rs:relinking_other_envelopes_preserves_an_index_even_while_its_body_is_in_flight`
- `crates/worth-relational/src/history/commit/envelope_residency_tests.rs:settlement_releases_a_journal_flight_but_preserves_memory_history_and_position`
- `crates/worth-relational/src/history/commit/envelope_residency_tests.rs:positioned_demand_owns_its_result_across_journal_settlement`
- `crates/worth-relational/src/publication/cdc/planning/checkpoint_resolution/tests.rs:subscriber_stream_recovers_from_durable_canonical_envelopes_when_patch_stream_index_outlives_envelope`
- `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs:publication_excludes_abandoned_preview_meanings_adopted_for_reading`
- `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs:concurrent_pending_meanings_deduplicate_and_publish_only_the_accepted_candidate`
- `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs:abandoning_the_last_custodian_spends_the_identity_and_removes_pending_text`
- `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs:independent_admission_uses_the_pending_identity`
- `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs:parallel_candidates_share_pending_meaning_without_publishing_it`
- `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs:checkpoint_recovery_preserves_abandoned_symbol_identity_frontiers`
- `crates/worth-relational/src/runtime/state/subsystems/services/symbol_table/reservations/tests.rs:canonical_symbol_sealing_and_cross_owner_adoption_refuse_before_publication`
- `crates/worth-relational/src/runtime/state/subsystems/visibility/snapshot_handles/published_registry/original_cleanup/tests.rs:result_snapshot_cleanup_registry_contention_preserves_exact_cell_and_competing_task`
- `crates/worth-relational/src/storage/residency/cross_region_forgery_tests.rs:rehashed_cross_region_kind_forgery_is_refused_by_the_canonical_root`
- `crates/worth-relational/src/tests/domains/cad/topology_identity_survival.rs:topology_identity_survival_preserves_reidentification_truth_across_recovery`
- `crates/worth-relational/src/tests/domains/fintech/workflows/persistence/checkpoint_fallback.rs:fintech_recovery_refuses_a_damaged_checkpoint_instead_of_losing_truth`
- `crates/worth-relational/src/tests/durability/contracts/branch_and_aspect_history.rs:durability_contract_recovery_preserves_relation_aspect_history_for_retained_audit_relations`
- `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:checkpoint_recovery_rejects_missing_exact_branch_root_artifact`
- `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:checkpoint_recovery_rejects_root_artifact_without_commit_envelope`
- `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:checkpoint_recovery_rejects_duplicate_commit_envelopes_before_root_readmission`
- `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:checkpoint_recovery_readmits_shared_root_once_for_sibling_heads`
- `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:tail_recovery_resolves_the_checkpoint_target_root_not_the_fork_source_head`
- `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:checkpoint_recovery_rejects_duplicate_global_partition_images`
- `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint.rs:checkpoint_recovery_rejects_duplicate_branch_root_partition_images`
- `crates/worth-relational/src/tests/durability/contracts/branch_root_checkpoint/foreign_targets.rs:checkpoint_recovery_rejects_foreign_branch_target_substitution`
- `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:checkpoint_rejects_previous_branch_root_format_before_readmission`
- `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:checkpoint_rejects_altered_root_content_with_recomputed_image_digest`
- `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:checkpoint_rejects_omitted_root_partition_with_recomputed_image_digest`
- `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:checkpoint_rejects_same_commit_truth_root_relabeling`
- `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:checkpoint_rejects_same_commit_schema_root_relabeling`
- `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:tail_rejects_same_commit_root_descriptor_substitution`
- `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:tail_replay_reconstructs_the_owner_content_root`
- `crates/worth-relational/src/tests/durability/contracts/branch_root_content_binding.rs:checkpoint_owner_root_is_stable_across_fingerprint_symbol_ids`
- `crates/worth-relational/src/tests/durability/contracts/branch_root_schema_binding.rs:branch_roots_with_one_schema_deduplicate_the_durable_carrier`
- `crates/worth-relational/src/tests/durability/contracts/branch_root_schema_binding.rs:recovery_rejects_a_missing_branch_root_schema_carrier`
- `crates/worth-relational/src/tests/durability/contracts/branch_root_schema_binding.rs:recovery_rejects_a_tampered_branch_root_schema_carrier`
- `crates/worth-relational/src/tests/durability/contracts/branch_root_shared_target_binding.rs:checkpoint_rejects_later_shared_root_cell_with_relabeled_schema_root`
- `crates/worth-relational/src/tests/durability/contracts/branch_root_shared_target_binding.rs:checkpoint_rejects_later_shared_root_cell_with_relabeled_truth_root`
- `crates/worth-relational/src/tests/durability/contracts/branch_transition/checkpoint_state.rs:checkpoint_size_tracks_branch_state_not_covered_transitions`
- `crates/worth-relational/src/tests/durability/contracts/branch_transition/checkpoint_state.rs:a_deleting_checkpoint_cell_requires_its_retired_name`
- `crates/worth-relational/src/tests/durability/contracts/branch_transition/fates.rs:every_transition_fate_preserves_the_durable_before_live_boundary`
- `crates/worth-relational/src/tests/durability/contracts/branch_transition/fates.rs:an_unresolved_commit_also_blocks_branch_transitions_with_its_original_fate`
- `crates/worth-relational/src/tests/durability/contracts/branch_transition/reconstruction.rs:memory_checkpoint_keeps_replay_reconstructible`
- `crates/worth-relational/src/tests/durability/contracts/branch_transition/reconstruction.rs:replay_reads_only_the_gap_after_its_checkpoint`
- `crates/worth-relational/src/tests/durability/contracts/branch_transition/reconstruction.rs:recovery_plan_after_native_capture_keeps_covered_forks`
- `crates/worth-relational/src/tests/durability/contracts/branch_transition/reconstruction.rs:a_memory_fork_can_replay_its_compacted_durable_source`
- `crates/worth-relational/src/tests/durability/contracts/branch_transition/recovery.rs:a_durable_fork_without_a_subsequent_commit_reopens_and_commits`
- `crates/worth-relational/src/tests/durability/contracts/branch_transition/recovery.rs:a_fork_covered_by_a_checkpoint_after_its_source_moves_accepts_a_target_tail`
- `crates/worth-relational/src/tests/durability/contracts/branch_transition/recovery.rs:a_fork_in_the_transition_tail_is_merged_before_the_target_commit`
- `crates/worth-relational/src/tests/durability/contracts/branch_transition/recovery.rs:lifecycle_postures_and_retired_names_survive_checkpoints_and_crashes`
- `crates/worth-relational/src/tests/durability/contracts/branch_transition/recovery.rs:independent_runtimes_encode_byte_identical_transition_histories`
- `crates/worth-relational/src/tests/durability/contracts/caller_association.rs:caller_association_bounds_and_rebinding_refuse_without_retaining_publication`
- `crates/worth-relational/src/tests/durability/contracts/caller_association.rs:caller_association_survives_embedded_checkpoint_and_refuses_old_semantics`
- `crates/worth-relational/src/tests/durability/contracts/caller_association.rs:caller_association_survives_cold_checkpoint_history_and_journal_tail`
- `crates/worth-relational/src/tests/durability/contracts/canonical_artifact_neutrality.rs:independent_runtimes_publish_byte_identical_canonical_artifacts`
- `crates/worth-relational/src/tests/durability/contracts/canonical_artifact_neutrality.rs:a_recovered_runtime_continues_with_the_reference_artifacts`
- `crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs:durability_contract_checkpoint_tail_recovery_preserves_post_checkpoint_commits`
- `crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs:durability_tail_replay_releases_each_commit_snapshot_beyond_live_capacity`
- `crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs:durability_contract_recovery_fails_closed_without_exact_branch_cells`
- `crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs:durability_contract_recovery_rejects_branch_cell_target_without_catalog_artifact`
- `crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs:durability_contract_checkpoint_recovers_index_metadata`
- `crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery.rs:durability_contract_checkpoint_recovers_lineage_authority`
- `crates/worth-relational/src/tests/durability/contracts/checkpoint_recovery/empty_head.rs:durability_contract_empty_checkpoint_restores_a_live_retained_head`
- `crates/worth-relational/src/tests/durability/contracts/declared_schema_binding.rs:another_label_with_identical_contracts_refuses_before_service`
- `crates/worth-relational/src/tests/durability/contracts/declared_schema_binding.rs:after_a_durable_transition_only_its_target_is_served`
- `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs:live_fork_scoped_generation_readmits_from_shared_basis`
- `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs:repeated_edits_checkpoint_only_the_live_index_payload`
- `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs:checkpoint_index_artifact_survives_tail_replay_without_envelope_caches`
- `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs:checkpoint_index_artifact_missing_payload_denies_before_recovery`
- `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs:checkpoint_recovery_denies_a_digest_valid_foreign_generation_affinity`
- `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint.rs:repeated_closed_branches_leave_no_latest_generation_residue`
- `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/artifact_affinity.rs:legacy_index_affinity_is_checked_before_replacing_a_live_recovery_target`
- `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/deleted_branch_reader.rs:deleted_branch_reader_retains_only_its_exact_scoped_generation`
- `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/historical_reader.rs:historical_reader_from_live_head_survives_head_retirement`
- `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/lagging_scoped.rs:lagging_inherited_and_own_stream_scoped_generations_survive_checkpoint_reopen`
- `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/scoped_affinity.rs:real_shared_parent_root_scoped_checkpoint_restores_lookup_and_content`
- `crates/worth-relational/src/tests/durability/contracts/derived_index_checkpoint/scoped_affinity.rs:digest_valid_unissued_scoped_branch_denies_without_replacing_target_state`
- `crates/worth-relational/src/tests/durability/contracts/descriptor_continuity.rs:durability_contract_failure_schema_mismatch_is_explicit`
- `crates/worth-relational/src/tests/durability/contracts/descriptor_continuity.rs:durability_contract_failure_descriptor_semantics_version_mismatch_is_explicit`
- `crates/worth-relational/src/tests/durability/contracts/descriptor_continuity.rs:durability_recovery_plan_preserves_explicit_verification_mode`
- `crates/worth-relational/src/tests/durability/contracts/descriptor_continuity.rs:durability_recovery_emits_authority_continuity_diagnostic_before_execution`
- `crates/worth-relational/src/tests/durability/contracts/descriptor_continuity.rs:durability_certification_recovery_authority_continuity_is_explained_and_counted`
- `crates/worth-relational/src/tests/durability/contracts/fork_provenance_checkpoint.rs:recovery_rejects_one_sided_fork_provenance_in_checkpoint`
- `crates/worth-relational/src/tests/durability/contracts/fork_provenance_checkpoint.rs:recovery_rejects_mismatched_fork_provenance_identity_in_checkpoint`
- `crates/worth-relational/src/tests/durability/contracts/fork_provenance_checkpoint.rs:recovery_rejects_one_sided_fork_provenance_in_tail`
- `crates/worth-relational/src/tests/durability/contracts/global_generation_reclamation.rs:repeated_global_same_basis_builds_reclaim_duplicates_but_keep_pinned_root`
- `crates/worth-relational/src/tests/durability/contracts/lineage_allocator_denials.rs:live_lineage_allocator_exhaustion_denies_before_public_effects`
- `crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs:checkpoint_lineage_node_deletion_denies_before_installation`
- `crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs:checkpoint_lineage_node_remap_denies_before_installation`
- `crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs:checkpoint_lineage_node_swap_denies_before_installation`
- `crates/worth-relational/src/tests/durability/contracts/lineage_artifact_recovery.rs:checkpoint_duplicate_lineage_entity_mapping_denies_before_installation`
- `crates/worth-relational/src/tests/durability/contracts/lineage_event_allocator_recovery.rs:multi_event_reservation_exhaustion_denies_before_public_effects`
- `crates/worth-relational/src/tests/durability/contracts/lineage_event_allocator_recovery.rs:branch_local_sparse_slot_create_publishes_and_recovers_exact_lineage`
- `crates/worth-relational/src/tests/durability/contracts/managed_field_text.rs:native_field_text_checkpoint_then_tail_recovers_exact_json_after_original_owners_drop`
- `crates/worth-relational/src/tests/durability/contracts/managed_field_text.rs:native_field_text_checkpoint_readmission_denial_keeps_bytes_and_target_root_intact`
- `crates/worth-relational/src/tests/durability/contracts/managed_field_text.rs:native_field_text_configuration_requires_current_explicit_readmission`
- `crates/worth-relational/src/tests/durability/contracts/managed_field_text.rs:native_field_text_prior_checkpoint_format_refuses_before_managed_text`
- `crates/worth-relational/src/tests/durability/contracts/merge_replay_continuity.rs:durability_contract_recovery_preserves_merge_parent_order`
- `crates/worth-relational/src/tests/durability/contracts/merge_replay_continuity.rs:durability_contract_replays_empty_intent_merge_currentness_once`
- `crates/worth-relational/src/tests/durability/contracts/merge_replay_continuity.rs:durability_contract_replays_merge_from_typed_authority_when_diagnostics_are_absent`
- `crates/worth-relational/src/tests/durability/contracts/native_checkpoint.rs:native_checkpoint_round_trip_restores_a_live_editable_world`
- `crates/worth-relational/src/tests/durability/contracts/native_checkpoint.rs:recovery_authority_only_admits_a_basis_from_the_recovered_image`
- `crates/worth-relational/src/tests/durability/contracts/native_checkpoint.rs:native_checkpoint_round_trip_restores_current_relation_adjacency`
- `crates/worth-relational/src/tests/durability/contracts/native_checkpoint.rs:repeated_native_captures_do_not_retain_checkpoint_images`
- `crates/worth-relational/src/tests/durability/contracts/native_checkpoint.rs:corrupt_native_checkpoint_is_denied_without_publishing_recovered_state`
- `crates/worth-relational/src/tests/durability/contracts/native_checkpoint.rs:native_checkpoint_rejects_a_foreign_runtime_name_without_publishing_state`
- `crates/worth-relational/src/tests/durability/contracts/native_checkpoint.rs:native_checkpoint_preserves_unsealed_builder_invariant_authority`
- `crates/worth-relational/src/tests/durability/contracts/native_checkpoint/allocation.rs:native_checkpoint_retains_all_admitted_capture_owners_and_restores_editable_truth`
- `crates/worth-relational/src/tests/durability/contracts/native_checkpoint/speculative_symbols.rs:unpublished_client_keys_leave_checkpoint_dictionary_equal_and_published_keys_restore`
- `crates/worth-relational/src/tests/durability/contracts/native_checkpoint_restore_benchmark.rs:synthetic_native_restore_and_image_digest_benchmark`
- `crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs:tail_replay_consumes_canonical_reused_record_allocation`
- `crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs:checkpoint_restores_burned_append_frontier_beyond_all_retained_roots`
- `crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs:recovery_releases_orphaned_append_reservation_without_reusing_its_gap`
- `crates/worth-relational/src/tests/durability/contracts/record_allocation_recovery.rs:recovery_returns_orphaned_reclaimed_reservation_to_reuse`
- `crates/worth-relational/src/tests/durability/contracts/recorded_component_checkpoint.rs:recorded_component_checkpoint_readmits_cold_canonical_predecessor_and_successor`
- `crates/worth-relational/src/tests/durability/contracts/recorded_component_checkpoint.rs:recorded_component_checkpoint_readmits_original_empty_predecessor`
- `crates/worth-relational/src/tests/durability/contracts/recorded_component_checkpoint.rs:recorded_component_checkpoint_omits_released_components_and_plain_observations`
- `crates/worth-relational/src/tests/durability/contracts/recorded_component_checkpoint.rs:recorded_component_checkpoint_refuses_chk15_before_source_admission`
- `crates/worth-relational/src/tests/durability/contracts/recorded_component_checkpoint.rs:recorded_original_retirement_preserves_both_leases_across_busy_and_stop`
- `crates/worth-relational/src/tests/durability/contracts/rejected_and_parent_closure.rs:durability_contract_recovery_ignores_rejected_relation_integrity_attempts`
- `crates/worth-relational/src/tests/durability/contracts/rejected_and_parent_closure.rs:durability_contract_failure_missing_authoritative_parent_closure_is_explicit`
- `crates/worth-relational/src/tests/durability/contracts/retention_inspection_store.rs:durability_contract_recovery_rebuilds_branch_head_root_obligations`
- `crates/worth-relational/src/tests/durability/contracts/retention_inspection_store.rs:durability_contract_recovery_preserves_inspection_truth_bundle`
- `crates/worth-relational/src/tests/durability/contracts/schema_transition_continuity.rs:durability_contract_failure_aspect_plan_mismatch_is_explicit`
- `crates/worth-relational/src/tests/durability/contracts/schema_transition_continuity.rs:durability_contract_failure_relation_integrity_plan_mismatch_is_explicit`
- `crates/worth-relational/src/tests/durability/contracts/semantic_result.rs:semantic_result_discovery_must_be_present_in_each_actual_index_entry`
- `crates/worth-relational/src/tests/durability/contracts/semantic_result/bulk.rs:semantic_result_bulk_entities_and_relations_seal_and_readmit_original_bindings`
- `crates/worth-relational/src/tests/durability/contracts/semantic_result/bulk.rs:semantic_result_bulk_malformed_correspondence_refuses_after_frame_reseal`
- `crates/worth-relational/src/tests/durability/contracts/semantic_result/custom_versions.rs:semantic_result_preserves_custom_enforcement_version_distinct_from_aggregate_request`
- `crates/worth-relational/src/tests/durability/medium/branch_transition_races.rs:durable_fork_and_source_commit_complete_without_deferral`
- `crates/worth-relational/src/tests/durability/medium/branch_transition_races.rs:durable_fork_and_source_archive_complete_in_lock_order`
- `crates/worth-relational/src/tests/durability/medium/branch_transition_races.rs:durable_fork_waits_for_another_branch_commit`
- `crates/worth-relational/src/tests/durability/medium/branch_transition_races.rs:memory_commit_waits_behind_a_transition_without_consuming_its_candidate`
- `crates/worth-relational/src/tests/durability/medium/checkpoint_member_storage.rs:filed_external_members_survive_checkpoint_replacement_and_obey_read_limits`
- `crates/worth-relational/src/tests/durability/medium/durable_before_live.rs:a_memory_only_runtime_publishes_without_a_medium`
- `crates/worth-relational/src/tests/durability/medium/embedded_cost.rs:embedded_restore_allocation_grows_linearly_in_relation_bearing_regions`
- `crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs:fork_against_checkpoint_capture`
- `crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs:fork_against_runtime_fork`
- `crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs:fork_against_reclamation`
- `crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs:archive_against_checkpoint_capture`
- `crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs:archive_against_runtime_fork`
- `crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs:archive_against_reclamation`
- `crates/worth-relational/src/tests/durability/medium/lifecycle_lock_order.rs:checkpoint_registry_is_readable_while_a_fork_is_durable_not_yet_live`
- `crates/worth-relational/src/tests/durability/medium/lineage_frontiers.rs:checkpoint_restore_protects_published_lineage_identities_in_each_lane`
- `crates/worth-relational/src/tests/durability/medium/lineage_frontiers.rs:inflight_tail_replay_resumes_after_the_largest_durable_lineage_values`
- `crates/worth-relational/src/tests/durability/medium/recovered_publication.rs:a_recovered_runtime_publishes_its_next_commit_at_the_next_position`
- `crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs:an_artifact_that_does_not_readmit_refuses_as_corrupt`
- `crates/worth-relational/src/tests/durability/medium/recovery_refusals.rs:a_checkpoint_filed_under_the_wrong_position_refuses_as_corrupt`
- `crates/worth-relational/src/tests/durability/medium/reservation_wait.rs:a_durable_publisher_waits_for_the_reservation_instead_of_deferring`
- `crates/worth-relational/src/tests/durability/medium/reservation_wait.rs:a_cancelled_waiter_stops_before_any_effect`
- `crates/worth-relational/src/tests/durability/medium/single_owner.rs:a_forked_runtime_never_inherits_the_medium`
- `crates/worth-relational/src/tests/durability/medium/single_owner.rs:an_attachment_the_medium_moved_out_of_publishes_nothing`
- `crates/worth-relational/src/tests/history/milestone_7a/counter_breadth.rs:merge_ready_history_shape_reports_counter_breadth_explicitly`
- `crates/worth-relational/src/tests/history/milestone_7b.rs:merge_planning_artifact_certification_is_stable_across_recovery`
- `crates/worth-relational/src/tests/history/milestone_7b.rs:merge_planning_schema_snapshot_changes_when_schema_semantics_change`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_a/compilation_denials.rs:compile_execution_ready_merge_plan_rejects_missing_source_record`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_a/compilation_denials.rs:verify_prepared_merge_execution_rejects_corrupted_compiled_plan`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_a/prepared_merge_verification.rs:verify_prepared_merge_execution_accepts_fresh_prepared_merge`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_a/prepared_merge_verification.rs:verify_prepared_merge_execution_rejects_runtime_instance_mismatch`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_a/prepared_merge_verification.rs:verify_prepared_merge_execution_rejects_target_head_drift`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_a/prepared_merge_verification.rs:verify_prepared_merge_execution_rejects_source_head_drift`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_a/prepared_merge_verification.rs:verify_prepared_merge_execution_rejects_schema_semantic_drift`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_a/prepared_merge_verification.rs:verify_prepared_merge_execution_rejects_merge_base_drift`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_a/readiness_admission.rs:prepare_merge_execution_admits_fully_ready_source_only_addition`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_a/readiness_admission.rs:prepare_merge_execution_rejects_blocked_merge_plans`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_a/readiness_admission.rs:prepare_merge_execution_rejects_rejected_merge_plans`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_a/readiness_admission.rs:runtime_prepare_merge_execution_matches_merge_access_surface`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_a/record_plan_compilation.rs:prepare_merge_execution_compiles_source_addition_record_plan`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_a/record_plan_compilation.rs:prepare_merge_execution_compiles_exact_shared_record_plan`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_a/record_plan_compilation.rs:prepare_merge_execution_compiles_reconcile_record_plan`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_a/verification_counters.rs:verify_prepared_merge_execution_does_not_increment_planning_counters`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_d.rs:derive_merge_commit_mutation_plan_emits_source_authorized_create_intent`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_d.rs:derive_merge_commit_mutation_plan_preserves_exact_shared_truth_without_mutation`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_d.rs:derive_merge_commit_mutation_plan_reconciles_target_with_source_authorized_aspects`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_e/canonical_envelope_publication.rs:execute_prepared_merge_publishes_ordered_multi_parent_commit_through_canonical_envelope`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_e/canonical_envelope_publication.rs:execute_prepared_merge_preserves_reserved_summary_when_optional_diagnostics_budget_is_zero`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_e/commit_context_admission.rs:merge_commit_context_rejects_mismatched_parent_branch_metadata`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_e/durability_recovery_diagnostics.rs:execute_prepared_merge_survives_durability_append_and_recovery`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_e/execution_counters_diagnostics.rs:execute_prepared_merge_reports_execution_counters_and_structural_summary`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_e/failure_attempt_diagnostics.rs:execute_prepared_merge_records_attempt_without_success_on_failure`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_g/stale_prepared_merge_denials.rs:authoritative_merge_execution_certification_rejects_stale_prepared_merge_after_target_advances`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_g/stale_prepared_merge_denials.rs:authoritative_merge_execution_certification_rejects_stale_prepared_merge_after_source_advances`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_g/stale_prepared_merge_denials.rs:authoritative_merge_execution_certification_rejects_metadata_reference_movement`
- `crates/worth-relational/src/tests/history/milestone_7c_phase_g/stale_prepared_merge_denials.rs:authoritative_merge_execution_certification_rejects_schema_semantic_drift_after_planning`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_a.rs:merge_planning_conflict_classification_carries_target_view_visibility_evidence_for_exact_shared_truth`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_a.rs:merge_planning_conflict_classification_carries_base_window_evidence_for_target_deleted_record`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_a.rs:merge_planning_conflict_classification_carries_target_lookup_evidence_for_source_deleted_record`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_a.rs:merge_planning_digest_basis_carries_visibility_evidence_rows`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_a.rs:merge_planning_policy_surface_is_explicitly_runtime_owned_before_lowering`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_a.rs:merge_planning_digest_basis_carries_policy_ownership_surface_rows`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_b/deletion_block_reasons.rs:lowered_plan_preserves_source_deleted_target_live_block_reason`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_b/deletion_block_reasons.rs:lowered_plan_preserves_source_live_target_deleted_block_reason`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_b/deletion_block_reasons.rs:lowered_plan_preserves_deleted_vs_modified_block_reason`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_b/executable_admission.rs:admitted_source_addition_carries_executable_class`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_b/executable_admission.rs:compile_rejects_corrupted_non_executable_resolution_class`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_b/policy_boundaries.rs:lowered_plan_carries_explicit_manual_resolution_policy_boundary_for_generic_denial`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_b/policy_boundaries.rs:lowered_plan_carries_explicit_hard_reject_policy_boundary_for_fail_on_conflict`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_c.rs:promoted_deleted_on_both_sides_compiles_to_an_explicit_executable_record_variant`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_c.rs:promoted_deleted_on_both_sides_derives_zero_mutation_intent_execution_plan`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_c.rs:promoted_deleted_on_both_sides_executes_through_authoritative_merge_publication`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_c.rs:real_feature_branch_delete_after_main_delete_is_authorable_and_classifies_as_deleted_on_both_sides`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_c.rs:branch_local_delete_allowance_does_not_make_same_branch_stale_delete_legal`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_d/deleted_convergence.rs:deleted_on_both_sides_merge_commit_has_replay_and_recovery_parity`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_d/prepared_merge_denials.rs:deleted_on_both_sides_prepared_merge_rejects_target_head_drift`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_d/prepared_merge_denials.rs:deleted_on_both_sides_prepared_merge_rejects_schema_semantic_drift`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_d/prepared_merge_denials.rs:non_executable_deletion_denial_is_stable_across_recovery`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_d/topology_local_rewire.rs:topology_endpoint_divergence_denial_is_stable_across_recovery`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_d/topology_local_rewire.rs:disjoint_rewire_neighborhoods_do_not_escalate_to_topology_region_conflict`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_d/topology_local_rewire/neighborhood_detection.rs:unrelated_relation_additions_do_not_inflate_topology_region_detection_counters`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_d/topology_region.rs:topology_region_conflict_detection_reports_bounded_neighborhood_counters`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_d/topology_region.rs:topology_region_conflict_denial_is_stable_across_recovery`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_e.rs:inspection_input_round_trips_deleted_vs_modified_without_host_projection`
- `crates/worth-relational/src/tests/history/milestone_7d_phase_e.rs:execution_surface_preserves_topology_region_conflict_authority_rows`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_a.rs:merge_branch_basis_matches_planning_artifact_and_history_scope_authority`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_a.rs:merge_branch_basis_denial_happens_before_planning_for_missing_or_disconnected_branches`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_a.rs:merge_branch_basis_survives_published_merge_outcome_and_durability_recovery`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_b.rs:foundational_current_basis_lowering_preserves_exact_relational_basis_truth`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_b.rs:foundational_current_basis_bridge_and_readmission_preserve_exact_basis_truth`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_b.rs:foundational_current_basis_lowering_rejects_published_historical_basis_in_live_and_recovered_lanes`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_b.rs:foundational_current_basis_lowering_rejects_stale_relational_basis_before_publication`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_c.rs:normalized_merge_request_equivalence_is_exact_across_authoring_lanes`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_c.rs:unsupported_request_posture_denies_before_history_or_planning_work`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_c.rs:planning_execution_and_publication_consume_normalized_request_authority`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_c.rs:normalized_request_deserialization_revalidates_admitted_truth`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_d.rs:foundational_lowering_preserves_exact_relational_request_truth`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_d.rs:foundational_lowering_does_not_collapse_distinct_relational_meanings`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_d.rs:execution_authoring_and_explicit_normalization_lower_identically`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_d.rs:foundational_lowering_uses_foundational_admission_vocabulary`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_e.rs:retained_merge_proof_packet_preserves_exact_authority_truth`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_e.rs:published_merge_outcome_retains_proof_packet_as_authority`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_e.rs:proof_packet_deserialization_rejects_forged_retained_truth`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_e.rs:merge_proof_packet_survives_publication_and_recovery_without_drift`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_f.rs:merge_proof_packet_canonical_basis_parity_survives_publication_and_recovery`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_f.rs:merge_proof_packet_canonical_basis_localizes_single_field_drift`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_g/mod.rs:retained_merge_correspondence_witness_preserves_exact_admitted_truth`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_g/mod.rs:retained_merge_correspondence_witness_rejects_forged_or_incomplete_truth`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_g/mod.rs:retained_merge_correspondence_witness_preserves_authority_basis_distinctions`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_g/mod.rs:retained_merge_correspondence_witness_preserves_runtime_declared_key_ambiguity_as_denial`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_g/mod.rs:retained_merge_correspondence_witness_survives_publication_and_recovery`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_h/mod.rs:retained_schema_reconciliation_witness_preserves_exact_category_and_linkage_truth`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_h/mod.rs:retained_schema_reconciliation_witness_rejects_forged_or_rehashed_truth`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_h/mod.rs:retained_schema_reconciliation_witness_preserves_runtime_additive_truth_across_authority_lanes`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_h/mod.rs:retained_schema_reconciliation_witness_preserves_runtime_narrowing_truth`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_h/mod.rs:retained_schema_reconciliation_witness_keeps_runtime_type_denial_typed`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_h/mod.rs:retained_schema_reconciliation_witness_keeps_runtime_structural_denial_typed`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_m/mod.rs:merge_collaboration_truth_replay_equivalence_survives_live_publication_and_recovery`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_m/mod.rs:merge_collaboration_support_surface_reflects_correspondence_posture_over_real_history`
- `crates/worth-relational/src/tests/history/milestone_7e_phase_m/mod.rs:merge_collaboration_truth_localizes_single_family_drift`
- `crates/worth-relational/src/tests/history/queries/aspect_history.rs:record_local_aspect_history_reads_committed_patch_truth`
- `crates/worth-relational/src/tests/history/queries/aspect_history.rs:aspect_history_projection_filter_matches_field_level_patch_locus`
- `crates/worth-relational/src/tests/history/queries/aspect_history.rs:bulk_like_aspect_history_filters_and_query_packets_stay_stable_after_recovery`
- `crates/worth-relational/src/tests/history/replay/derived_index_and_workload.rs:replay_and_recovery_preserve_aspect_bearing_truth_across_a_hostile_mixed_workload`
- `crates/worth-relational/src/tests/history/replay/derived_index_and_workload.rs:hostile_commit_replay_equivalence_test`
- `crates/worth-relational/src/tests/inspection/structural_identity.rs:structural_identity_comparison_only_uses_fingerprint_truth`
- `crates/worth-relational/src/tests/inspection/structural_identity.rs:inspection_truth_bundle_recovery_parity_holds_for_current_and_historical_surfaces`
- `crates/worth-relational/src/tests/lineage/resolution.rs:historical_lineage_resolution_follows_replace_events`
- `crates/worth-relational/src/tests/lineage/resolution.rs:historical_lineage_resolution_does_not_scan_unrelated_branch_events`
- `crates/worth-relational/src/tests/lineage/resolution.rs:lineage_aspect_history_keeps_origin_events_and_marks_resolution_context`
- `crates/worth-relational/src/tests/performance_profiles/artifact_recoverability_matrix.rs:perf_artifact_recoverability_matrix`
- `crates/worth-relational/src/tests/performance_profiles/recoverability_policy_matrix.rs:perf_recoverability_policy_matrix`
- `crates/worth-relational/src/tests/publication/cdc/certification/durable_recovery.rs:cdc_certification_persisted_seeded_matrix_survives_checkpoint_compaction_and_recovery`
- `crates/worth-relational/src/tests/publication/cdc/certification/durable_recovery.rs:cdc_certification_retention_truncation_recovers_exact_suffix_from_old_checkpoint`
- `crates/worth-relational/src/tests/publication/cdc/replay_parity.rs:subscriber_stream_matches_patch_stream_for_committed_history`
- `crates/worth-relational/src/tests/publication/cdc/replay_parity.rs:subscriber_stream_keeps_patch_parity_when_later_retained_envelope_is_missing`
- `crates/worth-relational/src/tests/publication/cdc/subscriber_failures/checkpoint_identity.rs:subscriber_stream_rejects_schema_unsupported_checkpoint`
- `crates/worth-relational/src/tests/publication/cdc/subscriber_failures/checkpoint_identity.rs:subscriber_stream_rejects_checkpoint_without_history_or_durable_coverage`
- `crates/worth-relational/src/tests/publication/cdc/subscriber_failures/checkpoint_identity.rs:subscriber_stream_rejects_checkpoint_with_mismatched_contract_identity`
- `crates/worth-relational/src/tests/publication/patch/batching.rs:patch_stream_resume_batches_commits_without_duplication`
- `crates/worth-relational/src/tests/publication/patch/batching.rs:patch_stream_rejects_unknown_resume_position`
- `crates/worth-relational/src/tests/publication/patch/batching.rs:patch_stream_uses_durable_canonical_history_when_retained_envelope_is_missing`
- `crates/worth-relational/src/tests/query/indexes/related_entity_schema_recovery.rs:recovered_related_ordering_uses_each_exact_roots_schema_contract`
- `crates/worth-relational/src/tests/transactions/core/aggregate_identity/logical_components/checkpoint.rs:physical_membership_recipe_and_alternating_derived_catalog_restore_from_real_checkpoints`
- `crates/worth-relational/src/tests/transactions/core/aggregate_identity/logical_components/checkpoint.rs:changed_configured_domain_cannot_replace_the_actual_recovered_root_or_its_handles`
- `crates/worth-relational/src/tests/transactions/core/authored_write_history/adjacency/audit.rs:audit_lifecycle_coverage_survives_unrelated_history_discard_and_durable_recovery`
- `crates/worth-relational/src/tests/transactions/core/authored_write_history/adjacency/recovery.rs:adjacency_occurrence_identities_rebuild_from_cold_canonical_history`
- `crates/worth-relational/src/tests/transactions/core/owner_lifecycle_phase3b.rs:phase3b_owner_drop_during_off_thread_settlement_completes`
- `crates/worth-relational/src/tests/transactions/core/owner_lifecycle_phase3b.rs:phase3b_settlement_admitted_after_owner_drop_is_denied`
- `crates/worth-relational/src/tests/transactions/core/owner_lifecycle_phase3b.rs:phase3b_recovery_replacement_closes_the_prior_runtimes_ports`
- `crates/worth-relational/src/tests/transactions/core/owner_lifecycle_phase3b.rs:phase3b_removed_lifecycle_premises_stay_removed`
- `crates/worth-relational/src/tests/transactions/core/owner_service_phase3.rs:phase3_settlement_receivers_take_a_shared_borrow`
- `crates/worth-relational/src/tests/transactions/core/owner_service_phase3.rs:phase3b_owner_authority_and_owner_services_keep_their_receivers`
- `crates/worth-relational/src/tests/transactions/core/owner_service_phase3.rs:phase3_settlement_port_is_a_cloneable_shared_borrow_service`
- `crates/worth-relational/src/tests/transactions/core/owner_service_phase3.rs:phase3_settlement_port_denies_after_its_runtime_owner_closes`
- `crates/worth-relational/src/tests/transactions/core/owner_service_phase3.rs:phase3_settlement_port_settles_a_performed_publication_from_another_thread`
- `crates/worth-relational/src/tests/transactions/core/owner_service_phase3.rs:phase3_settlement_port_repairs_a_lost_witness_by_identity`
- `crates/worth-relational/src/tests/transactions/core/owner_service_phase3.rs:phase3_foreign_settlement_denial_preserves_source_recovery`
- `crates/worth-relational/src/tests/transactions/core/pending_settlement_recovery.rs:dropping_the_performed_witness_leaves_settlement_recoverable`
- `crates/worth-relational/src/tests/transactions/core/pending_settlement_recovery.rs:repeated_terminal_settlement_returns_one_receipt_and_one_durable_append`
- `crates/worth-relational/src/tests/transactions/core/pending_settlement_recovery.rs:repair_and_witness_settlement_execute_exactly_once`
- `crates/worth-relational/src/tests/transactions/core/pending_settlement_recovery.rs:owner_loss_retires_every_pending_settlement_exactly_once`
- `crates/worth-relational/src/tests/transactions/core/pending_settlement_recovery.rs:interruption_after_movement_retains_its_pending_settlement`
- `crates/worth-relational/src/tests/transactions/core/publication_settlement.rs:direct_publication_settles_before_a_child_can_acknowledge`
- `crates/worth-relational/src/transactions/data/outcomes/commit_result/snapshot_cleanup_tests.rs:native_custody_correction_result_cleanup_is_explicit_idempotent_and_keeps_external_read`
- `crates/worth-relational/src/transactions/data/outcomes/commit_result/snapshot_cleanup_tests.rs:native_custody_correction_result_cleanup_refuses_foreign_and_private_altered_pairing`
