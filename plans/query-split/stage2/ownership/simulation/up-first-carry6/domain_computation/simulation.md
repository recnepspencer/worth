# Helper-move simulation: `domain_computation` (21 modules)

Before: component 12, 581 references against the current layering.
After: component 9, 245 against the current layering, 239 against a fresh layering of the result.
Moved: 621 helpers and 452 units over 7 rounds. Of 1557 helpers blocking coupled units, 351 could move at once.

Remaining seams in the largest component: 11 (9 contract, 2 nest or merge).

## Helpers blocking coupled units

| helper | owner | destination | units blocked | verdict |
|---|---|---|---|---|
| `primary_graph::application_query::managed_index::provider_failure::equal_cause` | primary_graph | domain_evidence_binding | 71 | could move |
| `primary_graph::application_attempt::denial::request_read::relational_read_kind` | primary_graph | domain_evidence_binding | 68 | blocked: primary_graph::provider::relational_execution_denial::relational_execution_kind (fn) |
| `primary_graph::provider::relational_execution_denial::query_read_execution_kind` | primary_graph | domain_evidence_binding | 68 | blocked: primary_graph::provider::relational_execution_denial::relational_execution_kind (fn) |
| `primary_graph::application_contribution::conditional::output_readiness_contract::denial` | primary_graph | domain_evidence_binding | 55 | blocked: primary_graph::application_contribution::conditional::output_readiness_contract::WorthQueryOutputReadinessContractDenial (struct) |
| `primary_graph::schema_layout::installation_primitives::planned_field_locator` | primary_graph | domain_evidence_binding | 55 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::valid_aspect_key (fn), primary_graph::schema_layout::installation_primitives::valid_field_key (fn) |
| `primary_graph::application_attempt::retained_decision_facts::endpoints::heap_sort` | primary_graph | domain_evidence_binding | 54 | blocked: primary_graph::application_attempt::retained_decision_facts::endpoints::sift (fn), primary_graph::application_attempt::retained_decision_facts::endpoints::swap (fn) |
| `primary_graph::application_attempt::retained_decision_facts::endpoints::union_visit` | primary_graph | domain_evidence_binding | 54 | could move |
| `primary_graph::freshness::comparison_observation` | primary_graph | domain_evidence_binding | 54 | could move |
| `primary_graph::freshness::principal_description_is_portable` | primary_graph | domain_evidence_binding | 54 | could move |
| `primary_graph::observations::observe_exact_principal_target` | primary_graph | domain_evidence_binding | 54 | blocked: primary_graph::observations::observe_principal_identity (fn), primary_graph::observations::principal_adjacency_revision (fn), primary_graph::observations::stale_proof_denial (fn) |
| `primary_graph::observations::principal_mapping::observe_mapping` | primary_graph | domain_evidence_binding | 54 | blocked: primary_graph::observations::authoritative (fn), primary_graph::observations::mapping_projection_scope (fn), primary_graph::observations::projected_field (fn) |
| `primary_graph::application_attempt::observation::observe_field_value` | primary_graph | domain_evidence_binding | 53 | blocked: primary_graph::application_attempt::observation::observe_field (fn) |
| `primary_graph::observations::authoritative` | primary_graph | domain_evidence_binding | 53 | could move |
| `primary_graph::schema_layout::aggregate_identity::lower` | primary_graph | domain_evidence_binding | 53 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::aggregate_identity::component_layout::retain (fn), primary_graph::schema_layout::aggregate_identity::definitions::collect (fn) |
| `primary_graph::schema_layout::application_layout_lowering::field_capability_keys` | primary_graph | domain_evidence_binding | 53 | could move |
| `primary_graph::schema_layout::application_layout_lowering::lower_fields` | primary_graph | domain_evidence_binding | 53 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::planned_field_locator (fn), primary_graph::schema_layout::installation_primitives::required_kind (fn) |
| `primary_graph::schema_layout::application_layout_lowering::lower_relation_layouts` | primary_graph | domain_evidence_binding | 53 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::required_kind (fn) |
| `primary_graph::schema_layout::authorization_selectors::insert` | primary_graph | domain_evidence_binding | 53 | could move |
| `primary_graph::schema_layout::capability_grant_join::lower_capability_grant_joins` | primary_graph | domain_evidence_binding | 53 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::capability_grant_join::WorthQueryCapabilityGrantJoinLayout (struct) |
| `primary_graph::schema_layout::capability_grant_join::required_relation` | primary_graph | domain_evidence_binding | 53 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::invalid_member (fn) |
| `primary_graph::schema_layout::content::lower` | primary_graph | domain_evidence_binding | 53 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::content::refusal (fn) |
| `primary_graph::schema_layout::continuation_ordering::lower_continuation_orderings` | primary_graph | domain_evidence_binding | 53 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::invalid_member (fn), primary_graph::schema_layout::installation_primitives::planned_field_locator (fn) |
| `primary_graph::schema_layout::derived_relation::lower` | primary_graph | domain_evidence_binding | 53 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::aggregate_identity::definitions::derived::orbit_identity (fn), primary_graph::schema_layout::derived_relation::expression::lower (fn) |
| `primary_graph::schema_layout::installation_primitives::invalid_member` | primary_graph | domain_evidence_binding | 53 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct) |
| `primary_graph::schema_layout::installation_primitives::required_kind` | primary_graph | domain_evidence_binding | 53 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::invalid_member (fn) |
| `primary_graph::schema_layout::live_control_roles::lower` | primary_graph | domain_evidence_binding | 53 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::required_kind (fn) |
| `primary_graph::schema_layout::managed_source::lower` | primary_graph | domain_evidence_binding | 53 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::required_kind (fn), primary_graph::schema_layout::managed_source::membership::path (fn) |
| `primary_graph::schema_layout::platform_entity_lowering::lower_platform_entities` | primary_graph | domain_evidence_binding | 53 | blocked: primary_graph::application_attempt::workflow::schema::lower_workflow (fn), primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::kind_space_exhausted (fn) |
| `primary_graph::schema_layout::principal_binding::lower_principal_bindings` | primary_graph | domain_evidence_binding | 53 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::contract_space_exhausted (fn), primary_graph::schema_layout::installation_primitives::planned_field_locator (fn) |
| `primary_graph::schema_layout::projected_member_domain::lower` | primary_graph | domain_evidence_binding | 53 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::invalid_member (fn), primary_graph::schema_layout::installation_primitives::required_kind (fn) |
| `primary_graph::schema_layout::registry_lowering::lower_application_contract_bindings` | primary_graph | domain_evidence_binding | 53 | could move |
| `primary_graph::schema_layout::registry_lowering::lower_kind_ids` | primary_graph | domain_evidence_binding | 53 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::kind_space_exhausted (fn), primary_graph::schema_layout::registry_lowering::next_kind_id (fn) |
| `primary_graph::schema_layout::registry_lowering::next_provider_kind_id` | primary_graph | domain_evidence_binding | 53 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::kind_space_exhausted (fn), primary_graph::schema_layout::registry_lowering::next_kind_id (fn) |
| `primary_graph::schema_layout::registry_lowering::register_application_relation` | primary_graph | domain_evidence_binding | 53 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::registry_lowering::register_relation_to_kinds_with_contracts (fn) |
| `primary_graph::schema_layout::registry_lowering::register_entity` | primary_graph | domain_evidence_binding | 53 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::relational_schema_denial (fn) |
| `primary_graph::schema_layout::registry_lowering::relational_schema_basis` | primary_graph | domain_evidence_binding | 53 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::relational_schema_denial (fn) |
| `primary_graph::schema_layout::signed_entities::lower` | primary_graph | domain_evidence_binding | 53 | blocked: primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial (struct), primary_graph::schema_layout::installation_primitives::required_kind (fn) |
| `primary_graph::application_attempt::workflow::actor::record::actor_record_corruption` | primary_graph | domain_evidence_binding | 52 | blocked: primary_graph::application_attempt::denial::WorthQueryApplicationAttemptDenial (struct) |
| `primary_graph::application_attempt::workflow::actor::record::decode_actor_components` | primary_graph | domain_evidence_binding | 52 | could move |
| `primary_graph::application_checkpoint::facts::payload::array_storage_bytes` | primary_graph | domain_evidence_binding | 52 | could move |
| `primary_graph::application_checkpoint::facts::payload::chunk::bytes` | primary_graph | domain_evidence_binding | 52 | could move |
| `primary_graph::application_checkpoint::facts::payload::chunk::deep_clone_bytes` | primary_graph | domain_evidence_binding | 52 | blocked: primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact (enum), primary_graph::application_checkpoint::facts::payload::chunk::bytes (fn) |
| `primary_graph::application_checkpoint::facts::payload::concatenation::join` | primary_graph | domain_evidence_binding | 52 | blocked: primary_graph::application_checkpoint::facts::payload::FactPayload (struct), primary_graph::application_checkpoint::facts::payload::concatenation::balance (fn), primary_graph::application_checkpoint::facts::payload::concatenation::node (fn) |
| `primary_graph::application_checkpoint::facts::payload::original_storage_bytes` | primary_graph | domain_evidence_binding | 52 | blocked: primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact (enum) |
| `primary_graph::application_checkpoint::facts::payload::patch::preparation::build` | primary_graph | domain_evidence_binding | 52 | blocked: primary_graph::application_checkpoint::facts::payload::FactPayload (struct), primary_graph::application_checkpoint::facts::payload::patch::PreparedNode (enum), primary_graph::application_checkpoint::facts::payload::patch::leaf::PreparedLeaf (struct) |
| `primary_graph::application_checkpoint::facts::payload::patch::preparation::count_nodes` | primary_graph | domain_evidence_binding | 52 | blocked: primary_graph::application_checkpoint::facts::payload::FactPayload (struct), primary_graph::application_checkpoint::facts::payload::patch::preparation::children (fn), primary_graph::application_checkpoint::facts::payload::patch::preparation::intersects (fn) |
| `primary_graph::application_checkpoint::facts::payload::patch::take_published` | primary_graph | domain_evidence_binding | 52 | blocked: primary_graph::application_checkpoint::facts::payload::FactPayload (struct), primary_graph::application_checkpoint::facts::payload::patch::PreparedNode (enum) |
| `primary_graph::application_checkpoint::facts::payload::resident_bytes` | primary_graph | domain_evidence_binding | 52 | blocked: primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact (enum), primary_graph::fact_verification_observation::original_payload_size (fn) |
| `primary_graph::fact_verification_observation::native_patch_clone` | primary_graph | domain_evidence_binding | 52 | blocked: primary_graph::fact_verification_observation::add (fn), primary_graph::fact_verification_observation::record (fn) |
| `primary_graph::fact_verification_observation::native_patch_path` | primary_graph | domain_evidence_binding | 52 | blocked: primary_graph::fact_verification_observation::add (fn), primary_graph::fact_verification_observation::record (fn) |
| `primary_graph::fact_verification_observation::native_payload_chunk` | primary_graph | domain_evidence_binding | 52 | blocked: primary_graph::fact_verification_observation::add (fn), primary_graph::fact_verification_observation::record (fn) |
| `primary_graph::fact_verification_observation::native_payload_fact_move` | primary_graph | domain_evidence_binding | 52 | blocked: primary_graph::fact_verification_observation::add (fn), primary_graph::fact_verification_observation::record (fn) |
| `primary_graph::fact_verification_observation::native_payload_join` | primary_graph | domain_evidence_binding | 52 | blocked: primary_graph::fact_verification_observation::add (fn), primary_graph::fact_verification_observation::record (fn) |
| `primary_graph::fact_verification_observation::native_payload_size` | primary_graph | domain_evidence_binding | 52 | blocked: primary_graph::fact_verification_observation::add (fn), primary_graph::fact_verification_observation::record (fn) |
| `primary_graph::provider::relational_execution_denial::relational_execution_kind` | primary_graph | domain_evidence_binding | 52 | could move |
| `primary_graph::entity_resolution::entity_denial` | primary_graph | domain_evidence_binding | 51 | could move |
| `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::execution::allocation_control` | primary_graph | domain_evidence_binding | 50 | could move |
| `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::execution::authority_cause::authority_execution` | primary_graph | domain_evidence_binding | 50 | could move |
| `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::execution::control_kind` | primary_graph | domain_evidence_binding | 50 | could move |
| `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::execution::execution_kind` | primary_graph | domain_evidence_binding | 50 | could move |

## Remaining seams

### `authorization` ↔ `primary_graph`: 128 references up in 45 items, default **nest or merge**

Items: `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` (16), `primary_graph::authenticated_principal::WorthQueryAuthenticatedPrincipal` (11), `primary_graph::application_attempt::workflow_transition_program::actor_selection::WorkflowRecordedActorObservation` (10), `primary_graph::application_attempt::precondition_binding::WorthQueryBoundMutationPreconditions` (8), `primary_graph::application_attempt::elevation_approval_outcome::WorthQueryApprovedElevation` (6), `primary_graph::causal_inspection::application_inspection::target::WorthQueryApplicationInspectionTarget` (5), `primary_graph::application_attempt::workflow::actor::encoding::encode` (4), `primary_graph::application_attempt::elevation_request_outcome::WorthQueryRequestedElevation` (4)

- `primary_graph::application_contribution::partitioned_computation::incremental::retained::SealedComputationRun` held by `primary_graph::application_contribution::partitioned_computation::incremental::retained::RetainedComputation` in `impl SealedComputationRun`
- `primary_graph::provider::application_decision_fact::application_table::ApplicationFactTable` held by `primary_graph::provider::application_decision_fact::WorthQueryPrimaryGraphApplicationDecisionFact` in `impl ApplicationFactTable`
- `primary_graph::application_attempt::provider_binding::WorthQueryPreparedApplicationProviderAttempt` held by `primary_graph::application_attempt::provider_binding::effect_accumulator::WorthQueryRegisteredProviderEffects` in `impl WorthQueryPreparedApplicationProviderAttempt`
- `primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease` held by `primary_graph::root::WorthQueryPrimaryGraphIntegrationHandle` in `impl WorthQueryApplicationSnapshotLease`
- `primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease` held by `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` in `impl WorthQueryApplicationSnapshotLease`
- `primary_graph::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding` held by `primary_graph::application_attempt::idempotency::encoding::encode_identity` in `impl WorthQueryApplicationIdempotencyBinding`

### `convergence_epoch` ↔ `managed_run`: 51 references up in 32 items, default **nest or merge**

Items: `managed_run::workflow_graph_execution::WorthQueryActiveWorkflowGraphExecution` (5), `managed_run::direct_graph_execution::WorthQueryActiveDirectGraphExecution` (5), `managed_run::direct::WorthQueryAdmittedDirectRun` (3), `managed_run::direct_terminal::owner::WorthQueryDirectRunTerminal` (3), `managed_run::workflow::terminal::owner::WorthQueryWorkflowRunTerminal` (3), `managed_run::direct_graph_chunk::WorthQueryPendingDirectGraphChunk` (2), `managed_run::direct::WorthQueryRunningDirectRun` (2), `managed_run::direct_yield_recovery::WorthQueryDirectYieldRecoveryRequired` (2)

- `managed_run::direct_graph_execution::WorthQueryActiveDirectGraphExecution` held by `managed_run::direct_graph_execution::WorthQueryDirectGraphStepOutcome` in `impl WorthQueryActiveDirectGraphExecution`
- `managed_run::direct_graph_execution::WorthQueryActiveDirectGraphExecution` held by `managed_run::direct_graph_terminalization::terminal_outcome` in `impl WorthQueryActiveDirectGraphExecution`
- `managed_run::direct::WorthQueryRunningDirectRun` held by `managed_run::direct_terminal::owner::WorthQueryDirectRunTerminal` in `impl WorthQueryRunningDirectRun`
- `managed_run::direct::WorthQueryRunningDirectRun` held by `provider_session::attempt_evidence::WorthQueryExecutionResourceAttemptEvidence` in `impl WorthQueryRunningDirectRun`
- `managed_run::workflow::WorthQueryRunningWorkflowRun` held by `managed_run::workflow::terminal::owner::WorthQueryWorkflowRunTerminal` in `impl WorthQueryRunningWorkflowRun`
- `managed_run::workflow::WorthQueryRunningWorkflowRun` held by `provider_session::attempt_evidence::WorthQueryExecutionResourceAttemptEvidence` in `impl WorthQueryRunningWorkflowRun`

### `provider_session` ↔ `primary_graph`: 23 references up in 10 items, default **contract**

Items: `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` (6), `primary_graph::causal_inspection::application_inspection::target::WorthQueryApplicationInspectionTarget` (4), `primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease` (3), `primary_graph::application_attempt::provider_execution::application_attempt_affinity::WorthQueryApplicationAttemptBasis` (2), `primary_graph::application_attempt::provider_execution::application_attempt_affinity::WorthQueryApplicationAttemptAffinity` (2), `primary_graph::provider::application_attempt_state::registered_reads::RegisteredPrimaryRead` (2), `primary_graph::source_facts::allocation::SourceFactsAllocation` (1), `primary_graph::root::WorthQueryPrimaryGraphIntegrationHandle` (1)

- `primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease` held by `primary_graph::root::WorthQueryPrimaryGraphIntegrationHandle` in `impl WorthQueryApplicationSnapshotLease`
- `primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease` held by `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` in `impl WorthQueryApplicationSnapshotLease`
- `primary_graph::root::WorthQueryPrimaryGraphIntegrationHandle` held by `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` in `impl WorthQueryPrimaryGraphIntegrationHandle`
- `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` held by `primary_graph::denial::WorthQueryPrimaryGraphInstallationDenial` in `impl WorthQueryPrimaryGraphLayout`
- `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` held by `primary_graph::application_attempt::workflow::schema::layout::WorthQueryWorkflowLayout` in `declaration`
- `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` held by `primary_graph::application_discovery::workflow::indexes::WorkflowReadIndexes` in `declaration`

### `application_aftermath` ↔ `primary_graph`: 9 references up in 4 items, default **contract**

Items: `primary_graph::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding` (4), `primary_graph::product_operation::correction_source::preparation::WorthQueryPreparedCorrectionSource` (3), `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenial` (1), `primary_graph::provider::mutation_work::WorthQueryPrimaryMutationWorkEvidence` (1)

- `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenial` held by `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::denial_cause::DenialCause` in `impl WorthQueryApplicationCommitDenial`
- `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenial` held by `provider_session::provisional_attempt::invariant_execution::denial::WorthQueryCustomInvariantDenial` in `impl WorthQueryApplicationCommitDenial`
- `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenial` held by `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::denial_cause::DenialCause` in `impl Denial`
- `primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenial` held by `provider_session::provisional_attempt::invariant_execution::denial::WorthQueryInvariantExecutionFailure` in `impl WorthQueryApplicationCommitDenial`
- `primary_graph::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding` held by `primary_graph::application_attempt::idempotency::encoding::encode_identity` in `impl WorthQueryApplicationIdempotencyBinding`
- `primary_graph::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding` held by `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation` in `impl WorthQueryApplicationIdempotencyBinding`

### `provider_session` ↔ `managed_run`: 7 references up in 4 items, default **contract**

Items: `managed_run::direct::WorthQueryRunningDirectRun` (4), `managed_run::direct_terminal::cleanup::WorthQueryDirectRunCleanupFailure` (1), `managed_run::workflow::WorthQueryRunningWorkflowRun` (1), `managed_run::direct_terminal::inspection::WorthQueryDirectRunCleanupReceipt` (1)

- `managed_run::direct::WorthQueryRunningDirectRun` held by `managed_run::direct_terminal::owner::WorthQueryDirectRunTerminal` in `impl WorthQueryRunningDirectRun`
- `managed_run::direct::WorthQueryRunningDirectRun` held by `provider_session::attempt_evidence::WorthQueryExecutionResourceAttemptEvidence` in `impl WorthQueryRunningDirectRun`
- `managed_run::workflow::WorthQueryRunningWorkflowRun` held by `managed_run::workflow::terminal::owner::WorthQueryWorkflowRunTerminal` in `impl WorthQueryRunningWorkflowRun`
- `managed_run::workflow::WorthQueryRunningWorkflowRun` held by `provider_session::attempt_evidence::WorthQueryExecutionResourceAttemptEvidence` in `impl WorthQueryRunningWorkflowRun`
- `managed_run::workflow::WorthQueryRunningWorkflowRun` held by `provider_session::session_identity::WorthQueryExecutionProviderSession` in `impl WorthQueryRunningWorkflowRun`
- `managed_run::direct::WorthQueryRunningDirectRun` held by `managed_run::run_affinity::WorthQueryDirectRunAffinity` in `declaration`

### `execution_runtime` ↔ `primary_graph`: 6 references up in 4 items, default **contract**

Items: `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` (2), `primary_graph::application_contribution::execution_denial` (2), `primary_graph::root::WorthQueryPrimaryGraphIntegrationHandle` (1), `primary_graph::root::WorthQueryPrimaryGraph` (1)

- `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` held by `primary_graph::output_lineage::invalidation::source_alignment::SnapshotAlignedMarkState` in `impl SourceInvalidationOwner`
- `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` held by `primary_graph::output_lineage::invalidation::settlement::SettlementRegistration` in `impl SourceInvalidationOwner`
- `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` held by `primary_graph::output_lineage::invalidation::output_availability::ObservedSourcePublication` in `impl SourceInvalidationOwner`
- `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` held by `primary_graph::output_lineage::source_facts::ComparableSourceFacts` in `impl SourceInvalidationOwner`
- `primary_graph::root::WorthQueryPrimaryGraphIntegrationHandle` held by `primary_graph::schema_layout::WorthQueryPrimaryGraphLayout` in `impl WorthQueryPrimaryGraphIntegrationHandle`
- `primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner` held by `primary_graph::output_lineage::invalidation::computation_changes::ComputationChanges` in `impl SourceInvalidationOwner`

### `convergence_epoch` ↔ `provider_session`: 5 references up in 2 items, default **contract**

Items: `provider_session::graph_provider::receipt::WorthQueryBoundGraphExecutionReceipt` (3), `provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor` (2)

- `provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor` held by `provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryErasedGraphParticipationProvider` in `declaration`
- `provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor` held by `provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryErasedProviderSessionLifecycle` in `declaration`
- `provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor` held by `provider_session::graph_provider::bounded_step::semantic_provider_ports::WorthQueryErasedDecisionFactProvider` in `declaration`
- `provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor` held by `provider_session::graph_provider::bounded_step::semantic_provider_ports::WorthQueryErasedInvariantExecutionProvider` in `declaration`
- `provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor` held by `provider_session::graph_provider::bounded_step::semantic_provider_ports::WorthQueryErasedProvisionalGraphProvider` in `declaration`
- `provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor` held by `provider_session::decision_read_set::fact::WorthQueryDecisionFactAdmission` in `impl WorthQueryGraphProviderAnchor`

### `product_publication` ↔ `primary_graph`: 5 references up in 2 items, default **contract**

Items: `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` (3), `primary_graph::application_contribution::execution_denial` (2)

- `primary_graph::application_contribution::execution_denial` held by `(module: not movable)`
- `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` held by `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyStore` in `declaration`
- `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` held by `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyStore` in `impl WorthQueryUnpublishedIdempotencyDisposition`
- `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` held by `primary_graph::provider::unpublished_idempotency::release::WorthQueryUnpublishedReleaseReservation` in `impl WorthQueryUnpublishedIdempotencyDisposition`

### `application_aftermath` ↔ `authorization`: 4 references up in 2 items, default **contract**

Items: `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation` (3), `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::capability_admission::preparation::observation::admitted_access::WorthQueryAdmittedApplicationCapabilityAccess` (1)

- `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation` held by `authorization::decision_facts::WorthQueryRetainedAuthorizationDecisionFacts` in `impl<Schema, Operation, Input, Scope> WorthQueryAdmittedApplicationOperation<Sch`
- `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation` held by `authorization::decision_facts::provider_commit_authorization::WorthQueryProviderCommitAuthorization` in `impl<Schema, Operation, Input, Scope> WorthQueryAdmittedApplicationOperation<Sch`
- `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation` held by `authorization::elevation_progression::approval::binding::WorthQueryElevationApprovalBinding` in `impl<Schema, Operation, Input, Scope> WorthQueryAdmittedApplicationOperation<Sch`
- `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation` held by `authorization::elevation_progression::elevation_close::binding::WorthQueryElevationCloseBinding` in `impl<Schema, Operation, Input, Scope> WorthQueryAdmittedApplicationOperation<Sch`
- `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation` held by `authorization::elevation_progression::mandatory_review::binding::WorthQueryMandatoryReviewBinding` in `impl<Schema, Operation, Input, Scope> WorthQueryAdmittedApplicationOperation<Sch`
- `authorization::operation_progression::authority_validation::precondition_binding::transition::admitted_operation::WorthQueryAdmittedApplicationOperation` held by `authorization::elevation_progression::request_binding::WorthQueryElevationRequestBinding` in `impl<Schema, Operation, Input, Scope> WorthQueryAdmittedApplicationOperation<Sch`

### `execution_runtime` ↔ `product_publication`: 1 references up in 1 items, default **contract**

Items: `product_publication::unpublished::WorthQueryProductUnpublishedApplication` (1)

- `product_publication::unpublished::WorthQueryProductUnpublishedApplication` held by `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` in `declaration`
- `product_publication::unpublished::WorthQueryProductUnpublishedApplication` held by `primary_graph::provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` in `impl WorthQueryProductUnpublishedApplication`

### `execution_runtime` ↔ `managed_run`: 1 references up in 1 items, default **contract**

Items: `managed_run::direct_admission::WorthQueryManagedRunAdmission` (1)

- `managed_run::direct_admission::WorthQueryManagedRunAdmission` held by `managed_run::lower_admission::admit_managed_lower_execution_basis` in `impl WorthQueryManagedRunAdmission<'_>`
- `managed_run::direct_admission::WorthQueryManagedRunAdmission` held by `managed_run::admission::validate_direct_run_head` in `impl WorthQueryManagedRunAdmission<'_>`
- `managed_run::direct_admission::WorthQueryManagedRunAdmission` held by `managed_run::admission::validate_direct_run_lower` in `impl WorthQueryManagedRunAdmission<'_>`
- `managed_run::direct_admission::WorthQueryManagedRunAdmission` held by `managed_run::direct::WorthQueryAdmittedDirectRun` in `impl WorthQueryManagedRunAdmission<'_>`
- `managed_run::direct_admission::WorthQueryManagedRunAdmission` held by `managed_run::direct_admission_failure::WorthQueryManagedDirectRunAdmissionFailure` in `impl WorthQueryManagedRunAdmission<'_>`
- `managed_run::direct_admission::WorthQueryManagedRunAdmission` held by `managed_run::lower_admission::admit_atomic_lower_execution_basis` in `impl WorthQueryManagedRunAdmission<'_>`

