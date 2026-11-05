# Vocabulary simulation: `domain_computation`

Before: component 12, 396 references against the layering.
After: component 12, 396 against a fresh layering; 101 items moved over 1 rounds [101].

## Vocabulary modules

| module | items | lines | kinds | top source modules |
|---|---|---|---|---|
| `vocabulary:vocabulary` | 101 | 2976 | struct 46, enum 40, fn 10, static 4, const 1 | `vocabulary::provider_session::session_identity` 4, `vocabulary::application_aftermath::outbox` 3, `vocabulary::authorization::operation_admission` 3, `vocabulary::execution_runtime::runtime_identity` 3, `vocabulary::managed_run::evidence` 3 |

Cycles among vocabulary modules: none

Vocabulary modules naming modules above them (should be none): none

## Remaining seams

- `authorization` -> `primary_graph`: 198 refs in 67 items: `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout`, `primary_graph::authenticated_principal::WorthQueryAuthenticatedPrincipal`, `primary_graph::application_attempt::workflow_transition_program::actor_selection::WorkflowRecordedActorObservation`, `primary_graph::entity_resolution::resolved::WorthQueryApplicationEntityIdentity`
  - `primary_graph::application_attempt::workflow_transition_program::actor_selection::WorkflowRecordedActorObservation` held by `primary_graph::application_attempt::workflow_transition_program::actor_selection::WorkflowRecordedActorObservation` (not vocabulary: struct)
  - `primary_graph::authenticated_principal::WorthQueryAuthenticatedPrincipal` held by `primary_graph::authenticated_principal::WorthQueryAuthenticatedPrincipal` (not vocabulary: struct)
  - `primary_graph::application_attempt::workflow::actor::encoding::encode` held by `authorization::retained_capability_request::portable::WorkflowApprovalRequestEncodingDenial` (body)
- `application_aftermath` -> `primary_graph`: 26 refs in 12 items: `primary_graph::provider::committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxObservation`, `primary_graph::application_attempt::compare_and_commit::commit_receipt::WorthQueryApplicationCommitReceipt`, `primary_graph::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding`, `primary_graph::product_operation::correction_source::preparation::WorthQueryPreparedCorrectionSource`
  - `primary_graph::application_attempt::compare_and_commit::commit_receipt::WorthQueryApplicationCommitReceipt` held by `primary_graph::application_attempt::compare_and_commit::commit_receipt::WorthQueryApplicationCommitReceipt` (not vocabulary: struct)
  - `primary_graph::product_operation::correction_source::preparation::WorthQueryPreparedCorrectionSource` held by `primary_graph::product_operation::correction_source::preparation::WorthQueryPreparedCorrectionSource` (not vocabulary: struct)
  - `primary_graph::provider::committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxObservation` held by `primary_graph::provider::committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxObservation` (not vocabulary: struct)
- `provider_session` -> `primary_graph`: 25 refs in 12 items: `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout`, `primary_graph::causal_inspection::application_inspection::target::WorthQueryApplicationInspectionTarget`, `primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease`, `primary_graph::application_attempt::provider_execution::application_attempt_affinity::WorthQueryApplicationAttemptBasis`
  - `primary_graph::root::WorthQueryPrimaryGraphIntegrationHandle` held by `primary_graph::root::WorthQueryPrimaryGraphIntegrationHandle` (not vocabulary: struct)
  - `primary_graph::application_attempt::retained_decision_facts::control::StorageControl` held by `primary_graph::application_attempt::retained_decision_facts::control::StorageControl` (not vocabulary: struct)
  - `primary_graph::application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` held by `primary_graph::application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` (not vocabulary: enum)
- `provider_session` -> `managed_run`: 10 refs in 5 items: `managed_run::direct::WorthQueryRunningDirectRun`, `managed_run::direct_graph_execution::completion::WorthQueryCompletedDirectEvidenceOwner`, `managed_run::workflow_graph_execution::step_outcome::WorthQueryCompletedWorkflowEvidenceOwner`, `managed_run::direct_terminal::cleanup::WorthQueryDirectRunCleanupFailure`
  - `managed_run::direct::WorthQueryRunningDirectRun` held by `managed_run::direct::WorthQueryRunningDirectRun` (not vocabulary: struct)
  - `managed_run::direct_graph_execution::completion::WorthQueryCompletedDirectEvidenceOwner` held by `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` (declaration)
  - `managed_run::direct_graph_execution::completion::WorthQueryCompletedDirectEvidenceOwner` held by `provider_session::graph_provider::receipt::WorthQueryBoundGraphExecutionReceipt` (declaration)
- `execution_runtime` -> `primary_graph`: 5 refs in 4 items: `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner`, `primary_graph::output_lineage::invalidation::head_cell::HeadCellRegistrationStop`, `primary_graph::root::WorthQueryPrimaryGraph`, `primary_graph::root::WorthQueryPrimaryGraphIntegrationHandle`
  - `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` held by `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` (not vocabulary: struct)
  - `primary_graph::output_lineage::invalidation::head_cell::HeadCellRegistrationStop` held by `primary_graph::output_lineage::invalidation::head_cell::HeadCellRegistrationStop` (not vocabulary: enum)
  - `primary_graph::root::WorthQueryPrimaryGraph` held by `primary_graph::root::WorthQueryPrimaryGraph` (not vocabulary: struct)
- `product_publication` -> `primary_graph`: 4 refs in 2 items: `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition`, `primary_graph::application_output_demand::denial::recovery_release::WorthQueryProductRecoveryClose`
  - `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` held by `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyStore` (declaration)
  - `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` held by `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyStore` (impl WorthQueryUnpublishedIdempotencyDisposition)
  - `primary_graph::application_output_demand::denial::recovery_release::WorthQueryProductRecoveryClose` held by `primary_graph::application_output_demand::denial::recovery_release::WorthQueryProductRecoveryClose` (not vocabulary: trait)
- `provider_session` -> `convergence_epoch`: 2 refs in 1 items: `convergence_epoch::domain_provider::WorthQueryConvergenceDomainProvider`
  - `convergence_epoch::domain_provider::WorthQueryConvergenceDomainProvider` held by `convergence_epoch::domain_provider::WorthQueryConvergenceDomainProvider` (not vocabulary: trait)
- `domain_evidence_binding` -> `provider_session`: 2 refs in 2 items: `provider_session::graph_provider::receipt_association::WorthQueryBoundGraphExecutionAssociation`, `provider_session::graph_provider::receipt_association::WorthQueryCompletedDomainEvidenceDerivation`
  - `provider_session::graph_provider::receipt_association::WorthQueryBoundGraphExecutionAssociation` held by `provider_session::graph_provider::receipt_association::WorthQueryBoundGraphExecutionAssociation` (not vocabulary: struct)
  - `provider_session::graph_provider::receipt_association::WorthQueryCompletedDomainEvidenceDerivation` held by `provider_session::graph_provider::receipt_association::WorthQueryBoundGraphExecutionAssociation` (declaration)
  - `provider_session::graph_provider::receipt_association::WorthQueryCompletedDomainEvidenceDerivation` held by `provider_session::graph_provider::receipt_association::WorthQueryBoundGraphExecutionAssociation` (impl WorthQueryCompletedDomainEvidenceDerivation)
- `operation_binding` -> `provider_session`: 2 refs in 2 items: `provider_session::protocol::execution_plan::WorthQueryValidatedProviderPlan`, `provider_session::protocol::plan_contract::WorthQueryProviderExecutionPlanContract`
  - `provider_session::protocol::execution_plan::WorthQueryValidatedProviderPlan` held by `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` (declaration)
  - `provider_session::protocol::execution_plan::WorthQueryValidatedProviderPlan` held by `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` (impl WorthQueryValidatedProviderPlan<'_>)
  - `provider_session::protocol::plan_contract::WorthQueryProviderExecutionPlanContract` held by `provider_session::protocol::execution_plan::WorthQueryValidatedProviderPlan` (impl WorthQueryProviderExecutionPlanContract)
- `artifact_owner` -> `operation_binding`: 1 refs in 1 items: `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority`
  - `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` held by `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` (not vocabulary: struct)
- `execution_runtime` -> `product_publication`: 1 refs in 1 items: `product_publication::unpublished::WorthQueryProductUnpublishedApplication`
  - `product_publication::unpublished::WorthQueryProductUnpublishedApplication` held by `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` (declaration)
  - `product_publication::unpublished::WorthQueryProductUnpublishedApplication` held by `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` (impl WorthQueryProductUnpublishedApplication)
- `execution_runtime` -> `managed_run`: 1 refs in 1 items: `managed_run::direct_admission::WorthQueryManagedRunAdmission`
  - `managed_run::direct_admission::WorthQueryManagedRunAdmission` held by `execution_runtime::runtime_root::WorthQueryExecutionRuntime` (declaration)
  - `managed_run::direct_admission::WorthQueryManagedRunAdmission` held by `managed_run::admission::validate_direct_run_head` (impl WorthQueryManagedRunAdmission<'_>)
- `execution_runtime` -> `invalidation_equivalence_control`: 1 refs in 1 items: `invalidation_equivalence_control::attach_constructed_receiver`
  - `invalidation_equivalence_control::attach_constructed_receiver` held by `invalidation_equivalence_control::attach_constructed_receiver` (not vocabulary: fn)
