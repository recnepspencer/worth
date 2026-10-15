# Owning-operation cutover inventory

Engineering inventory for public Query9.17.6.4. Baseline `045d4ec` precedes the
owning-operation cutover; audited destination is `f7a6d83` plus the scoped audit
repair. This records source deletion and audience placement, not an authority,
receipt registry, consumer usage guide, or milestone certification.

## Ordinary and native responsibility

| Previous caller surface | Current ordinary owner | Current native responsibility |
|---|---|---|
| Request mutation object executed/prepared/recovered fixed source and outputs | `WorthQueryOperationBuilder` captures a description; fixed action retains `WorthQueryFixedOperation`, then settled/terminal custody | `request.advanced().mutate(..)` returns the real native request for independently needed staging/retention/recovery |
| Discovered request coordinated source, registration, traversal and settlement | Declared discovered action retains `WorthQueryDiscoveredOperation`; original unregistered carrier, full required-set guard and delivery stay owned | Native discovered intake, partial promotion, registered receipt recovery and retention retry remain current lower owners |
| Program action without an output root | Declared SourceOnly operation, fresh advance, completion/terminal and lawful close | Program-less/non-output manual execution requires explicit advanced native request |
| Workflow operation funnel | Description binds `for_workflow_operation`; installed action retains workflow owner and original issued slot | Bound workflow `.advanced()` delegates the same native funnel; raw native request is not exposed by the ordinary bound view |
| Assessment caller retained a native demand across fresh calls | `WorthQueryWorkflowAssessmentOperation` retains actual admitted demand/cleanup | Explicit advanced assessment start/settle remain native controls, not workflow approval or acceptance |

`application_entry/mod.rs` no longer exports native mutation request types at the
ordinary root. `advanced::{WorthQueryApplicationMutationRequest,
WorthQueryApplicationMutationRequestWithIdempotency}` are the actual current
native types, not aliases back to an ordinary legacy lane. `ordinary/builder.rs`
and `ordinary/keyed.rs` expose no manual fixed/discovered/source-only execution,
recovery or idempotency-resolution method. The borrowed-key pending view can bind
an issued workflow requirement; it cannot execute that effect without selecting
its explicit native bound entrance. Owned workflow construction is separate.

## Moved versus deleted Query vocabulary

- MOVED audience: the two native mutation request types above. Their meaningful
  native `execute`, `execute_in_program`, `execute_retained_in_program`,
  `execute_performed`, `execute_performed_discovered`, preparation, retained
  recovery and idempotency-resolution methods remain on the native owner.
  Removing these would remove real native coordination; they are not retired
  wrappers sheltered by `advanced`.
- DELETED alias: `request/output_currentness.rs`'s old
  `WorthQueryProgramOutputCurrentnessDenial = WorthQueryOutputCurrentnessDenial`.
  The current complete-set denial is `WorthQueryOutputCurrentnessDenial`; no
  alias survives at ordinary or advanced scope. Host `cutover_contract.rs`
  checks its absence with otherwise-valid current native imports and a valid
  current diagnostic counterpart.
- New ordinary descriptions are immutable pure meaning plus shared funded input,
  not cached requests/grants. Native phase, selected installed action, binding
  dispatcher, original source/conditions/key/branch and cleanup owners remain
  authoritative. Advanced native APIs do not reopen a retired format decoder.
- Compared all public type declarations and public method names under the old
  and current Publication application-entry tree: the removed Query type name is
  only the alias above; no native request wrapper was deleted outright. Public
  method-name retention is not evidence that it remains ordinary: receiver and
  export placement above are the cutover.

## Deleted Bank lifecycle wrappers and methods

These actual public wrapper/result aliases are absent from `bank-server`, rather
than reexported through a compatibility module:

- `BankAccountAccessExecution`
- `BankBusinessAccountCreationExecution`
- `BankMoneyMovementExecution`
- `BankMoneyMovementRetainedExecution`
- `BankPaymentDecisionExecution`
- `BankPaymentInitiationOutcome`
- `BankPersonalAccountCreationExecution`
- `BankProgramMutationExecution`

`bank-estate-certification/src/cutover_contract.rs` has an independent missing
import fence for EACH removed name, in a context where the current Query
`request.advanced()` is valid, plus a valid current native/owning-refusal twin.
The assertion is about Bank's real facade, not importing a Bank name from an
unrelated Query namespace. Existing eleven-family Bank compile fences also
reject removed one-shot methods with valid `.retain_operation()` counterparts.

`ordinary/mutation/query/execution/money_movement.rs` was deleted. Deposit,
withdrawal and send-money construction have named files, returning real owning
SourceOnly operations and whole capture/declaration refusal. Account creation,
access, payment initiation/rejection, estate administration/settlement, opening
funding, reversal and program-deployment revocation likewise retain owners.

Compared baseline/current public method vocabulary: removed legacy methods are
`execute_retained`, `into_continuation`, `into_execution`,
`notify_estate_death_with_key`, `retransmit_estate_death_notice_with_key`,
`open_estate_case_with_key`, `freeze_estate_account_with_key`,
`recognize_estate_executor_with_key`, `disburse_estate_with_key`,
`release_estate_with_key`, `perform_apply`, `prepare_apply_recovery`,
`accept_applied`, `accept_recovered_applied`, `begin_payment_assessment`, and
`settle_payment_assessment`. Current assessment retain/advance/accept is an
observation owner. Approved effects use `retain_apply` / owned acceptance.
`BankAdvancedApprovedPaymentWorkflow::execute_operation` retains deliberate
native control through the current bound `.advanced()` funnel; removed old
method spellings are not compatibility aliases. HTTP's removed diagnostic names
`BankHttpMutationFailureKind` and `BankHttpProviderRecoveryKind` are not live
execution wrappers and do not provide an alternate progression door.

## Covered consumer boundaries and remaining UI bridge

- Neutral ConsumerRoot/TopologyEntry ordinary courts use installed action owners;
  their native counterpart, restoration and hostile custody courts explicitly
  select current native controls. Existing compiled-doc drift binds exact real
  fixed/discovered/source-only/workflow bodies, not handwritten retry loops.
- Bank's eleven owning factory families return original construction refusal and
  owning progression; HTTP transports store those owners, not cloned diagnostic
  continuation authority. Genuine Bank workflow native control and installation,
  assessment, inbound occurrence and external rail owners keep separate duties.
- At frozen `f7a6d83`, three UI callers still use implicit native receivers:
  product_projection/application_action_owner.rs,
  product_projection/application_source_owner.rs and status_integrity/lease_tests.rs
  under `workspaces/worth-ui/crates/worth-ui-query-binding/src`. Available commit
  `777144081749021d9afde8c18ecbb1403692aa3d` contains the exact three explicit-advanced
  bridges. Its successful `--lib --tests` compilation took 9m34s; no runtime cases
  were executed. Root integration has joined that commit; this Source worktree
  remains at `f7a6d83` and has not joined it here. The full owning UI consumer
  conversion remains the distinct future successor; mechanical receiver changes
  and compilation do not establish that conversion.
- Private adopter cutover is not certified by neutral fixtures. No old-format,
  alias, fallback or advanced compatibility support is added for it.

## Exact native-reference census

The following is the complete tracked Rust FILE census for the nine current
manual execution/recovery method spellings in the three covered consumer roots
(Query certification, Bank crates, UI query binding). It includes compiler
contracts and native fixture bodies, not only calls executed by one test. It was
read with scoped `git grep -l -E` without truncating output. It is an insertion
inventory, not permission for ordinary tasks to use advanced. Lower engine
implementation files and pure declaration/reference crates are not application
consumers and are deliberately outside this census. UI findings above remain
explicit pending external work; no all-consumer closure is inferred from names.

Census: 135 files.

- `workspaces/worth-query-bank-world/crates/bank-server/src/approved_payment_workflow/native_operation.rs`
- `workspaces/worth-query-bank-world/crates/bank-server/src/approved_payment_workflow/owning.rs`
- `workspaces/worth-query/crates/worth-query-certification/examples/advanced_product_branching.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/continuation_basis.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/custody.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/dependent_recovery.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/descriptor_result_set/hostile.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/discovered/continuation.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/discovered/currentness.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/discovered/currentness/deep_output.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/discovered/currentness/dependent.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/discovered/currentness/retired.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/discovered/initial_unpublished.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/discovered/publication_basis.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/discovered/publication_lifecycle.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/discovered/publication_lifecycle/running.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/discovered/recovery.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/discovered/retention.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/discovered/unpublished.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/fixed_missing_carrier.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/lifecycle.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/operation_cost/native.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/owner_demand_boundary/producer_lifecycle.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/readiness_recovery/dependency_aba.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/readiness_recovery/preserved_noop.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/recovery.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/recovery/exact_refresh.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/recovery/resources.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/recovery/snapshot_pressure.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/recovery/supersession.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/required_basis.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/required_unpublished_hostile.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/root_selection.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/selected_root_selection.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/settlement.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/application_program/source_only_pinned.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/current_output_selector.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/generated_materialization/retained.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/generated_materialization/typed_reconstruction_preserves_query_authority.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/output_correspondence.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/output_correspondence/source_aba.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/prior_output_family.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/consumer_root/src/application_invariant_acceptance/proof/suspension_recovery_read_stop.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/advancement_custody.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/advancement_custody/mutation_entries.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/advancement_discovered.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/advancement_discovered/recovery_entries.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/advancement_discovered/recovery_entries/cancellation.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/advancement_lifetime.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/checkpoint_reopens_ready_output_without_producer_contact_and_rec.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/clean_reuse.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/compiled_examples.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/oracle/courtroom/adoption.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/oracle/courtroom/equal_output.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/oracle/courtroom/equal_output/indexed_decision.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/oracle/courtroom/equal_output/witness.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/oracle/courtroom/request_work/first_admission.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/oracle/courtroom/request_work/population.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/oracle/courtroom/request_work/settled_rows.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/oracle/courtroom/wide.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/oracle/cutoff.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/oracle/demand.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/oracle/parallel_history_reuse/execution.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/oracle/seeded_absence/unretained.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/owner.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/current_family_output.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/current_output.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/demand_only.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/input_cutoff.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/late_cancellation.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/mixed_retirement.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/parallel_history/matrix/runtime.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/parallel_history/observation.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/concurrent_waves/abandonment_thaw.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/concurrent_waves/application.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/concurrent_waves/checkpoint_restore.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/concurrent_waves/deferral_freeze.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/concurrent_waves/frozen_truth.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/concurrent_waves/history.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/concurrent_waves/ordinary_reads.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/concurrent_waves/queue_failures.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/concurrent_waves/three_members.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/concurrent_waves/two_owners.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/diamond.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/exact_invalidation/commits.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/exact_invalidation/observation_room.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/mixed_mode.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/performed_head_movement.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/reopened_consumer.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/required_queue.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/required_queue/custody_stops/joined_refresh/current_source.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/required_queue/custody_stops/reclamation.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/required_queue/recovery_isolation.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/restored_currentness.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/stable_alias_currentness.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/stable_alias_currentness/differential.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/reuse_opt_out/required_refresh.rs`
- `workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/stable_refresh.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/adoption/authorization.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/allocation_custody.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/document_retention_model/host/tests.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/document_retention_model/presented_request.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/document_retention_model/workflow/review_requirement.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/document_retention_model/workflow/review_requirement/unlink.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/expressions/conditions.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/native_staging_allocation.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/producer_predicate_checkpoint/ordinary_source.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/producer_predicate_checkpoint/ordinary_source/unpublished.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/restored_replay.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/source_bound_replay.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/source_bound_replay/recovery.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/unpublished_recovery.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/workflow_actor_wait.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/workflow_approval.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/workflow_approval/atomic_settlement.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/workflow_approval/back_custody.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/workflow_approval/effect_currentness.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/workflow_approval/instance_cancellation.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/workflow_approval/instance_deadline.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/workflow_approval/instance_migration.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/workflow_approval/operation_input_identity.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/workflow_approval/wait_revocation.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/workflow_execution.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/workflow_execution/review.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/application_graph/workflow_owning/approval_revocation.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/mutation_attempt_report.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/program_example_authority.rs`
- `workspaces/worth-query/crates/worth-query-certification/tests/temporal_integrity_lease.rs`
- `workspaces/worth-ui/crates/worth-ui-query-binding/src/product_projection/application_action_owner.rs`
- `workspaces/worth-ui/crates/worth-ui-query-binding/src/product_projection/application_source_owner.rs`
- `workspaces/worth-ui/crates/worth-ui-query-binding/src/product_projection/status_integrity/lease_tests.rs`
