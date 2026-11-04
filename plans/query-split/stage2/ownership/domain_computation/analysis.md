# Ownership analysis: `domain_computation`

## Draft target stack (bottom to top)


### identity_time_and_observation

- `runtime_time` (rank 0): the one time source for grant validity, timeouts and recovery expiry; uses below nothing; above nothing; used by authorization 25, primary_graph 8, application_aftermath 2. **greedy disagrees** (greedy rank 6)
- `artifact_identity` (rank 1): artifact identity; uses below nothing; above nothing; used by artifact_owner 5.
- `application_outcome_identity` (rank 2): application outcome identity; uses below nothing; above nothing; used by authorization 2, primary_graph 1.
- `evidence_material` (rank 3): evidence material; uses below nothing; above nothing; used by primary_graph 7.
- `evidence_identity` (rank 4): the canonical Query diagnostic identity protocol; uses below nothing; above nothing; used by primary_graph 80.
- `execution_observation` (rank 5): owned observation lifetime for Query execution; uses below nothing; above nothing; used by primary_graph 39, provider_session 3, execution_runtime 1.
- `application_contract_admission` (rank 6): execution-side consumption of installed read and program meaning; uses below nothing; above nothing; used by primary_graph 10, authorization 1. **greedy disagrees** (greedy rank 0)

### execution_resources

- `execution_runtime` (rank 7): the product world and execution runtime resources; uses below execution_observation 1; above primary_graph 17, provider_session 1, product_publication 1; used by primary_graph 253, authorization 16, managed_run 13.
- `invalidation_equivalence_control` (rank 8): the receiver-owned verification arm; uses below execution_runtime 2; above nothing; used by execution_runtime 1.
- `product_publication` (rank 9): product publication and unpublished recovery; uses below execution_runtime 6; above primary_graph 9; used by primary_graph 70, provider_session 6, execution_runtime 1.
- `artifact_owner` (rank 10): artifact ownership and production authority; uses below artifact_identity 5; above operation_binding 6; used by managed_run 57, provider_session 16, operation_binding 5.
- `operation_binding` (rank 11): operation binding authority and resource topology; uses below execution_runtime 12, artifact_owner 5; above provider_session 13, authorization 3; used by managed_run 16, provider_session 12, artifact_owner 6.
- `domain_evidence_binding` (rank 12): domain evidence binding; uses below nothing; above provider_session 2; used by provider_session 7, managed_run 6, convergence_epoch 2.

### sessions_authority_and_aftermath

- `provider_session` (rank 13): provider session protocol, failures and commit stops; uses below artifact_owner 16, operation_binding 12, execution_runtime 12; above primary_graph 61, managed_run 18, authorization 3; used by primary_graph 657, managed_run 215, authorization 78.
- `managed_run` (rank 14): managed provider runs; uses below provider_session 215, artifact_owner 57, operation_binding 16; above primary_graph 30; used by convergence_epoch 119, primary_graph 38, provider_session 18.
- `authorization` (rank 15): operation authorization, capabilities and disclosure; uses below provider_session 78, runtime_time 25, execution_runtime 16; above primary_graph 321, application_aftermath 3; used by primary_graph 669, application_aftermath 9, provider_session 3.
- `application_aftermath` (rank 16): external effects, recovery custody and recorded correction causality; uses below managed_run 14, execution_runtime 12, authorization 9; above primary_graph 41; used by primary_graph 244, authorization 3, provider_session 1.

### primary_graph

- `primary_graph` (rank 17): the primary application graph; uses below authorization 669, provider_session 657, execution_runtime 253; above nothing; used by authorization 321, provider_session 61, application_aftermath 41.

### convergence_and_attempts

- `convergence_epoch` (rank 18): convergence epochs over managed runs; uses below managed_run 119, primary_graph 36, provider_session 10; above nothing; used by provider_session 2.
- `execution_resource_attempt` (rank 19): execution resource attempts; uses below provider_session 2, operation_binding 1, execution_runtime 1; above nothing; used by nothing.

## Units referenced against the stack

- 136 units: 68 movable, 68 behaviour-coupled, 0 pinned; 479 references
- 23 functions, constants and modules: 55 references

| unit | owner -> destination | refs | class | detail |
|---|---|---|---|---|
| `primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime` | primary_graph -> product_publication | 56 | behaviour-coupled | 17 impl(s) need items above; blocked by 1574: `receipt_backing_bytes`, `dispatch_external_effect`, `provider_commit_identity`, `reserve` … |
| `primary_graph::application_contribution::request_execution::advancement::phase_custody::WorthQueryAdvancementPhase` | primary_graph -> execution_runtime | 34 | behaviour-coupled | 2 impl(s) need items above; blocked by 130: `demanded_field_slot`, `artifact_authority_denial_detail`, `ensure_live`, `mark_disposed_if_unowned` … |
| `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` | primary_graph -> provider_session | 22 | behaviour-coupled | 3 impl(s) need items above; blocked by 32: `relational_read_kind`, `observe_field_value`, `denial`, `equal_cause` … |
| `primary_graph::application_query::resource_lifecycle::basis_registry::WorthQueryApplicationBasisIdentity` | primary_graph -> provider_session | 17 | movable | carries 1 |
| `primary_graph::authenticated_principal::WorthQueryAuthenticatedPrincipal` | primary_graph -> authorization | 11 | behaviour-coupled | 2 impl(s) need items above; blocked by 10: `relational_read_kind`, `heap_sort`, `union_visit`, `equal_cause` … |
| `primary_graph::application_attempt::workflow_transition_program::actor_selection::WorkflowRecordedActorObservation` | primary_graph -> authorization | 10 | behaviour-coupled | 2 impl(s) need items above; blocked by 21: `relational_read_kind`, `observe_field_value`, `AUTHOR_CHUNK`, `compare_bytes` … |
| `primary_graph::causal_inspection::application_inspection::target::WorthQueryApplicationInspectionTarget` | primary_graph -> provider_session | 9 | behaviour-coupled | 2 impl(s) need items above; blocked by 1407: `receipt_backing_bytes`, `dispatch_external_effect`, `provider_commit_identity`, `reserve` … |
| `primary_graph::entity_resolution::resolved::WorthQueryApplicationEntityIdentity` | primary_graph -> authorization | 9 | behaviour-coupled | 1 impl(s) need items above; blocked by 3: `publication_execution_kind`, `scope_stop_kind`, `bytes` |
| `primary_graph::application_attempt::precondition_binding::WorthQueryBoundMutationPreconditions` | primary_graph -> authorization | 8 | behaviour-coupled | 1 impl(s) need items above; blocked by 1319: `receipt_backing_bytes`, `dispatch_external_effect`, `provider_commit_identity`, `reserve` … |
| `primary_graph::workflow::actor::encoding::FundedActorText` | primary_graph -> authorization | 8 | movable |  |
| `primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact` | primary_graph -> authorization | 7 | behaviour-coupled | 4 impl(s) need items above; blocked by 1319: `receipt_backing_bytes`, `dispatch_external_effect`, `provider_commit_identity`, `reserve` … |
| `primary_graph::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding` | primary_graph -> authorization | 7 | behaviour-coupled | 1 impl(s) need items above; blocked by 28: `tree_clone_bytes`, `checkpoint_names`, `relational_read_kind`, `append_identity_slot` … |
| `primary_graph::application_contribution::request_execution::advancement::WorthQueryAdvancementDenial` | primary_graph -> execution_runtime | 7 | behaviour-coupled | 1 impl(s) need items above; blocked by 148: `demanded_field_slot`, `artifact_authority_denial_detail`, `ensure_live`, `mark_disposed_if_unowned` … |
| `primary_graph::application_query::resource_lifecycle::basis_registry::WorthQueryApplicationBasisReleaseReceipt` | primary_graph -> provider_session | 7 | movable | carries 3 |
| `primary_graph::application_attempt::elevation_approval_outcome::WorthQueryApprovedElevation` | primary_graph -> authorization | 6 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `checkpoint` |
| `primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest` | primary_graph -> managed_run | 6 | behaviour-coupled | 2 impl(s) need items above; blocked by 1442: `receipt_backing_bytes`, `dispatch_external_effect`, `provider_commit_identity`, `reserve` … |
| `primary_graph::provider::committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxObservation` | primary_graph -> application_aftermath | 6 | movable | carries 5 |
| `primary_graph::workflow::actor::ActorCodecFunding` | primary_graph -> authorization | 6 | movable | carries 7 |
| `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryOperationAdmissionIdentity` | authorization -> operation_binding | 5 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `NEXT_OPERATION_ADMISSION_IDENTITY` |
| `operation_binding::installed_domain_authority::WorthQueryInstalledDomainExecutionAuthority` | operation_binding -> artifact_owner | 5 | movable |  |
| `primary_graph::admitted_capture::retained_stage::RetainedStorageDenial` | primary_graph -> authorization | 5 | movable |  |
| `primary_graph::application_attempt::compare_and_commit::commit_receipt::WorthQueryApplicationCommitReceipt` | primary_graph -> authorization | 5 | behaviour-coupled | 2 impl(s) need items above; blocked by 918: `receipt_backing_bytes`, `provider_commit_identity`, `reserve`, `bind_dispatch_outbox_create_intent` … |
| `primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease` | primary_graph -> provider_session | 5 | behaviour-coupled | 3 impl(s) need items above; blocked by 201: `demanded_field_slot`, `NEXT_OPERATION_ADMISSION_IDENTITY`, `tree_clone_bytes`, `checkpoint_names` … |
| `primary_graph::application_contribution::computation::denial::WorthQueryManagedComputationResourceDenial` | primary_graph -> execution_runtime | 5 | movable | carries 1 |
| `primary_graph::application_contribution::request_execution::advancement::phase_custody::WorthQueryForeignAdvancementPhase` | primary_graph -> execution_runtime | 5 | movable | carries 4 |
| `primary_graph::source_facts::allocation::SourceFactsAllocation` | primary_graph -> provider_session | 5 | behaviour-coupled | 1 impl(s) need items above; blocked by 7: `tree_clone_bytes`, `equal_cause`, `tree_insertion_bytes`, `derive` … |
| `managed_run::direct::WorthQueryRunningDirectRun` | managed_run -> provider_session | 4 | behaviour-coupled | 2 impl(s) need items above; blocked by 1482: `receipt_backing_bytes`, `dispatch_external_effect`, `provider_commit_identity`, `reserve` … |
| `primary_graph::admitted_capture::CaptureStop` | primary_graph -> authorization | 4 | movable |  |
| `primary_graph::application_attempt::elevation_request_outcome::WorthQueryRequestedElevation` | primary_graph -> authorization | 4 | movable | carries 11 |
| `primary_graph::application_entry::mutation::execution::MutationHandlerExecutionDenial` | primary_graph -> authorization | 4 | behaviour-coupled | 1 impl(s) need items above; blocked by 5: `relational_read_kind`, `native_decision_cause`, `equal_cause`, `native_index_decision_cause` … |
| `primary_graph::product_operation::shared_query_basis::SharedSelectedProductOperation` | primary_graph -> authorization | 4 | movable | carries 1 |
| `primary_graph::provider::branch_commit_coordination::WorthQueryApplicationBranchCommitCoordination` | primary_graph -> authorization | 4 | movable |  |
| `provider_session::protocol::declared_closure::WorthQueryProviderPlanDeclarations` | provider_session -> operation_binding | 4 | behaviour-coupled | 1 impl(s) need items above; blocked by 7: `reversal_posture`, `bind_application_candidate_effect_closure`, `bind_native_reference_invariants`, `bind_direct_role_closure` … |
| `application_aftermath::correction::selection::original::OriginalCorrectionDisclosure` | application_aftermath -> authorization | 3 | movable | carries 2 |
| `primary_graph::application_attempt::denial::WorthQueryApplicationAttemptDenial` | primary_graph -> authorization | 3 | behaviour-coupled | 4 impl(s) need items above; blocked by 3: `relational_read_kind`, `equal_cause`, `query_read_execution_kind` |
| `primary_graph::application_attempt::denial::kind::WorthQueryApplicationAttemptDenialKind` | primary_graph -> authorization | 3 | movable | carries 1 |
| `primary_graph::application_attempt::elevation_close_outcome::WorthQueryElevationClosureKind` | primary_graph -> authorization | 3 | movable |  |
| `primary_graph::application_attempt::idempotency_resolution::WorthQueryApplicationIdempotencyResolution` | primary_graph -> authorization | 3 | movable | carries 1 |
| `primary_graph::application_attempt::idempotency_resolution::denial::WorthQueryApplicationIdempotencyResolutionDenial` | primary_graph -> authorization | 3 | movable | carries 3 |
| `primary_graph::application_attempt::provider_execution::application_attempt_affinity::WorthQueryApplicationAttemptBasis` | primary_graph -> provider_session | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 201: `demanded_field_slot`, `NEXT_OPERATION_ADMISSION_IDENTITY`, `tree_clone_bytes`, `checkpoint_names` … |
| `primary_graph::application_attempt::read_set::handler_fact_boundary::CompletedHandlerFactBoundary` | primary_graph -> authorization | 3 | movable | carries 6 |
| `primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` | primary_graph -> authorization | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 104: `demanded_field_slot`, `tree_clone_bytes`, `checkpoint_names`, `empty` … |
| `primary_graph::application_output_demand::registry::required_context::RequiredOutputDemandContext` | primary_graph -> authorization | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 416: `demanded_field_slot`, `insert`, `lookup`, `remove` … |
| `primary_graph::application_query::managed_index::invocation::WorthQueryManagedIndexWorkDenial` | primary_graph -> provider_session | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `equal_cause` |
| `primary_graph::application_query::observed_source::WorthQuerySourceExpectationKind` | primary_graph -> authorization | 3 | movable |  |
| `primary_graph::application_query::observed_source::query_context_binding::ObservedQueryExpressionContext` | primary_graph -> authorization | 3 | movable | carries 1 |
| `primary_graph::entity_resolution::product::issued_scope::WorthQueryIssuedSelectedScope` | primary_graph -> authorization | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 3: `publication_execution_kind`, `scope_stop_kind`, `bytes` |
| `primary_graph::entity_resolution::retained_selector::WorthQueryObservedScopeSelector` | primary_graph -> authorization | 3 | movable |  |
| `primary_graph::product_operation::correction_source::preparation::WorthQueryPreparedCorrectionSource` | primary_graph -> application_aftermath | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 1314: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `primary_graph::provider::application_decision_fact::WorthQueryPrimaryGraphApplicationDecisionFact` | primary_graph -> authorization | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 21: `relational_read_kind`, `array_storage_bytes`, `bytes`, `deep_clone_bytes` … |
| `primary_graph::resolution::WorthQueryPrincipalResolutionMode` | primary_graph -> authorization | 3 | movable |  |
| `primary_graph::resolution::selected_product::admitted::issued_principal::WorthQueryIssuedSelectedPrincipal` | primary_graph -> authorization | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 10: `relational_read_kind`, `heap_sort`, `union_visit`, `equal_cause` … |
| `primary_graph::schema_layout::principal_binding::WorthQueryPrimaryPrincipalBindingLayout` | primary_graph -> authorization | 3 | movable |  |
| `primary_graph::source_disclosure::inspection::OriginalSourceInspectionDisclosure` | primary_graph -> authorization | 3 | movable | carries 3 |
| `convergence_epoch::domain_provider::WorthQueryConvergenceDomainProvider` | convergence_epoch -> provider_session | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 3: `validate_incumbent_removals`, `portable_family`, `portable_identity` |
| `managed_run::direct_graph_execution::completion::WorthQueryCompletedDirectEvidenceOwner` | managed_run -> provider_session | 2 | movable |  |
| `managed_run::terminal::WorthQueryManagedRunTerminalKind` | managed_run -> provider_session | 2 | movable |  |
| `managed_run::workflow::run_affinity::WorthQueryWorkflowRunTransitionPermit` | managed_run -> provider_session | 2 | movable |  |
| `managed_run::workflow_graph_execution::step_outcome::WorthQueryCompletedWorkflowEvidenceOwner` | managed_run -> provider_session | 2 | movable |  |
| `primary_graph::application_attempt::commit_authority_binding::WorthQueryRetainedGovernedInput` | primary_graph -> application_aftermath | 2 | movable | carries 1 |
| `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenialStage` | primary_graph -> authorization | 2 | movable |  |
| `primary_graph::application_attempt::elevation_approval_outcome::WorthQueryElevationApprovalOutcome` | primary_graph -> authorization | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 11: `allocation_control`, `authority_execution`, `control_kind`, `execution_kind` … |
| `primary_graph::application_attempt::elevation_close_outcome::WorthQueryMandatoryReview` | primary_graph -> authorization | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `checkpoint` |
| `primary_graph::application_attempt::provider_execution::application_attempt_affinity::WorthQueryApplicationAttemptAffinity` | primary_graph -> provider_session | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 201: `demanded_field_slot`, `NEXT_OPERATION_ADMISSION_IDENTITY`, `tree_clone_bytes`, `checkpoint_names` … |
| `primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotRelease` | primary_graph -> provider_session | 2 | movable | carries 1 |
| `primary_graph::application_attempt::workflow_transition_program::actor_context::WorthQueryWorkflowActorContext` | primary_graph -> authorization | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 223: `demanded_field_slot`, `with_checkpoint`, `with_resources`, `tree_clone_bytes` … |
| `primary_graph::application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` | primary_graph -> authorization | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 105: `demanded_field_slot`, `tree_clone_bytes`, `checkpoint_names`, `empty` … |
| `primary_graph::application_query::access_context::WorthQueryApplicationQueryAccessContext` | primary_graph -> authorization | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 13: `relational_read_kind`, `heap_sort`, `union_visit`, `publication_execution_kind` … |
| `primary_graph::application_query::resource_lifecycle::basis_registry::WorthQueryApplicationBasisSelectionIdentity` | primary_graph -> provider_session | 2 | movable |  |
| `primary_graph::application_runtime::runtime_core::WorthQueryApplicationRuntimeCore` | primary_graph -> authorization | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 576: `provider_commit_identity`, `reserve`, `bind_dispatch_outbox_create_intent`, `demanded_field_slot` … |
| `primary_graph::entity_resolution::WorthQueryEntityResolutionTruth` | primary_graph -> authorization | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 8: `publication_execution_kind`, `scope_stop_kind`, `entity_denial`, `validate` … |
| `primary_graph::entity_resolution::resolved::WorthQueryResolvedEntity` | primary_graph -> authorization | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 3: `publication_execution_kind`, `scope_stop_kind`, `bytes` |
| `primary_graph::freshness::WorthQueryDurablePrincipalCurrentness` | primary_graph -> authorization | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 36: `relational_read_kind`, `observe_field_value`, `denial`, `equal_cause` … |
| `primary_graph::handler::decision_context_use::DecisionContextUse` | primary_graph -> authorization | 2 | movable |  |
| `primary_graph::invariant_projection::consumed_output::verification::capacity_stop::VerificationCapacityStop` | primary_graph -> provider_session | 2 | movable |  |
| `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` | primary_graph -> execution_runtime | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 115: `authorization_denial`, `field_locator`, `relation`, `compile_capability_plan` … |
| `primary_graph::program_occurrence::original_meaning::OriginalProgramMeaning` | primary_graph -> authorization | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 911: `receipt_backing_bytes`, `provider_commit_identity`, `reserve`, `bind_dispatch_outbox_create_intent` … |
| `primary_graph::provider::application_attempt_state::registered_reads::RegisteredPrimaryRead` | primary_graph -> provider_session | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 212: `demanded_field_slot`, `principal_currentness`, `NEXT_OPERATION_ADMISSION_IDENTITY`, `tree_clone_bytes` … |
| `primary_graph::provider::session_commit::outbox_commit::WorthQueryCommittedDispatchOutboxBinding` | primary_graph -> application_aftermath | 2 | movable | carries 1 |
| `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` | primary_graph -> product_publication | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 224: `diagnostic_shape_supported`, `diagnostic_rule`, `slot_lookup_work`, `field_tuple` … |
| `primary_graph::root::WorthQueryPrimaryGraphIntegrationHandle` | primary_graph -> execution_runtime | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 1492: `receipt_backing_bytes`, `dispatch_external_effect`, `provider_commit_identity`, `reserve` … |
| `provider_session::graph_obligation::session_identity::WorthQueryGraphWorkManagedRunIdentity` | provider_session -> operation_binding | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `NEXT_GRAPH_WORK_MANAGED_RUN_IDENTITY` |
| `provider_session::graph_obligation::session_identity::WorthQueryGraphWorkSessionIdentity` | provider_session -> operation_binding | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `NEXT_GRAPH_WORK_SESSION_IDENTITY` |
| `provider_session::graph_provider::call_kind::WorthQueryGraphProviderCallKind` | provider_session -> operation_binding | 2 | movable |  |
| `application_aftermath::retained_preimage::WorthQueryPreImageRetentionDenial` | application_aftermath -> provider_session | 1 | movable |  |
| `authorization::denial::WorthQueryOperationAuthorizationDenialKind` | authorization -> provider_session | 1 | movable | carries 1 |
| `managed_run::direct_admission::WorthQueryManagedRunAdmission` | managed_run -> execution_runtime | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1581: `receipt_backing_bytes`, `dispatch_external_effect`, `provider_commit_identity`, `reserve` … |
| `managed_run::direct_terminal::cleanup::WorthQueryDirectRunCleanupFailure` | managed_run -> provider_session | 1 | movable | carries 14 |
| `managed_run::direct_terminal::inspection::WorthQueryDirectRunCleanupReceipt` | managed_run -> provider_session | 1 | movable | carries 11 |
| `managed_run::readmission::direct_lane::WorthQueryDirectReadmissionTransitionPermit` | managed_run -> provider_session | 1 | movable |  |
| `managed_run::terminal::WorthQueryManagedRunCleanupFailureKind` | managed_run -> provider_session | 1 | movable |  |
| `managed_run::workflow::WorthQueryRunningWorkflowRun` | managed_run -> provider_session | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 27: `consumer_terminal_kind`, `producer_terminal_kind`, `classify_suspension_invocation`, `retain_suspended_checkpoint` … |
| `managed_run::workflow::provider_plan_admission::WorthQueryWorkflowProviderPlanPermit` | managed_run -> provider_session | 1 | movable |  |
| `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` | operation_binding -> artifact_owner | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 15: `NEXT_OPERATION_ADMISSION_IDENTITY`, `installed_artifact_identity`, `exact_graph_labels`, `exact_labels` … |
| `primary_graph::application_attempt::compare_and_commit::commit_outcome::WorthQueryApplicationCommitOutcome` | primary_graph -> authorization | 1 | behaviour-coupled | 3 impl(s) need items above; blocked by 1332: `receipt_backing_bytes`, `dispatch_external_effect`, `provider_commit_identity`, `reserve` … |
| `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenial` | primary_graph -> application_aftermath | 1 | behaviour-coupled | 3 impl(s) need items above; blocked by 10: `allocation_control`, `authority_execution`, `control_kind`, `execution_kind` … |
| `primary_graph::application_attempt::compare_and_commit::committed_publication::WorthQueryCommittedProductPublication` | primary_graph -> application_aftermath | 1 | movable | carries 2 |
| `primary_graph::application_attempt::precondition_binding::EmptyMutationPreconditionAdmissionStop` | primary_graph -> authorization | 1 | movable |  |
| `primary_graph::application_attempt::provider_binding::WorthQueryPreparedApplicationProviderAttempt` | primary_graph -> authorization | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 918: `receipt_backing_bytes`, `provider_commit_identity`, `reserve`, `bind_dispatch_outbox_create_intent` … |
| `primary_graph::application_attempt::provider_execution::external_dispatch::WorthQueryPerformedExternalRedispatchSeal` | primary_graph -> application_aftermath | 1 | movable | carries 1 |
| `primary_graph::application_attempt::provider_execution::external_dispatch::denial::WorthQueryExternalDispatchAttemptDenial` | primary_graph -> application_aftermath | 1 | movable |  |
| `primary_graph::application_attempt::provider_execution::outcome::WorthQueryProviderProgressionOutcome` | primary_graph -> authorization | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 12: `bind_dispatch_outbox_create_intent`, `allocation_control`, `authority_execution`, `control_kind` … |
| `primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::registered::WorthQueryProviderAttemptRegistrationContext` | primary_graph -> authorization | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 917: `receipt_backing_bytes`, `provider_commit_identity`, `reserve`, `bind_dispatch_outbox_create_intent` … |
| `primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::registered::WorthQueryRegisteredProviderAttempt` | primary_graph -> authorization | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 56: `bind_dispatch_outbox_create_intent`, `tree_clone_bytes`, `checkpoint_names`, `allocation_control` … |
| `primary_graph::application_attempt::retained_decision_facts::control::StorageControl` | primary_graph -> provider_session | 1 | movable | carries 4 |
| `primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLeaseDenial` | primary_graph -> authorization | 1 | movable |  |
| `primary_graph::application_contribution::computation::denial::WorthQueryMemoryLimitLevel` | primary_graph -> managed_run | 1 | movable |  |
| `primary_graph::application_contribution::partitioned_computation::incremental::retained::SealedComputationRun` | primary_graph -> authorization | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 104: `demanded_field_slot`, `tree_clone_bytes`, `checkpoint_names`, `empty` … |
| `primary_graph::application_contribution::producer::demand::denial::WorthQueryOutputDemandDenial` | primary_graph -> provider_session | 1 | behaviour-coupled | 3 impl(s) need items above; blocked by 1461: `receipt_backing_bytes`, `dispatch_external_effect`, `provider_commit_identity`, `reserve` … |
| `primary_graph::application_contribution::producer::demand::denial::kind::WorthQueryOutputDemandDenialKind` | primary_graph -> provider_session | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 3: `is_temporarily_deferred`, `equal_cause`, `relational_execution_kind` |
| `primary_graph::application_runtime::external_dispatch_attempt::WorthQueryExternalDispatchAttemptOrdinal` | primary_graph -> application_aftermath | 1 | movable |  |
| `primary_graph::application_runtime::inbound_publication::outcome::WorthQueryPerformedInboundCompletion` | primary_graph -> application_aftermath | 1 | movable |  |
| `primary_graph::application_runtime::inbound_publication::outcome::WorthQueryUnpublishedInboundCompletion` | primary_graph -> application_aftermath | 1 | movable |  |
| `primary_graph::entity_resolution::WorthQueryInstalledEntityResolutionContext` | primary_graph -> authorization | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 40: `relational_read_kind`, `observe_field_value`, `denial`, `publication_execution_kind` … |
| `primary_graph::entity_resolution_denial::WorthQueryEntityResolutionDenialKind` | primary_graph -> authorization | 1 | movable | carries 1 |
| `primary_graph::freshness::WorthQueryPrincipalFreshnessEvidence` | primary_graph -> authorization | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 10: `relational_read_kind`, `heap_sort`, `union_visit`, `equal_cause` … |
| `primary_graph::output_lineage::invalidation::head_cell::HeadCellRegistrationStop` | primary_graph -> execution_runtime | 1 | movable |  |
| `primary_graph::product_operation::generated_materialization::restoration::invariant_denial::WorthQueryGeneratedOutputInvariantAdmissionDenial` | primary_graph -> provider_session | 1 | movable |  |
| `primary_graph::provider::application_attempt_state::commit_preparation::preimage_retention::selection::WorthQueryRetainedPreImageSeal` | primary_graph -> application_aftermath | 1 | movable | carries 1 |
| `primary_graph::provider::application_decision_fact::application_facts::CompletedApplicationFacts` | primary_graph -> authorization | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 205: `demanded_field_slot`, `with_checkpoint`, `with_resources`, `tree_clone_bytes` … |
| `primary_graph::provider::application_decision_fact::application_table::ApplicationFactTable` | primary_graph -> authorization | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 204: `demanded_field_slot`, `with_checkpoint`, `with_resources`, `tree_clone_bytes` … |
| `primary_graph::provider::committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxReadDenial` | primary_graph -> application_aftermath | 1 | movable |  |
| `primary_graph::provider::decision_facts::comparison::PrimaryDecisionReadSetComparison` | primary_graph -> provider_session | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 219: `demanded_field_slot`, `principal_currentness`, `NEXT_OPERATION_ADMISSION_IDENTITY`, `tree_clone_bytes` … |
| `primary_graph::provider::mutation_work::WorthQueryPrimaryMutationWorkEvidence` | primary_graph -> application_aftermath | 1 | movable | carries 7 |
| `primary_graph::resolution_denial::WorthQueryPrincipalResolutionDenial` | primary_graph -> authorization | 1 | movable | carries 2 |
| `primary_graph::resolution_denial::WorthQueryPrincipalResolutionDenialKind` | primary_graph -> authorization | 1 | movable | carries 1 |
| `primary_graph::root::WorthQueryPrimaryGraph` | primary_graph -> execution_runtime | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 479: `demanded_field_slot`, `artifact_authority_denial_detail`, `ensure_live`, `mark_disposed_if_unowned` … |
| `primary_graph::selected_read_indexes::PreparedSelectedReadIndexes` | primary_graph -> authorization | 1 | movable |  |
| `primary_graph::source_disclosure::original::OriginalSourceDisclosure` | primary_graph -> application_aftermath | 1 | movable | carries 1 |
| `product_publication::unpublished::WorthQueryProductUnpublishedApplication` | product_publication -> execution_runtime | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 1547: `receipt_backing_bytes`, `dispatch_external_effect`, `provider_commit_identity`, `reserve` … |
| `provider_session::graph_provider::receipt_association::WorthQueryBoundGraphExecutionAssociation` | provider_session -> domain_evidence_binding | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 8: `NEXT_EXECUTION_ATTEMPT`, `next_execution_attempt_ordinal`, `same_execution_basis`, `NEXT_GRAPH_CALL_AUTHORITY` … |
| `provider_session::graph_provider::receipt_association::WorthQueryCompletedDomainEvidenceDerivation` | provider_session -> domain_evidence_binding | 1 | movable | carries 1 |
| `provider_session::protocol::control_stopped::WorthQueryProviderSessionControlStopKind` | provider_session -> execution_runtime | 1 | movable |  |
| `provider_session::protocol::declared_closure::application::WorthQueryApplicationEffectPosture` | provider_session -> operation_binding | 1 | movable |  |
| `provider_session::protocol::execution_plan::WorthQueryValidatedProviderPlan` | provider_session -> operation_binding | 1 | movable |  |
| `provider_session::protocol::plan_contract::WorthQueryProviderExecutionPlanContract` | provider_session -> operation_binding | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 8: `NEXT_OPERATION_ADMISSION_IDENTITY`, `reversal_posture`, `bind_application_candidate_effect_closure`, `bind_native_reference_invariants` … |

### Types whose impls live across many modules

A unit moves with all its inherent impls, so these types cannot move, or be split into
another crate, until their impls are regrouped or turned into owned extension traits.

| type | owner | impl blocks | impl lines | modules hosting impls |
|---|---|---|---|---|
| `execution_runtime::runtime_root::WorthQueryExecutionRuntime` | execution_runtime | 9 | 392 | 5 |
| `primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime` | primary_graph | 310 | 28109 | 4 |
| `execution_runtime::product_world::runtime::WorthQueryProductRuntime` | execution_runtime | 32 | 1070 | 2 |
| `primary_graph::application_runtime::runtime_core::WorthQueryApplicationRuntimeCore` | primary_graph | 32 | 388 | 2 |
| `execution_runtime::installation_authority::WorthQueryExecutionInstallationAuthority` | execution_runtime | 6 | 349 | 2 |
| `provider_session::protocol::terminal_binding::WorthQueryProviderSessionTerminalBinding` | provider_session | 6 | 129 | 2 |
| `execution_observation::WorthQueryExecutionObservation` | execution_observation | 4 | 62 | 2 |
| `application_aftermath::recovery_handle::denial::WorthQueryRecoveryHandleDenial` | application_aftermath | 2 | 57 | 2 |
| `provider_session::decision_read_set::fact::WorthQueryDecisionFactProvider` | provider_session | 1 | 45 | 1 |
| `provider_session::graph_provider::provider_contract::WorthQueryGraphParticipationProvider` | provider_session | 1 | 24 | 1 |
| `provider_session::graph_provider::provider_contract::WorthQueryProviderSessionLifecycle` | provider_session | 1 | 84 | 1 |
| `provider_session::provisional_attempt::invariant_execution::denial::WorthQueryCustomInvariantDenial` | provider_session | 1 | 24 | 1 |
| `provider_session::provisional_attempt::invariant_execution::provider_port::WorthQueryInvariantExecutionProvider` | provider_session | 1 | 68 | 1 |
| `provider_session::provisional_attempt::provider_port::WorthQueryProvisionalGraphProvider` | provider_session | 1 | 41 | 1 |

### Direct blockers

Non-type items above the destination that a coupled unit's own declaration or impls call.

| item | units held | their references |
|---|---|---|
| `primary_graph::provider::application_decision_fact::binding_failure` | 3 | 5 |
| `primary_graph::provider::relational_execution_denial::relational_execution_kind` | 3 | 58 |
| `primary_graph::application_attempt::observation::observe_field_value` | 2 | 63 |
| `primary_graph::application_checkpoint::facts::WIRE_VERSION` | 2 | 66 |
| `primary_graph::application_contribution::producer::demand::bridge_denial::bridge_denial_kind` | 2 | 57 |
| `primary_graph::application_contribution::request_execution::advancement::advancement_denial` | 2 | 90 |
| `primary_graph::entity_resolution::entity_denial` | 2 | 3 |
| `primary_graph::fact_verification_observation::provider_application_fact` | 2 | 2 |
| `primary_graph::freshness::comparison_observation` | 2 | 3 |
| `primary_graph::index_maintenance_budget::cold_index_reconstruction_budget` | 2 | 58 |
| `primary_graph::index_maintenance_budget::refresh_with_cold_fallback` | 2 | 58 |
| `primary_graph::observations::observe_exact_principal_target` | 2 | 3 |
| `primary_graph::observations::principal_mapping::observe_mapping` | 2 | 3 |
| `primary_graph::output_reuse::selection::preflight_stop` | 2 | 57 |
| `primary_graph::output_reuse::selection::read_denial` | 2 | 57 |
| `primary_graph::output_reuse::selection::retained_fact_denial` | 2 | 57 |
| `primary_graph::provider::decision_facts::storage_read_set_failure` | 2 | 2 |
| `primary_graph::structural_walk::memory::reserve` | 2 | 61 |
| `application_aftermath::external_effect::dispatch::dispatch_external_effect` | 1 | 56 |
| `application_aftermath::recovery_handle::mint::claim_for` | 1 | 56 |
| `application_aftermath::recovery_handle::mint::ensure_receipt_belongs_to_runtime` | 1 | 56 |
| `application_aftermath::recovery_handle::mint::mint_recovery_handle_in_registry` | 1 | 56 |
| `application_aftermath::recovery_progression::authority::admission_authorization_denial` | 1 | 56 |
| `application_aftermath::recovery_progression::authority::ensure_same_runtime` | 1 | 56 |
| `application_aftermath::recovery_progression::authority::require_fresh_effect_authority` | 1 | 56 |
| `application_aftermath::recovery_progression::binding_axis::check_binding_axes` | 1 | 56 |
| `application_aftermath::recovery_progression::expiry::deny_if_expired` | 1 | 56 |
| `application_aftermath::recovery_progression::expiry::evaluate_expiry` | 1 | 56 |
| `authorization::admission::validation::denial` | 1 | 56 |
| `authorization::application_disclosure::contract::compilation::compile_disclosure_contract` | 1 | 56 |

### What holds the behaviour-coupled units (transitively)

Each non-type item above a destination that some coupled unit (directly or through a
coupled type it needs) depends on; inverting the top rows frees the most units.

| item | units held | their references |
|---|---|---|
| `primary_graph::application_query::managed_index::provider_failure::equal_cause` | 53 | 265 |
| `primary_graph::application_attempt::denial::request_read::relational_read_kind` | 47 | 250 |
| `primary_graph::provider::relational_execution_denial::query_read_execution_kind` | 47 | 250 |
| `primary_graph::application_contribution::conditional::output_readiness_contract::denial` | 39 | 218 |
| `primary_graph::provider::relational_execution_denial::relational_execution_kind` | 38 | 154 |
| `primary_graph::admitted_capture::storage::tree_clone_bytes` | 35 | 154 |
| `primary_graph::application_checkpoint::facts::payload::array_storage_bytes` | 35 | 185 |
| `primary_graph::application_checkpoint::facts::payload::chunk::bytes` | 35 | 185 |
| `primary_graph::application_checkpoint::facts::payload::chunk::deep_clone_bytes` | 35 | 185 |
| `primary_graph::application_checkpoint::facts::payload::concatenation::join` | 35 | 185 |
| `primary_graph::application_checkpoint::facts::payload::original_storage_bytes` | 35 | 185 |
| `primary_graph::application_checkpoint::facts::payload::patch::preparation::build` | 35 | 185 |
| `primary_graph::application_checkpoint::facts::payload::patch::preparation::count_nodes` | 35 | 185 |
| `primary_graph::application_checkpoint::facts::payload::patch::take_published` | 35 | 185 |
| `primary_graph::application_checkpoint::facts::payload::resident_bytes` | 35 | 185 |
| `primary_graph::fact_verification_observation::native_patch_clone` | 35 | 185 |
| `primary_graph::fact_verification_observation::native_patch_path` | 35 | 185 |
| `primary_graph::fact_verification_observation::native_payload_chunk` | 35 | 185 |
| `primary_graph::fact_verification_observation::native_payload_fact_move` | 35 | 185 |
| `primary_graph::fact_verification_observation::native_payload_join` | 35 | 185 |

### Functions, constants and modules referenced against the stack

| item | kind | owner | refs | from |
|---|---|---|---|---|
| `primary_graph::admitted_capture::checkpoint` | fn | primary_graph | 8 | authorization 8 |
| `primary_graph::workflow::actor::encoding::check_live` | fn | primary_graph | 5 | authorization 5 |
| `primary_graph::workflow::actor::fund_actor_codec` | fn | primary_graph | 5 | authorization 5 |
| `primary_graph::admitted_capture::retained_stage::run_retained_stage` | fn | primary_graph | 4 | authorization 4 |
| `primary_graph::application_attempt::fact::consult_observation` | module | primary_graph | 4 | authorization 4 |
| `primary_graph::application_contribution::execution_denial` | module | primary_graph | 4 | execution_runtime 2, product_publication 2 |
| `primary_graph::admitted_capture::storage::tree_clone_bytes` | fn | primary_graph | 3 | authorization 3 |
| `primary_graph::provider::relational_execution_denial::relational_execution_kind` | fn | primary_graph | 3 | authorization 3 |
| `primary_graph::application_attempt::precondition_binding::bind_mutation_preconditions` | fn | primary_graph | 2 | authorization 2 |
| `primary_graph::freshness::validate_freshness_at_snapshot` | fn | primary_graph | 2 | authorization 2 |
| `primary_graph::workflow::actor::encoding` | module | primary_graph | 2 | authorization 2 |
| `primary_graph::workflow::actor::encoding::encode` | fn | primary_graph | 2 | authorization 2 |
| `invalidation_equivalence_control::attach_constructed_receiver` | fn | invalidation_equivalence_control | 1 | execution_runtime 1 |
| `primary_graph::application_attempt::precondition_binding::bind_empty_mutation_preconditions_admitted` | fn | primary_graph | 1 | authorization 1 |
| `primary_graph::application_attempt::provider_execution::outcome::progression_denied` | fn | primary_graph | 1 | authorization 1 |
| `primary_graph::application_attempt::provider_execution::resource_request::application_resource_request` | fn | primary_graph | 1 | authorization 1 |
| `primary_graph::application_contribution::partitioned_computation::incremental::published_state::observe_discarded` | fn | primary_graph | 1 | authorization 1 |
| `primary_graph::content::retention_work::checkpoint` | fn | primary_graph | 1 | application_aftermath 1 |
| `primary_graph::index_maintenance_budget::ordinary_index_maintenance_budget` | fn | primary_graph | 1 | authorization 1 |
| `primary_graph::principal_currentness_capture::capture_principal_currentness_admitted` | fn | primary_graph | 1 | authorization 1 |
| `primary_graph::publication_request_denial::publication` | fn | primary_graph | 1 | execution_runtime 1 |
| `primary_graph::source_facts::authorization_currentness::validate_source_principal_facts` | fn | primary_graph | 1 | authorization 1 |
| `primary_graph::workflow::actor` | module | primary_graph | 1 | authorization 1 |

## Ruling sheet: heaviest mutual pairs

### `authorization` ↔ `primary_graph`

- references: authorization -> primary_graph 321, primary_graph -> authorization 669
- against the stack (authorization -> primary_graph): 321 references; 28 movable units, 42 behaviour-coupled, 0 pinned, 19 functions/constants/modules
- crossing up: `WorthQueryPrimaryGraphApplicationRuntime` (behaviour-coupled, 50), `WorthQueryPrimaryGraphLayout` (behaviour-coupled, 16), `WorthQueryAuthenticatedPrincipal` (behaviour-coupled, 11), `WorkflowRecordedActorObservation` (behaviour-coupled, 10), `WorthQueryAdvancementPhase` (behaviour-coupled, 10), `WorthQueryApplicationEntityIdentity` (behaviour-coupled, 9), `WorthQueryBoundMutationPreconditions` (behaviour-coupled, 8), `FundedActorText` (movable, 8), `WorthQueryApplicationObservedFact` (behaviour-coupled, 7), `WorthQueryApprovedElevation` (behaviour-coupled, 6), `ActorCodecFunding` (movable, 6), `RetainedStorageDenial` (movable, 5), `WorthQueryApplicationInspectionTarget` (behaviour-coupled, 5), `CaptureStop` (movable, 4) …
- crossing down (top): `WorthQueryAdmittedApplicationOperation` 97, `WorthQueryOperationAuthorizationDenial` 67, `WorthQueryOperationScopeEntityBinding` 64, `WorthQueryOperationAuthorizationDenialKind` 54, `WorthQueryApplicationQueryGovernance` 44, `WorthQueryAdmittedApplicationCapabilityAccess` 32, `WorthQueryOperationAuthorizationRefusalKind` 32, `WorthQueryPendingApplicationQueryGovernance` 27
- option nest: move 30136 lines of authorization under primary_graph; 15 references from other modules below primary_graph would then point up
- option move units: 28 units by script (78 references)
- option contract trait: a contract owned by authorization, implemented in primary_graph, 61 items (243 references)
- **default**: nest authorization under primary_graph (30136 lines); its other users then go through primary_graph

### `provider_session` ↔ `primary_graph`

- references: provider_session -> primary_graph 61, primary_graph -> provider_session 657
- against the stack (provider_session -> primary_graph): 61 references; 8 movable units, 14 behaviour-coupled, 0 pinned, 0 functions/constants/modules
- crossing up: `WorthQueryApplicationBasisIdentity` (movable, 16), `WorthQueryApplicationBasisReleaseReceipt` (movable, 7), `WorthQueryPrimaryGraphLayout` (behaviour-coupled, 6), `WorthQueryApplicationInspectionTarget` (behaviour-coupled, 4), `WorthQueryApplicationSnapshotLease` (behaviour-coupled, 3), `WorthQueryManagedIndexWorkDenial` (behaviour-coupled, 3), `WorthQueryApplicationAttemptAffinity` (behaviour-coupled, 2), `WorthQueryApplicationAttemptBasis` (behaviour-coupled, 2), `WorthQueryApplicationSnapshotRelease` (movable, 2), `WorthQueryApplicationBasisSelectionIdentity` (movable, 2), `VerificationCapacityStop` (movable, 2), `RegisteredPrimaryRead` (behaviour-coupled, 2), `StorageControl` (movable, 1), `WorthQueryManagedComputationResourceDenial` (movable, 1) …
- crossing down (top): `WorthQueryProviderSessionControlStopKind` 78, `WorthQueryProviderSessionDenialKind` 65, `WorthQueryManagedGraphWorkSession` 42, `WorthQueryApplicationEffectPosture` 30, `WorthQueryProviderSessionFailure` 29, `WorthQueryGraphWorkSessionIdentity` 23, `WorthQueryProviderSessionProtocolStage` 19, `WorthQueryProviderSessionCommitStop` 16
- option nest: move 16886 lines of provider_session under primary_graph; 311 references from other modules below primary_graph would then point up
- option move units: 8 units by script (32 references)
- option contract trait: a contract owned by provider_session, implemented in primary_graph, 14 items (29 references)
- **default**: move the 8 movable units by script; a contract owned by provider_session for the 14 items that stay behaviour-bound in primary_graph

### `application_aftermath` ↔ `primary_graph`

- references: application_aftermath -> primary_graph 41, primary_graph -> application_aftermath 244
- against the stack (application_aftermath -> primary_graph): 41 references; 15 movable units, 6 behaviour-coupled, 0 pinned, 1 functions/constants/modules
- crossing up: `WorthQueryCommittedDispatchOutboxObservation` (movable, 6), `WorthQueryApplicationCommitReceipt` (behaviour-coupled, 4), `WorthQueryApplicationIdempotencyBinding` (behaviour-coupled, 4), `WorthQueryPrimaryGraphApplicationRuntime` (behaviour-coupled, 4), `WorthQueryPreparedCorrectionSource` (behaviour-coupled, 3), `WorthQueryRetainedGovernedInput` (movable, 2), `WorthQueryApplicationIdempotencyResolution` (movable, 2), `WorthQueryCommittedDispatchOutboxBinding` (movable, 2), `WorthQueryApplicationCommitDenial` (behaviour-coupled, 1), `WorthQueryApplicationCommitDenialStage` (movable, 1), `WorthQueryCommittedProductPublication` (movable, 1), `WorthQueryPerformedExternalRedispatchSeal` (movable, 1), `WorthQueryExternalDispatchAttemptDenial` (movable, 1), `WorthQueryAdvancementDenial` (behaviour-coupled, 1) …
- crossing down (top): `WorthQueryPendingAftermathCausality` 22, `ExternalEffectCorrelationIdentity` 22, `ApplicationCommitCausality` 17, `WorthQueryDispatchOutboxRecord` 17, `WorthQueryRetainedPreImage` 13, `WorthQueryAcceptedInboundOccurrence` 10, `WorthQueryCommittedAftermathCausality` 10, `WorthQueryDispatchOutboxLayout` 9
- option nest: move 6004 lines of application_aftermath under primary_graph; 4 references from other modules below primary_graph would then point up
- option move units: 15 units by script (23 references)
- option contract trait: a contract owned by application_aftermath, implemented in primary_graph, 7 items (18 references)
- **default**: move the 15 movable units by script; a contract owned by application_aftermath for the 7 items that stay behaviour-bound in primary_graph

### `execution_runtime` ↔ `primary_graph`

- references: execution_runtime -> primary_graph 17, primary_graph -> execution_runtime 253
- against the stack (execution_runtime -> primary_graph): 17 references; 3 movable units, 5 behaviour-coupled, 0 pinned, 2 functions/constants/modules
- crossing up: `WorthQueryAdvancementDenial` (behaviour-coupled, 4), `WorthQueryAdvancementPhase` (behaviour-coupled, 3), `SourceInvalidationOwner` (behaviour-coupled, 2), `WorthQueryManagedComputationResourceDenial` (movable, 1), `WorthQueryForeignAdvancementPhase` (movable, 1), `HeadCellRegistrationStop` (movable, 1), `WorthQueryPrimaryGraph` (behaviour-coupled, 1), `WorthQueryPrimaryGraphIntegrationHandle` (behaviour-coupled, 1), `execution_denial` (module, 2), `publication` (fn, 1)
- crossing down (top): `WorthQueryRuntimeAuthorityIdentity` 37, `WorthQueryProductPublicationBinding` 18, `WorthQueryOutputDemandLimits` 17, `WorthQueryProductPublicationOutcome` 16, `WorthQueryProductPublicationDenial` 16, `OwnerCleanupRetirements` 15, `WorthQueryExecutionRuntime` 14, `WorthQueryProductRuntime` 14
- option nest: move 5743 lines of execution_runtime under primary_graph; 73 references from other modules below primary_graph would then point up
- option move units: 3 units by script (3 references)
- option contract trait: a contract owned by execution_runtime, implemented in primary_graph, 7 items (14 references)
- **default**: move the 3 movable units by script; a contract owned by execution_runtime for the 7 items that stay behaviour-bound in primary_graph

### `provider_session` ↔ `managed_run`

- references: provider_session -> managed_run 18, managed_run -> provider_session 215
- against the stack (provider_session -> managed_run): 18 references; 9 movable units, 2 behaviour-coupled, 0 pinned, 0 functions/constants/modules
- crossing up: `WorthQueryRunningDirectRun` (behaviour-coupled, 4), `WorthQueryCompletedDirectEvidenceOwner` (movable, 2), `WorthQueryManagedRunTerminalKind` (movable, 2), `WorthQueryWorkflowRunTransitionPermit` (movable, 2), `WorthQueryCompletedWorkflowEvidenceOwner` (movable, 2), `WorthQueryDirectRunCleanupFailure` (movable, 1), `WorthQueryDirectRunCleanupReceipt` (movable, 1), `WorthQueryDirectReadmissionTransitionPermit` (movable, 1), `WorthQueryManagedRunCleanupFailureKind` (movable, 1), `WorthQueryRunningWorkflowRun` (behaviour-coupled, 1), `WorthQueryWorkflowProviderPlanPermit` (movable, 1)
- crossing down (top): `WorthQueryProviderCheckpointReleaseEvidence` 17, `WorthQueryProviderExecutionReleaseEvidence` 17, `WorthQueryGraphProviderCall` 10, `WorthQueryExecutionResourceAttemptEvidence` 8, `WorthQueryProviderCheckpointEvidence` 7, `WorthQueryGraphProviderCallReadmissionPlan` 7, `WorthQueryGraphProviderStepArtifactContext` 7, `WorthQueryGraphProviderMemoryArena` 7
- option nest: move 16886 lines of provider_session under managed_run; 16 references from other modules below managed_run would then point up
- option move units: 9 units by script (13 references)
- option contract trait: a contract owned by provider_session, implemented in managed_run, 2 items (5 references)
- **default**: move the 9 movable units by script; a contract owned by provider_session for the 2 items that stay behaviour-bound in managed_run

### `provider_session` ↔ `authorization`

- references: provider_session -> authorization 3, authorization -> provider_session 78
- against the stack (provider_session -> authorization): 3 references; 1 movable units, 1 behaviour-coupled, 0 pinned, 0 functions/constants/modules
- crossing up: `WorthQueryOperationAdmissionIdentity` (behaviour-coupled, 2), `WorthQueryOperationAuthorizationDenialKind` (movable, 1)
- crossing down (top): `WorthQueryGraphWorkSessionIdentity` 31, `WorthQueryManagedGraphWorkSession` 23, `WorthQueryGraphWorkManagedRunIdentity` 7, `WorthQueryProviderSessionControlStopKind` 5, `WorthQueryGraphWorkAccessContextAffinity` 5, `WorthQueryProviderSessionDenialKind` 3, `WorthQuerySessionBoundReadsAndEffects` 1, `WorthQueryDecisionReadSetFailure` 1
- option nest: move 16886 lines of provider_session under authorization; 231 references from other modules below authorization would then point up
- option move units: 1 units by script (1 references)
- option contract trait: a contract owned by provider_session, implemented in authorization, 1 items (2 references)
- **default**: move the 1 movable units by script; a contract owned by provider_session for the 1 items that stay behaviour-bound in authorization

### `product_publication` ↔ `primary_graph`

- references: product_publication -> primary_graph 9, primary_graph -> product_publication 70
- against the stack (product_publication -> primary_graph): 9 references; 0 movable units, 4 behaviour-coupled, 0 pinned, 1 functions/constants/modules
- crossing up: `WorthQueryAdvancementPhase` (behaviour-coupled, 2), `WorthQueryPrimaryGraphApplicationRuntime` (behaviour-coupled, 2), `WorthQueryUnpublishedIdempotencyDisposition` (behaviour-coupled, 2), `WorthQueryAdvancementDenial` (behaviour-coupled, 1), `execution_denial` (module, 2)
- crossing down (top): `WorthQueryProductUnpublishedRecovery` 21, `WorthQueryProductUnpublishedApplication` 21, `WorthQueryProductUnpublishedRecoveryReleaseFailure` 13, `WorthQueryProductStaleApplication` 12, `WorthQueryProductUnpublishedRecoveryReleaseDenial` 3
- option nest: move 622 lines of product_publication under primary_graph; 8 references from other modules below primary_graph would then point up
- option move units: 0 units by script (0 references)
- option contract trait: a contract owned by product_publication, implemented in primary_graph, 5 items (9 references)
- **default**: move the 0 movable units by script; a contract owned by product_publication for the 5 items that stay behaviour-bound in primary_graph

### `managed_run` ↔ `primary_graph`

- references: managed_run -> primary_graph 30, primary_graph -> managed_run 38
- against the stack (managed_run -> primary_graph): 30 references; 3 movable units, 2 behaviour-coupled, 0 pinned, 0 functions/constants/modules
- crossing up: `WorthQueryAdvancementPhase` (behaviour-coupled, 18), `TestHostRequest` (behaviour-coupled, 6), `WorthQueryForeignAdvancementPhase` (movable, 4), `WorthQueryManagedComputationResourceDenial` (movable, 1), `WorthQueryMemoryLimitLevel` (movable, 1)
- crossing down (top): `WorthQueryRunningDirectRun` 8, `WorthQueryManagedLowerExecutionBasis` 8, `WorthQueryDirectRunCleanupReceipt` 6, `WorthQueryRecoveryHandleRegistry` 4, `WorthQueryManagedRunTerminalKind` 3, `WorthQueryManagedRelationalObservation` 2, `WorthQueryRunningWorkflowRun` 2, `WorthQueryManagedTruthReadRequest` 2
- option nest: move 17529 lines of managed_run under primary_graph; 33 references from other modules below primary_graph would then point up
- option move units: 3 units by script (6 references)
- option contract trait: a contract owned by managed_run, implemented in primary_graph, 2 items (24 references)
- **default**: move the 3 movable units by script; a contract owned by managed_run for the 2 items that stay behaviour-bound in primary_graph

### `operation_binding` ↔ `provider_session`

- references: operation_binding -> provider_session 13, provider_session -> operation_binding 12
- against the stack (operation_binding -> provider_session): 13 references; 3 movable units, 4 behaviour-coupled, 0 pinned, 0 functions/constants/modules
- crossing up: `WorthQueryProviderPlanDeclarations` (behaviour-coupled, 4), `WorthQueryGraphWorkManagedRunIdentity` (behaviour-coupled, 2), `WorthQueryGraphWorkSessionIdentity` (behaviour-coupled, 2), `WorthQueryGraphProviderCallKind` (movable, 2), `WorthQueryApplicationEffectPosture` (movable, 1), `WorthQueryValidatedProviderPlan` (movable, 1), `WorthQueryProviderExecutionPlanContract` (behaviour-coupled, 1)
- crossing down (top): `WorthQueryExecutionBoundOperationAuthority` 12
- option nest: move 1752 lines of operation_binding under provider_session; 6 references from other modules below provider_session would then point up
- option move units: 3 units by script (4 references)
- option contract trait: a contract owned by operation_binding, implemented in provider_session, 4 items (9 references)
- **default**: move the 3 movable units by script; a contract owned by operation_binding for the 4 items that stay behaviour-bound in provider_session

### `execution_runtime` ↔ `managed_run`

- references: execution_runtime -> managed_run 1, managed_run -> execution_runtime 13
- against the stack (execution_runtime -> managed_run): 1 references; 0 movable units, 1 behaviour-coupled, 0 pinned, 0 functions/constants/modules
- crossing up: `WorthQueryManagedRunAdmission` (behaviour-coupled, 1)
- crossing down (top): `WorthQueryExecutionRuntime` 12, `WorthQueryRuntimeAuthorityIdentity` 1
- option nest: move 5743 lines of execution_runtime under managed_run; 32 references from other modules below managed_run would then point up
- option move units: 0 units by script (0 references)
- option contract trait: a contract owned by execution_runtime, implemented in managed_run, 1 items (1 references)
- **default**: move the 0 movable units by script; a contract owned by execution_runtime for the 1 items that stay behaviour-bound in managed_run

## Estimate

- defaults for the pairs above: largest component 11, 26 references against the stack; 42 units by script, 43 items by hand, 1 nests
- defaults plus every movable unit moved: largest component 8, 18 references against the stack; 68 units by script, 100 items by hand
- and the five widest hub types (`WorthQueryExecutionInstallationAuthority`, `WorthQueryProductRuntime`, `WorthQueryExecutionRuntime`, `WorthQueryPrimaryGraphApplicationRuntime`, `WorthQueryApplicationRuntimeCore`) behind contracts: largest component 8, 18 references against the stack

