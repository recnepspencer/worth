# Landing 2a coherence ledger — DRAFT

## Current batch status (2026-11-10, 9ed22bccaa)

The historical draft below is not current qualification. Batch 1 contains P-C 2dce9c178c, H8 1ce3832b78 and P-A e4d3b01c8d; the runtime repair batch is committed at 54b4fa230d. The 2a/2b Delivery-tracks update is committed at 9ed22bccaa. H5 has not landed or merged. No final 2a gate has started.

The serial permitted Tier-2 run (able-batch1-*) is still executing. Its Root start header names 1e8bd528be; subsequent documentation edits do not change that test source. Old SEED/ADDR/Q-NR rows below describe pre-repair evidence only. They must be replaced with actual post-repair results before this ledger certifies a candidate; no previously masked row is deemed green by inference.

Current contract corrections, with qualification still pending:
- B8: DispatchOutbox and DispatchAttempts are now Supported by the installed Home policy, from the H8 merge. Outstanding dispatch, inbound completion and recovery claims remain refused. H8's policy explicitly records that the dispatched receipt-basis owner still refuses graceful close; that partial journey needs H8's next safe point, not a false Supported graceful row here.
- B13/B15 and related partial H6/H7/H9 issuers: incoming P-C 765e8ccdb0 adds typed Directory refusals. That commit is not merged yet; its courts and coherence rows will be joined in batch 2 before these rows change.
- Durable World history/branch retirement is Refused on H, with C2 removal obligations, not a partial H9 promise.
- The 60-second close-maintenance deadline is a C2 timing trigger. A timeout is not passing graceful-close evidence, and no retry or raised deadline certifies the row.


Status: **draft, read-only research; nothing here is a pass.** Written 2026-11-09 against
`codex/m3-host-home` HEAD `171d380829` plus the uncommitted repair batch. No cargo, test or
runner was launched for this draft. Gate: `orchestration/m3/spec/landing_2a.md` item 1.

What is merged here (read from `git log` and source, not assumed): H3, H4, P-C's H6/H7
progress (`2dbd704a37` at `6ea7a7126e`), H8 outbox rebinding (`5791602d56`). **H5 is not merged**;
H9/H10 are not here. P-C's own rows (`landing2a-coherence-pc.md`) are not in this worktree and
must be folded in at the batch merge.

## How to read the evidence column

All census evidence is pre-repair, on `171d380829`. Three shared defects mask almost every Home court:

| Tag | Meaning | Source |
|---|---|---|
| **SEED** | red pre-repair (Native seed family: `BundleAssembly` / `original semantic result: ValidationMismatch`); re-run pending | `master2-merge-families.md`, `master2-store-runtime-failure-rows.json` |
| **ADDR** | red pre-repair (Store physical-profile family: backend build script recorded `PROFILE=unsupported` because of the committed `--verbose` flag, giving `InvalidAddress` / `UnqualifiedPathStorage`); re-run pending. This is a second shared defect, separate from SEED | `master2-store-runtime-families.md` |
| **Q-NR** | not run: the whole Query workspace runtime was UNRUN at the census (native preparation exit 101, PDB capacity) | `master2-merge-families.md` row "Query debug PDB capacity" |

The Store census is also **incomplete**: 2,102 of 4,506 selected cases started (highest failing ordinal 1984)
before the driver died. "Not in failure rows" below means the case's ordinal range was executed and it is absent
from the 479 failure rows; I did not read its `ok` event, so treat it as *probably passed, unconfirmed*.

Court path aliases:
- **QH** = `worth-query-court-query-decl-host`, `workspaces/worth-query/crates/worth-query-court-query-decl-host/tests/worth_query_court_query_decl_host/`
- **SA** = `worth-store-court-application`, `workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/`
- **RS** = `worth-court-server-certification-suite`, `crates/worth-court-server-certification-suite/tests/worth_court_server_certification_suite/`

## Finding that shapes every Refused row: the D15 type and its two enforcement points

- The spec's `UnsupportedDurableFamily { family }` **does not exist as a type** (zero matches in `crates/` and `workspaces/`).
  The realised refusal is `WorthQueryDurableStateAdmissionDenial::Unsupported { family, supported_from_phase }`
  (`worth-query-execution/src/domain_computation/execution_runtime/product_world/durable_admission.rs`).
- The Host table (`worth-query-host/src/application_home/support.rs`, `directory_requirement_refusal`) is consulted at exactly two places:
  1. **Open, declared seeds**: `admit_home_seed` (`.../application_contribution/contracts/home_seed.rs`) gives
     `WorthQueryApplicationHomeOpenRefused::Preparation(WorthQueryApplicationPreparationDenial::UnsupportedSeed(Unsupported { .. }))`.
     It checks only three things: producers (`SignalNodeDefinitions`), conditionals (`BridgeConditionalDefinitions`), resource members (`BridgeOwnedAsyncDefinitions`). **Has courts.**
  2. **Ordinary mutation admission**: `admit_directory_mutation` (`.../application_runtime/durable_admission.rs`, called from
     `.../application_entry/mutation/execution/preparation.rs:152`) gives
     `MutationHandlerExecutionDenial::DurableStateAdmission(Unsupported { .. })` before any handler contact. Any capability whose phase is above 4 refuses.
     **No court found** (`DurableStateAdmission` appears in no test or fixture file).
- "Opening a directory home that holds the family refuses" is implemented separately and is **not family-typed**:
  `home_preparation/existing/native_families.rs::readmit` refuses Native outbox / inbound / aftermath rows with a string
  `WorthQueryPrimaryGraphInstallationDenial` of kind `CheckpointRecoveryRejected` (surfaced as `ExistingNativeFinalization`), and Store refuses non-empty reserved
  journals with `ApplicationHomeOwnerAdmissionDenial::JournalNotEmpty`.
- Entry points that are **not** ordinary mutations (workflow definition publication, program adoption, support retirement) never consult the table (inferred from source: no gate call or gate vocabulary in those modules).

## Ledger

Status key: **Supported** / **Refused** / **PARTIAL — needs gating**. "(court gap)" means the status is what the code does but no court proves it, so the row is not yet ledger-complete.

### A. Home lifecycle and cross-cutting decisions

| # | Surface | H item | Status | Court(s) | Latest evidence | Refusal removed by |
|---|---|---|---|---|---|---|
| A1 | Fresh Directory open, `initial_state`, serve | H3 | Supported | QH `host::application_home::fresh_source::authored_fresh_home_postbootstrap_complete_and_automatic_maintenance_preserve_business_value`; SA `process::application::host_oracle::opening::actual_host_opening_observes_every_owner_stage_without_granting_a_phase` | Q-NR; SEED | — |
| A2 | Graceful close, reopen, keep working | H3 | Supported | QH `host::application_home::reopening::same_directory_reopens_reads_and_commits_without_replaying_initializer_or_old_key`, `reopening::flushed_and_pending_multibatch_homes_readmit_their_completed_publication`, `reopening::separate_process::separate_processes_readmit_old_keys_and_restore_the_actual_outcome_frontier`; SA `process::application::world_durable_release::unclean::graceful_close_reopens_the_checkpointed_native_and_world_heads`, `no_pending_owners::query_publication::a_gracefully_reopened_query_owner_can_publish_after_the_prior_no_effect` | Q-NR; SA first: red pre-repair, child panics in `prepare_native(..)` with the typed error not printed (cause unobserved, probably SEED); SA second: ADDR | — |
| A3 | Quiescent kill, reopen | H3/H4 | Supported | SA `world_durable_release::unclean::quiescent_kill_reopens_the_acknowledged_native_and_world_heads`, `query_readmission::native_artifacts::home_close::killed_writer::killed_host_without_close_reopens_its_acknowledged_value_and_original_idempotency_record`; QH `reopening::checkpoint_restart::journal_only_killed_home_reopens_original_inputs_and_commits_without_initializing`, `killed_home_reopens_normalized_image_plus_nonempty_tail` | SA first: red, child cause unobserved; SA second: SEED; Q-NR | — |
| A4 | Close settlement, `CloseRefused { posture }`, refused close leaves a serving home (D11) | H3 | Supported | QH `host::application_home::close_admission::active_installed_call_refuses_before_world_seal_and_resumes_the_same_home`; SA `publication_close::performed_publication_custody_refuses_close_until_its_real_settlement`, `query_readmission::native_artifacts::home_close::held_original_publication_receipt_refuses_close_and_same_home_resumes`, `interrupted_close::*` (3 cases) | Q-NR; SEED; SEED; ADDR | — |
| A5 | Checkpoint at quiesced close and explicit application checkpoint (D12, track H) | H3 | Supported | QH `reopening::checkpoint_state::close_checkpoint_uses_host_allowance_with_an_ordinary_close_caller`, `reopening::checkpoint_restart::selected_complete_home_reopens_and_selects_another_complete_checkpoint`; SA `native_components::checkpoint::known_checkpoint_failure_allows_close_and_reopen_preserves_the_journal_tail` | Q-NR; SEED | — |
| A6 | Automatic checkpoint maintenance (D12 assigns triggers to C2, but `application_home/maintenance.rs` is live) | H3 code, C2 bounds | Supported (ahead of track) | QH `reopening::checkpoint_restart::automatic_checkpoints_bound_host_reopen_journal_reads_with_headroom`, `reopening::checkpoint_state::automatic_capture_at_head_forty_one_preserves_ordinary_request_results`, `reopening::checkpoint_funding::*` (2); `worth-store-application` unit `world_section::readmission::tests::composed_home_manual_and_automatic_checkpoint_share_actual_complete_attempt` | Q-NR; SEED. If a threshold or bound assertion is red after repair it is a C2 move-out candidate, not a happy gate | — |
| A7 | Program transition, graceful scope (plan, intent, steps, binding, promotion); `TransitionRequired` | H3 (D16-H) | Supported | QH `host::application_home::program_transition::host_transition_plans_on_nonempty_a_then_serves_writes_closes_and_reopens_b`, `host_transition_requires_finite_resources_before_creating_a_home`, `fresh_and_current_home_skip_the_optional_transition_author`, `host_transition_refuses_private_record_bodies_before_intent_and_preserves_a`, `transition_required::changed_program_requires_a_transition_before_initialization_or_supplier_execution`; SA `no_pending_owners::transition_*` (about 45 cases) | Q-NR; SA mostly ADDR, some SEED | — |
| A8 | Reopen with a **pending** transition (resume, `TransitionPlanDrift`, `TransitionPending`; D16 track C / C1) | C1 | **PARTIAL — needs ruling (unverified)** | In-tree courts exist: SA `no_pending_owners::transition_open_resume::a_readmitted_transition_open_keeps_its_original_attempt_commit_and_custody`, `transition_unopened_resume::*` (2), `transition_history::a_pending_transition_refuses_a_different_successor_before_predecessor_admission` | ADDR. I did not establish what track-H open does with an intent-without-binding after a quiescent kill (resume, typed refusal, or D18). Three sibling transition-source recovery courts are already moved to C1 | C1 |
| A9 | `discard_home`: refuses a live home; closed home is removed and next open initializes | H3 | Supported (graceful part) | SA `process::application::discard::discard_refuses_the_live_lease_and_closed_disposal_reinitializes_with_a_new_identity`. Tombstone / kill-during-discard courts (`torn_discard_intent_refuses_open_and_explicit_retry_finishes_it`, `discard::interruption::killed_discard_retains_the_tombstone_during_removal_and_retry_initializes`, `discard_completes_damaged_store_and_namespace_identity_without_store_recovery`, `damaged_tombstone_directory_is_completed_without_losing_the_opening_fence`) are C1 scope and still in the tree. No Query-host court calls `discard_home` | SEED for the two named first; other three not in failure rows | — |
| A10 | Contention, single writer, incarnation fencing, stale capability/session/selector refusals | H3/H4 | Supported | SA `admission::ownership::competing_host_processes_admit_exactly_one_writer_until_process_death`, `admission::world_incarnation::reopened_world_refuses_old_live_selectors_and_readmits_original_durable_occurrence`, `process::relational::relational_home::incarnation::home_identity_is_stable_and_each_successful_open_advances_once`; QH `process::host::durable_retry::affinity::capability_home::reopened_home_refuses_prior_open_capability_before_disclosing_new_observations`, `affinity::session_grant::reopened_home_refuses_the_authentic_prior_provider_session_before_any_provider_effect`, `affinity::reopened_home_refuses_the_prior_open_branch_selector_before_effect` | SEED; ADDR; ADDR; Q-NR | — |
| A11 | Descriptor binding; reopen under a changed build identity | H3 | Supported | SA `admission::changed_build::changed_host_build_keeps_initializer_closed_and_reaches_native_readmission`; `worth-store-application` unit `tests::descriptor_admission::changed_implementation_build_preserves_authored_binding_and_exact_durable_history` | SEED; SEED | — |
| A12 | Memory-only location (never refused, never durable) | H3 (D13) | Supported | QH `process::host::durable_retry::affinity::capability_home::trusted_time::memory_home_uses_its_supplied_trusted_authorization_clock`, `host::application_home::workflow_configuration::memory_home_workflow_configuration_survives_close_refusal_and_resumes_the_original_key`; SA `host_oracle::released_original::memory_release_preserves_original_key_and_refuses_handler_replay` | Q-NR; SEED | — |
| A13 | **D18** unclean open | H3 (track H only) | **Refused**: `ApplicationHomeOpenRefusal::UncleanRecoveryUnsupported(UncleanRecoveryUnsupported)` (Store `open/readmission/unclean/relational.rs`), surfaced by Host as `WorthQueryApplicationHomeOpenRefused::{WorldReadmission, ExistingWorldReadmission}` with accessor `unclean_recovery_unsupported()` | SA `process::application::world_durable_release::unclean::unclean_native_tail_refuses_before_world_service_and_names_the_original_effect`. Host accessor: **no court found** | **Failed** at census: child `original_world_attempt_child` panicked at testkit `world_home/publication.rs:53` (`prepare_native(..).expect`), typed payload absent; parent never reached the kill, the refusal or the owner/effect comparison. Not confirmed SEED. D18 is unproved; re-run pending | C5 (after C1) |
| A14 | **D13** interim door `WorthQueryInterimCheckpointRestore`, frozen callers | H3 | Supported (interim) — **static gate red** | boundary-check against `tools/boundary-check/config/interim-checkpoint-callers.txt` (54 rows) | Six incoming master callers are outside the frozen list; independent review verdict REJECT (`master2-d13-six-caller-boundary-design.md`). Ruling 1 (delete the six, list under Moved courts H5/H6/H9) is in flight. Separately, seven public `in_memory*` constructors still exist under `worth-query-execution/.../primary_graph/application_installation/` (`program/construction.rs`, `program/bootstrap_installation.rs`, `mod.rs`) although H3 says every `in_memory*` path is deleted; Bank installs through `in_memory_rostered_program_in_advancement` | C5 deletes the door |
| A15 | Phase 3 open-refusal matrix (foreign, damaged, retired format) | C1 by track; H3 for clean-cutover typed refusals | Refused (typed, permanent) | SA `process::relational::relational_home::open_refusals::*` (6), `incarnation::retired_descriptor_versions_and_the_retired_anchor_refuse_without_advancing`, `query_readmission::bootstrap_publication::retired_format::retired_bootstrap_completion_format_refuses_existing_home_without_replaying_seeds`, `native_components::admission_refusals::selected_home_admission_refuses_predecessors_missing_foreign_and_damaged_images` | SEED, except `retired_descriptor_versions..` which fails an expected `Descriptor/UnsupportedVersion1` assertion with the actual value not printed (cause unobserved) | permanent |

### B. Durable state families (from `support.rs`)

| # | Families | H item | Status | Court(s) | Latest evidence | Refusal removed by |
|---|---|---|---|---|---|---|
| B1 | `ProgramActivation`, `PortablePackageMeaning` | H3 + H6 slice (`1c9c4ac01a`) | Supported | QH `host::application_home::program_transition::activation_siblings::directory_reopen_validates_rostered_activation_on_every_unadopted_branch`, `directory_open_refuses_an_unrostered_activation_on_a_non_main_branch` (typed `QueryHomeProgramActivationDenial::UnrosteredCurrentActivation`) | Q-NR | — |
| B2 | `AggregateIdentity`, `AggregateMembership`, `AggregateLineage`, `AggregateProjectedCoordinates` (Native graph state) | H3/H4 | Supported | SA `host_oracle::graceful::graceful_home_oracle_seed_{zero,one}_preserves_relational_and_world_prefixes`, `process::relational::relational_home::journeys::a_home_recovers_exactly_what_it_committed_and_keeps_going` | SEED | — |
| B3 | `WorldIdentityFrontiers`, `WorldProductBranches`, `WorldProductHeads`, `WorldCompositeHistory`, `WorldPerformedPublication` | H4 | Supported | SA `world_occurrence_readmission::ordinary_occurrences_reopen_original_history_after_later_native_advance`, `host_oracle::partition_reopen::reopened_host_never_reuses_a_committed_private_mutation_partition`, graceful oracle seeds | SEED | — |
| B4 | `QueryIdempotencyBinding`, `UnpublishedIdempotency`; original-result retry; D9 receipts | H4 | Supported | QH `process::host::durable_retry::lost_response_kill_reopen_retry_preserves_receipt_before_checkpoint`, `..._after_checkpoint`, `concurrent_identical_directory_keys_publish_one_original_receipt`, `changed_intent_refuses_after_later_directory_publications`, `affinity::inherited_directory_key_refuses_a_sibling_product_affinity`; RS `process::durable_query_retry::restarted_server_returns_the_original_receipt_after_later_history`, `concurrent_server_identical_keys_publish_one_original_occurrence`, `server_changed_intent_refuses_without_reentering_the_handler`, `child_reexecution::killed_server_reopens_home_and_returns_its_original_query_receipt`; SA `host_oracle::concurrent::concurrent_same_key_retries_return_one_original_occurrence_without_reexecuting_its_retry` | Q-NR; RS all four SEED (explicit, `master2-root-process-failure-review.md` rows 21-24); SA SEED | — |
| B5 | `WorldAttemptProgress`, `WorldUnpublishedCustody`, `WorldComponentBranchCustody`, `PendingApplicationPublication`, `RelationalPendingSettlement` (open custody readmitted as recovered custody, D5/D11 track H) | H4 | Supported | QH `host::application_home::recovered_custody::recovered_original_custody_reaches_service_and_continues_without_reexecution`, `recovered_original_custody_reaches_service_and_release_stays_released`, `continued_recovered_custody_closes_cooperatively_and_resumes_after_reopen`; SA `host_oracle::recovered_custody::*` (3), `host_oracle::abandonment::caller_abandonment_uses_the_actual_typed_outcome_and_retained_owner_effect`, `host_oracle::no_effect_release::recovered_no_effect_release_permits_one_fresh_execution_of_the_original_key` | Q-NR (the three QH cases were listed red on 2026-11-06 in `merge-cleanup-reds.md`; current state unknown); SA SEED | — (`Indeterminate`, `OpenAttempt` are C1) |
| B6 | `ApplicationBranchCleanup` (Native, phase 4) | H4 | Supported per table (court gap) | no court identified in this pass (searched the Home court directories only) | not run | — |
| B7 | `PublicationDiagnosticMaterial` | H4 | Supported per table (court gap) | no court identified in this pass | not run | — |
| B8 | `DispatchOutbox` | H8 | **Refused in effect; table says `Supported`** (court gap on admission) | Admission: an operation with `external_emission` needs `ExternalEmission` (phase 8) and refuses with `DurableStateAdmission(Unsupported { DispatchOutbox, 8 })` — no court. Open: Native outbox rows refuse — SA `query_readmission::native_artifacts::latest_native_owner_rows_refuse_their_actual_family_kind_and_record_before_service` (string denial, not family-typed) | SEED | H8 |
| B9 | `AftermathCausality` | H8 | **PARTIAL — needs gating (inferred)** | Table says `Supported`, yet reopen refuses any Native aftermath row (`native_families.rs::readmit`, court as B8). I did not verify whether a phase-4 operation with an aftermath but no external emission can create such a row on a Directory Home; if it can, the write succeeds and the next open refuses | SEED (open side) | H8 |
| B10 | `CorrectionIntent` | H9 | **Refused in effect; table says `Supported`** (court gap) | An operation whose installed binding has a correction binding needs `RetainedCorrectionMaterial` (phase 9) and refuses at admission — no court. Store journal `query.corrections` non-empty refuses: SA `no_pending_owners::journal_absence::each_unsupported_journal_entry_refuses_admission_and_returns_the_same_home` (`ApplicationHomeOwnerAdmissionDenial::JournalNotEmpty`) | not in failure rows | H9 |
| B11 | Phase 5: `WorldConditionalDefinitionCustody`, `SignalNodeDefinitions`, `SignalTopology`, `SignalBranchCells`, `SignalOwnerFrontiers`, `SignalBranchMetadata`, `SignalRuntimeDefinitions`, `BridgeConditionalDefinitions`, `BridgeSemanticRegistrations`, `BridgeCorrespondenceAllocation`, `BridgeOwnedAsyncDefinitions`, `BridgeDiagnosticRetention` | H5 | **Refused**: `Preparation(UnsupportedSeed(Unsupported { family, supported_from_phase: 5 }))` at open | QH `process::host::home_seed::directory_producer_seed_refuses_before_initialization_or_owner_effect`, `directory_conditional_seed_refuses_before_initialization_or_owner_effect`; `worth-query-court-decl-host` `certification::application_graph::source_only::capability::resources::home_seed::directory_resource_seed_refuses_before_clock_provider_or_owner_effect`; Store journals `signal.definitions`, `signal.advances`: SA `each_unsupported_journal_entry_refuses_admission_and_returns_the_same_home`. Operation side (`SignalMutation`, `BridgeMutation`): no court | Q-NR; SA not in failure rows | **H5** (in the 2a batch: these rows flip to Supported at the final candidate and the three seed courts must be replaced by H5's graceful courts) |
| B12 | `ProgramSupportLifecycle` | H6 slice | **Supported by court; table says `RefusedUntil(6)`** (mismatch to reconcile) | QH `host::application_home::program_support::directory_program_support_retirement_stays_retired_after_graceful_reopen`, `refused_directory_support_retirement_records_active_abort_before_reopen`, `program_support::separate_process::retired_support_is_readmitted_in_a_fresh_process_without_initializer_or_reactivation` (`retire_program_support` is not a mutation, so it never consults the table) | Q-NR | — |
| B13 | `BranchAdoptionRecovery`, `StoppedBranchSetAdoption` (program adoption) | H6 | **PARTIAL — needs gating (inferred)** | no court found on a Directory Home. The table refuses only when a mutation handler declares `ProgramAdoption`; the adoption entries (`worth-query-publication/src/application_entry/programs/request.rs::adopt`, `programs/branch_set.rs::adopt`) and `product_operation/program_adoption/` contain no directory check | not run | H6 |
| B14 | `OutputDemandRecords`, `OutputSourceCustody`, `PerformedSourcePreparation`, `ProgramOutputReadCustody` | H6 | Refused today, transitively: a program with producers refuses at open (B11) | as B11 producer seed court | Q-NR | H6. **Becomes PARTIAL at the H5 merge** unless output demand is complete or gets its own gate: once H5 lifts the producer seed, `RequiredOutputDemand` is checked only when a handler declares it |
| B15 | Workflow: `WorkflowLineages`, `WorkflowDefinitions`, `WorkflowNodes`, `WorkflowConnections`, `WorkflowInstances`, `WorkflowTransitions`, `WorkflowProposals`, `WorkflowProposalCoverage`, `WorkflowAssessmentEvidence`, `WorkflowEvidenceDependencies`, `WorkflowApprovals`, `WorkflowOperationOutputs`, `WorkflowCapturedInputs`, plus `ManagedWorkflowRuns`, `YieldedWorkflowRuns`, `WorkflowYieldRecovery`, `WorkflowTerminalCleanup` | H7 (P-C partial) | **PARTIAL — needs gating** | Only Memory is proven: QH `host::application_home::workflow_configuration::memory_home_workflow_configuration_survives_close_refusal_and_resumes_the_original_key`, `foreign_workflow_binding_returns_the_original_home_spec_and_signing_owner`, `workflow_configuration_keeps_no_runtime_backing_and_refuses_a_foreign_receiver`; transition side QH `program_transition::workflow_dispositions::home_transition_refuses_a_foreign_workflow_choice_before_intent_and_preserves_a`. No Directory graceful court and no typed refusal court. Details below | Q-NR | H7 |
| B16 | Rest of phase 7: `ConditionalDecisions`, `RetainedConditionalCleanup`, `TemporalIdempotency`, `ResourceOccurrences`, `ResourceIngress`, `SignalResourceRequestCustody`, `OwnedAsyncCustody`, `SignalOwnedAsyncCustody`, `SignalOwnedAsyncQueue`, `BridgeManagedTemporalIntents`, `BridgeOwnedAsyncCustody`, `ManagedDirectRuns`, `YieldedDirectRuns`, `ProviderSuspensions`, `DirectYieldRecovery`, `ManagedRestoreRecovery`, `DirectTerminalCleanup`, `PendingApplicationCleanup`, `ProducerCleanupCustody`, `SourceOperationCustody`, `OperationCleanupCustody`, `BankManagedCleanup`, `HttpNativeCleanup`, `UiPresentationCustody`, `CapturedExternalInput`, `FixedOperationCustody` | H7 | Refused today: conditional and resource seeds refuse at open (B11 courts); handler-declared capabilities refuse at mutation admission (no court) | B11 conditional and resource seed courts | Q-NR | H7. **At risk at the H5 merge** for the same reason as B14: lifting the phase-5 seed refusal removes the only proven gate in front of conditional, temporal and resource work |
| B17 | Phase 8: `DispatchAttempts`, `OutstandingDispatch`, `InboundCompletionRows`, `InboundMessageCustody`, `InstalledTransportCompletionCustody`, `TerminalAcknowledgement`, `InboundSourceControls`, `RecoveryClaims`, `RecoveryClaimDispositions`, `RecoveryHandleCustody`, `BankRecoveryClaim` | H8 | **Refused** (admission-side court gap) | Open: Native inbound rows — SA `latest_native_owner_rows_refuse_their_actual_family_kind_and_record_before_service`; journal `query.dispatch` — SA `each_unsupported_journal_entry_refuses_admission_and_returns_the_same_home`. Admission (`ExternalEmission`/`ExternalDispatch`, phase 8): no court | SEED; not in failure rows | H8 |
| B18 | H8 slice merged: original outbox contract rebinding (`5791602d56`) | H8 | Supported on the non-Home runtime only (Bank opens no Directory Home: zero uses of `WorthQueryApplicationHomeLocation` in the Bank crates); on a Directory Home it is behind B8/B17 | `worth-query-bank-court-external-rail-server` `process::server_owned_effects::server::ordinary_mutations::estate_operations::c4_retry_outbox_rebind::ordinary_equivalent_retry_skips_materialization_and_the_external_rail`, `equivalent_binding_does_not_bypass_fresh_authorization_currentness` | SEED (both are among the 65 Bank category failures, all confirmed seed family) | — |
| B19 | Phase 9: `CorrectionPreImages`, `CorrectionGovernedInput`, `CorrectionOriginalMeaning`, `OriginalOperationDescription`, `FixedDeliveryCustody`, `ServerSourceCustody`, `HttpOriginalOwners`, `UiStatusMutationOwner` | H9 | **Refused** (admission-side court gap) | as B10 | not in failure rows | H9 |

### C. Cross-cutting proofs

| # | Surface | H item | Status | Court(s) | Latest evidence |
|---|---|---|---|---|---|
| C1 | Graceful oracle (Memory reference vs Directory Home, `DurableIdentityCorrespondence`) | H3 | Supported | SA `process::application::host_oracle::graceful::graceful_home_oracle_seed_zero_preserves_relational_and_world_prefixes`, `..._seed_one_...`; also `host_oracle::continuation::bounded_host_oracle_seed_{zero,one}_continues_across_real_process_reopens` | **Failed**, SEED explicit: the Memory reference open fails at `host_observation/opening.rs:34` before any Directory session; no graceful assertion is proved. `host_oracle_service_child` standalone `ok` is an env-noop, not proof |
| C2 | Server retry route across Server restart | H4 | Supported | RS courts in B4 | SEED |
| C3 | D15 operation-admission refusal mechanism itself | H3/H4 | Refused (**no court**) | none: nothing asserts `MutationHandlerExecutionDenial::DurableStateAdmission` or `WorthQueryApplicationRequestMutationDenialKind::DurableStateAdmission` | not run |
| C4 | P-C spec seam: `AuthoritativeDurableHistory`, `DurableBranchPublicationRequired` (durable World retirement moves to C2) | H9 / C2 | Refused (to fold in) | rows live in P-C's `landing2a-coherence-pc.md`, not in this worktree; both names exist in `crates/worth-runtime-world/src` (6 files) | not assessed here |

## Row counts

38 rows: A1–A15, B1–B19, C1–C4.

| Status | Rows | Count |
|---|---|---|
| Supported, court named | A1–A7, A9–A12, A14, B1–B5, B12, B18, C1, C2 | 21 |
| Supported per table, no court identified | B6, B7 | 2 |
| Refused, court named (B17 and B19 only for the open side) | A13, A15, B11, B14, B16, B17, B19 | 7 |
| Refused, no court in this worktree / table mismatch | B8, B10, C3, C4 | 4 |
| PARTIAL | A8, B9, B13, B15 | 4 |

None of the 38 rows has passing evidence on this head. Every Supported row is "pending".

## PARTIAL rows, ranked by risk

1. **B15 workflow on a Directory Home (highest).** Three facts, all read from source:
   - `6ea7a7126e` removed the open-time refusal of workflow rows from `home_preparation/existing/native_families.rs::readmit` and replaced it with validation
     (`bootstrap/branch_activation_history.rs::verify` calls `workflow::readmission::readmit`, denial `QueryHomeProgramActivationDenial::Workflow`). Workflow rows are now *accepted* at reopen.
   - `WorthQueryApplicationHome::retain_workflow_spec` (`worth-query-host/src/application_home/workflow.rs`) and
     `install_workflow_configuration` (`.../application_installation/program/workflow_runtime/configuration.rs`) have no Directory check.
   - `prepare_workflow_publication` (`worth-query-execution/.../primary_graph/workflow/definition/preparation.rs:119`) does not go through `admit_directory_mutation`.
     Only workflow *execution* through an ordinary mutation binding refuses (`WorkflowExecution`, phase 7).
   So a workflow definition can apparently be published into a Directory Home and readmitted, while runs refuse: half a slice, no Directory court.
   **Where a typed refusal would go:** (a) `retain_workflow_spec` in `application_home/workflow.rs`: when `self.is_directory()`, refuse through `support::directory_requirement_refusal(WorkflowDefinitions, 0)` and return the Home, spec and signing owner as it does today; and (b) `prepare_workflow_publication` in `workflow/definition/preparation.rs`: call `product_runtime.admit_directory_capability(WorkflowExecution)` before projection. Either Landing 2b removes both, or P-C supplies a Directory close/reopen/continue court and the table row moves to Supported.
   **Stale court:** SA `latest_native_owner_rows_refuse_their_actual_family_kind_and_record_before_service` still has a `("worth-query-workflow-lineage", "workflow")` arm expecting `"Home unsupported Native workflow entity kind .."`. It was not updated by `6ea7a7126e` (last touched by `089aef778a` and the master merge). I expect that arm to fail once SEED is repaired (inferred, never observed).
2. **B9 `AftermathCausality`.** Table `Supported`, reopen refuses rows. If a phase-4 operation can write one, a Home becomes unopenable after a successful commit. Needs a source check of which operations create aftermath causality rows. **Gate placement if needed:** `admit_directory_mutation` in `.../application_runtime/durable_admission.rs`, next to the `external_emission` check, requiring phase 8 for any operation with an installed aftermath; and change `AftermathCausality` and `DispatchOutbox` to `RefusedUntil(8)` in `support.rs`.
3. **B13 program adoption.** No gate at the adoption entry on a Directory Home; only B1 (activation) and B12 (support retirement) have courts. **Gate placement:** the two `adopt` entries in `worth-query-publication/src/application_entry/programs/{request.rs, branch_set.rs}`, or their shared preparation under `worth-query-execution/.../product_operation/program_adoption/preparation/`, calling `admit_directory_capability(ProgramAdoption)`. I did not trace whether adoption needs something a Directory Home cannot provide and therefore fails earlier for another reason.
4. **A8 pending-transition reopen.** Track C by the decision split, but resume courts are in the happy tree and I did not confirm the track-H behaviour. Needs the owner's statement of which typed outcome a pending intent gets at open before landing.

Two rows are not PARTIAL today but **become** PARTIAL when H5 merges (B14 output demand, B16 conditional/temporal/resource): the phase-5 seed refusal is the only *proven* gate in front of them. The final-candidate ledger must re-derive both after the batch merge.

## Other things the owner should know

- **Table mismatches to settle in the spec update:** `ProgramSupportLifecycle` (table refuses, court supports), `DispatchOutbox`, `AftermathCausality`, `CorrectionIntent` (table supports, code refuses). The ledger must tie Refused rows to phases; today the table is not a reliable source for four families.
- **Spec wording:** item 7 and H10 name `UnsupportedDurableFamily`; the type to delete is `WorthQueryDurableStateAdmissionDenial::Unsupported` with its two carriers.
- **D18 has no evidence yet** and its failure was not shown to be SEED. It is a required gate item (item 5), so it should be among the first re-runs.
- **Query Home courts have never run on this head.** Most Supported rows rest on QH; their first observation will come only after the PDB fix links.

## Re-run list after the repair (turns "pending" into evidence)

Selection only; exact names as above. No ignored bodies, no exhaustive Home oracle, no scenario suite, no Query scale.

**Store** — `worth-store-court-application`:
- `process::application::host_oracle::graceful::graceful_home_oracle_seed_zero_preserves_relational_and_world_prefixes`, `..._seed_one_...`
- `process::application::world_durable_release::unclean::unclean_native_tail_refuses_before_world_service_and_names_the_original_effect` (D18), `unclean::graceful_close_reopens_the_checkpointed_native_and_world_heads`, `unclean::quiescent_kill_reopens_the_acknowledged_native_and_world_heads`
- `process::application::host_oracle::{opening, concurrent, continuation, recovered_custody, abandonment, no_effect_release, released_original, partition_reopen}::*`
- `process::application::admission::{changed_build, ownership, world_incarnation}::*`
- `process::application::discard::*`
- `process::application::publication_close::*`, `interrupted_close::*`, `native_components::checkpoint::*`
- `process::application::query_readmission::native_artifacts::latest_native_owner_rows_refuse_their_actual_family_kind_and_record_before_service` (expect the workflow arm red), `home_close::*`, `idempotency::*`
- `process::application::no_pending_owners::journal_absence::*`, `query_publication::*`, `transition_*`
- `process::application::world_occurrence_readmission::*`
- `process::relational::relational_home::{journeys, open_refusals, incarnation}::*` (except `anchor_replay`, a C1 court)
- `worth-store-application` unit: `tests::lifecycle::*`, `tests::descriptor_admission::*`, `world_section::readmission::tests::*`
- plus the remaining 2,404 Store cases the census never started.

**Query** — `worth-query-court-query-decl-host`:
- `host::application_home::{fresh_source, reopening, close_admission, recovered_custody, program_transition, transition_required, program_support, workflow_configuration, handler_retirement, signing_owner_retirement}::*`
- `process::host::durable_retry::*` (without `request_stops::original_directory_receipt_preserves_cancellation_at_every_consult`, a cleanup-class sweep listed in `merge-cleanup-reds.md`)
- `process::host::home_seed::*`
- `worth-query-court-decl-host`: `certification::application_graph::source_only::capability::resources::home_seed::directory_resource_seed_refuses_before_clock_provider_or_owner_effect`

**Root** — `worth-court-server-certification-suite`:
- `process::durable_query_retry::restarted_server_returns_the_original_receipt_after_later_history`, `concurrent_server_identical_keys_publish_one_original_occurrence`, `server_changed_intent_refuses_without_reentering_the_handler`, `child_reexecution::killed_server_reopens_home_and_returns_its_original_query_receipt`
- `worth-court-relational-certification`: `process::home_program_transition_source::ordinary_recovery_still_refuses_a_declared_schema_that_differs_from_the_durable_head`
- `worth-runtime-world` example `runtime_world_publication` (gate item 5, examples on default-stack mains)

**Bank** — `worth-query-bank-court-external-rail-server`:
- `process::server_owned_effects::server::ordinary_mutations::estate_operations::c4_retry_outbox_rebind::*`

**Static:** boundary-check (D13 frozen callers) after Ruling 1.

## What I inferred rather than read

- That workflow definition publication and program adoption reach a Directory Home ungated: inferred from the absence of any gate call or gate vocabulary in those modules; I did not trace every callee.
- That `AftermathCausality` rows can be created by a phase-4 operation: not verified either way.
- That the stale workflow arm of the SA native-owner-rows court will fail after repair: predicted from source.
- "Not in failure rows" as a pass: unconfirmed.
- The exact boundary-check test file for D13 is cited from the design note, not opened.
