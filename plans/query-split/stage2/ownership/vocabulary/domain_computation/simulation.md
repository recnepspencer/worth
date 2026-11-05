# Vocabulary simulation: `domain_computation`

Before: component 12, 581 references against the layering.
After: component 12, 405 against a fresh layering; 123 items moved over 1 rounds [123].

## Vocabulary modules

| module | items | lines | kinds | top source modules |
|---|---|---|---|---|
| `vocabulary:primary_graph` | 58 | 2026 | struct 26, enum 24, fn 8 | `primary_graph::admitted_capture::retained_stage` 3, `primary_graph::vocabulary::application_query::provider_failure` 3, `primary_graph::vocabulary::resolution_denial` 3, `primary_graph::admitted_capture` 2, `primary_graph::application_attempt::snapshot_lease` 2 |
| `vocabulary:provider_session` | 27 | 957 | struct 9, enum 8, fn 8, static 2 | `provider_session::protocol::declared_closure` 6, `provider_session::graph_obligation::session_identity` 4, `provider_session::graph_provider::bounded_step::provider_execution_release` 3, `provider_session::protocol::declared_closure::application` 3, `provider_session::graph_provider::bounded_step::denial` 2 |
| `vocabulary:managed_run` | 16 | 586 | struct 12, enum 4 | `managed_run::direct_terminal::inspection` 3, `managed_run::provider_work::evidence` 3, `managed_run::terminal` 3, `managed_run::counters` 1, `managed_run::provider_work::release_summary` 1 |
| `vocabulary:application_aftermath` | 10 | 343 | struct 6, fn 3, enum 1 | `application_aftermath::external_effect::outbox` 7, `application_aftermath::correction::selection::original` 1, `application_aftermath::external_effect::correlation` 1, `application_aftermath::retained_preimage` 1 |
| `vocabulary:authorization` | 6 | 243 | struct 3, enum 2, static 1 | `authorization::denial` 2, `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation` 2, `authorization::operation_scope_binding` 2 |
| `vocabulary:request_execution` | 2 | 67 | enum 2 | `request_execution::execution_refusal` 2 |
| `vocabulary:operation_binding` | 1 | 50 | struct 1 | `operation_binding::installed_domain_authority` 1 |
| `vocabulary:execution_runtime` | 3 | 45 | static 1, struct 1, fn 1 | `execution_runtime::runtime_identity` 3 |

Cycles among vocabulary modules: none

Vocabulary modules naming modules above them (should be none): none

## Remaining seams

- `authorization` -> `primary_graph`: 198 refs in 67 items: `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout`, `primary_graph::authenticated_principal::WorthQueryAuthenticatedPrincipal`, `primary_graph::application_attempt::workflow_transition_program::actor_selection::WorkflowRecordedActorObservation`, `primary_graph::entity_resolution::resolved::WorthQueryApplicationEntityIdentity`
  - `primary_graph::application_contribution::partitioned_computation::incremental::published_state::observe_discarded` held by `primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` (body)
  - `primary_graph::program_occurrence::original_meaning::OriginalProgramMeaning` held by `primary_graph::application_installation::program::supported_program::WorthQuerySupportedProgramRecord` (declaration)
  - `primary_graph::entity_resolution::resolved::WorthQueryApplicationEntityIdentity` held by `primary_graph::entity_resolution::resolved::WorthQueryApplicationEntityIdentity` (not vocabulary: struct)
- `application_aftermath` -> `primary_graph`: 30 refs in 13 items: `primary_graph::provider::committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxObservation`, `primary_graph::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding`, `primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime`, `primary_graph::application_attempt::compare_and_commit::commit_receipt::WorthQueryApplicationCommitReceipt`
  - `primary_graph::application_attempt::idempotency_resolution::WorthQueryApplicationIdempotencyResolution` held by `primary_graph::application_attempt::compare_and_commit::commit_receipt::WorthQueryApplicationCommitReceipt` (declaration)
  - `primary_graph::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding` held by `primary_graph::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding` (not vocabulary: struct)
  - `primary_graph::application_inbound::inbound_publication::outcome::WorthQueryUnpublishedInboundCompletion` held by `application_aftermath::external_effect::inbound::custody::WorthQueryAcceptedInboundOccurrence` (declaration)
- `provider_session` -> `primary_graph`: 25 refs in 12 items: `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout`, `primary_graph::causal_inspection::application_inspection::target::WorthQueryApplicationInspectionTarget`, `primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease`, `primary_graph::application_attempt::provider_execution::application_attempt_affinity::WorthQueryApplicationAttemptAffinity`
  - `primary_graph::application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` held by `primary_graph::application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` (not vocabulary: enum)
  - `primary_graph::root::WorthQueryPrimaryGraphIntegrationHandle` held by `primary_graph::root::WorthQueryPrimaryGraphIntegrationHandle` (not vocabulary: struct)
  - `primary_graph::causal_inspection::application_inspection::target::WorthQueryApplicationInspectionTarget` held by `primary_graph::causal_inspection::application_inspection::target::InspectionOriginal` (declaration)
- `provider_session` -> `managed_run`: 10 refs in 5 items: `managed_run::direct::WorthQueryRunningDirectRun`, `managed_run::direct_graph_execution::completion::WorthQueryCompletedDirectEvidenceOwner`, `managed_run::workflow_graph_execution::step_outcome::WorthQueryCompletedWorkflowEvidenceOwner`, `managed_run::direct_terminal::cleanup::WorthQueryDirectRunCleanupFailure`
  - `managed_run::direct_terminal::cleanup::WorthQueryDirectRunCleanupFailure` held by `managed_run::direct_terminal::owner::WorthQueryDirectRunTerminal` (declaration)
  - `managed_run::direct_terminal::cleanup::WorthQueryDirectRunCleanupFailure` held by `managed_run::direct_terminal::owner::WorthQueryDirectRunTerminal` (impl WorthQueryDirectRunCleanupFailure)
  - `managed_run::direct::WorthQueryRunningDirectRun` held by `managed_run::direct::WorthQueryRunningDirectRun` (not vocabulary: struct)
- `execution_runtime` -> `primary_graph`: 7 refs in 5 items: `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner`, `primary_graph::application_contribution::execution_denial`, `primary_graph::root::WorthQueryPrimaryGraph`, `primary_graph::root::WorthQueryPrimaryGraphIntegrationHandle`
  - `primary_graph::root::WorthQueryPrimaryGraph` held by `primary_graph::root::WorthQueryPrimaryGraph` (not vocabulary: struct)
  - `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` held by `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` (not vocabulary: struct)
  - `primary_graph::root::WorthQueryPrimaryGraphIntegrationHandle` held by `primary_graph::root::WorthQueryPrimaryGraphIntegrationHandle` (not vocabulary: struct)
- `product_publication` -> `primary_graph`: 7 refs in 4 items: `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition`, `primary_graph::application_contribution::execution_denial`, `primary_graph::application_output_demand::denial::recovery_release::WorthQueryProductRecoveryClose`, `primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime`
  - `primary_graph::application_contribution::execution_denial` held by `primary_graph::application_contribution::execution_denial` (not vocabulary: module)
  - `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` held by `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyStore` (declaration)
  - `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` held by `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyStore` (impl WorthQueryUnpublishedIdempotencyDisposition)
- `provider_session` -> `convergence_epoch`: 2 refs in 1 items: `convergence_epoch::domain_provider::WorthQueryConvergenceDomainProvider`
  - `convergence_epoch::domain_provider::WorthQueryConvergenceDomainProvider` held by `convergence_epoch::domain_provider::WorthQueryConvergenceDomainProvider` (not vocabulary: trait)
- `domain_evidence_binding` -> `provider_session`: 2 refs in 2 items: `provider_session::graph_provider::receipt_association::WorthQueryCompletedDomainEvidenceDerivation`, `provider_session::graph_provider::receipt_association::WorthQueryBoundGraphExecutionAssociation`
  - `provider_session::graph_provider::receipt_association::WorthQueryCompletedDomainEvidenceDerivation` held by `provider_session::graph_provider::receipt_association::WorthQueryBoundGraphExecutionAssociation` (declaration)
  - `provider_session::graph_provider::receipt_association::WorthQueryCompletedDomainEvidenceDerivation` held by `provider_session::graph_provider::receipt_association::WorthQueryBoundGraphExecutionAssociation` (impl WorthQueryCompletedDomainEvidenceDerivation)
  - `provider_session::graph_provider::receipt_association::WorthQueryBoundGraphExecutionAssociation` held by `provider_session::graph_provider::receipt_association::WorthQueryBoundGraphExecutionAssociation` (not vocabulary: struct)
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
