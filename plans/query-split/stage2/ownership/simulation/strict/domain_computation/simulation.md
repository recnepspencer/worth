# Helper-move simulation: `domain_computation` (21 modules)

Before: component 12, 581 references against the current layering.
After: component 11, 264 against the current layering, 264 against a fresh layering of the result.
Moved: 223 helpers and 216 units over 3 rounds. Of 1557 helpers blocking coupled units, 206 could move at once.

Remaining seams in the largest component: 13 (11 contract, 2 nest or merge).

## Helpers blocking coupled units

| helper | owner | destination | units blocked | verdict |
|---|---|---|---|---|
| `primary_graph::application_query::managed_index::provider_failure::equal_cause` | primary_graph | domain_evidence_binding | 61 | blocked: primary_graph::application_query::managed_index::provider_failure::PaidProviderDenial (struct) |
| `primary_graph::application_attempt::denial::request_read::relational_read_kind` | primary_graph | domain_evidence_binding | 57 | blocked: primary_graph::application_attempt::denial::kind::WorthQueryApplicationAttemptDenialKind (enum), primary_graph::provider::relational_execution_denial::relational_execution_kind (fn), provider_session::protocol::control_stopped::WorthQueryProviderSessionControlStopKind (enum) |
| `primary_graph::provider::relational_execution_denial::query_read_execution_kind` | primary_graph | domain_evidence_binding | 57 | blocked: primary_graph::provider::relational_execution_denial::relational_execution_kind (fn), provider_session::protocol::control_stopped::WorthQueryProviderSessionControlStopKind (enum), provider_session::protocol::denial::WorthQueryProviderSessionDenialKind (enum) |
| `provider_session::graph_provider::read_row::pair_vector_bytes` | provider_session | domain_evidence_binding | 48 | could move |
| `provider_session::graph_provider::read_row::retained_arc_str_bytes` | provider_session | domain_evidence_binding | 48 | could move |
| `provider_session::graph_provider::read_row::value_by_key` | provider_session | domain_evidence_binding | 48 | could move |
| `provider_session::graph_provider::call_identity::NEXT_GRAPH_CALL_AUTHORITY` | provider_session | domain_evidence_binding | 47 | could move |
| `primary_graph::application_contribution::conditional::output_readiness_contract::denial` | primary_graph | domain_evidence_binding | 44 | blocked: primary_graph::application_contribution::conditional::output_readiness_contract::WorthQueryOutputReadinessContractDenial (struct) |
| `primary_graph::schema_layout::installation_primitives::planned_field_locator` | primary_graph | domain_evidence_binding | 44 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::valid_aspect_key (fn), primary_graph::schema_layout::installation_primitives::valid_field_key (fn) |
| `provider_session::execution_attempt_identity::NEXT_EXECUTION_ATTEMPT` | provider_session | domain_evidence_binding | 44 | could move |
| `provider_session::execution_attempt_identity::next_execution_attempt_ordinal` | provider_session | domain_evidence_binding | 44 | could move |
| `provider_session::graph_provider::call::readmission::same_execution_basis` | provider_session | domain_evidence_binding | 44 | blocked: provider_session::attempt_evidence::WorthQueryExecutionResourceAttemptEvidence (struct) |
| `provider_session::graph_provider::receipt::graph_evidence_identity` | provider_session | domain_evidence_binding | 44 | blocked: provider_session::graph_provider::call::WorthQueryGraphProviderCall (struct), provider_session::graph_provider::receipt::WorthQueryGraphProjectionEvidence (enum) |
| `primary_graph::application_attempt::observation::observe_field_value` | primary_graph | domain_evidence_binding | 42 | blocked: primary_graph::application_attempt::observation::WorthQueryApplicationFieldObservation (enum), primary_graph::application_attempt::observation::observe_field (fn) |
| `primary_graph::application_attempt::retained_decision_facts::endpoints::heap_sort` | primary_graph | domain_evidence_binding | 42 | blocked: primary_graph::application_attempt::retained_decision_facts::StoreDenial (enum), primary_graph::application_attempt::retained_decision_facts::control::StorageControl (struct), primary_graph::application_attempt::retained_decision_facts::endpoints::sift (fn) |
| `primary_graph::application_attempt::retained_decision_facts::endpoints::union_visit` | primary_graph | domain_evidence_binding | 42 | blocked: primary_graph::application_attempt::retained_decision_facts::StoreDenial (enum), primary_graph::application_attempt::retained_decision_facts::control::StorageControl (struct) |
| `primary_graph::application_attempt::workflow::actor::record::actor_record_corruption` | primary_graph | domain_evidence_binding | 42 | blocked: primary_graph::application_attempt::denial::WorthQueryApplicationAttemptDenial (struct), primary_graph::application_attempt::denial::kind::WorthQueryApplicationAttemptDenialKind (enum) |
| `primary_graph::application_attempt::workflow::actor::record::decode_actor_components` | primary_graph | domain_evidence_binding | 42 | could move |
| `primary_graph::freshness::comparison_observation` | primary_graph | domain_evidence_binding | 42 | blocked: primary_graph::resolution_denial::WorthQueryPrincipalResolutionDenial (struct), primary_graph::resolution_denial::WorthQueryPrincipalResolutionDenialKind (enum), provider_session::protocol::control_stopped::WorthQueryProviderSessionControlStopKind (enum) |
| `primary_graph::freshness::principal_description_is_portable` | primary_graph | domain_evidence_binding | 42 | could move |
| `primary_graph::observations::observe_exact_principal_target` | primary_graph | domain_evidence_binding | 42 | blocked: primary_graph::observations::WorthQueryPrincipalTargetObservation (struct), primary_graph::observations::observe_principal_identity (fn), primary_graph::observations::principal_adjacency_revision (fn) |
| `primary_graph::observations::principal_mapping::observe_mapping` | primary_graph | domain_evidence_binding | 42 | blocked: primary_graph::observations::WorthQueryPrincipalMappingObservation (struct), primary_graph::observations::authoritative (fn), primary_graph::observations::mapping_projection_scope (fn) |
| `primary_graph::application_checkpoint::facts::payload::array_storage_bytes` | primary_graph | domain_evidence_binding | 41 | could move |
| `primary_graph::application_checkpoint::facts::payload::chunk::bytes` | primary_graph | domain_evidence_binding | 41 | could move |
| `primary_graph::application_checkpoint::facts::payload::chunk::deep_clone_bytes` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact (enum), primary_graph::application_checkpoint::facts::payload::chunk::bytes (fn) |
| `primary_graph::application_checkpoint::facts::payload::concatenation::join` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::application_attempt::retained_decision_facts::StoreDenial (enum), primary_graph::application_attempt::retained_decision_facts::control::StorageControl (struct), primary_graph::application_checkpoint::facts::payload::FactPayload (struct) |
| `primary_graph::application_checkpoint::facts::payload::original_storage_bytes` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact (enum) |
| `primary_graph::application_checkpoint::facts::payload::patch::preparation::build` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::application_attempt::retained_decision_facts::StoreDenial (enum), primary_graph::application_attempt::retained_decision_facts::control::StorageControl (struct), primary_graph::application_checkpoint::facts::payload::FactPayload (struct) |
| `primary_graph::application_checkpoint::facts::payload::patch::preparation::count_nodes` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::application_attempt::retained_decision_facts::StoreDenial (enum), primary_graph::application_attempt::retained_decision_facts::control::StorageControl (struct), primary_graph::application_checkpoint::facts::payload::FactPayload (struct) |
| `primary_graph::application_checkpoint::facts::payload::patch::take_published` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::application_checkpoint::facts::payload::FactPayload (struct), primary_graph::application_checkpoint::facts::payload::patch::PreparedNode (enum) |
| `primary_graph::application_checkpoint::facts::payload::resident_bytes` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact (enum), primary_graph::fact_verification_observation::original_payload_size (fn) |
| `primary_graph::fact_verification_observation::native_patch_clone` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::fact_verification_observation::add (fn), primary_graph::fact_verification_observation::record (fn) |
| `primary_graph::fact_verification_observation::native_patch_path` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::fact_verification_observation::add (fn), primary_graph::fact_verification_observation::record (fn) |
| `primary_graph::fact_verification_observation::native_payload_chunk` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::fact_verification_observation::add (fn), primary_graph::fact_verification_observation::record (fn) |
| `primary_graph::fact_verification_observation::native_payload_fact_move` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::fact_verification_observation::add (fn), primary_graph::fact_verification_observation::record (fn) |
| `primary_graph::fact_verification_observation::native_payload_join` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::fact_verification_observation::add (fn), primary_graph::fact_verification_observation::record (fn) |
| `primary_graph::fact_verification_observation::native_payload_size` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::fact_verification_observation::add (fn), primary_graph::fact_verification_observation::record (fn) |
| `primary_graph::observations::authoritative` | primary_graph | domain_evidence_binding | 41 | could move |
| `primary_graph::provider::relational_execution_denial::relational_execution_kind` | primary_graph | domain_evidence_binding | 41 | blocked: provider_session::protocol::control_stopped::WorthQueryProviderSessionControlStopKind (enum), provider_session::protocol::denial::WorthQueryProviderSessionDenialKind (enum) |
| `primary_graph::schema_layout::aggregate_identity::lower` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::aggregate_identity::component_layout::ComponentSetLayout (struct), primary_graph::schema_layout::aggregate_identity::component_layout::retain (fn) |
| `primary_graph::schema_layout::application_layout_lowering::field_capability_keys` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::schema_layout::WorthQueryPrimaryFieldLayout (struct) |
| `primary_graph::schema_layout::application_layout_lowering::lower_fields` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::WorthQueryPrimaryFieldLayout (struct), primary_graph::schema_layout::installation_primitives::planned_field_locator (fn) |
| `primary_graph::schema_layout::application_layout_lowering::lower_relation_layouts` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::WorthQueryPrimaryRelationLayout (struct), primary_graph::schema_layout::installation_primitives::required_kind (fn) |
| `primary_graph::schema_layout::authorization_selectors::insert` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::schema_layout::authorization_selectors::EntityFields (type) |
| `primary_graph::schema_layout::capability_grant_join::lower_capability_grant_joins` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::WorthQueryPrimaryRelationLayout (struct), primary_graph::schema_layout::capability_grant_join::WorthQueryCapabilityGrantJoinLayout (struct) |
| `primary_graph::schema_layout::capability_grant_join::required_relation` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::WorthQueryPrimaryRelationLayout (struct), primary_graph::schema_layout::installation_primitives::invalid_member (fn) |
| `primary_graph::schema_layout::content::lower` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::content::codec::Codec (struct), primary_graph::schema_layout::content::refusal (fn) |
| `primary_graph::schema_layout::continuation_ordering::lower_continuation_orderings` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::WorthQueryPrimaryRelationLayout (struct), primary_graph::schema_layout::continuation_ordering::WorthQueryPrimaryContinuationOrderingLayout (struct) |
| `primary_graph::schema_layout::derived_relation::lower` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::aggregate_identity::definitions::derived::orbit_identity (fn), primary_graph::schema_layout::derived_relation::DerivedRelationLayout (struct) |
| `primary_graph::schema_layout::installation_primitives::invalid_member` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::denial::WorthQueryPrimaryGraphInstallationDenialKind (enum) |
| `primary_graph::schema_layout::installation_primitives::required_kind` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::invalid_member (fn) |
| `primary_graph::schema_layout::live_control_roles::lower` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::required_kind (fn) |
| `primary_graph::schema_layout::managed_source::lower` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::WorthQueryPrimaryFieldLayout (struct), primary_graph::schema_layout::installation_primitives::required_kind (fn) |
| `primary_graph::schema_layout::platform_entity_lowering::lower_platform_entities` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::application_attempt::workflow::schema::lower_workflow (fn), primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::kind_space_exhausted (fn) |
| `primary_graph::schema_layout::principal_binding::lower_principal_bindings` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::contract_space_exhausted (fn), primary_graph::schema_layout::installation_primitives::planned_field_locator (fn) |
| `primary_graph::schema_layout::projected_member_domain::lower` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::invalid_member (fn), primary_graph::schema_layout::installation_primitives::required_kind (fn) |
| `primary_graph::schema_layout::registry_lowering::lower_application_contract_bindings` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::schema_layout::registry_lowering::LoweredApplicationContractBindings (struct) |
| `primary_graph::schema_layout::registry_lowering::lower_kind_ids` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::kind_space_exhausted (fn), primary_graph::schema_layout::registry_lowering::next_kind_id (fn) |
| `primary_graph::schema_layout::registry_lowering::next_provider_kind_id` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::kind_space_exhausted (fn), primary_graph::schema_layout::registry_lowering::next_kind_id (fn) |
| `primary_graph::schema_layout::registry_lowering::register_application_relation` | primary_graph | domain_evidence_binding | 41 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::registry_lowering::register_relation_to_kinds_with_contracts (fn) |

## Remaining seams

### `authorization` ↔ `primary_graph`: 160 references up in 53 items, default **nest or merge**

Items: `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` (16), `primary_graph::authenticated_principal::WorthQueryAuthenticatedPrincipal` (11), `primary_graph::application_attempt::workflow_transition_program::actor_selection::WorkflowRecordedActorObservation` (10), `primary_graph::entity_resolution::resolved::WorthQueryApplicationEntityIdentity` (9), `primary_graph::application_attempt::precondition_binding::WorthQueryBoundMutationPreconditions` (8), `primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact` (7), `primary_graph::application_attempt::workflow::actor::encoding::check_live` (5), `primary_graph::application_attempt::workflow::actor::fund_actor_codec` (5)

- `primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact` held by `primary_graph::application_attempt::fact::source_currentness::WorthQuerySourceCurrentnessFailure` in `impl WorthQueryApplicationObservedFact`
- `primary_graph::application_contribution::partitioned_computation::incremental::retained::SealedComputationRun` held by `primary_graph::application_contribution::partitioned_computation::incremental::retained::RetainedComputation` in `impl SealedComputationRun`
- `primary_graph::provider::application_decision_fact::application_table::ApplicationFactTable` held by `primary_graph::provider::application_decision_fact::WorthQueryPrimaryGraphApplicationDecisionFact` in `impl ApplicationFactTable`
- `primary_graph::causal_inspection::application_inspection::target::WorthQueryApplicationInspectionTarget` held by `primary_graph::causal_inspection::application_observation::declared_query::WorthQueryApplicationQueryObservation` in `impl WorthQueryApplicationInspectionTarget`
- `primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease` held by `primary_graph::root::WorthQueryPrimaryGraphIntegrationHandle` in `impl WorthQueryApplicationSnapshotLease`
- `primary_graph::application_attempt::provider_binding::WorthQueryPreparedApplicationProviderAttempt` held by `primary_graph::application_attempt::provider_binding::effect_accumulator::WorthQueryRegisteredProviderEffects` in `impl WorthQueryPreparedApplicationProviderAttempt`

### `convergence_epoch` ↔ `managed_run`: 33 references up in 23 items, default **nest or merge**

Items: `managed_run::workflow_graph_execution::WorthQueryActiveWorkflowGraphExecution` (5), `managed_run::direct::WorthQueryAdmittedDirectRun` (3), `managed_run::workflow::terminal::owner::WorthQueryWorkflowRunTerminal` (3), `managed_run::workflow_graph_chunk::WorthQueryPendingWorkflowGraphChunk` (2), `managed_run::direct_yield_recovery::WorthQueryDirectYieldRecoveryRequired` (2), `managed_run::direct_yield::WorthQueryDirectYieldOutcome` (1), `managed_run::workflow_yield_cleanup::WorthQueryWorkflowYieldCleanupPending` (1), `managed_run::direct_yield_cleanup::WorthQueryDirectYieldCleanupOutcome` (1)

- `managed_run::direct::WorthQueryAdmittedDirectRun` held by `managed_run::run_affinity::WorthQueryDirectRunAffinity` in `declaration`
- `managed_run::direct::WorthQueryAdmittedDirectRun` held by `managed_run::run_affinity::WorthQueryDirectRunAffinity` in `impl WorthQueryAdmittedDirectRun`
- `managed_run::direct_yield::WorthQueryDirectYieldOutcome` held by `managed_run::direct_yield_recovery::WorthQueryDirectYieldRecoveryRequired` in `declaration`
- `managed_run::workflow_yield_cleanup::WorthQueryWorkflowYieldCleanupPending` held by `managed_run::workflow_yield_cleanup::cleanup_without_checkpoint_owner` in `impl WorthQueryWorkflowYieldCleanupPending`
- `managed_run::direct_yield_cleanup::WorthQueryDirectYieldCleanupOutcome` held by `managed_run::direct_yield_cleanup::inspection::WorthQueryDirectYieldCleanupReceipt` in `declaration`
- `managed_run::workflow_graph_execution::WorthQueryActiveWorkflowGraphExecution` held by `managed_run::managed_graph_execution::WorthQueryManagedGraphExecution` in `declaration`

### `provider_session` ↔ `primary_graph`: 27 references up in 12 items, default **contract**

Items: `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` (6), `primary_graph::causal_inspection::application_inspection::target::WorthQueryApplicationInspectionTarget` (4), `primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease` (3), `primary_graph::application_query::managed_index::invocation::WorthQueryManagedIndexWorkDenial` (3), `primary_graph::application_attempt::provider_execution::application_attempt_affinity::WorthQueryApplicationAttemptBasis` (2), `primary_graph::application_attempt::provider_execution::application_attempt_affinity::WorthQueryApplicationAttemptAffinity` (2), `primary_graph::provider::application_attempt_state::registered_reads::RegisteredPrimaryRead` (2), `primary_graph::source_facts::allocation::SourceFactsAllocation` (1)

- `primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial` held by `primary_graph::application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl WorthQueryOutputDemandDenial`
- `primary_graph::causal_inspection::application_inspection::target::WorthQueryApplicationInspectionTarget` held by `primary_graph::causal_inspection::application_observation::declared_query::WorthQueryApplicationQueryObservation` in `impl WorthQueryApplicationInspectionTarget`
- `primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease` held by `primary_graph::root::WorthQueryPrimaryGraphIntegrationHandle` in `impl WorthQueryApplicationSnapshotLease`
- `primary_graph::root::WorthQueryPrimaryGraphIntegrationHandle` held by `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` in `impl WorthQueryPrimaryGraphIntegrationHandle`
- `primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial` held by `primary_graph::application_contribution::producer::denial_value::WorthQueryProducerDenialValue` in `impl WorthQueryOutputDemandDenial`
- `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` held by `primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial` in `impl WorthQueryPrimaryGraphLayout`

### `application_aftermath` ↔ `primary_graph`: 12 references up in 4 items, default **contract**

Items: `primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime` (4), `primary_graph::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding` (4), `primary_graph::product_operation::correction_source::preparation::WorthQueryPreparedCorrectionSource` (3), `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenial` (1)

- `primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime` held by `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation` in `impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationR`
- `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenial` held by `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::denial_cause::DenialCause` in `impl WorthQueryApplicationCommitDenial`
- `primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime` held by `primary_graph::application_attempt::compare_and_commit::commit_receipt::WorthQueryApplicationCommitReceipt` in `impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationR`
- `primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime` held by `primary_graph::program_occurrence::WorthQueryInstalledProgramSupport` in `impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>`
- `primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime` held by `primary_graph::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding` in `impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationR`
- `primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime` held by `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::capability_admission::preparation::observation::admitted_access::WorthQueryAdmittedApplicationCapabilityAccess` in `impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationR`

### `application_aftermath` ↔ `authorization`: 9 references up in 4 items, default **contract**

Items: `primary_graph::application_attempt::compare_and_commit::commit_receipt::WorthQueryApplicationCommitReceipt` (4), `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation` (3), `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::capability_admission::preparation::observation::admitted_access::WorthQueryAdmittedApplicationCapabilityAccess` (1), `authorization::denial::WorthQueryOperationAuthorizationDenial` (1)

- `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation` held by `authorization::denial::WorthQueryOperationAuthorizationDenial` in `impl<Schema, Operation, Input, Scope> WorthQueryAdmittedApplicationOperation<Sch`
- `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation` held by `authorization::retained_capability_request::portable::WorkflowApprovalRequestEncodingDenial` in `impl<Schema, Operation, Input, Scope> WorthQueryAdmittedApplicationOperation<Sch`
- `primary_graph::application_attempt::compare_and_commit::commit_receipt::WorthQueryApplicationCommitReceipt` held by `primary_graph::application_attempt::commit_authority_binding::WorthQueryApplicationCommitAuthorityBinding` in `impl WorthQueryApplicationCommitReceipt`
- `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation` held by `authorization::decision_facts::WorthQueryRetainedAuthorizationDecisionFacts` in `impl<Schema, Operation, Input, Scope> WorthQueryAdmittedApplicationOperation<Sch`
- `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation` held by `primary_graph::application_entry::mutation::execution::MutationHandlerExecutionDenial` in `impl<Schema, Operation, Input, Scope> WorthQueryAdmittedApplicationOperation<Sch`
- `primary_graph::application_attempt::compare_and_commit::commit_receipt::WorthQueryApplicationCommitReceipt` held by `primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondence` in `impl WorthQueryApplicationCommitReceipt`

### `product_publication` ↔ `primary_graph`: 6 references up in 3 items, default **contract**

Items: `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` (3), `primary_graph::application_contribution::execution_denial` (2), `primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime` (1)

- `primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime` held by `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation` in `impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationR`
- `primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime` held by `primary_graph::application_attempt::compare_and_commit::commit_receipt::WorthQueryApplicationCommitReceipt` in `impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationR`
- `primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime` held by `primary_graph::program_occurrence::WorthQueryInstalledProgramSupport` in `impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>`
- `primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime` held by `primary_graph::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding` in `impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationR`
- `primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime` held by `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::capability_admission::preparation::observation::admitted_access::WorthQueryAdmittedApplicationCapabilityAccess` in `impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationR`
- `primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime` held by `primary_graph::application_query::projection::WorthQueryApplicationProjection` in `impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationR`

### `execution_runtime` ↔ `primary_graph`: 6 references up in 4 items, default **contract**

Items: `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` (2), `primary_graph::application_contribution::execution_denial` (2), `primary_graph::root::WorthQueryPrimaryGraph` (1), `primary_graph::root::WorthQueryPrimaryGraphIntegrationHandle` (1)

- `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` held by `primary_graph::output_lineage::invalidation::derived::SettlementRegistrationStop` in `impl SourceInvalidationOwner`
- `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` held by `primary_graph::output_lineage::invalidation::source_alignment::SnapshotAlignedMarkState` in `impl SourceInvalidationOwner`
- `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` held by `primary_graph::output_lineage::invalidation::settlement::SettlementRegistration` in `impl SourceInvalidationOwner`
- `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` held by `primary_graph::output_lineage::invalidation::verification::SettlementVerificationStop` in `impl SourceInvalidationOwner`
- `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` held by `primary_graph::output_lineage::invalidation::mark_state::FullVerificationReason` in `impl SourceInvalidationOwner`
- `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` held by `primary_graph::output_lineage::invalidation::query_coverage::QueryCoverageStop` in `impl SourceInvalidationOwner`

### `operation_binding` ↔ `provider_session`: 5 references up in 2 items, default **contract**

Items: `provider_session::protocol::declared_closure::WorthQueryProviderPlanDeclarations` (4), `provider_session::protocol::plan_contract::WorthQueryProviderExecutionPlanContract` (1)

- `provider_session::protocol::declared_closure::WorthQueryProviderPlanDeclarations` held by `provider_session::protocol::declared_closure::bind_direct_role_closure` in `impl WorthQueryProviderPlanDeclarations`
- `provider_session::protocol::plan_contract::WorthQueryProviderExecutionPlanContract` held by `provider_session::protocol::declared_closure::WorthQueryProviderPlanDeclarations` in `impl WorthQueryProviderExecutionPlanContract`
- `provider_session::protocol::declared_closure::WorthQueryProviderPlanDeclarations` held by `provider_session::protocol::declared_closure::aftermath_posture::reversal_posture` in `impl WorthQueryProviderPlanDeclarations`
- `provider_session::protocol::declared_closure::WorthQueryProviderPlanDeclarations` held by `provider_session::protocol::declared_closure::application::bind_application_candidate_effect_closure` in `impl WorthQueryProviderPlanDeclarations`
- `provider_session::protocol::declared_closure::WorthQueryProviderPlanDeclarations` held by `provider_session::protocol::declared_closure::application::bind_native_reference_invariants` in `impl WorthQueryProviderPlanDeclarations`

### `convergence_epoch` ↔ `provider_session`: 2 references up in 1 items, default **contract**

Items: `provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor` (2)

- `provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor` held by `provider_session::provisional_attempt::provider_port::WorthQueryProvisionalEffectProgramView` in `impl WorthQueryGraphProviderAnchor`
- `provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor` held by `provider_session::provisional_attempt::provider_port::WorthQueryProvisionalOverlayAdmission` in `impl WorthQueryGraphProviderAnchor`
- `provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor` held by `provider_session::provisional_attempt::provider_port::WorthQueryProvisionalOverlayEvidence` in `impl WorthQueryGraphProviderAnchor`
- `provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor` held by `provider_session::provisional_attempt::provider_port::WorthQueryProvisionalOverlayEvidenceView` in `impl WorthQueryGraphProviderAnchor`
- `provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor` held by `provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryErasedGraphParticipationProvider` in `declaration`
- `provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor` held by `provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryErasedProviderSessionLifecycle` in `declaration`

### `execution_runtime` ↔ `invalidation_equivalence_control`: 1 references up in 1 items, default **contract**

Items: `invalidation_equivalence_control::attach_constructed_receiver` (1)


### `artifact_owner` ↔ `operation_binding`: 1 references up in 1 items, default **contract**

Items: `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` (1)

- `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` held by `provider_session::protocol::declared_closure::WorthQueryProviderPlanDeclarations` in `impl WorthQueryExecutionBoundOperationAuthority`
- `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` held by `operation_binding::authority::topology::WorthQueryExecutionResourceTopology` in `impl WorthQueryExecutionBoundOperationAuthority`
- `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` held by `operation_binding::authority::WorthQueryWorkflowStageResourceAuthority` in `declaration`
- `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` held by `operation_binding::authority::topology::WorthQueryExecutionResourceTopology` in `declaration`
- `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` held by `provider_session::protocol::declared_closure::WorthQueryProviderPlanDeclarations` in `declaration`
- `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` held by `execution_runtime::runtime_root::WorthQueryExecutionRuntime` in `impl WorthQueryExecutionBoundOperationAuthority`

### `execution_runtime` ↔ `product_publication`: 1 references up in 1 items, default **contract**

Items: `product_publication::unpublished::WorthQueryProductUnpublishedApplication` (1)


### `execution_runtime` ↔ `managed_run`: 1 references up in 1 items, default **contract**

Items: `managed_run::direct_admission::WorthQueryManagedRunAdmission` (1)

- `managed_run::direct_admission::WorthQueryManagedRunAdmission` held by `managed_run::lower_admission::admit_managed_lower_execution_basis` in `impl WorthQueryManagedRunAdmission<'_>`
- `managed_run::direct_admission::WorthQueryManagedRunAdmission` held by `managed_run::truth_read_request::WorthQueryManagedTruthReadRequest` in `impl WorthQueryManagedRunAdmission<'_>`
- `managed_run::direct_admission::WorthQueryManagedRunAdmission` held by `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` in `impl WorthQueryManagedRunAdmission<'_>`
- `managed_run::direct_admission::WorthQueryManagedRunAdmission` held by `managed_run::admission::validate_direct_run_head` in `impl WorthQueryManagedRunAdmission<'_>`
- `managed_run::direct_admission::WorthQueryManagedRunAdmission` held by `managed_run::admission::validate_direct_run_lower` in `impl WorthQueryManagedRunAdmission<'_>`
- `managed_run::direct_admission::WorthQueryManagedRunAdmission` held by `managed_run::direct::WorthQueryAdmittedDirectRun` in `impl WorthQueryManagedRunAdmission<'_>`

