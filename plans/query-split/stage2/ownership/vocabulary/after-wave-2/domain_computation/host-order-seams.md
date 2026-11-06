# Vocabulary simulation: `domain_computation`

Before: component 11, 234 references against the layering.
After: component 11, 229 against a fresh layering; 6 items moved over 1 rounds [6].

## Vocabulary modules

| module | items | lines | kinds | top source modules |
|---|---|---|---|---|
| `vocabulary:vocabulary` | 4 | 136 | enum 4 | `vocabulary::request_execution::execution_refusal` 2, `vocabulary::provider_session::control_stopped` 1, `vocabulary::provider_session::protocol_denial` 1 |
| `vocabulary:primary_graph` | 2 | 116 | enum 1, fn 1 | `primary_graph::vocabulary::application_attempt::denial` 1, `primary_graph::vocabulary::provider::relational_execution_denial` 1 |

Cycles among vocabulary modules: none

Vocabulary modules naming modules above them (should be none): none

## Remaining seams

- `authorization` -> `primary_graph`: 158 refs in 63 items: `primary_graph::application_attempt::workflow_transition_program::actor_selection::WorkflowRecordedActorObservation`, `primary_graph::application_attempt::precondition_binding::WorthQueryBoundMutationPreconditions`, `primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact`, `primary_graph::application_attempt::elevation_approval_outcome::WorthQueryApprovedElevation`
  - `primary_graph::authenticated_principal::WorthQueryAuthenticatedPrincipal` held by `primary_graph::authenticated_principal::WorthQueryAuthenticatedPrincipal` (not vocabulary: struct)
  - `primary_graph::product_operation::shared_query_basis::SharedSelectedProductOperation` held by `primary_graph::product_operation::shared_query_basis::SharedSelectedProductOperation` (not vocabulary: struct)
  - `primary_graph::resolution::selected_product::admitted::issued_principal::WorthQueryIssuedSelectedPrincipal` held by `primary_graph::authenticated_principal::WorthQueryAuthenticatedPrincipal` (declaration)
- `application_aftermath` -> `primary_graph`: 25 refs in 11 items: `primary_graph::provider::committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxObservation`, `primary_graph::vocabulary::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding`, `primary_graph::application_attempt::compare_and_commit::commit_receipt::WorthQueryApplicationCommitReceipt`, `primary_graph::product_operation::correction_source::preparation::WorthQueryPreparedCorrectionSource`
  - `primary_graph::provider::mutation_work::WorthQueryPrimaryMutationWorkEvidence` held by `primary_graph::provider::mutation_work::WorthQueryPrimaryMutationWorkEvidence` (not vocabulary: struct)
  - `primary_graph::provider::committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxObservation` held by `primary_graph::provider::committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxObservation` (not vocabulary: struct)
  - `primary_graph::vocabulary::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding` held by `primary_graph::vocabulary::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding` (not vocabulary: struct)
- `provider_session` -> `primary_graph`: 19 refs in 11 items: `primary_graph::causal_inspection::application_inspection::target::WorthQueryApplicationInspectionTarget`, `primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease`, `primary_graph::application_attempt::provider_execution::application_attempt_affinity::WorthQueryApplicationAttemptAffinity`, `primary_graph::provider::application_attempt_state::registered_reads::RegisteredPrimaryRead`
  - `primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial` held by `primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial` (not vocabulary: struct)
  - `primary_graph::causal_inspection::application_inspection::target::WorthQueryApplicationInspectionTarget` held by `primary_graph::causal_inspection::application_inspection::target::InspectionOriginal` (declaration)
  - `primary_graph::causal_inspection::application_inspection::target::WorthQueryApplicationInspectionTarget` held by `primary_graph::causal_inspection::application_inspection::source_original::SourceOriginal` (impl WorthQueryApplicationInspectionTarget)
- `provider_session` -> `managed_run`: 10 refs in 5 items: `managed_run::direct::WorthQueryRunningDirectRun`, `managed_run::workflow_graph_execution::step_outcome::WorthQueryCompletedWorkflowEvidenceOwner`, `managed_run::direct_graph_execution::completion::WorthQueryCompletedDirectEvidenceOwner`, `managed_run::workflow::WorthQueryRunningWorkflowRun`
  - `managed_run::direct::WorthQueryRunningDirectRun` held by `managed_run::direct::WorthQueryRunningDirectRun` (not vocabulary: struct)
  - `managed_run::workflow_graph_execution::step_outcome::WorthQueryCompletedWorkflowEvidenceOwner` held by `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` (declaration)
  - `managed_run::workflow_graph_execution::step_outcome::WorthQueryCompletedWorkflowEvidenceOwner` held by `provider_session::graph_provider::receipt::WorthQueryBoundGraphExecutionReceipt` (declaration)
- `execution_runtime` -> `primary_graph`: 5 refs in 4 items: `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner`, `primary_graph::root::WorthQueryPrimaryGraph`, `primary_graph::output_lineage::invalidation::head_cell::HeadCellRegistrationStop`, `primary_graph::root::WorthQueryPrimaryGraphIntegrationHandle`
  - `primary_graph::root::WorthQueryPrimaryGraph` held by `primary_graph::root::WorthQueryPrimaryGraph` (not vocabulary: struct)
  - `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` held by `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` (not vocabulary: struct)
  - `primary_graph::output_lineage::invalidation::head_cell::HeadCellRegistrationStop` held by `primary_graph::output_lineage::invalidation::head_cell::HeadCellRegistrationStop` (not vocabulary: enum)
- `product_publication` -> `primary_graph`: 4 refs in 2 items: `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition`, `primary_graph::application_output_demand::denial::recovery_release::WorthQueryProductRecoveryClose`
  - `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` held by `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyStore` (declaration)
  - `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` held by `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyStore` (impl WorthQueryUnpublishedIdempotencyDisposition)
  - `primary_graph::application_output_demand::denial::recovery_release::WorthQueryProductRecoveryClose` held by `primary_graph::application_output_demand::denial::recovery_release::WorthQueryProductRecoveryClose` (not vocabulary: trait)
- `domain_evidence_binding` -> `provider_session`: 2 refs in 2 items: `provider_session::graph_provider::receipt_association::WorthQueryCompletedDomainEvidenceDerivation`, `provider_session::graph_provider::receipt_association::WorthQueryBoundGraphExecutionAssociation`
  - `provider_session::graph_provider::receipt_association::WorthQueryCompletedDomainEvidenceDerivation` held by `provider_session::graph_provider::receipt_association::WorthQueryBoundGraphExecutionAssociation` (declaration)
  - `provider_session::graph_provider::receipt_association::WorthQueryCompletedDomainEvidenceDerivation` held by `provider_session::graph_provider::receipt_association::WorthQueryBoundGraphExecutionAssociation` (impl WorthQueryCompletedDomainEvidenceDerivation)
  - `provider_session::graph_provider::receipt_association::WorthQueryBoundGraphExecutionAssociation` held by `provider_session::graph_provider::receipt_association::WorthQueryBoundGraphExecutionAssociation` (not vocabulary: struct)
- `operation_binding` -> `provider_session`: 2 refs in 2 items: `provider_session::protocol::plan_contract::WorthQueryProviderExecutionPlanContract`, `provider_session::protocol::execution_plan::WorthQueryValidatedProviderPlan`
  - `provider_session::protocol::plan_contract::WorthQueryProviderExecutionPlanContract` held by `provider_session::protocol::execution_plan::WorthQueryValidatedProviderPlan` (impl WorthQueryProviderExecutionPlanContract)
  - `provider_session::protocol::execution_plan::WorthQueryValidatedProviderPlan` held by `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` (declaration)
  - `provider_session::protocol::execution_plan::WorthQueryValidatedProviderPlan` held by `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` (impl WorthQueryValidatedProviderPlan<'_>)
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
