# Helper-move simulation: `domain_computation` (21 modules)

Before: component 12, 432 references against the current layering.
After: component 10, 154 against the current layering, 154 against a fresh layering of the result.
Moved: 536 helpers and 525 units over 5 rounds. Of 1450 helpers blocking coupled units, 319 could move at once.

Remaining seams in the largest component: 10 (9 contract, 1 nest or merge).

## Helpers blocking coupled units

| helper | owner | destination | units blocked | verdict |
|---|---|---|---|---|
| `primary_graph::application_query::managed_index::provider_failure::equal_cause` | primary_graph | execution_runtime | 43 | could move |
| `primary_graph::application_attempt::denial::request_read::relational_read_kind` | primary_graph | execution_runtime | 39 | blocked: primary_graph::provider::relational_execution_denial::relational_execution_kind (fn) |
| `primary_graph::provider::relational_execution_denial::query_read_execution_kind` | primary_graph | execution_runtime | 39 | blocked: primary_graph::provider::relational_execution_denial::relational_execution_kind (fn) |
| `primary_graph::application_contribution::conditional::output_readiness_contract::denial` | primary_graph | execution_runtime | 37 | blocked: primary_graph::application_contribution::conditional::output_readiness_contract::WorthQueryOutputReadinessContractDenial (struct) |
| `primary_graph::schema_layout::installation_primitives::planned_field_locator` | primary_graph | execution_runtime | 35 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::valid_aspect_key (fn), primary_graph::schema_layout::installation_primitives::valid_field_key (fn) |
| `primary_graph::schema_layout::aggregate_identity::lower` | primary_graph | execution_runtime | 32 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::aggregate_identity::component_layout::retain (fn), primary_graph::schema_layout::aggregate_identity::definitions::collect (fn) |
| `primary_graph::schema_layout::application_layout_lowering::field_capability_keys` | primary_graph | execution_runtime | 32 | could move |
| `primary_graph::schema_layout::application_layout_lowering::lower_fields` | primary_graph | execution_runtime | 32 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::planned_field_locator (fn), primary_graph::schema_layout::installation_primitives::required_kind (fn) |
| `primary_graph::schema_layout::application_layout_lowering::lower_relation_layouts` | primary_graph | execution_runtime | 32 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::required_kind (fn) |
| `primary_graph::schema_layout::authorization_selectors::insert` | primary_graph | execution_runtime | 32 | could move |
| `primary_graph::schema_layout::capability_grant_join::lower_capability_grant_joins` | primary_graph | execution_runtime | 32 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::capability_grant_join::WorthQueryCapabilityGrantJoinLayout (struct) |
| `primary_graph::schema_layout::capability_grant_join::required_relation` | primary_graph | execution_runtime | 32 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::invalid_member (fn) |
| `primary_graph::schema_layout::content::lower` | primary_graph | execution_runtime | 32 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::content::refusal (fn) |
| `primary_graph::schema_layout::continuation_ordering::lower_continuation_orderings` | primary_graph | execution_runtime | 32 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::invalid_member (fn), primary_graph::schema_layout::installation_primitives::planned_field_locator (fn) |
| `primary_graph::schema_layout::derived_relation::lower` | primary_graph | execution_runtime | 32 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::aggregate_identity::definitions::derived::orbit_identity (fn), primary_graph::schema_layout::derived_relation::expression::lower (fn) |
| `primary_graph::schema_layout::installation_primitives::invalid_member` | primary_graph | execution_runtime | 32 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct) |
| `primary_graph::schema_layout::installation_primitives::required_kind` | primary_graph | execution_runtime | 32 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::invalid_member (fn) |
| `primary_graph::schema_layout::live_control_roles::lower` | primary_graph | execution_runtime | 32 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::required_kind (fn) |
| `primary_graph::schema_layout::managed_source::lower` | primary_graph | execution_runtime | 32 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::required_kind (fn), primary_graph::schema_layout::managed_source::membership::path (fn) |
| `primary_graph::schema_layout::platform_entity_lowering::lower_platform_entities` | primary_graph | execution_runtime | 32 | blocked: primary_graph::application_attempt::workflow::schema::lower_workflow (fn), primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::kind_space_exhausted (fn) |
| `primary_graph::schema_layout::principal_binding::lower_principal_bindings` | primary_graph | execution_runtime | 32 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::contract_space_exhausted (fn), primary_graph::schema_layout::installation_primitives::planned_field_locator (fn) |
| `primary_graph::schema_layout::projected_member_domain::lower` | primary_graph | execution_runtime | 32 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::invalid_member (fn), primary_graph::schema_layout::installation_primitives::required_kind (fn) |
| `primary_graph::schema_layout::registry_lowering::lower_application_contract_bindings` | primary_graph | execution_runtime | 32 | could move |
| `primary_graph::schema_layout::registry_lowering::lower_kind_ids` | primary_graph | execution_runtime | 32 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::kind_space_exhausted (fn), primary_graph::schema_layout::registry_lowering::next_kind_id (fn) |
| `primary_graph::schema_layout::registry_lowering::next_provider_kind_id` | primary_graph | execution_runtime | 32 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::kind_space_exhausted (fn), primary_graph::schema_layout::registry_lowering::next_kind_id (fn) |
| `primary_graph::schema_layout::registry_lowering::register_application_relation` | primary_graph | execution_runtime | 32 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::registry_lowering::register_relation_to_kinds_with_contracts (fn) |
| `primary_graph::schema_layout::registry_lowering::register_entity` | primary_graph | execution_runtime | 32 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::relational_schema_denial (fn) |
| `primary_graph::schema_layout::registry_lowering::relational_schema_basis` | primary_graph | execution_runtime | 32 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::relational_schema_denial (fn) |
| `primary_graph::schema_layout::signed_entities::lower` | primary_graph | execution_runtime | 32 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::required_kind (fn) |
| `primary_graph::application_checkpoint::facts::payload::array_storage_bytes` | primary_graph | execution_runtime | 31 | could move |
| `primary_graph::application_checkpoint::facts::payload::chunk::bytes` | primary_graph | execution_runtime | 31 | could move |
| `primary_graph::application_checkpoint::facts::payload::chunk::deep_clone_bytes` | primary_graph | execution_runtime | 31 | blocked: primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact (enum), primary_graph::application_checkpoint::facts::payload::chunk::bytes (fn) |
| `primary_graph::application_checkpoint::facts::payload::concatenation::join` | primary_graph | execution_runtime | 31 | blocked: primary_graph::application_checkpoint::facts::payload::FactPayload (struct), primary_graph::application_checkpoint::facts::payload::concatenation::balance (fn), primary_graph::application_checkpoint::facts::payload::concatenation::node (fn) |
| `primary_graph::application_checkpoint::facts::payload::original_storage_bytes` | primary_graph | execution_runtime | 31 | blocked: primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact (enum) |
| `primary_graph::application_checkpoint::facts::payload::patch::preparation::build` | primary_graph | execution_runtime | 31 | blocked: primary_graph::application_checkpoint::facts::payload::FactPayload (struct), primary_graph::application_checkpoint::facts::payload::patch::PreparedNode (enum), primary_graph::application_checkpoint::facts::payload::patch::leaf::PreparedLeaf (struct) |
| `primary_graph::application_checkpoint::facts::payload::patch::preparation::count_nodes` | primary_graph | execution_runtime | 31 | blocked: primary_graph::application_checkpoint::facts::payload::FactPayload (struct), primary_graph::application_checkpoint::facts::payload::patch::preparation::children (fn), primary_graph::application_checkpoint::facts::payload::patch::preparation::intersects (fn) |
| `primary_graph::application_checkpoint::facts::payload::patch::take_published` | primary_graph | execution_runtime | 31 | blocked: primary_graph::application_checkpoint::facts::payload::FactPayload (struct), primary_graph::application_checkpoint::facts::payload::patch::PreparedNode (enum) |
| `primary_graph::application_checkpoint::facts::payload::resident_bytes` | primary_graph | execution_runtime | 31 | blocked: primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact (enum), primary_graph::fact_verification_observation::original_payload_size (fn) |
| `primary_graph::fact_verification_observation::native_patch_clone` | primary_graph | execution_runtime | 31 | blocked: primary_graph::fact_verification_observation::add (fn), primary_graph::fact_verification_observation::record (fn) |
| `primary_graph::fact_verification_observation::native_patch_path` | primary_graph | execution_runtime | 31 | blocked: primary_graph::fact_verification_observation::add (fn), primary_graph::fact_verification_observation::record (fn) |
| `primary_graph::fact_verification_observation::native_payload_chunk` | primary_graph | execution_runtime | 31 | blocked: primary_graph::fact_verification_observation::add (fn), primary_graph::fact_verification_observation::record (fn) |
| `primary_graph::fact_verification_observation::native_payload_fact_move` | primary_graph | execution_runtime | 31 | blocked: primary_graph::fact_verification_observation::add (fn), primary_graph::fact_verification_observation::record (fn) |
| `primary_graph::fact_verification_observation::native_payload_join` | primary_graph | execution_runtime | 31 | blocked: primary_graph::fact_verification_observation::add (fn), primary_graph::fact_verification_observation::record (fn) |
| `primary_graph::fact_verification_observation::native_payload_size` | primary_graph | execution_runtime | 31 | blocked: primary_graph::fact_verification_observation::add (fn), primary_graph::fact_verification_observation::record (fn) |
| `primary_graph::provider::relational_execution_denial::relational_execution_kind` | primary_graph | execution_runtime | 30 | could move |
| `primary_graph::application_attempt::retained_decision_facts::endpoints::heap_sort` | primary_graph | execution_runtime | 29 | blocked: primary_graph::application_attempt::retained_decision_facts::endpoints::sift (fn), primary_graph::application_attempt::retained_decision_facts::endpoints::swap (fn) |
| `primary_graph::application_attempt::retained_decision_facts::endpoints::union_visit` | primary_graph | execution_runtime | 29 | could move |
| `primary_graph::entity_resolution::entity_denial` | primary_graph | execution_runtime | 29 | could move |
| `primary_graph::freshness::comparison_observation` | primary_graph | execution_runtime | 29 | could move |
| `primary_graph::freshness::principal_description_is_portable` | primary_graph | execution_runtime | 29 | could move |
| `primary_graph::observations::observe_exact_principal_target` | primary_graph | execution_runtime | 29 | blocked: primary_graph::observations::observe_principal_identity (fn), primary_graph::observations::principal_adjacency_revision (fn), primary_graph::observations::stale_proof_denial (fn) |
| `primary_graph::observations::principal_mapping::observe_mapping` | primary_graph | execution_runtime | 29 | blocked: primary_graph::observations::authoritative (fn), primary_graph::observations::mapping_projection_scope (fn), primary_graph::observations::projected_field (fn) |
| `primary_graph::aggregate_observation::member_work::checkpoint_names` | primary_graph | execution_runtime | 28 | could move |
| `primary_graph::component_selection::key::write_entity` | primary_graph | execution_runtime | 28 | could move |
| `primary_graph::component_selection::key::write_member` | primary_graph | execution_runtime | 28 | blocked: primary_graph::component_selection::key::write_entity (fn) |
| `primary_graph::observations::authoritative` | primary_graph | execution_runtime | 28 | could move |
| `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::execution::allocation_control` | primary_graph | execution_runtime | 27 | could move |
| `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::execution::authority_cause::authority_execution` | primary_graph | execution_runtime | 27 | could move |
| `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::execution::control_kind` | primary_graph | execution_runtime | 27 | could move |
| `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::execution::execution_kind` | primary_graph | execution_runtime | 27 | could move |

## Remaining seams

### `authorization` ↔ `primary_graph`: 103 references up in 37 items, default **nest or merge**

Items: `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` (16), `primary_graph::authenticated_principal::WorthQueryAuthenticatedPrincipal` (11), `primary_graph::application_attempt::workflow_transition_program::actor_selection::WorkflowRecordedActorObservation` (10), `primary_graph::causal_inspection::application_inspection::target::WorthQueryApplicationInspectionTarget` (5), `primary_graph::application_attempt::workflow::actor::encoding::encode` (4), `primary_graph::source_facts::allocation::SourceFactsAllocation` (4), `primary_graph::application_attempt::fact::consult_observation` (4), `primary_graph::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding` (3)

- `primary_graph::provider::application_decision_fact::application_table::ApplicationFactTable` held by `primary_graph::provider::application_decision_fact::WorthQueryPrimaryGraphApplicationDecisionFact` in `impl ApplicationFactTable`
- `primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease` held by `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` in `impl WorthQueryApplicationSnapshotLease`
- `primary_graph::application_attempt::provider_binding::WorthQueryPreparedApplicationProviderAttempt` held by `primary_graph::application_attempt::provider_binding::effect_accumulator::WorthQueryRegisteredProviderEffects` in `impl WorthQueryPreparedApplicationProviderAttempt`
- `primary_graph::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding` held by `primary_graph::application_attempt::idempotency::encoding::encode_identity` in `impl WorthQueryApplicationIdempotencyBinding`
- `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` held by `primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial` in `impl WorthQueryPrimaryGraphLayout`
- `primary_graph::application_attempt::workflow_transition_program::actor_selection::WorkflowRecordedActorObservation` held by `primary_graph::application_attempt::workflow::actor::dependencies::WorkflowActorDependencies` in `impl WorkflowRecordedActorObservation`

### `provider_session` ↔ `primary_graph`: 22 references up in 9 items, default **contract**

Items: `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` (6), `primary_graph::causal_inspection::application_inspection::target::WorthQueryApplicationInspectionTarget` (4), `primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease` (3), `primary_graph::application_attempt::provider_execution::application_attempt_affinity::WorthQueryApplicationAttemptBasis` (2), `primary_graph::application_attempt::provider_execution::application_attempt_affinity::WorthQueryApplicationAttemptAffinity` (2), `primary_graph::provider::application_attempt_state::registered_reads::RegisteredPrimaryRead` (2), `primary_graph::source_facts::allocation::SourceFactsAllocation` (1), `primary_graph::provider::decision_facts::comparison::PrimaryDecisionReadSetComparison` (1)

- `primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease` held by `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` in `impl WorthQueryApplicationSnapshotLease`
- `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` held by `primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial` in `impl WorthQueryPrimaryGraphLayout`
- `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` held by `primary_graph::application_attempt::workflow::schema::layout::WorthQueryWorkflowLayout` in `declaration`
- `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` held by `primary_graph::application_discovery::workflow::indexes::WorkflowReadIndexes` in `declaration`
- `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` held by `primary_graph::schema_layout::capability_grant_join::WorthQueryCapabilityGrantJoinLayout` in `declaration`
- `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` held by `primary_graph::schema_layout::relation_payload::ApplicationRelationPayloadLayout` in `declaration`

### `application_aftermath` ↔ `primary_graph`: 8 references up in 3 items, default **contract**

Items: `primary_graph::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding` (4), `primary_graph::product_operation::correction_source::preparation::WorthQueryPreparedCorrectionSource` (3), `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenial` (1)

- `primary_graph::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding` held by `primary_graph::application_attempt::idempotency::encoding::encode_identity` in `impl WorthQueryApplicationIdempotencyBinding`
- `primary_graph::product_operation::correction_source::preparation::WorthQueryPreparedCorrectionSource` held by `primary_graph::application_attempt::workflow::correction::PreparedWorkflowCorrection` in `declaration`
- `primary_graph::product_operation::correction_source::preparation::WorthQueryPreparedCorrectionSource` held by `primary_graph::product_operation::correction_source::authored_identity::PreparedAuthoredIdentity` in `declaration`
- `primary_graph::product_operation::correction_source::preparation::WorthQueryPreparedCorrectionSource` held by `primary_graph::product_operation::correction_source::intent::WorthQueryCorrectionIntent` in `declaration`
- `primary_graph::product_operation::correction_source::preparation::WorthQueryPreparedCorrectionSource` held by `primary_graph::application_attempt::workflow::correction::PreparedWorkflowCorrection` in `impl WorthQueryPreparedCorrectionSource`
- `primary_graph::product_operation::correction_source::preparation::WorthQueryPreparedCorrectionSource` held by `primary_graph::product_operation::correction_source::authored_identity::PreparedAuthoredIdentity` in `impl WorthQueryPreparedCorrectionSource`

### `execution_runtime` ↔ `primary_graph`: 6 references up in 4 items, default **contract**

Items: `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` (2), `primary_graph::application_contribution::execution_denial` (2), `primary_graph::publication_request_denial::publication` (1), `primary_graph::root::WorthQueryPrimaryGraph` (1)

- `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` held by `primary_graph::output_lineage::invalidation::source_alignment::SnapshotAlignedMarkState` in `impl SourceInvalidationOwner`
- `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` held by `primary_graph::output_lineage::invalidation::settlement::SettlementRegistration` in `impl SourceInvalidationOwner`
- `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` held by `primary_graph::output_lineage::invalidation::output_availability::ObservedSourcePublication` in `impl SourceInvalidationOwner`
- `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` held by `primary_graph::output_lineage::source_facts::ComparableSourceFacts` in `impl SourceInvalidationOwner`
- `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` held by `primary_graph::output_lineage::invalidation::computation_changes::ComputationChanges` in `impl SourceInvalidationOwner`
- `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` held by `primary_graph::output_lineage::source_facts::RetainedSourceFacts` in `impl SourceInvalidationOwner`

### `operation_binding` ↔ `provider_session`: 5 references up in 2 items, default **contract**

Items: `provider_session::protocol::declared_closure::WorthQueryProviderPlanDeclarations` (4), `provider_session::protocol::plan_contract::WorthQueryProviderExecutionPlanContract` (1)

- `provider_session::protocol::plan_contract::WorthQueryProviderExecutionPlanContract` held by `provider_session::protocol::declared_closure::WorthQueryProviderPlanDeclarations` in `impl WorthQueryProviderExecutionPlanContract`
- `provider_session::protocol::declared_closure::WorthQueryProviderPlanDeclarations` held by `provider_session::protocol::declared_closure::aftermath_posture::reversal_posture` in `impl WorthQueryProviderPlanDeclarations`

### `provider_session` ↔ `managed_run`: 5 references up in 2 items, default **contract**

Items: `managed_run::direct::WorthQueryRunningDirectRun` (4), `managed_run::workflow::WorthQueryRunningWorkflowRun` (1)

- `managed_run::workflow::WorthQueryRunningWorkflowRun` held by `managed_run::workflow::terminal::owner::WorthQueryWorkflowRunTerminal` in `impl WorthQueryRunningWorkflowRun`
- `managed_run::direct::WorthQueryRunningDirectRun` held by `managed_run::run_affinity::WorthQueryDirectRunAffinity` in `declaration`
- `managed_run::direct::WorthQueryRunningDirectRun` held by `managed_run::direct_graph_execution::WorthQueryActiveDirectGraphExecution` in `impl WorthQueryRunningDirectRun`
- `managed_run::direct::WorthQueryRunningDirectRun` held by `managed_run::direct_graph_execution_start::begin` in `impl WorthQueryRunningDirectRun`
- `managed_run::workflow::WorthQueryRunningWorkflowRun` held by `managed_run::workflow::run_affinity::WorthQueryWorkflowRunAffinity` in `declaration`
- `managed_run::workflow::WorthQueryRunningWorkflowRun` held by `managed_run::workflow::run_affinity::WorthQueryWorkflowRunAffinity` in `impl WorthQueryRunningWorkflowRun`

### `product_publication` ↔ `primary_graph`: 2 references up in 1 items, default **contract**

Items: `primary_graph::application_contribution::execution_denial` (2)

- `primary_graph::application_contribution::execution_denial` held by `(module: not movable)`

### `artifact_owner` ↔ `operation_binding`: 1 references up in 1 items, default **contract**

Items: `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` (1)

- `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` held by `provider_session::protocol::declared_closure::WorthQueryProviderPlanDeclarations` in `impl WorthQueryExecutionBoundOperationAuthority`
- `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` held by `provider_session::protocol::declared_closure::WorthQueryProviderPlanDeclarations` in `declaration`
- `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` held by `operation_binding::authority::topology::operation_workflow_topology` in `impl WorthQueryExecutionBoundOperationAuthority`
- `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` held by `operation_binding::authority::topology::resource_topology` in `impl WorthQueryExecutionBoundOperationAuthority`
- `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` held by `provider_session::protocol::plan_contract::WorthQueryProviderExecutionPlanContract` in `impl WorthQueryExecutionBoundOperationAuthority`

### `execution_runtime` ↔ `invalidation_equivalence_control`: 1 references up in 1 items, default **contract**

Items: `invalidation_equivalence_control::attach_constructed_receiver` (1)


### `execution_runtime` ↔ `managed_run`: 1 references up in 1 items, default **contract**

Items: `managed_run::direct_admission::WorthQueryManagedRunAdmission` (1)

- `managed_run::direct_admission::WorthQueryManagedRunAdmission` held by `managed_run::lower_admission::admit_managed_lower_execution_basis` in `impl WorthQueryManagedRunAdmission<'_>`
- `managed_run::direct_admission::WorthQueryManagedRunAdmission` held by `managed_run::truth_read_request::WorthQueryManagedTruthReadRequest` in `impl WorthQueryManagedRunAdmission<'_>`
- `managed_run::direct_admission::WorthQueryManagedRunAdmission` held by `operation_binding::authority::WorthQueryExecutionBoundOperationAuthority` in `impl WorthQueryManagedRunAdmission<'_>`
- `managed_run::direct_admission::WorthQueryManagedRunAdmission` held by `managed_run::admission::validate_direct_run_head` in `impl WorthQueryManagedRunAdmission<'_>`
- `managed_run::direct_admission::WorthQueryManagedRunAdmission` held by `managed_run::admission::validate_direct_run_lower` in `impl WorthQueryManagedRunAdmission<'_>`
- `managed_run::direct_admission::WorthQueryManagedRunAdmission` held by `managed_run::direct::WorthQueryAdmittedDirectRun` in `impl WorthQueryManagedRunAdmission<'_>`

