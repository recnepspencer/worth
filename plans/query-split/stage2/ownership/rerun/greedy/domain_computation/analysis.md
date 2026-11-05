# Ownership analysis: `domain_computation`

## Draft target stack (bottom to top)


### application_contract_admission

- `application_contract_admission` (rank 0): greedy rank 0; uses below nothing; above nothing; used by primary_graph 10, authorization 1.

### application_outcome_identity

- `application_outcome_identity` (rank 1): greedy rank 1; uses below nothing; above nothing; used by authorization 2, primary_graph 1.

### artifact_identity

- `artifact_identity` (rank 2): greedy rank 2; uses below nothing; above nothing; used by artifact_owner 5.

### evidence_identity

- `evidence_identity` (rank 3): greedy rank 3; uses below nothing; above nothing; used by primary_graph 80.

### evidence_material

- `evidence_material` (rank 4): greedy rank 4; uses below nothing; above nothing; used by primary_graph 7.

### execution_observation

- `execution_observation` (rank 5): greedy rank 5; uses below nothing; above nothing; used by primary_graph 33, request_execution 6, provider_session 3.

### request_execution

- `request_execution` (rank 6): greedy rank 6; uses below execution_observation 6; above nothing; used by primary_graph 728, convergence_epoch 36, managed_run 33.

### runtime_time

- `runtime_time` (rank 7): greedy rank 7; uses below nothing; above nothing; used by authorization 25, primary_graph 9, application_aftermath 2.

### domain_evidence_binding

- `domain_evidence_binding` (rank 8): greedy rank 8; uses below nothing; above provider_session 2; used by provider_session 7, managed_run 6, convergence_epoch 2.

### artifact_owner

- `artifact_owner` (rank 9): greedy rank 9; uses below artifact_identity 5; above operation_binding 6; used by managed_run 57, provider_session 16, operation_binding 5.

### execution_runtime

- `execution_runtime` (rank 10): greedy rank 10; uses below request_execution 19, execution_observation 1; above primary_graph 8, provider_session 1, product_publication 1; used by primary_graph 290, authorization 16, managed_run 13.

### invalidation_equivalence_control

- `invalidation_equivalence_control` (rank 11): greedy rank 11; uses below execution_runtime 2; above nothing; used by execution_runtime 1.

### product_publication

- `product_publication` (rank 12): greedy rank 12; uses below execution_runtime 7, request_execution 3; above primary_graph 7; used by primary_graph 94, provider_session 6, execution_runtime 1.

### operation_binding

- `operation_binding` (rank 13): greedy rank 13; uses below execution_runtime 12, artifact_owner 5; above provider_session 13, authorization 3; used by managed_run 16, provider_session 12, artifact_owner 6.

### convergence_epoch

- `convergence_epoch` (rank 14): greedy rank 14; uses below request_execution 36, execution_runtime 7, operation_binding 3; above managed_run 119, provider_session 10; used by provider_session 2.

### application_aftermath

- `application_aftermath` (rank 15): greedy rank 15; uses below execution_runtime 13, runtime_time 2, request_execution 1; above primary_graph 40, managed_run 15, authorization 9; used by primary_graph 283, authorization 3, provider_session 1.

### provider_session

- `provider_session` (rank 16): greedy rank 16; uses below artifact_owner 16, operation_binding 12, execution_runtime 12; above primary_graph 58, managed_run 18, authorization 3; used by primary_graph 661, managed_run 215, authorization 78.

### authorization

- `authorization` (rank 17): greedy rank 17; uses below provider_session 78, runtime_time 25, execution_runtime 16; above primary_graph 264; used by primary_graph 721, application_aftermath 9, provider_session 3.

### managed_run

- `managed_run` (rank 18): greedy rank 18; uses below provider_session 215, artifact_owner 57, request_execution 33; above nothing; used by convergence_epoch 119, primary_graph 36, provider_session 18.

### primary_graph

- `primary_graph` (rank 19): greedy rank 19; uses below request_execution 728, authorization 721, provider_session 661; above nothing; used by authorization 264, provider_session 58, application_aftermath 40.

### execution_resource_attempt

- `execution_resource_attempt` (rank 20): greedy rank 20; uses below provider_session 2, operation_binding 1, execution_runtime 1; above nothing; used by nothing.

## Units referenced against the stack

- 198 units: 89 movable, 109 behaviour-coupled, 0 pinned; 525 references
- 23 functions, constants and modules: 56 references

| unit | owner -> destination | refs | class | detail |
|---|---|---|---|---|
| `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` | primary_graph -> provider_session | 22 | behaviour-coupled | 3 impl(s) need items above; blocked by 32: `relational_read_kind`, `observe_field_value`, `actor_record_corruption`, `decode_actor_components` … |
| `primary_graph::application_query::resource_lifecycle::basis_registry::WorthQueryApplicationBasisIdentity` | primary_graph -> provider_session | 17 | movable | carries 1 |
| `primary_graph::authenticated_principal::WorthQueryAuthenticatedPrincipal` | primary_graph -> authorization | 11 | behaviour-coupled | 2 impl(s) need items above; blocked by 10: `relational_read_kind`, `heap_sort`, `union_visit`, `equal_cause` … |
| `managed_run::readmission::evidence::WorthQueryReadmissionEvidence` | managed_run -> convergence_epoch | 10 | movable | carries 1 |
| `primary_graph::application_attempt::workflow_transition_program::actor_selection::WorkflowRecordedActorObservation` | primary_graph -> authorization | 10 | behaviour-coupled | 2 impl(s) need items above; blocked by 21: `relational_read_kind`, `observe_field_value`, `AUTHOR_CHUNK`, `compare_bytes` … |
| `primary_graph::causal_inspection::application_inspection::target::WorthQueryApplicationInspectionTarget` | primary_graph -> provider_session | 9 | behaviour-coupled | 2 impl(s) need items above; blocked by 985: `request_lapse`, `compile_document_history_disclosure_contract`, `add_counters`, `bridge_dependency_count` … |
| `primary_graph::entity_resolution::resolved::WorthQueryApplicationEntityIdentity` | primary_graph -> authorization | 9 | behaviour-coupled | 1 impl(s) need items above; blocked by 3: `publication_execution_kind`, `scope_stop_kind`, `bytes` |
| `managed_run::recovery_registry::WorthQueryRecoveryResourceTerminal` | managed_run -> application_aftermath | 8 | movable |  |
| `primary_graph::application_attempt::precondition_binding::WorthQueryBoundMutationPreconditions` | primary_graph -> authorization | 8 | behaviour-coupled | 1 impl(s) need items above; blocked by 1141: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `primary_graph::application_attempt::workflow::actor::encoding::FundedActorText` | primary_graph -> authorization | 8 | movable |  |
| `managed_run::terminal::WorthQueryManagedRunTerminalKind` | managed_run -> convergence_epoch | 7 | movable |  |
| `primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact` | primary_graph -> authorization | 7 | behaviour-coupled | 4 impl(s) need items above; blocked by 1141: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `primary_graph::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding` | primary_graph -> application_aftermath | 7 | behaviour-coupled | 1 impl(s) need items above; blocked by 17: `append_identity_slot`, `append_scope_slot`, `encode_identity`, `append_identity_slot` … |
| `primary_graph::application_query::resource_lifecycle::basis_registry::WorthQueryApplicationBasisReleaseReceipt` | primary_graph -> provider_session | 7 | movable | carries 3 |
| `managed_run::direct::WorthQueryRunningDirectRun` | managed_run -> convergence_epoch | 6 | behaviour-coupled | 1 impl(s) need items above; blocked by 105: `demanded_field_slot`, `principal_currentness`, `NEXT_OPERATION_ADMISSION_IDENTITY`, `release_or_retain_queue` … |
| `primary_graph::application_attempt::elevation_approval_outcome::WorthQueryApprovedElevation` | primary_graph -> authorization | 6 | movable | carries 14 |
| `primary_graph::application_attempt::workflow::actor::ActorCodecFunding` | primary_graph -> authorization | 6 | movable | carries 7 |
| `primary_graph::provider::committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxObservation` | primary_graph -> application_aftermath | 6 | movable | carries 5 |
| `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryOperationAdmissionIdentity` | authorization -> operation_binding | 5 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `NEXT_OPERATION_ADMISSION_IDENTITY` |
| `managed_run::direct_graph_execution::WorthQueryActiveDirectGraphExecution` | managed_run -> convergence_epoch | 5 | behaviour-coupled | 2 impl(s) need items above; blocked by 35: `release_or_retain_queue`, `terminal_outcome`, `finish`, `cleanup_terminalized` … |
| `managed_run::direct_graph_execution::WorthQueryPausedDirectGraphExecution` | managed_run -> convergence_epoch | 5 | behaviour-coupled | 1 impl(s) need items above; blocked by 30: `cleanup_terminalized`, `cleanup_yielded_direct`, `recovery_checkpoint_release`, `yield_direct_run` … |
| `managed_run::direct_yield::WorthQueryYieldedDirectRun` | managed_run -> convergence_epoch | 5 | behaviour-coupled | 1 impl(s) need items above; blocked by 28: `cleanup_yielded_direct`, `recovery_checkpoint_release`, `classify_suspension_invocation`, `retain_suspended_checkpoint` … |
| `managed_run::workflow_graph_execution::WorthQueryActiveWorkflowGraphExecution` | managed_run -> convergence_epoch | 5 | behaviour-coupled | 2 impl(s) need items above; blocked by 39: `consumer_terminal_kind`, `producer_terminal_kind`, `classify_suspension_invocation`, `retain_suspended_checkpoint` … |
| `managed_run::workflow_graph_execution::step_outcome::WorthQueryPausedWorkflowGraphExecution` | managed_run -> convergence_epoch | 5 | behaviour-coupled | 1 impl(s) need items above; blocked by 34: `classify_suspension_invocation`, `retain_suspended_checkpoint`, `split_managed_execution`, `finish_restore` … |
| `managed_run::workflow_yield::WorthQueryYieldedWorkflowRun` | managed_run -> convergence_epoch | 5 | behaviour-coupled | 2 impl(s) need items above; blocked by 33: `classify_suspension_invocation`, `retain_suspended_checkpoint`, `split_managed_execution`, `finish_restore` … |
| `operation_binding::installed_domain_authority::WorthQueryInstalledDomainExecutionAuthority` | operation_binding -> artifact_owner | 5 | behaviour-coupled | 2 impl(s) need items above; blocked by 2: `NEXT_RUNTIME_AUTHORITY_IDENTITY`, `next_runtime_authority_ordinal` |
| `primary_graph::admitted_capture::retained_stage::RetainedStorageDenial` | primary_graph -> authorization | 5 | movable |  |
| `primary_graph::application_attempt::compare_and_commit::commit_receipt::WorthQueryApplicationCommitReceipt` | primary_graph -> application_aftermath | 5 | behaviour-coupled | 2 impl(s) need items above; blocked by 1263: `request_lapse`, `compile_document_history_disclosure_contract`, `prepare_pending_disclosure`, `authorization_denial` … |
| `primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease` | primary_graph -> provider_session | 5 | behaviour-coupled | 3 impl(s) need items above; blocked by 34: `relational_read_kind`, `observe_field_value`, `release_custody`, `actor_record_corruption` … |
| `primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime` | primary_graph -> product_publication | 5 | behaviour-coupled | 9 impl(s) need items above; blocked by 1361: `receipt_backing_bytes`, `provider_commit_identity`, `reserve`, `bind_dispatch_outbox_create_intent` … |
| `primary_graph::source_facts::allocation::SourceFactsAllocation` | primary_graph -> provider_session | 5 | behaviour-coupled | 1 impl(s) need items above; blocked by 7: `tree_clone_bytes`, `equal_cause`, `tree_insertion_bytes`, `derive` … |
| `managed_run::provider_work::request::WorthQueryManagedGraphCallRequest` | managed_run -> convergence_epoch | 4 | movable | carries 1 |
| `managed_run::recovery_registry::WorthQueryRecoveryHandleRegistry` | managed_run -> application_aftermath | 4 | movable | carries 5 |
| `managed_run::terminal::WorthQueryManagedRunCleanupDisposition` | managed_run -> convergence_epoch | 4 | movable |  |
| `primary_graph::admitted_capture::CaptureStop` | primary_graph -> authorization | 4 | movable |  |
| `primary_graph::application_attempt::elevation_request_outcome::WorthQueryRequestedElevation` | primary_graph -> authorization | 4 | movable | carries 11 |
| `primary_graph::application_entry::mutation::execution::MutationHandlerExecutionDenial` | primary_graph -> authorization | 4 | behaviour-coupled | 1 impl(s) need items above; blocked by 5: `relational_read_kind`, `native_decision_cause`, `equal_cause`, `native_index_decision_cause` … |
| `primary_graph::product_operation::shared_query_basis::SharedSelectedProductOperation` | primary_graph -> authorization | 4 | movable | carries 1 |
| `primary_graph::provider::branch_commit_coordination::WorthQueryApplicationBranchCommitCoordination` | primary_graph -> authorization | 4 | movable |  |
| `provider_session::graph_provider::read_material::WorthQueryGraphReadMaterial` | provider_session -> convergence_epoch | 4 | behaviour-coupled | 2 impl(s) need items above; blocked by 3: `pair_vector_bytes`, `retained_arc_str_bytes`, `value_by_key` |
| `provider_session::protocol::declared_closure::WorthQueryProviderPlanDeclarations` | provider_session -> operation_binding | 4 | behaviour-coupled | 1 impl(s) need items above; blocked by 7: `reversal_posture`, `bind_application_candidate_effect_closure`, `bind_native_reference_invariants`, `bind_direct_role_closure` … |
| `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation` | authorization -> application_aftermath | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 1337: `request_lapse`, `diagnostic_shape_supported`, `compile_document_history_disclosure_contract`, `diagnostic_rule` … |
| `authorization::operation_scope_binding::WorthQueryOperationScopeBinding` | authorization -> application_aftermath | 3 | movable | carries 2 |
| `managed_run::direct::WorthQueryAdmittedDirectRun` | managed_run -> convergence_epoch | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 1353: `receipt_backing_bytes`, `provider_commit_identity`, `reserve`, `bind_dispatch_outbox_create_intent` … |
| `managed_run::direct_terminal::owner::WorthQueryDirectRunTerminal` | managed_run -> convergence_epoch | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 13: `finish`, `as_usize`, `NEXT_EXECUTION_ATTEMPT`, `next_execution_attempt_ordinal` … |
| `managed_run::workflow::WorthQueryAdmittedWorkflowRun` | managed_run -> convergence_epoch | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 43: `consumer_terminal_kind`, `producer_terminal_kind`, `classify_suspension_invocation`, `retain_suspended_checkpoint` … |
| `managed_run::workflow::WorthQueryRunningWorkflowRun` | managed_run -> convergence_epoch | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 43: `consumer_terminal_kind`, `producer_terminal_kind`, `classify_suspension_invocation`, `retain_suspended_checkpoint` … |
| `managed_run::workflow::terminal::owner::WorthQueryWorkflowRunTerminal` | managed_run -> convergence_epoch | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 13: `as_usize`, `finish`, `NEXT_EXECUTION_ATTEMPT`, `next_execution_attempt_ordinal` … |
| `primary_graph::application_attempt::denial::WorthQueryApplicationAttemptDenial` | primary_graph -> authorization | 3 | behaviour-coupled | 4 impl(s) need items above; blocked by 3: `relational_read_kind`, `equal_cause`, `query_read_execution_kind` |
| `primary_graph::application_attempt::denial::kind::WorthQueryApplicationAttemptDenialKind` | primary_graph -> authorization | 3 | movable | carries 1 |
| `primary_graph::application_attempt::elevation_close_outcome::WorthQueryElevationClosureKind` | primary_graph -> authorization | 3 | movable |  |
| `primary_graph::application_attempt::idempotency_resolution::WorthQueryApplicationIdempotencyResolution` | primary_graph -> application_aftermath | 3 | movable | carries 1 |
| `primary_graph::application_attempt::idempotency_resolution::denial::WorthQueryApplicationIdempotencyResolutionDenial` | primary_graph -> authorization | 3 | movable | carries 3 |
| `primary_graph::application_attempt::provider_execution::application_attempt_affinity::WorthQueryApplicationAttemptBasis` | primary_graph -> provider_session | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 1201: `request_lapse`, `diagnostic_shape_supported`, `compile_document_history_disclosure_contract`, `diagnostic_rule` … |
| `primary_graph::application_attempt::read_set::handler_fact_boundary::CompletedHandlerFactBoundary` | primary_graph -> authorization | 3 | movable | carries 6 |
| `primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` | primary_graph -> authorization | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 93: `tree_clone_bytes`, `checkpoint_names`, `empty`, `pair` … |
| `primary_graph::application_output_demand::registry::required_context::RequiredOutputDemandContext` | primary_graph -> authorization | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 304: `with_checkpoint`, `with_resources`, `checkpoint`, `tree_clone_bytes` … |
| `primary_graph::application_query::managed_index::invocation::WorthQueryManagedIndexWorkDenial` | primary_graph -> provider_session | 3 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `equal_cause` |
| `primary_graph::application_query::observed_source::WorthQuerySourceExpectationKind` | primary_graph -> authorization | 3 | movable |  |
| `primary_graph::application_query::observed_source::query_context_binding::ObservedQueryExpressionContext` | primary_graph -> authorization | 3 | movable | carries 1 |
| `primary_graph::entity_resolution::product::issued_scope::WorthQueryIssuedSelectedScope` | primary_graph -> authorization | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 3: `publication_execution_kind`, `scope_stop_kind`, `bytes` |
| `primary_graph::entity_resolution::retained_selector::WorthQueryObservedScopeSelector` | primary_graph -> authorization | 3 | movable |  |
| `primary_graph::product_operation::correction_source::preparation::WorthQueryPreparedCorrectionSource` | primary_graph -> application_aftermath | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 483: `request_lapse`, `authorization_denial`, `field_locator`, `relation` … |
| `primary_graph::provider::application_decision_fact::WorthQueryPrimaryGraphApplicationDecisionFact` | primary_graph -> authorization | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 21: `relational_read_kind`, `array_storage_bytes`, `bytes`, `deep_clone_bytes` … |
| `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` | primary_graph -> product_publication | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 606: `receipt_backing_bytes`, `bind_dispatch_outbox_create_intent`, `NEXT_SECRET`, `demanded_field_slot` … |
| `primary_graph::resolution::WorthQueryPrincipalResolutionMode` | primary_graph -> authorization | 3 | movable |  |
| `primary_graph::resolution::selected_product::admitted::issued_principal::WorthQueryIssuedSelectedPrincipal` | primary_graph -> authorization | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 10: `relational_read_kind`, `heap_sort`, `union_visit`, `equal_cause` … |
| `primary_graph::schema_layout::principal_binding::WorthQueryPrimaryPrincipalBindingLayout` | primary_graph -> authorization | 3 | movable |  |
| `primary_graph::source_disclosure::inspection::OriginalSourceInspectionDisclosure` | primary_graph -> authorization | 3 | movable | carries 3 |
| `provider_session::graph_provider::receipt::WorthQueryBoundGraphExecutionReceipt` | provider_session -> convergence_epoch | 3 | behaviour-coupled | 2 impl(s) need items above; blocked by 4: `NEXT_GRAPH_CALL_AUTHORITY`, `pair_vector_bytes`, `retained_arc_str_bytes`, `value_by_key` |
| `authorization::denial::WorthQueryOperationAuthorizationDenialKind` | authorization -> application_aftermath | 2 | movable | carries 3 |
| `managed_run::direct_graph_chunk::WorthQueryPendingDirectGraphChunk` | managed_run -> convergence_epoch | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 33: `release_or_retain_queue`, `finish`, `cleanup_terminalized`, `cleanup_yielded_direct` … |
| `managed_run::direct_graph_execution::completion::WorthQueryCompletedDirectEvidenceOwner` | managed_run -> provider_session | 2 | movable |  |
| `managed_run::direct_terminal::cleanup::WorthQueryDirectRunCleanupFailure` | managed_run -> convergence_epoch | 2 | movable | carries 23 |
| `managed_run::direct_yield::WorthQueryDirectYieldDenied` | managed_run -> convergence_epoch | 2 | movable | carries 4 |
| `managed_run::direct_yield_recovery::WorthQueryDirectYieldRecoveryRequired` | managed_run -> convergence_epoch | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 10: `cleanup_terminalized`, `recovery_checkpoint_release`, `NEXT_EXECUTION_ATTEMPT`, `next_execution_attempt_ordinal` … |
| `managed_run::readmission::direct_outcome::WorthQueryDirectReadmissionDenied` | managed_run -> convergence_epoch | 2 | movable | carries 5 |
| `managed_run::readmission::workflow_outcome::WorthQueryWorkflowReadmissionDenied` | managed_run -> convergence_epoch | 2 | movable | carries 5 |
| `managed_run::recovery_registry::WorthQueryRecoveryRegistrySlot` | managed_run -> application_aftermath | 2 | movable |  |
| `managed_run::workflow::WorthQueryWorkflowRunStartRejection` | managed_run -> convergence_epoch | 2 | movable | carries 2 |
| `managed_run::workflow::run_affinity::WorthQueryWorkflowRunTransitionPermit` | managed_run -> provider_session | 2 | movable |  |
| `managed_run::workflow_graph_chunk::WorthQueryPendingWorkflowGraphChunk` | managed_run -> convergence_epoch | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 37: `consumer_terminal_kind`, `classify_suspension_invocation`, `retain_suspended_checkpoint`, `split_managed_execution` … |
| `managed_run::workflow_graph_execution::step_outcome::WorthQueryCompletedWorkflowEvidenceOwner` | managed_run -> provider_session | 2 | movable |  |
| `managed_run::workflow_yield::WorthQueryWorkflowYieldDenied` | managed_run -> convergence_epoch | 2 | movable | carries 4 |
| `managed_run::workflow_yield_recovery::WorthQueryWorkflowYieldRecoveryRequired` | managed_run -> convergence_epoch | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 8: `NEXT_EXECUTION_ATTEMPT`, `next_execution_attempt_ordinal`, `same_execution_basis`, `NEXT_GRAPH_CALL_AUTHORITY` … |
| `primary_graph::application_attempt::commit_authority_binding::WorthQueryRetainedGovernedInput` | primary_graph -> application_aftermath | 2 | movable | carries 1 |
| `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenialStage` | primary_graph -> application_aftermath | 2 | movable |  |
| `primary_graph::application_attempt::elevation_approval_outcome::WorthQueryElevationApprovalOutcome` | primary_graph -> authorization | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 10: `allocation_control`, `authority_execution`, `control_kind`, `execution_kind` … |
| `primary_graph::application_attempt::elevation_close_outcome::WorthQueryMandatoryReview` | primary_graph -> authorization | 2 | movable | carries 16 |
| `primary_graph::application_attempt::provider_execution::application_attempt_affinity::WorthQueryApplicationAttemptAffinity` | primary_graph -> provider_session | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 1229: `request_lapse`, `diagnostic_shape_supported`, `compile_document_history_disclosure_contract`, `diagnostic_rule` … |
| `primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotRelease` | primary_graph -> provider_session | 2 | movable | carries 1 |
| `primary_graph::application_attempt::workflow_transition_program::actor_context::WorthQueryWorkflowActorContext` | primary_graph -> authorization | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 1028: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `primary_graph::application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` | primary_graph -> authorization | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 176: `tree_clone_bytes`, `checkpoint_names`, `relational_read_kind`, `empty` … |
| `primary_graph::application_query::access_context::WorthQueryApplicationQueryAccessContext` | primary_graph -> authorization | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 13: `relational_read_kind`, `heap_sort`, `union_visit`, `publication_execution_kind` … |
| `primary_graph::application_query::resource_lifecycle::basis_registry::WorthQueryApplicationBasisSelectionIdentity` | primary_graph -> provider_session | 2 | movable |  |
| `primary_graph::entity_resolution::WorthQueryEntityResolutionTruth` | primary_graph -> authorization | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 8: `publication_execution_kind`, `scope_stop_kind`, `entity_denial`, `validate` … |
| `primary_graph::entity_resolution::resolved::WorthQueryResolvedEntity` | primary_graph -> authorization | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 3: `publication_execution_kind`, `scope_stop_kind`, `bytes` |
| `primary_graph::freshness::WorthQueryDurablePrincipalCurrentness` | primary_graph -> authorization | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 36: `relational_read_kind`, `observe_field_value`, `actor_record_corruption`, `decode_actor_components` … |
| `primary_graph::handler::decision_context_use::DecisionContextUse` | primary_graph -> authorization | 2 | movable |  |
| `primary_graph::invariant_projection::consumed_output::verification::capacity_stop::VerificationCapacityStop` | primary_graph -> provider_session | 2 | movable |  |
| `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` | primary_graph -> execution_runtime | 2 | behaviour-coupled | 3 impl(s) need items above; blocked by 317: `demanded_field_slot`, `request_lapse`, `authorization_denial`, `field_locator` … |
| `primary_graph::program_occurrence::original_meaning::OriginalProgramMeaning` | primary_graph -> authorization | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 1022: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `primary_graph::provider::application_attempt_state::registered_reads::RegisteredPrimaryRead` | primary_graph -> provider_session | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 60: `add_counters`, `bridge_dependency_count`, `bridge_matches`, `capture` … |
| `primary_graph::provider::session_commit::outbox_commit::WorthQueryCommittedDispatchOutboxBinding` | primary_graph -> application_aftermath | 2 | movable | carries 1 |
| `primary_graph::root::WorthQueryPrimaryGraphIntegrationHandle` | primary_graph -> execution_runtime | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 1279: `receipt_backing_bytes`, `provider_commit_identity`, `reserve`, `bind_dispatch_outbox_create_intent` … |
| `primary_graph::selected_read_indexes::PreparedSelectedReadIndexes` | primary_graph -> authorization | 2 | movable |  |
| `provider_session::graph_obligation::session_identity::WorthQueryGraphWorkManagedRunIdentity` | provider_session -> operation_binding | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `NEXT_GRAPH_WORK_MANAGED_RUN_IDENTITY` |
| `provider_session::graph_obligation::session_identity::WorthQueryGraphWorkSessionIdentity` | provider_session -> operation_binding | 2 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `NEXT_GRAPH_WORK_SESSION_IDENTITY` |
| `provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor` | provider_session -> convergence_epoch | 2 | behaviour-coupled | 2 impl(s) need items above; blocked by 1350: `receipt_backing_bytes`, `provider_commit_identity`, `reserve`, `bind_dispatch_outbox_create_intent` … |
| `provider_session::graph_provider::call_kind::WorthQueryGraphProviderCallKind` | provider_session -> operation_binding | 2 | movable |  |
| `provider_session::protocol::control_stopped::WorthQueryProviderSessionControlStopKind` | provider_session -> execution_runtime | 2 | movable |  |
| `authorization::denial::WorthQueryOperationAuthorizationDenial` | authorization -> application_aftermath | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 3: `relational_read_kind`, `equal_cause`, `query_read_execution_kind` |
| `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::capability_admission::preparation::observation::admitted_access::WorthQueryAdmittedApplicationCapabilityAccess` | authorization -> application_aftermath | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 194: `authorization_denial`, `field_locator`, `relation`, `compile_capability_plan` … |
| `managed_run::denial::WorthQueryManagedRunDenial` | managed_run -> convergence_epoch | 1 | movable | carries 2 |
| `managed_run::direct_admission::WorthQueryManagedRunAdmission` | managed_run -> execution_runtime | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1368: `receipt_backing_bytes`, `provider_commit_identity`, `reserve`, `bind_dispatch_outbox_create_intent` … |
| `managed_run::direct_graph_execution::WorthQueryDirectGraphStepOutcome` | managed_run -> convergence_epoch | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 31: `finish`, `cleanup_terminalized`, `cleanup_yielded_direct`, `recovery_checkpoint_release` … |
| `managed_run::direct_graph_execution::completion::WorthQueryCompletedDirectGraphExecution` | managed_run -> convergence_epoch | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 4: `NEXT_GRAPH_CALL_AUTHORITY`, `pair_vector_bytes`, `retained_arc_str_bytes`, `value_by_key` |
| `managed_run::direct_graph_execution_start::WorthQueryDirectGraphExecutionStartFailureKind` | managed_run -> convergence_epoch | 1 | movable | carries 1 |
| `managed_run::direct_terminal::inspection::WorthQueryDirectRunCleanupReceipt` | managed_run -> provider_session | 1 | movable | carries 11 |
| `managed_run::direct_yield::WorthQueryDirectYieldOutcome` | managed_run -> convergence_epoch | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 29: `cleanup_terminalized`, `cleanup_yielded_direct`, `recovery_checkpoint_release`, `classify_suspension_invocation` … |
| `managed_run::direct_yield_cleanup::WorthQueryDirectYieldCleanupOutcome` | managed_run -> convergence_epoch | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `recovery_checkpoint_release` |
| `managed_run::readmission::direct_lane::WorthQueryDirectReadmissionTransitionPermit` | managed_run -> provider_session | 1 | movable |  |
| `managed_run::readmission::direct_lane::preflight::preparation::progression::recovery::WorthQueryDirectReadmissionRecoveryRequired` | managed_run -> convergence_epoch | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 25: `classify_suspension_invocation`, `retain_suspended_checkpoint`, `split_managed_execution`, `finish_restore` … |
| `managed_run::readmission::direct_lane::preflight::preparation::progression::recovery::WorthQueryDirectReadmissionTerminalRecovery` | managed_run -> convergence_epoch | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 24: `classify_suspension_invocation`, `retain_suspended_checkpoint`, `split_managed_execution`, `finish_restore` … |
| `managed_run::readmission::direct_lane::preflight::preparation::progression::recovery::WorthQueryDirectReadmissionYieldReassemblyOutcome` | managed_run -> convergence_epoch | 1 | movable | carries 5 |
| `managed_run::readmission::direct_lane::preflight::preparation::progression::recovery::WorthQueryDirectReadmissionYieldReassemblyRecovery` | managed_run -> convergence_epoch | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 25: `classify_suspension_invocation`, `retain_suspended_checkpoint`, `split_managed_execution`, `finish_restore` … |
| `managed_run::readmission::direct_lane::preflight::preparation::progression::recovery::cleanup::WorthQueryDirectReadmissionCleanupOutcome` | managed_run -> convergence_epoch | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `finish_bridge_cleanup` |
| `managed_run::readmission::direct_lane::preflight::preparation::progression::recovery::cleanup::WorthQueryDirectReadmissionCleanupPending` | managed_run -> convergence_epoch | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `finish_bridge_cleanup` |
| `managed_run::readmission::direct_lane::preflight::preparation::progression::recovery::cleanup::WorthQueryDirectReadmissionCleanupRequired` | managed_run -> convergence_epoch | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 24: `classify_suspension_invocation`, `retain_suspended_checkpoint`, `split_managed_execution`, `finish_restore` … |
| `managed_run::readmission::direct_outcome::WorthQueryDirectReadmissionOutcome` | managed_run -> convergence_epoch | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 25: `classify_suspension_invocation`, `retain_suspended_checkpoint`, `split_managed_execution`, `finish_restore` … |
| `managed_run::readmission::workflow_outcome::WorthQueryWorkflowReadmissionOutcome` | managed_run -> convergence_epoch | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 26: `classify_suspension_invocation`, `retain_suspended_checkpoint`, `split_managed_execution`, `finish_restore` … |
| `managed_run::readmission::workflow_recovery::WorthQueryWorkflowReadmissionRecoveryRequired` | managed_run -> convergence_epoch | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 26: `classify_suspension_invocation`, `retain_suspended_checkpoint`, `split_managed_execution`, `finish_restore` … |
| `managed_run::readmission::workflow_recovery::WorthQueryWorkflowReadmissionTerminalRecovery` | managed_run -> convergence_epoch | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 26: `classify_suspension_invocation`, `retain_suspended_checkpoint`, `split_managed_execution`, `finish_restore` … |
| `managed_run::readmission::workflow_recovery::WorthQueryWorkflowReadmissionYieldReassemblyOutcome` | managed_run -> convergence_epoch | 1 | movable | carries 11 |
| `managed_run::readmission::workflow_recovery::WorthQueryWorkflowReadmissionYieldReassemblyRecovery` | managed_run -> convergence_epoch | 1 | movable | carries 8 |
| `managed_run::recovery_registry::WorthQueryRecoveryMintClaim` | managed_run -> application_aftermath | 1 | movable |  |
| `managed_run::terminal::WorthQueryManagedRunCleanupFailureKind` | managed_run -> provider_session | 1 | movable |  |
| `managed_run::workflow::provider_plan_admission::WorthQueryWorkflowProviderPlanPermit` | managed_run -> provider_session | 1 | movable |  |
| `managed_run::workflow::terminal::cleanup::WorthQueryWorkflowRunCleanupFailure` | managed_run -> convergence_epoch | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `finish` |
| `managed_run::workflow::terminal::cleanup::WorthQueryWorkflowRunCleanupOutcome` | managed_run -> convergence_epoch | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `finish` |
| `managed_run::workflow::terminal::cleanup::WorthQueryWorkflowRunCleanupPending` | managed_run -> convergence_epoch | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 1: `finish` |
| `managed_run::workflow_graph_execution::step_outcome::WorthQueryCompletedWorkflowGraphExecution` | managed_run -> convergence_epoch | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 4: `NEXT_GRAPH_CALL_AUTHORITY`, `pair_vector_bytes`, `retained_arc_str_bytes`, `value_by_key` |
| `managed_run::workflow_graph_execution::step_outcome::WorthQueryWorkflowGraphStepOutcome` | managed_run -> convergence_epoch | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 35: `classify_suspension_invocation`, `retain_suspended_checkpoint`, `split_managed_execution`, `finish_restore` … |
| `managed_run::workflow_graph_execution_start::WorthQueryWorkflowGraphExecutionStartFailureKind` | managed_run -> convergence_epoch | 1 | movable | carries 1 |
| `managed_run::workflow_yield::WorthQueryWorkflowYieldOutcome` | managed_run -> convergence_epoch | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 33: `classify_suspension_invocation`, `retain_suspended_checkpoint`, `split_managed_execution`, `finish_restore` … |
| `managed_run::workflow_yield_cleanup::WorthQueryWorkflowYieldCleanupOutcome` | managed_run -> convergence_epoch | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 9: `cleanup_without_checkpoint_owner`, `NEXT_EXECUTION_ATTEMPT`, `next_execution_attempt_ordinal`, `same_execution_basis` … |
| `managed_run::workflow_yield_cleanup::WorthQueryWorkflowYieldCleanupPending` | managed_run -> convergence_epoch | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 9: `cleanup_without_checkpoint_owner`, `NEXT_EXECUTION_ATTEMPT`, `next_execution_attempt_ordinal`, `same_execution_basis` … |
| `managed_run::workflow_yield_recovery::terminal_cleanup::WorthQueryWorkflowYieldRecoveryReleaseOutcome` | managed_run -> convergence_epoch | 1 | movable | carries 34 |
| `managed_run::workflow_yield_recovery::terminal_cleanup::WorthQueryWorkflowYieldRecoveryReleasePending` | managed_run -> convergence_epoch | 1 | movable | carries 2 |
| `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` | operation_binding -> artifact_owner | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 1475: `receipt_backing_bytes`, `provider_commit_identity`, `reserve`, `bind_dispatch_outbox_create_intent` … |
| `primary_graph::application_attempt::compare_and_commit::commit_outcome::WorthQueryApplicationCommitOutcome` | primary_graph -> authorization | 1 | behaviour-coupled | 3 impl(s) need items above; blocked by 1155: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenial` | primary_graph -> application_aftermath | 1 | behaviour-coupled | 3 impl(s) need items above; blocked by 10: `allocation_control`, `authority_execution`, `control_kind`, `execution_kind` … |
| `primary_graph::application_attempt::compare_and_commit::committed_publication::WorthQueryCommittedProductPublication` | primary_graph -> application_aftermath | 1 | movable | carries 2 |
| `primary_graph::application_attempt::elevation_approval_replay::ElevationApprovalReplayScopedOps` | primary_graph -> authorization | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 1155: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `primary_graph::application_attempt::external_dispatch_attempt::WorthQueryExternalDispatchAttemptOrdinal` | primary_graph -> application_aftermath | 1 | movable |  |
| `primary_graph::application_attempt::idempotency_resolution::IdempotencyResolutionScopedOps` | primary_graph -> authorization | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 988: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `primary_graph::application_attempt::precondition_binding::EmptyMutationPreconditionAdmissionStop` | primary_graph -> authorization | 1 | movable |  |
| `primary_graph::application_attempt::provider_binding::WorthQueryPreparedApplicationProviderAttempt` | primary_graph -> authorization | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 1023: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `primary_graph::application_attempt::provider_execution::external_dispatch::WorthQueryPerformedExternalRedispatchSeal` | primary_graph -> application_aftermath | 1 | movable | carries 1 |
| `primary_graph::application_attempt::provider_execution::external_dispatch::denial::WorthQueryExternalDispatchAttemptDenial` | primary_graph -> application_aftermath | 1 | movable |  |
| `primary_graph::application_attempt::provider_execution::outcome::WorthQueryProviderProgressionOutcome` | primary_graph -> authorization | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 10: `allocation_control`, `authority_execution`, `control_kind`, `execution_kind` … |
| `primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::registered::WorthQueryProviderAttemptRegistrationContext` | primary_graph -> authorization | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 1022: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `primary_graph::application_attempt::provider_execution::phase::prepared::running::progression::registered::WorthQueryRegisteredProviderAttempt` | primary_graph -> authorization | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 890: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `primary_graph::application_attempt::retained_decision_facts::control::StorageControl` | primary_graph -> provider_session | 1 | movable | carries 4 |
| `primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLeaseDenial` | primary_graph -> authorization | 1 | movable |  |
| `primary_graph::application_contribution::partitioned_computation::incremental::retained::SealedComputationRun` | primary_graph -> authorization | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 93: `tree_clone_bytes`, `checkpoint_names`, `empty`, `pair` … |
| `primary_graph::application_inbound::inbound_publication::outcome::WorthQueryPerformedInboundCompletion` | primary_graph -> application_aftermath | 1 | movable |  |
| `primary_graph::application_inbound::inbound_publication::outcome::WorthQueryUnpublishedInboundCompletion` | primary_graph -> application_aftermath | 1 | movable |  |
| `primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial` | primary_graph -> provider_session | 1 | behaviour-coupled | 3 impl(s) need items above; blocked by 28: `tree_clone_bytes`, `allocation_control`, `authority_execution`, `control_kind` … |
| `primary_graph::application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` | primary_graph -> provider_session | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 3: `is_temporarily_deferred`, `equal_cause`, `relational_execution_kind` |
| `primary_graph::application_output_demand::denial::recovery_release::WorthQueryProductRecoveryClose` | primary_graph -> product_publication | 1 | movable |  |
| `primary_graph::entity_resolution::WorthQueryInstalledEntityResolutionContext` | primary_graph -> authorization | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 40: `relational_read_kind`, `observe_field_value`, `actor_record_corruption`, `decode_actor_components` … |
| `primary_graph::entity_resolution_denial::WorthQueryEntityResolutionDenialKind` | primary_graph -> authorization | 1 | movable | carries 1 |
| `primary_graph::freshness::WorthQueryPrincipalFreshnessEvidence` | primary_graph -> authorization | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 10: `relational_read_kind`, `heap_sort`, `union_visit`, `equal_cause` … |
| `primary_graph::output_lineage::invalidation::head_cell::HeadCellRegistrationStop` | primary_graph -> execution_runtime | 1 | movable |  |
| `primary_graph::principal_currentness_capture::host::PrincipalCurrentnessCaptureHost` | primary_graph -> authorization | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 887: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `primary_graph::product_operation::generated_materialization::restoration::invariant_denial::WorthQueryGeneratedOutputInvariantAdmissionDenial` | primary_graph -> provider_session | 1 | movable |  |
| `primary_graph::product_operation::security_basis::WorthQueryProductObservationSource` | primary_graph -> authorization | 1 | movable |  |
| `primary_graph::product_operation::security_basis::WorthQueryProductSecurityBasis` | primary_graph -> authorization | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 1: `release` |
| `primary_graph::provider::application_attempt_state::commit_preparation::preimage_retention::selection::WorthQueryRetainedPreImageSeal` | primary_graph -> application_aftermath | 1 | movable | carries 1 |
| `primary_graph::provider::application_decision_fact::application_facts::CompletedApplicationFacts` | primary_graph -> authorization | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 56: `relational_read_kind`, `observe_field_value`, `release_custody`, `actor_record_corruption` … |
| `primary_graph::provider::application_decision_fact::application_table::ApplicationFactTable` | primary_graph -> authorization | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 55: `relational_read_kind`, `observe_field_value`, `release_custody`, `actor_record_corruption` … |
| `primary_graph::provider::committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxReadDenial` | primary_graph -> application_aftermath | 1 | movable | carries 2 |
| `primary_graph::provider::decision_facts::comparison::PrimaryDecisionReadSetComparison` | primary_graph -> provider_session | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 67: `add_counters`, `bridge_dependency_count`, `bridge_matches`, `capture` … |
| `primary_graph::provider::mutation_work::WorthQueryPrimaryMutationWorkEvidence` | primary_graph -> application_aftermath | 1 | movable | carries 7 |
| `primary_graph::resolution_denial::WorthQueryPrincipalResolutionDenial` | primary_graph -> authorization | 1 | movable | carries 2 |
| `primary_graph::resolution_denial::WorthQueryPrincipalResolutionDenialKind` | primary_graph -> authorization | 1 | movable | carries 1 |
| `primary_graph::root::WorthQueryPrimaryGraph` | primary_graph -> execution_runtime | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 198: `demanded_field_slot`, `authorization_denial`, `field_locator`, `relation` … |
| `primary_graph::runtime_host::WorthQueryRuntimeIdentityHost` | primary_graph -> authorization | 1 | behaviour-coupled | 1 impl(s) need items above; blocked by 887: `insert`, `lookup`, `remove`, `with_checkpoint` … |
| `primary_graph::source_disclosure::original::OriginalSourceDisclosure` | primary_graph -> application_aftermath | 1 | movable | carries 1 |
| `product_publication::unpublished::WorthQueryProductUnpublishedApplication` | product_publication -> execution_runtime | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 1334: `receipt_backing_bytes`, `provider_commit_identity`, `reserve`, `bind_dispatch_outbox_create_intent` … |
| `provider_session::graph_provider::receipt_association::WorthQueryBoundGraphExecutionAssociation` | provider_session -> domain_evidence_binding | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 1493: `receipt_backing_bytes`, `provider_commit_identity`, `reserve`, `bind_dispatch_outbox_create_intent` … |
| `provider_session::graph_provider::receipt_association::WorthQueryCompletedDomainEvidenceDerivation` | provider_session -> domain_evidence_binding | 1 | movable | carries 1 |
| `provider_session::graph_provider::work_report::WorthQueryProviderWorkReport` | provider_session -> convergence_epoch | 1 | movable |  |
| `provider_session::protocol::declared_closure::application::WorthQueryApplicationEffectPosture` | provider_session -> operation_binding | 1 | movable |  |
| `provider_session::protocol::denial::WorthQueryProviderSessionDenialKind` | provider_session -> application_aftermath | 1 | movable |  |
| `provider_session::protocol::execution_plan::WorthQueryValidatedProviderPlan` | provider_session -> operation_binding | 1 | movable |  |
| `provider_session::protocol::plan_contract::WorthQueryProviderExecutionPlanContract` | provider_session -> operation_binding | 1 | behaviour-coupled | 2 impl(s) need items above; blocked by 8: `NEXT_OPERATION_ADMISSION_IDENTITY`, `reversal_posture`, `bind_application_candidate_effect_closure`, `bind_native_reference_invariants` … |

### Types whose impls live across many modules

A unit moves with all its inherent impls, so these types cannot move, or be split into
another crate, until their impls are regrouped or turned into owned extension traits.

| type | owner | impl blocks | impl lines | modules hosting impls |
|---|---|---|---|---|
| `execution_runtime::runtime_root::WorthQueryExecutionRuntime` | execution_runtime | 9 | 392 | 5 |
| `execution_observation::WorthQueryExecutionObservation` | execution_observation | 4 | 62 | 3 |
| `execution_runtime::installation_authority::WorthQueryExecutionInstallationAuthority` | execution_runtime | 6 | 357 | 2 |
| `provider_session::protocol::terminal_binding::WorthQueryProviderSessionTerminalBinding` | provider_session | 6 | 129 | 2 |
| `request_execution::advancement::WorthQueryAdvancementDenial` | request_execution | 5 | 85 | 2 |
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
| `managed_run::workflow::terminal::cleanup::finish` | 3 | 5 |
| `primary_graph::provider::application_decision_fact::binding_failure` | 3 | 5 |
| `managed_run::interruption_classification::consumer_terminal_kind` | 2 | 4 |
| `managed_run::interruption_classification::producer_terminal_kind` | 2 | 10 |
| `managed_run::readmission::direct_lane::preflight::preparation::progression::recovery::cleanup::finish_bridge_cleanup` | 2 | 2 |
| `primary_graph::application_attempt::observation::observe_field_value` | 2 | 12 |
| `primary_graph::entity_resolution::entity_denial` | 2 | 3 |
| `primary_graph::fact_verification_observation::provider_application_fact` | 2 | 2 |
| `primary_graph::freshness::comparison_observation` | 2 | 3 |
| `primary_graph::index_maintenance_budget::cold_index_reconstruction_budget` | 2 | 7 |
| `primary_graph::index_maintenance_budget::refresh_with_cold_fallback` | 2 | 7 |
| `primary_graph::observations::observe_exact_principal_target` | 2 | 3 |
| `primary_graph::observations::principal_mapping::observe_mapping` | 2 | 3 |
| `primary_graph::provider::decision_facts::storage_read_set_failure` | 2 | 2 |
| `primary_graph::provider::relational_execution_denial::relational_execution_kind` | 2 | 2 |
| `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::NEXT_OPERATION_ADMISSION_IDENTITY` | 1 | 5 |
| `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::brand_custody::inconsistent` | 1 | 3 |
| `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::capability_admission::preparation::observation::admitted_access::exact_observation::with_exact_observation` | 1 | 1 |
| `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::capability_admission::preparation::observation::admitted_access::operation_progression::bind_capability_operation_preconditions` | 1 | 1 |
| `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::capability_admission::preparation::observation::admitted_access::operation_progression::transition_capability_operation` | 1 | 1 |
| `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::capability_admission::preparation::observation::admitted_access::operation_progression::validate_capability_operation_authority` | 1 | 1 |
| `managed_run::admission::validate_direct_run_head` | 1 | 1 |
| `managed_run::admission::validate_direct_run_lower` | 1 | 1 |
| `managed_run::admission::validate_workflow_run_head` | 1 | 1 |
| `managed_run::admission::validate_workflow_run_lower` | 1 | 1 |
| `managed_run::direct_graph_chunk::release_or_retain_queue` | 1 | 2 |
| `managed_run::direct_graph_execution_start::begin` | 1 | 6 |
| `managed_run::direct_graph_terminalization::terminal_outcome` | 1 | 5 |
| `managed_run::direct_terminal::cleanup::finish` | 1 | 3 |
| `managed_run::direct_yield_cleanup::cleanup_terminalized` | 1 | 2 |

### What holds the behaviour-coupled units (transitively)

Each non-type item above a destination that some coupled unit (directly or through a
coupled type it needs) depends on; inverting the top rows frees the most units.

| item | units held | their references |
|---|---|---|
| `primary_graph::application_query::managed_index::provider_failure::equal_cause` | 61 | 183 |
| `primary_graph::application_attempt::denial::request_read::relational_read_kind` | 54 | 164 |
| `primary_graph::provider::relational_execution_denial::query_read_execution_kind` | 54 | 164 |
| `provider_session::graph_provider::read_row::pair_vector_bytes` | 44 | 106 |
| `provider_session::graph_provider::read_row::retained_arc_str_bytes` | 44 | 106 |
| `provider_session::graph_provider::read_row::value_by_key` | 44 | 106 |
| `primary_graph::application_contribution::conditional::output_readiness_contract::denial` | 43 | 129 |
| `provider_session::graph_provider::call_identity::NEXT_GRAPH_CALL_AUTHORITY` | 43 | 102 |
| `primary_graph::provider::relational_execution_denial::relational_execution_kind` | 41 | 93 |
| `primary_graph::schema_layout::installation_primitives::planned_field_locator` | 41 | 125 |
| `primary_graph::application_attempt::observation::observe_field_value` | 40 | 122 |
| `primary_graph::application_attempt::workflow::actor::record::actor_record_corruption` | 40 | 122 |
| `primary_graph::application_attempt::workflow::actor::record::decode_actor_components` | 40 | 122 |
| `primary_graph::application_checkpoint::facts::payload::array_storage_bytes` | 40 | 98 |
| `primary_graph::application_checkpoint::facts::payload::chunk::bytes` | 40 | 98 |
| `primary_graph::application_checkpoint::facts::payload::chunk::deep_clone_bytes` | 40 | 98 |
| `primary_graph::application_checkpoint::facts::payload::concatenation::join` | 40 | 98 |
| `primary_graph::application_checkpoint::facts::payload::original_storage_bytes` | 40 | 98 |
| `primary_graph::application_checkpoint::facts::payload::patch::preparation::build` | 40 | 98 |
| `primary_graph::application_checkpoint::facts::payload::patch::preparation::count_nodes` | 40 | 98 |

### Functions, constants and modules referenced against the stack

| item | kind | owner | refs | from |
|---|---|---|---|---|
| `primary_graph::admitted_capture::checkpoint` | fn | primary_graph | 8 | authorization 8 |
| `primary_graph::application_attempt::workflow::actor::encoding::check_live` | fn | primary_graph | 5 | authorization 5 |
| `primary_graph::application_attempt::workflow::actor::fund_actor_codec` | fn | primary_graph | 5 | authorization 5 |
| `primary_graph::admitted_capture::retained_stage::run_retained_stage` | fn | primary_graph | 4 | authorization 4 |
| `primary_graph::application_attempt::fact::consult_observation` | module | primary_graph | 4 | authorization 4 |
| `primary_graph::application_attempt::workflow::actor::encoding::encode` | fn | primary_graph | 4 | authorization 4 |
| `primary_graph::application_contribution::execution_denial` | module | primary_graph | 4 | execution_runtime 2, product_publication 2 |
| `primary_graph::admitted_capture::storage::tree_clone_bytes` | fn | primary_graph | 3 | authorization 3 |
| `primary_graph::provider::relational_execution_denial::relational_execution_kind` | fn | primary_graph | 3 | authorization 3 |
| `primary_graph::application_attempt::precondition_binding::bind_mutation_preconditions` | fn | primary_graph | 2 | authorization 2 |
| `primary_graph::freshness::validate_freshness_at_snapshot` | fn | primary_graph | 2 | authorization 2 |
| `invalidation_equivalence_control::attach_constructed_receiver` | fn | invalidation_equivalence_control | 1 | execution_runtime 1 |
| `managed_run::readmission::workflow_progression::workflow_state::workflow_cleanup_owner` | module | managed_run | 1 | convergence_epoch 1 |
| `primary_graph::application_attempt::precondition_binding::bind_empty_mutation_preconditions_admitted` | fn | primary_graph | 1 | authorization 1 |
| `primary_graph::application_attempt::provider_execution::outcome::progression_denied` | fn | primary_graph | 1 | authorization 1 |
| `primary_graph::application_attempt::provider_execution::resource_request::application_resource_request` | fn | primary_graph | 1 | authorization 1 |
| `primary_graph::application_attempt::workflow::actor::storage_denial` | fn | primary_graph | 1 | authorization 1 |
| `primary_graph::application_contribution::partitioned_computation::incremental::published_state::observe_discarded` | fn | primary_graph | 1 | authorization 1 |
| `primary_graph::content::retention_work::checkpoint` | fn | primary_graph | 1 | application_aftermath 1 |
| `primary_graph::index_maintenance_budget::ordinary_index_maintenance_budget` | fn | primary_graph | 1 | authorization 1 |
| `primary_graph::principal_currentness_capture::capture_principal_currentness_admitted` | fn | primary_graph | 1 | authorization 1 |
| `primary_graph::publication_request_denial::publication` | fn | primary_graph | 1 | execution_runtime 1 |
| `primary_graph::source_facts::authorization_currentness::validate_source_principal_facts` | fn | primary_graph | 1 | authorization 1 |

## Ruling sheet: heaviest mutual pairs

### `authorization` ↔ `primary_graph`

- references: authorization -> primary_graph 264, primary_graph -> authorization 721
- against the stack (authorization -> primary_graph): 264 references; 30 movable units, 42 behaviour-coupled, 0 pinned, 18 functions/constants/modules
- crossing up: `WorthQueryPrimaryGraphLayout` (behaviour-coupled, 16), `WorthQueryAuthenticatedPrincipal` (behaviour-coupled, 11), `WorkflowRecordedActorObservation` (behaviour-coupled, 10), `WorthQueryApplicationEntityIdentity` (behaviour-coupled, 9), `WorthQueryBoundMutationPreconditions` (behaviour-coupled, 8), `FundedActorText` (movable, 8), `WorthQueryApplicationObservedFact` (behaviour-coupled, 7), `WorthQueryApprovedElevation` (movable, 6), `ActorCodecFunding` (movable, 6), `RetainedStorageDenial` (movable, 5), `WorthQueryApplicationInspectionTarget` (behaviour-coupled, 5), `CaptureStop` (movable, 4), `WorthQueryRequestedElevation` (movable, 4), `MutationHandlerExecutionDenial` (behaviour-coupled, 4) …
- crossing down (top): `WorthQueryAdmittedApplicationOperation` 105, `WorthQueryOperationAuthorizationDenial` 68, `WorthQueryOperationScopeEntityBinding` 64, `WorthQueryOperationAuthorizationDenialKind` 54, `WorthQueryApplicationQueryGovernance` 44, `WorthQueryAdmittedApplicationCapabilityAccess` 35, `WorthQueryOperationAuthorizationRefusalKind` 32, `WorthQueryPendingApplicationQueryGovernance` 27
- option nest: move 30608 lines of authorization under primary_graph; 15 references from other modules below primary_graph would then point up
- option move units: 30 units by script (86 references)
- option contract trait: a contract owned by authorization, implemented in primary_graph, 60 items (178 references)
- **default**: nest authorization under primary_graph (30608 lines); its other users then go through primary_graph

### `provider_session` ↔ `primary_graph`

- references: provider_session -> primary_graph 58, primary_graph -> provider_session 661
- against the stack (provider_session -> primary_graph): 58 references; 7 movable units, 12 behaviour-coupled, 0 pinned, 0 functions/constants/modules
- crossing up: `WorthQueryApplicationBasisIdentity` (movable, 16), `WorthQueryApplicationBasisReleaseReceipt` (movable, 7), `WorthQueryPrimaryGraphLayout` (behaviour-coupled, 6), `WorthQueryApplicationInspectionTarget` (behaviour-coupled, 4), `WorthQueryApplicationSnapshotLease` (behaviour-coupled, 3), `WorthQueryManagedIndexWorkDenial` (behaviour-coupled, 3), `WorthQueryApplicationAttemptAffinity` (behaviour-coupled, 2), `WorthQueryApplicationAttemptBasis` (behaviour-coupled, 2), `WorthQueryApplicationSnapshotRelease` (movable, 2), `WorthQueryApplicationBasisSelectionIdentity` (movable, 2), `VerificationCapacityStop` (movable, 2), `RegisteredPrimaryRead` (behaviour-coupled, 2), `StorageControl` (movable, 1), `WorthQueryOutputDemandDenial` (behaviour-coupled, 1) …
- crossing down (top): `WorthQueryProviderSessionControlStopKind` 77, `WorthQueryProviderSessionDenialKind` 64, `WorthQueryManagedGraphWorkSession` 42, `WorthQueryApplicationEffectPosture` 30, `WorthQueryProviderSessionFailure` 29, `WorthQueryGraphWorkSessionIdentity` 23, `WorthQueryProviderSessionProtocolStage` 19, `WorthQueryProviderSessionCommitStop` 16
- option nest: move 16925 lines of provider_session under primary_graph; 321 references from other modules below primary_graph would then point up
- option move units: 7 units by script (31 references)
- option contract trait: a contract owned by provider_session, implemented in primary_graph, 12 items (27 references)
- **default**: move the 7 movable units by script; a contract owned by provider_session for the 12 items that stay behaviour-bound in primary_graph

### `application_aftermath` ↔ `primary_graph`

- references: application_aftermath -> primary_graph 40, primary_graph -> application_aftermath 283
- against the stack (application_aftermath -> primary_graph): 40 references; 15 movable units, 5 behaviour-coupled, 0 pinned, 1 functions/constants/modules
- crossing up: `WorthQueryCommittedDispatchOutboxObservation` (movable, 6), `WorthQueryApplicationCommitReceipt` (behaviour-coupled, 4), `WorthQueryApplicationIdempotencyBinding` (behaviour-coupled, 4), `WorthQueryPrimaryGraphApplicationRuntime` (behaviour-coupled, 4), `WorthQueryPreparedCorrectionSource` (behaviour-coupled, 3), `WorthQueryRetainedGovernedInput` (movable, 2), `WorthQueryApplicationIdempotencyResolution` (movable, 2), `WorthQueryCommittedDispatchOutboxBinding` (movable, 2), `WorthQueryApplicationCommitDenial` (behaviour-coupled, 1), `WorthQueryApplicationCommitDenialStage` (movable, 1), `WorthQueryCommittedProductPublication` (movable, 1), `WorthQueryExternalDispatchAttemptOrdinal` (movable, 1), `WorthQueryPerformedExternalRedispatchSeal` (movable, 1), `WorthQueryExternalDispatchAttemptDenial` (movable, 1) …
- crossing down (top): `ExternalEffectCorrelationIdentity` 23, `WorthQueryPendingAftermathCausality` 22, `WorthQueryDispatchOutboxRecord` 19, `ApplicationCommitCausality` 18, `WorthQueryRetainedPreImage` 13, `WorthQueryAcceptedInboundOccurrence` 11, `WorthQueryCommittedAftermathCausality` 10, `WorthQueryDispatchOutboxLayout` 9
- option nest: move 6048 lines of application_aftermath under primary_graph; 4 references from other modules below primary_graph would then point up
- option move units: 15 units by script (23 references)
- option contract trait: a contract owned by application_aftermath, implemented in primary_graph, 6 items (17 references)
- **default**: move the 15 movable units by script; a contract owned by application_aftermath for the 6 items that stay behaviour-bound in primary_graph

### `execution_runtime` ↔ `primary_graph`

- references: execution_runtime -> primary_graph 8, primary_graph -> execution_runtime 290
- against the stack (execution_runtime -> primary_graph): 8 references; 1 movable units, 3 behaviour-coupled, 0 pinned, 2 functions/constants/modules
- crossing up: `SourceInvalidationOwner` (behaviour-coupled, 2), `HeadCellRegistrationStop` (movable, 1), `WorthQueryPrimaryGraph` (behaviour-coupled, 1), `WorthQueryPrimaryGraphIntegrationHandle` (behaviour-coupled, 1), `execution_denial` (module, 2), `publication` (fn, 1)
- crossing down (top): `WorthQueryRuntimeAuthorityIdentity` 40, `ApplicationAdvancementOps` 27, `WorthQueryOutputDemandLimits` 18, `WorthQueryProductPublicationBinding` 18, `OwnerCleanupRetirements` 16, `WorthQueryProductPublicationDenial` 16, `WorthQueryExecutionRuntime` 16, `WorthQueryProductPublicationOutcome` 16
- option nest: move 5943 lines of execution_runtime under primary_graph; 82 references from other modules below primary_graph would then point up
- option move units: 1 units by script (1 references)
- option contract trait: a contract owned by execution_runtime, implemented in primary_graph, 5 items (7 references)
- **default**: move the 1 movable units by script; a contract owned by execution_runtime for the 5 items that stay behaviour-bound in primary_graph

### `provider_session` ↔ `managed_run`

- references: provider_session -> managed_run 18, managed_run -> provider_session 215
- against the stack (provider_session -> managed_run): 18 references; 9 movable units, 2 behaviour-coupled, 0 pinned, 0 functions/constants/modules
- crossing up: `WorthQueryRunningDirectRun` (behaviour-coupled, 4), `WorthQueryCompletedDirectEvidenceOwner` (movable, 2), `WorthQueryManagedRunTerminalKind` (movable, 2), `WorthQueryWorkflowRunTransitionPermit` (movable, 2), `WorthQueryCompletedWorkflowEvidenceOwner` (movable, 2), `WorthQueryDirectRunCleanupFailure` (movable, 1), `WorthQueryDirectRunCleanupReceipt` (movable, 1), `WorthQueryDirectReadmissionTransitionPermit` (movable, 1), `WorthQueryManagedRunCleanupFailureKind` (movable, 1), `WorthQueryRunningWorkflowRun` (behaviour-coupled, 1), `WorthQueryWorkflowProviderPlanPermit` (movable, 1)
- crossing down (top): `WorthQueryProviderCheckpointReleaseEvidence` 17, `WorthQueryProviderExecutionReleaseEvidence` 17, `WorthQueryGraphProviderCall` 10, `WorthQueryExecutionResourceAttemptEvidence` 8, `WorthQueryGraphProviderStepArtifactContext` 7, `WorthQueryGraphProviderMemoryArena` 7, `WorthQueryWorkflowExecutionResourceAttempt` 7, `WorthQueryExecutionProviderSession` 7
- option nest: move 16925 lines of provider_session under managed_run; 106 references from other modules below managed_run would then point up
- option move units: 9 units by script (13 references)
- option contract trait: a contract owned by provider_session, implemented in managed_run, 2 items (5 references)
- **default**: move the 9 movable units by script; a contract owned by provider_session for the 2 items that stay behaviour-bound in managed_run

### `product_publication` ↔ `primary_graph`

- references: product_publication -> primary_graph 7, primary_graph -> product_publication 94
- against the stack (product_publication -> primary_graph): 7 references; 1 movable units, 2 behaviour-coupled, 0 pinned, 1 functions/constants/modules
- crossing up: `WorthQueryUnpublishedIdempotencyDisposition` (behaviour-coupled, 3), `WorthQueryProductRecoveryClose` (movable, 1), `WorthQueryPrimaryGraphApplicationRuntime` (behaviour-coupled, 1), `execution_denial` (module, 2)
- crossing down (top): `WorthQueryProductUnpublishedRecovery` 24, `WorthQueryProductUnpublishedApplication` 21, `WorthQueryProductUnpublishedRecoveryReleaseFailure` 15, `WorthQueryProductStaleApplication` 12, `DiscoveryScopedOps` 6, `ProductPublicationHost` 5, `WorthQueryProductUnpublishedRecoveryReleaseDenial` 4, `DiscoveryOps` 3
- option nest: move 700 lines of product_publication under primary_graph; 8 references from other modules below primary_graph would then point up
- option move units: 1 units by script (1 references)
- option contract trait: a contract owned by product_publication, implemented in primary_graph, 3 items (6 references)
- **default**: move the 1 movable units by script; a contract owned by product_publication for the 3 items that stay behaviour-bound in primary_graph

### `provider_session` ↔ `authorization`

- references: provider_session -> authorization 3, authorization -> provider_session 78
- against the stack (provider_session -> authorization): 3 references; 1 movable units, 1 behaviour-coupled, 0 pinned, 0 functions/constants/modules
- crossing up: `WorthQueryOperationAdmissionIdentity` (behaviour-coupled, 2), `WorthQueryOperationAuthorizationDenialKind` (movable, 1)
- crossing down (top): `WorthQueryGraphWorkSessionIdentity` 31, `WorthQueryManagedGraphWorkSession` 23, `WorthQueryGraphWorkManagedRunIdentity` 7, `WorthQueryProviderSessionControlStopKind` 5, `WorthQueryGraphWorkAccessContextAffinity` 5, `WorthQueryProviderSessionDenialKind` 3, `WorthQuerySessionBoundReadsAndEffects` 1, `WorthQueryDecisionReadSetDenialKind` 1
- option nest: move 16925 lines of provider_session under authorization; 28 references from other modules below authorization would then point up
- option move units: 1 units by script (1 references)
- option contract trait: a contract owned by provider_session, implemented in authorization, 1 items (2 references)
- **default**: move the 1 movable units by script; a contract owned by provider_session for the 1 items that stay behaviour-bound in authorization

### `operation_binding` ↔ `provider_session`

- references: operation_binding -> provider_session 13, provider_session -> operation_binding 12
- against the stack (operation_binding -> provider_session): 13 references; 3 movable units, 4 behaviour-coupled, 0 pinned, 0 functions/constants/modules
- crossing up: `WorthQueryProviderPlanDeclarations` (behaviour-coupled, 4), `WorthQueryGraphWorkManagedRunIdentity` (behaviour-coupled, 2), `WorthQueryGraphWorkSessionIdentity` (behaviour-coupled, 2), `WorthQueryGraphProviderCallKind` (movable, 2), `WorthQueryApplicationEffectPosture` (movable, 1), `WorthQueryValidatedProviderPlan` (movable, 1), `WorthQueryProviderExecutionPlanContract` (behaviour-coupled, 1)
- crossing down (top): `WorthQueryExecutionBoundOperationAuthority` 12
- option nest: move 1752 lines of operation_binding under provider_session; 9 references from other modules below provider_session would then point up
- option move units: 3 units by script (4 references)
- option contract trait: a contract owned by operation_binding, implemented in provider_session, 4 items (9 references)
- **default**: move the 3 movable units by script; a contract owned by operation_binding for the 4 items that stay behaviour-bound in provider_session

### `execution_runtime` ↔ `managed_run`

- references: execution_runtime -> managed_run 1, managed_run -> execution_runtime 13
- against the stack (execution_runtime -> managed_run): 1 references; 0 movable units, 1 behaviour-coupled, 0 pinned, 0 functions/constants/modules
- crossing up: `WorthQueryManagedRunAdmission` (behaviour-coupled, 1)
- crossing down (top): `WorthQueryExecutionRuntime` 12, `WorthQueryRuntimeAuthorityIdentity` 1
- option nest: move 5943 lines of execution_runtime under managed_run; 69 references from other modules below managed_run would then point up
- option move units: 0 units by script (0 references)
- option contract trait: a contract owned by execution_runtime, implemented in managed_run, 1 items (1 references)
- **default**: move the 0 movable units by script; a contract owned by execution_runtime for the 1 items that stay behaviour-bound in managed_run

### `execution_runtime` ↔ `provider_session`

- references: execution_runtime -> provider_session 1, provider_session -> execution_runtime 12
- against the stack (execution_runtime -> provider_session): 1 references; 1 movable units, 0 behaviour-coupled, 0 pinned, 0 functions/constants/modules
- crossing up: `WorthQueryProviderSessionControlStopKind` (movable, 1)
- crossing down (top): `WorthQueryRuntimeAuthorityIdentity` 10, `WorthQueryProductRuntime` 1, `WorthQueryRelationalSourceOwner` 1
- option nest: move 5943 lines of execution_runtime under provider_session; 41 references from other modules below provider_session would then point up
- option move units: 1 units by script (1 references)
- option contract trait: a contract owned by execution_runtime, implemented in provider_session, 0 items (0 references)
- **default**: move the 1 movable units down by script

## Estimate

- defaults for the pairs above: largest component 8, 168 references against the stack; 38 units by script, 34 items by hand, 1 nests
- defaults plus every movable unit moved: largest component 8, 103 references against the stack; 89 units by script, 135 items by hand
- and the five widest hub types (`WorthQueryExecutionObservation`, `WorthQueryExecutionInstallationAuthority`, `WorthQueryExecutionRuntime`, `WorthQueryProviderSessionTerminalBinding`, `WorthQueryAdvancementDenial`) behind contracts: largest component 8, 103 references against the stack

