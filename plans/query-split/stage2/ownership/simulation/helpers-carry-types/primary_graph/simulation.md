# Helper-move simulation: `domain_computation::primary_graph` (80 modules)

Before: component 59, 1506 references against the current layering.
After: component 57, 395 against the current layering, 520 against a fresh layering of the result.
Moved: 440 helpers and 628 units over 5 rounds. Of 1224 helpers blocking coupled units, 276 could move at once.

Remaining seams in the largest component: 63 (62 contract, 1 nest or merge).

## Helpers blocking coupled units

| helper | owner | destination | units blocked | verdict |
|---|---|---|---|---|
| `application_query::managed_index::provider_failure::equal_cause` | application_query | application_program | 145 | could move |
| `application_contribution::conditional::output_readiness_contract::denial` | application_contribution | denial | 120 | blocked: application_contribution::conditional::output_readiness_contract::WorthQueryOutputReadinessContractDenial (struct) |
| `application_contribution::native_field_contract::field_denial` | application_contribution | application_program | 118 | could move |
| `application_contribution::conditional::output_readiness_contract::typed::field_denial` | application_contribution | application_program | 117 | blocked: application_contribution::conditional::output_readiness_contract::WorthQueryOutputReadinessContractDenial (struct) |
| `application_contribution::producer::denial_value::mismatch_or` | application_contribution | application_program | 117 | could move |
| `application_contribution::producer::denial_value::declaration::encode_declared` | application_contribution | application_program | 114 | blocked: application_contribution::producer::denial_value::WorthQueryProducerDenialValue (enum), application_invariant::refusal::canonical_wire_maximum (fn) |
| `application_contribution::producer::registry::semantic_edition::posture_code` | application_contribution | application_program | 112 | could move |
| `application_query::resource_lifecycle::lifecycle_count::release` | application_query | application_program | 107 | could move |
| `application_query::observed_source::source_identity::material_storage::admit_footprint_comparison` | application_query | application_program | 106 | blocked: application_query::observed_source::WorthQueryObservedSourceFootprint (struct) |
| `application_contribution::producer::demand::progression::required_wave::capacity_denial` | application_contribution | application_program | 104 | blocked: application_contribution::producer::demand::progression::required_wave::denial (fn), application_output_demand::denial::WorthQueryOutputDemandDenial (struct), application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind (enum) |
| `application_query::one_shot::denial::authorization_denial` | application_query | application_program | 100 | blocked: application_query::one_shot::denial::OneShotDenialPayload (struct), application_query::one_shot::denial::WorthQueryApplicationOneShotDenial (struct), application_query::one_shot::denial::denial (fn) |
| `application_query::one_shot::denial::denial` | application_query | application_program | 100 | blocked: application_query::one_shot::denial::OneShotDenialPayload (struct), application_query::one_shot::denial::WorthQueryApplicationOneShotDenial (struct) |
| `invariant_projection::consumed_output::verification::work_budget::map_admission_stop` | invariant_projection | application_program | 98 | blocked: invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop (enum) |
| `application_query::component_result::inspect` | application_query | application_program | 97 | blocked: application_query::component_result::WorthQueryApplicationQueryComponents (struct), application_query::observed_source::result_set::WorthQueryObservedResultSet (struct) |
| `application_query::observed_source::source_identity::checkpoint_commitment::checkpoint_source_identity` | application_query | application_program | 97 | blocked: application_query::observed_source::WorthQueryObservedSourceFootprint (struct), application_query::observed_source::source_identity::checkpoint_commitment::checkpoint_source_identity_admitted (fn) |
| `application_query::orbit_result::inspect` | application_query | application_program | 97 | blocked: application_query::observed_source::result_set::WorthQueryObservedResultSet (struct) |
| `application_query::projection::projected_tree::content_comparison::same_relation` | application_query | application_program | 97 | blocked: application_query::projection::projected_tree::content_comparison::same_relation_header (fn), application_query::projection::projected_tree::content_comparison::same_row (fn) |
| `application_query::projection::projected_tree::content_comparison::same_relation_header` | application_query | application_program | 97 | blocked: application_query::projection::projected_tree::content_comparison::same_text (fn) |
| `application_query::projection::projected_tree::content_comparison::same_row` | application_query | application_program | 97 | blocked: application_query::projection::projected_tree::content_comparison::same_text (fn), application_query::projection::projected_tree::content_comparison::same_value (fn) |
| `application_query::projection::projected_tree::content_comparison::traversal::frames` | application_query | application_program | 97 | could move |
| `application_query::projection::projection_denial` | application_query | application_program | 96 | could move |
| `application_query::projection::relation_cardinality_denial` | application_query | application_program | 96 | blocked: application_query::projection::projection_denial (fn) |
| `application_contribution::producer::demand::disclosure::denial` | application_contribution | application_program | 95 | blocked: application_output_demand::denial::WorthQueryOutputDemandDenial (struct), application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind (enum) |
| `application_contribution::producer::demand::disclosure::required_successor::validate_required_successor` | application_contribution | application_program | 95 | blocked: application_contribution::producer::demand::WorthQueryAdmittedOutputDemand (struct), application_contribution::producer::demand::disclosure::denial (fn), application_contribution::producer::demand::disclosure::validated::FreshOutputDisclosure (struct) |
| `application_contribution::producer::demand::disclosure::validate_disclosure` | application_contribution | application_program | 95 | blocked: application_contribution::host::ApplicationContributionHost (trait), application_contribution::producer::demand::WorthQueryAdmittedOutputDemand (struct), application_contribution::producer::demand::disclosure::validate_source (fn) |
| `application_contribution::producer::demand::own_effect_observation::take` | application_contribution | application_program | 95 | could move |
| `application_contribution::producer::demand::progression::denial` | application_contribution | application_program | 95 | blocked: application_output_demand::denial::WorthQueryOutputDemandDenial (struct), application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind (enum) |
| `application_contribution::producer::demand::progression::required_wave::denial` | application_contribution | application_program | 95 | blocked: application_output_demand::denial::WorthQueryOutputDemandDenial (struct), application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind (enum) |
| `application_contribution::producer::demand::progression::required_wave::performed::member_keys::same_key` | application_contribution | application_program | 95 | blocked: application_output_demand::registry::demand_key::WorthQueryOutputDemandKey (struct) |
| `application_contribution::producer::demand::progression::required_wave::required_settlement_denial` | application_contribution | application_program | 95 | blocked: application_contribution::producer::demand::progression::required_wave::admission_denial (fn), application_contribution::producer::demand::progression::required_wave::foreign_denial (fn), application_output_demand::denial::WorthQueryOutputDemandDenial (struct) |
| `application_contribution::producer::demand::progression::required_wave::selection::select_required_basis` | application_contribution | application_program | 95 | blocked: application_contribution::host::ApplicationContributionHost (trait), application_contribution::producer::demand::progression::required_wave::admission_denial (fn), application_contribution::producer::demand::progression::required_wave::position_denial (fn) |
| `application_contribution::producer::demand::progression::retained_read::authorize_history_source` | application_contribution | application_program | 95 | blocked: application_contribution::host::ApplicationContributionHost (trait), application_contribution::producer::demand::progression::retained_read::history_permission_denial (fn), application_output_demand::denial::WorthQueryOutputDemandDenial (struct) |
| `application_contribution::producer::demand::required_continuations::capacity_denial` | application_contribution | application_program | 95 | blocked: application_output_demand::denial::WorthQueryOutputDemandDenial (struct), application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind (enum) |
| `application_contribution::producer::demand::required_continuations::held::held_key` | application_contribution | application_program | 95 | blocked: application_output_demand::registry::demand_key::WorthQueryOutputDemandKey (struct) |
| `application_contribution::producer::demand::required_continuations::progress::refused_custody` | application_contribution | application_program | 95 | blocked: application_output_demand::denial::WorthQueryOutputDemandDenial (struct), application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind (enum) |
| `application_contribution::producer::demand::required_provenance::validation::EDITION_SUBJECT` | application_contribution | application_program | 95 | could move |
| `application_contribution::producer::demand::required_provenance::validation::MODE_SUBJECT` | application_contribution | application_program | 95 | could move |
| `application_contribution::producer::execution::denial::denial` | application_contribution | application_program | 95 | blocked: application_output_demand::denial::WorthQueryOutputDemandDenial (struct), application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind (enum) |
| `application_contribution::producer::execution::source_readmission::ready::ready_currentness_denial` | application_contribution | application_program | 95 | blocked: application_contribution::producer::execution::denial::ProducerExecutionStop (enum), application_output_demand::denial::WorthQueryOutputDemandDenial (struct), application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind (enum) |
| `application_contribution::producer::invocation::confirm_history_source` | application_contribution | application_program | 95 | blocked: application_contribution::host::ApplicationContributionHost (trait), application_contribution::producer::invocation::history_source_denial (fn), application_output_demand::denial::WorthQueryOutputDemandDenial (struct) |
| `application_contribution::partitioned_computation::incremental::observed::observe_retained_full_tree` | application_contribution | application_program | 94 | blocked: application_contribution::partitioned_computation::incremental::observed::observe_tree (fn), application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause (enum), application_contribution::partitioned_computation::incremental::tree_report::WorthQueryPartitionedTreeRun (enum) |
| `application_query::observed_source::source_identity::membership::same_footprint_membership` | application_query | application_program | 94 | blocked: application_query::observed_source::WorthQueryObservedSourceFootprint (struct), application_query::observed_source::source_identity::membership::same_adjacencies (fn), application_query::observed_source::source_identity::membership::same_entities (fn) |
| `invariant_projection::consumed_output::verification::work_budget::reserve_pending` | invariant_projection | application_program | 94 | blocked: invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop (enum) |
| `invariant_projection::operation_reader::output_availability::denial::demand_stop::meter` | invariant_projection | application_program | 94 | blocked: application_output_demand::denial::WorthQueryOutputDemandDenial (struct), output_reuse::selection::preflight_stop (fn), output_reuse::selection::retained_fact_denial (fn) |
| `invariant_projection::operation_reader::output_availability::denial::demand_stop::query_stop` | invariant_projection | application_program | 94 | blocked: application_contribution::execution_denial::advancement_scope_stop (fn), application_output_demand::denial::WorthQueryOutputDemandDenial (struct), application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind (enum) |
| `application_contribution::producer::demand::bridge_denial::bridge_denial` | application_contribution | application_program | 93 | blocked: application_contribution::producer::demand::bridge_denial::bridge_denial_kind (fn), application_output_demand::denial::WorthQueryOutputDemandDenial (struct) |
| `application_contribution::producer::demand::progression::required_wave::admission_denial` | application_contribution | application_program | 93 | blocked: application_contribution::producer::demand::progression::required_wave::capacity_denial (fn), application_contribution::producer::demand::progression::required_wave::work_denial (fn), application_output_demand::denial::WorthQueryOutputDemandDenial (struct) |
| `application_contribution::producer::demand::progression::required_wave::resolved::match_one` | application_contribution | application_program | 93 | blocked: application_contribution::host::ApplicationContributionHost (trait), application_contribution::producer::demand::progression::required_wave::admission_denial (fn), application_contribution::producer::demand::progression::required_wave::required_settlement_denial (fn) |
| `application_contribution::producer::demand::required_continuations::promotion::prepare_typed_extraction` | application_contribution | application_program | 93 | blocked: application_contribution::producer::demand::required_continuations::RequiredFreshProgress (struct), application_contribution::producer::demand::required_continuations::TypedRequiredSuccessor (struct), application_contribution::producer::demand::required_continuations::promotion::FAMILY_SUBJECT (const) |
| `application_contribution::producer::output_evaluation::consult` | application_contribution | application_program | 93 | blocked: application_output_demand::denial::WorthQueryOutputDemandDenial (struct), output_reuse::selection::RetainedFactStop (enum), output_reuse::selection::retained_fact_denial (fn) |
| `application_contribution::producer::output_evaluation::native::prepare` | application_contribution | application_program | 93 | blocked: application_attempt::fact::WorthQueryApplicationObservedFact (enum), application_contribution::producer::output_equivalence_contract::InstalledProducerOutputEquivalence (struct), application_contribution::producer::output_evaluation::NativeOutputBasis (struct) |
| `application_contribution::producer::output_evaluation::next_comparison_attempt` | application_contribution | application_program | 93 | could move |
| `application_contribution::producer::execution::admitted::execute_fresh` | application_contribution | application_program | 92 | blocked: application_contribution::host::ApplicationContributionHost (trait), application_contribution::producer::ProducerSourceQuery (type), application_contribution::producer::ProducerSourceValue (type) |
| `application_contribution::producer::execution::current_control::select_invocation_control` | application_contribution | application_program | 92 | blocked: application_contribution::host::ApplicationContributionHost (trait), application_contribution::producer::execution::current_control::select_history_control (fn), application_contribution::producer::execution::denial::ProducerExecutionStop (enum) |
| `application_contribution::producer::execution::denial::failed` | application_contribution | application_program | 92 | blocked: application_contribution::producer::execution::denial::denial (fn), application_output_demand::denial::WorthQueryOutputDemandDenial (struct), application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind (enum) |
| `application_contribution::producer::execution::input_reuse::require_selected_program` | application_contribution | application_program | 92 | blocked: application_output_demand::denial::WorthQueryOutputDemandDenial (struct), application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind (enum), product_operation::context::WorthQuerySelectedProductOperation (struct) |
| `application_contribution::producer::execution::selected::execution_plan::prepare_selected_execution` | application_contribution | application_program | 92 | blocked: application_contribution::host::ApplicationContributionHost (trait), application_contribution::producer::ProducerSourceQuery (type), application_contribution::producer::ProducerSourceValue (type) |
| `application_contribution::producer::execution::selected_public::denial::preparation_stop` | application_contribution | application_program | 92 | blocked: application_contribution::producer::WorthQueryApplicationProducerBinding (trait), application_contribution::producer::execution::denial::ProducerExecutionStop (enum), application_contribution::producer::execution::denial::principal_rejected (fn) |
| `application_contribution::producer::execution::selected_public::prepare_recovery_public_mutation` | application_contribution | application_program | 92 | blocked: application_contribution::host::ApplicationContributionHost (trait), application_contribution::producer::WorthQueryApplicationProducerBinding (trait), application_contribution::producer::execution::installed::TypedInstalledProducer (struct) |
| `application_contribution::producer::execution::source_readmission::readmit_source` | application_contribution | application_program | 92 | blocked: application_contribution::host::ApplicationContributionHost (trait), application_contribution::producer::ProducerSourceQuery (type), application_contribution::producer::ProducerSourceValue (type) |

## Remaining seams

### `output_lineage` ↔ `application_contribution`: 22 references up in 8 items, default **contract**

Items: `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` (5), `application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` (4), `application_contribution::producer::demand::progression::required_wave::resolved::matched_predecessors::MatchedRequiredPredecessors` (4), `application_contribution::producer::registry::semantic_edition::InstalledProducerEdition` (3), `application_contribution::partitioned_computation::incremental::retained::SealedComputationRun` (2), `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` (2), `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` (1), `application_contribution::partitioned_computation::incremental::published_state::observe_published` (1)

- `application_contribution::partitioned_computation::incremental::retained::SealedComputationRun` held by `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` in `impl SealedComputationRun`
- `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` held by `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` in `impl PriorAbsence`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::producer::registry::semantic_edition::InstalledProducerEdition` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `output_lineage::retained_computation::PriorComputationRecord` in `impl ComputationPrior`

### `product_operation` ↔ `application_attempt`: 19 references up in 18 items, default **nest or merge**

Items: `application_attempt::workflow::correction::PreparedWorkflowCorrection` (2), `application_attempt::effect_program::model::WorthQueryApplicationEffectProgram` (1), `application_attempt::host::ApplicationAttemptHost` (1), `application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease` (1), `application_attempt::compare_and_commit::commit_outcome::WorthQueryApplicationCommitOutcome` (1), `application_attempt::workflow::correction::retain_current_continuation` (1), `application_attempt::idempotency_resolution::external_settlement::ExternalSettlementScopedOps` (1), `application_attempt::workflow::adoption::inventory::WorkflowAdoptionInventoryRequest` (1)

- `application_attempt::effect_program::model::WorthQueryApplicationEffectProgram` held by `application_attempt::host::ApplicationAttemptHost` in `impl<Schema, Operation, Input, Scope> WorthQueryApplicationEffectProgram<Schema,`
- `application_attempt::effect_program::model::WorthQueryApplicationEffectProgram` held by `application_attempt::workflow::actor::record::WorkflowAdmittedActor` in `impl<Schema, Operation, Input, Scope> WorthQueryApplicationEffectProgram<Schema,`
- `application_attempt::effect_program::model::WorthQueryApplicationEffectProgram` held by `application_attempt::workflow::instance::start_provenance::admit` in `impl<Schema, Operation, Input, Scope> WorthQueryApplicationEffectProgram<Schema,`
- `application_attempt::workflow_transition_program::actor_context::WorthQueryWorkflowActorContext` held by `application_attempt::workflow_transition_program::actor_context::Command` in `impl WorthQueryWorkflowActorContext`
- `application_attempt::workflow::actor::record::WorkflowAdmittedActor` held by `application_attempt::workflow::actor::record::WorkflowActorFields` in `impl WorkflowAdmittedActor`
- `application_attempt::effect_program::model::WorthQueryApplicationEffectProgram` held by `application_attempt::dependency_identity::output_postcondition::OutputCurrentnessFacts` in `declaration`

### `application_attempt` ↔ `provider`: 18 references up in 8 items, default **contract**

Items: `provider::WorthQueryPrimaryGraphProvider` (10), `provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::PreparedSourceFactRebase` (2), `provider::committed_dispatch_outbox::ProviderCommittedDispatchOutboxScopedOps` (1), `provider::relational_execution_denial::query_read_execution_kind` (1), `provider::application_decision_fact::application_facts::CompletedApplicationFacts` (1), `provider::host::ProviderHost` (1), `provider::application_decision_fact::application_table::ApplicationFactTable` (1), `provider::pending_application_publication::registry::WorthQueryApplicationPublicationRecoveryReservation` (1)

- `provider::application_decision_fact::application_table::ApplicationFactTable` held by `provider::application_decision_fact::WorthQueryPrimaryGraphApplicationDecisionFact` in `impl ApplicationFactTable`
- `provider::WorthQueryPrimaryGraphProvider` held by `application_query::derived_view::registry::predicate::preparation::PredicatePreparationRetirement` in `impl WorthQueryPrimaryGraphProvider`
- `provider::WorthQueryPrimaryGraphProvider` held by `application_query::derived_view::registry::predicate::preparation::PredicatePreparationOwner` in `impl WorthQueryPrimaryGraphProvider`
- `provider::WorthQueryPrimaryGraphProvider` held by `provider::application_attempt_state::WorthQueryPrimaryGraphApplicationAttemptStore` in `impl WorthQueryPrimaryGraphProvider`
- `provider::WorthQueryPrimaryGraphProvider` held by `provider::pending_application_publication::registry::WorthQueryPendingApplicationPublicationRegistry` in `impl WorthQueryPrimaryGraphProvider`
- `provider::WorthQueryPrimaryGraphProvider` held by `provider::invariant_execution_failure::map_validation_failure` in `impl WorthQueryPrimaryGraphProvider`

### `product_operation` ↔ `application_query`: 17 references up in 10 items, default **contract**

Items: `application_query::admission::AdmissionScopedOps` (6), `application_query::managed_index::denial::WorthQueryManagedIndexDenial` (2), `application_query::one_shot::WorthQueryApplicationOneShotResult` (2), `application_query::observed_source::retained_dependencies::WorthQueryRetainedQueryDependencies` (1), `application_query::one_shot::denial::WorthQueryApplicationOneShotDenial` (1), `application_query::document_history::output_intent::WorthQueryHistoryOutputIntent` (1), `application_query::document_history::output_read::OutputReadScopedOps` (1), `application_query::one_shot::correction_disclosure::OneShotCorrectionDisclosureScopedOps` (1)

- `application_query::observed_source::retained_dependencies::WorthQueryRetainedQueryDependencies` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl WorthQueryRetainedQueryDependencies`
- `application_query::observed_source::retained_dependencies::WorthQueryRetainedQueryDependencies` held by `application_query::observed_source::retained_dependencies::retain_joined_descriptors` in `impl WorthQueryRetainedQueryDependencies`
- `application_query::managed_index::denial::WorthQueryManagedIndexDenial` held by `application_query::managed_index::denial::ReceiverCause` in `declaration`
- `application_query::managed_index::denial::WorthQueryManagedIndexDenial` held by `application_query::managed_index::denial::ReceiverCause` in `impl WorthQueryManagedIndexDenial`
- `application_query::managed_index::denial::WorthQueryManagedIndexDenial` held by `application_query::managed_index::denial::causes::WorthQueryManagedIndexDenialCause` in `impl WorthQueryManagedIndexDenial`
- `application_query::managed_index::denial::WorthQueryManagedIndexDenial` held by `application_query::managed_index::denial::causes::provider` in `impl WorthQueryManagedIndexDenial`

### `product_operation` ↔ `application_contribution`: 16 references up in 3 items, default **contract**

Items: `application_contribution::producer::WorthQueryApplicationProducerBinding` (12), `application_contribution::producer::WorthQueryApplicationProducerProvider` (3), `application_contribution::producer::registry::WorthQueryInstalledApplicationProducerRegistry` (1)

- `application_contribution::producer::registry::WorthQueryInstalledApplicationProducerRegistry` held by `application_contribution::producer::registry::InstalledProducerProvider` in `impl<Schema> WorthQueryInstalledApplicationProducerRegistry<Schema>`
- `application_contribution::producer::registry::WorthQueryInstalledApplicationProducerRegistry` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl<Schema> WorthQueryInstalledApplicationProducerRegistry<Schema>`
- `application_contribution::producer::registry::WorthQueryInstalledApplicationProducerRegistry` held by `application_contribution::producer::registry::InstalledProducerProvider` in `declaration`
- `application_contribution::producer::registry::WorthQueryInstalledApplicationProducerRegistry` held by `application_contribution::producer::WorthQueryApplicationProducerBinding` in `impl<Schema> super::WorthQueryInstalledApplicationProducerRegistry<Schema>`
- `application_contribution::producer::registry::WorthQueryInstalledApplicationProducerRegistry` held by `application_contribution::producer::WorthQueryApplicationProducerBinding` in `impl<Schema> WorthQueryInstalledApplicationProducerRegistry<Schema>`
- `application_contribution::producer::registry::WorthQueryInstalledApplicationProducerRegistry` held by `application_contribution::producer::scheduling::WorthQueryInstalledOutputProducerRoutes` in `impl<Schema> WorthQueryInstalledApplicationProducerRegistry<Schema>`

### `application_discovery` ↔ `application_attempt`: 15 references up in 9 items, default **contract**

Items: `application_attempt::fact::adjacency::observe_adjacency` (4), `application_attempt::workflow::schema::layout::WorthQueryWorkflowLayout` (3), `application_attempt::workflow_instance_observation::governed_record` (2), `application_attempt::observation::observe_field_value` (1), `application_attempt::workflow_instance_observation::audit::observe` (1), `application_attempt::workflow_instance_observation::history_projection` (1), `application_attempt::workflow::schema::version::WORKFLOW_INSTANCE_FACT_PROTOCOL_VERSION` (1), `application_attempt::workflow_instance_observation::read_fields::ReadFields` (1)

- `application_attempt::observation::observe_field_value` held by `application_attempt::observation::observe_field` in `body`
- `application_attempt::fact::adjacency::observe_adjacency` held by `application_attempt::fact::adjacency::observe_adjacency_checked` in `body`
- `application_attempt::fact::adjacency::observe_adjacency` held by `application_attempt::fact::adjacency::resolve_observation` in `body`
- `application_attempt::workflow_instance_observation::audit::observe` held by `application_attempt::workflow::schema::layout::WorthQueryWorkflowLayout` in `body`
- `application_attempt::workflow_instance_observation::audit::observe` held by `application_attempt::workflow_instance_observation::actors::observe` in `body`
- `application_attempt::workflow_instance_observation::audit::observe` held by `application_attempt::workflow_instance_observation::transition_roles::observe` in `body`

### `application_attempt` ↔ `application_contribution`: 12 references up in 5 items, default **contract**

Items: `application_contribution::partitioned_computation::incremental::prior_absence::CompletedComputationRetention` (4), `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` (3), `application_contribution::partitioned_computation::incremental::native_reads::NativeComputationReads` (3), `application_contribution::execution_denial` (1), `application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` (1)

- `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` held by `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` in `impl PriorAbsence`
- `application_contribution::partitioned_computation::incremental::native_reads::NativeComputationReads` held by `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` in `declaration`
- `application_contribution::partitioned_computation::incremental::native_reads::NativeComputationReads` held by `output_lineage::invalidation::computation_changes::ComputationChanges` in `declaration`
- `application_contribution::partitioned_computation::incremental::native_reads::NativeComputationReads` held by `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` in `impl NativeComputationReads`
- `application_contribution::partitioned_computation::incremental::native_reads::NativeComputationReads` held by `invariant_projection::operation_reader::computation_calls::sequence::ComputationCallSequence` in `impl NativeComputationReads`
- `application_contribution::partitioned_computation::incremental::native_reads::NativeComputationReads` held by `output_lineage::invalidation::computation_changes::ComputationChanges` in `impl NativeComputationReads`

### `application_attempt` ↔ `application_output_demand`: 12 references up in 4 items, default **contract**

Items: `application_output_demand::settlement::WorthQueryOutputDemandSettlement` (5), `application_output_demand::registry::required_context::RequiredOutputDemandContext` (5), `application_output_demand::settlement::SettlementScopedOps` (1), `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` (1)

- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::readiness_delivery::WorthQueryOutputReadinessDeliveryEvidence` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::settlement::receipt_custody::SettlementReceiptCustody` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::registry::required_context::RequiredOutputDemandContext` held by `output_lineage::invalidation::owner::SourceInvalidationOwner` in `impl RequiredOutputDemandContext`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::readiness_delivery::WorthQueryOutputReadinessDeliveryEvidence` in `declaration`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::settlement::receipt_custody::SettlementReceiptCustody` in `declaration`

### `application_attempt` ↔ `invariant_projection`: 10 references up in 5 items, default **contract**

Items: `invariant_projection::operation_reader::decision_reads::expected_keys::ExpectedFactKeys` (6), `invariant_projection::realized_scope::WorthQueryRealizedProjectionScope` (1), `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionSnapshot` (1), `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` (1), `invariant_projection::operation_reader::borrowed::BorrowedOperationProjection` (1)

- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::WorthQueryApplicationInvariantProjectionSnapshot` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::locked_reader::WorthQueryApplicationInvariantProjectionReader` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::locked_reader::WorthQueryCompletedInvariantProjection` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease` in `impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionAuthorit`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` in `impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionAuthorit`

### `application_output_demand` ↔ `application_query`: 10 references up in 4 items, default **contract**

Items: `application_query::observed_source::retained_dependencies::WorthQueryRetainedQueryDependencies` (4), `application_query::observed_source::source_identity::WorthQueryObservedSourceEpoch` (3), `application_query::managed_index::denial::WorthQueryManagedIndexDenial` (2), `application_query::managed_index::denial::causes::WorthQueryManagedIndexDenialCause` (1)

- `application_query::observed_source::source_identity::WorthQueryObservedSourceEpoch` held by `application_query::observed_source::source_identity::WorthQueryObservedSourceMeaning` in `impl WorthQueryObservedSourceEpoch`
- `application_query::observed_source::source_identity::WorthQueryObservedSourceEpoch` held by `application_query::observed_source::source_identity::WorthQueryObservedSourceMeaning` in `declaration`
- `application_query::observed_source::source_identity::WorthQueryObservedSourceEpoch` held by `application_query::observed_source::source_identity::checkpoint_commitment::checkpoint_source_identity` in `impl WorthQueryObservedSourceEpoch`
- `application_query::observed_source::retained_dependencies::WorthQueryRetainedQueryDependencies` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl WorthQueryRetainedQueryDependencies`
- `application_query::observed_source::retained_dependencies::WorthQueryRetainedQueryDependencies` held by `application_query::observed_source::retained_dependencies::retain_joined_descriptors` in `impl WorthQueryRetainedQueryDependencies`
- `application_query::managed_index::denial::causes::WorthQueryManagedIndexDenialCause` held by `application_query::one_shot::denial::WorthQueryApplicationOneShotDenial` in `declaration`

### `product_operation` ↔ `application_installation`: 9 references up in 6 items, default **contract**

Items: `application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime` (3), `application_installation::program::program_owner::contract::WorthQueryProgramOwner` (2), `application_installation::program::demand::WorthQueryAdmittedProgramOutput` (1), `application_installation::program::demand::WorthQuerySettledProgramOutput` (1), `application_installation::program::specialized_action::WorthQueryAdmittedProgramOperation` (1), `application_installation::program::demand::root_completion::OriginalProgramRootCompletion` (1)

- `application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>`
- `application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>> WorthQu`
- `application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime` held by `application_installation::program::demand::WorthQueryAdmittedProgramOutput` in `impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>`
- `application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime` held by `application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenial` in `impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>`
- `application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime` held by `application_installation::program::program_owner::WorthQuerySelectedProgramOwner` in `impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>`
- `application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime` held by `application_installation::program::demand::WorthQuerySettledProgramOutput` in `impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>> WorthQu`

### `invariant_projection` ↔ `application_contribution`: 9 references up in 7 items, default **contract**

Items: `application_contribution::partitioned_computation::incremental::prior_absence::CompletedComputationRetention` (2), `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` (2), `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` (1), `application_contribution::partitioned_computation::incremental::native_reads::NativeComputationReads` (1), `application_contribution::producer::source_qualification::SourceQualificationScopedOps` (1), `application_contribution::partitioned_computation::incremental::deposit::ComputationDeposit` (1), `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` (1)

- `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` held by `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` in `impl PriorAbsence`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::producer::registry::semantic_edition::InstalledProducerEdition` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `output_lineage::retained_computation::PriorComputationRecord` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::native_reads::NativeComputationReads` held by `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` in `declaration`

### `operation_body` ↔ `handler`: 8 references up in 2 items, default **contract**

Items: `handler::candidate::CandidateWriter` (5), `handler::invariant::DecisionReader` (3)

- `handler::invariant::DecisionReader` held by `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` in `impl<'borrow, 'reader, 'runtime, Schema, Binding> DecisionReader<'borrow, 'reade`
- `handler::invariant::DecisionReader` held by `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` in `declaration`
- `handler::invariant::DecisionReader` held by `application_contribution::conditional::WorthQueryApplicationConditionalBinding` in `impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>> Dec`
- `handler::invariant::DecisionReader` held by `invariant_projection::component::result::WorthQueryInvariantProjectionComponent` in `impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>> Dec`
- `handler::invariant::DecisionReader` held by `invariant_projection::operation_reader::derived_relation::member::component::WorthQueryInvariantGraphComponent` in `impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>> Dec`
- `handler::invariant::DecisionReader` held by `operation_body::input::prepare_inputs` in `impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>> Dec`

### `application_output_demand` ↔ `application_contribution`: 8 references up in 5 items, default **contract**

Items: `application_contribution::producer::output_evaluation::ProducerOutputEvaluation` (3), `application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` (2), `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` (1), `application_contribution::producer::demand::bridge_denial::bridge_denial_kind` (1), `application_contribution::producer::execution::unpublished::PendingProducerPublication` (1)

- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::producer::registry::semantic_edition::InstalledProducerEdition` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `output_lineage::retained_computation::PriorComputationRecord` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` held by `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` in `declaration`
- `application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` held by `application_contribution::partitioned_computation::incremental::retained::SealedComputationRun` in `declaration`

### `application_attempt` ↔ `application_installation`: 7 references up in 2 items, default **contract**

Items: `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` (5), `application_installation::program::workflow_runtime::authentication::workflow_approval_authentication_intent` (2)

- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_installation::host::ApplicationInstallationHost` in `declaration`
- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_contribution::producer::demand::WorthQueryAdmittedOutputDemand` in `impl<Schema, Spec> WorthQueryWorkflowVocabulary<'_, Schema, Spec>`
- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl<Schema, Spec> WorthQueryWorkflowVocabulary<'_, Schema, Spec>`
- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_installation::host::ApplicationInstallationHost` in `impl<'vocabulary, Schema, Spec> WorthQueryWorkflowVocabulary<'vocabulary, Schema`
- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_installation::program::workflow_runtime::WorthQueryWorkflowApplicationRuntime` in `impl<'vocabulary, Schema, Spec, Program> From<&'vocabulary WorthQueryWorkflowApp`
- `application_installation::program::workflow_runtime::authentication::workflow_approval_authentication_intent` held by `application_installation::program::workflow_runtime::authentication::append` in `body`

### `application_attempt` ↔ `output_lineage`: 7 references up in 2 items, default **contract**

Items: `output_lineage::invalidation::computation_changes::postcondition_carry::PostconditionCarry` (4), `output_lineage::invalidation::output_availability::ObservedSourcePublication` (3)

- `output_lineage::invalidation::computation_changes::postcondition_carry::PostconditionCarry` held by `output_lineage::invalidation::computation_changes::postcondition_carry::visit` in `impl PostconditionCarry`
- `output_lineage::invalidation::output_availability::ObservedSourcePublication` held by `invariant_projection::output_availability_candidate::ObservedOutputCandidate` in `declaration`

### `product_operation` ↔ `application_output_demand`: 6 references up in 2 items, default **contract**

Items: `application_output_demand::settlement::WorthQueryOutputDemandSettlement` (3), `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` (3)

- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::readiness_delivery::WorthQueryOutputReadinessDeliveryEvidence` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::settlement::receipt_custody::SettlementReceiptCustody` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::readiness_delivery::WorthQueryOutputReadinessDeliveryEvidence` in `declaration`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::settlement::receipt_custody::SettlementReceiptCustody` in `declaration`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::registry::ready_backing::ReadyCompletion` in `impl WorthQueryOutputDemandSettlement`

### `application_discovery` ↔ `application_installation`: 5 references up in 1 items, default **contract**

Items: `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` (5)

- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_installation::host::ApplicationInstallationHost` in `declaration`
- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_contribution::producer::demand::WorthQueryAdmittedOutputDemand` in `impl<Schema, Spec> WorthQueryWorkflowVocabulary<'_, Schema, Spec>`
- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl<Schema, Spec> WorthQueryWorkflowVocabulary<'_, Schema, Spec>`
- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_installation::host::ApplicationInstallationHost` in `impl<'vocabulary, Schema, Spec> WorthQueryWorkflowVocabulary<'vocabulary, Schema`
- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_installation::program::workflow_runtime::WorthQueryWorkflowApplicationRuntime` in `impl<'vocabulary, Schema, Spec, Program> From<&'vocabulary WorthQueryWorkflowApp`

### `operation_body` ↔ `invariant_projection`: 5 references up in 2 items, default **contract**

Items: `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` (4), `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` (1)

- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::WorthQueryApplicationInvariantProjectionSnapshot` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::locked_reader::WorthQueryApplicationInvariantProjectionReader` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::locked_reader::WorthQueryCompletedInvariantProjection` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease` in `impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionAuthorit`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` in `impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionAuthorit`

### `access_strategy` ↔ `application_query`: 4 references up in 2 items, default **contract**

Items: `application_query::managed_index::source_structure::SourceTree` (3), `application_query::managed_index::source_rows::SourceImageView` (1)

- `application_query::managed_index::source_structure::SourceTree` held by `application_query::managed_index::source_rows::SourceImageView` in `impl SourceTree`
- `application_query::managed_index::source_rows::SourceImageView` held by `application_query::managed_index::source_rows::SourceImageRows` in `declaration`
- `application_query::managed_index::source_rows::SourceImageView` held by `application_query::managed_index::source_rows::SourceImageRows` in `impl SourceImageView<'_>`
- `application_query::managed_index::source_structure::SourceTree` held by `application_query::managed_index::source_structure::cold::acyclic` in `impl SourceTree`
- `application_query::managed_index::source_structure::SourceTree` held by `application_query::managed_index::source_structure::cold_postings::capture` in `impl SourceTree`

### `application_entry` ↔ `handler`: 4 references up in 2 items, default **contract**

Items: `handler::invariant::DecisionReader` (2), `handler::candidate::CandidateWriter` (2)

- `handler::invariant::DecisionReader` held by `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` in `impl<'borrow, 'reader, 'runtime, Schema, Binding> DecisionReader<'borrow, 'reade`
- `handler::invariant::DecisionReader` held by `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` in `declaration`
- `handler::invariant::DecisionReader` held by `application_contribution::conditional::WorthQueryApplicationConditionalBinding` in `impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>> Dec`
- `handler::invariant::DecisionReader` held by `invariant_projection::component::result::WorthQueryInvariantProjectionComponent` in `impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>> Dec`
- `handler::invariant::DecisionReader` held by `invariant_projection::operation_reader::derived_relation::member::component::WorthQueryInvariantGraphComponent` in `impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>> Dec`
- `handler::invariant::DecisionReader` held by `operation_body::input::prepare_inputs` in `impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>> Dec`

### `application_output_demand` ↔ `output_lineage`: 4 references up in 2 items, default **contract**

Items: `output_lineage::input_reuse_key::PreparedInputReuseKey` (2), `output_lineage::retained_computation::PriorComputationRecord` (2)

- `output_lineage::input_reuse_key::PreparedInputReuseKey` held by `application_contribution::producer::registry::semantic_edition::InstalledProducerEdition` in `impl PreparedInputReuseKey`
- `output_lineage::input_reuse_key::PreparedInputReuseKey` held by `application_query::observed_source::source_identity::membership::WorthQueryObservedSourceSelection` in `impl PreparedInputReuseKey`
- `output_lineage::input_reuse_key::PreparedInputReuseKey` held by `application_contribution::producer::registry::semantic_edition::InstalledProducerEdition` in `declaration`
- `output_lineage::input_reuse_key::PreparedInputReuseKey` held by `application_query::observed_source::source_identity::membership::WorthQueryObservedSourceSelection` in `declaration`
- `output_lineage::retained_computation::PriorComputationRecord` held by `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` in `impl PriorComputationRecord`
- `output_lineage::retained_computation::PriorComputationRecord` held by `output_lineage::invalidation::computation_changes::ComputationChanges` in `impl PriorComputationRecord`

### `output_reuse` ↔ `application_output_demand`: 4 references up in 2 items, default **contract**

Items: `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` (3), `output_lineage::invalidation::owner::SourceInvalidationOwner` (1)

- `output_lineage::invalidation::owner::SourceInvalidationOwner` held by `output_lineage::invalidation::source_alignment::SnapshotAlignedMarkState` in `impl SourceInvalidationOwner`
- `output_lineage::invalidation::owner::SourceInvalidationOwner` held by `output_lineage::invalidation::settlement::SettlementRegistration` in `impl SourceInvalidationOwner`
- `output_lineage::invalidation::owner::SourceInvalidationOwner` held by `output_lineage::source_facts::ComparableSourceFacts` in `impl SourceInvalidationOwner`
- `output_lineage::invalidation::owner::SourceInvalidationOwner` held by `output_lineage::invalidation::computation_changes::ComputationChanges` in `impl SourceInvalidationOwner`
- `output_lineage::invalidation::owner::SourceInvalidationOwner` held by `output_lineage::source_facts::RetainedSourceFacts` in `impl SourceInvalidationOwner`
- `output_lineage::invalidation::owner::SourceInvalidationOwner` held by `output_lineage::invalidation::full_verification::FullVerificationImage` in `impl SourceInvalidationOwner`

### `application_entry` ↔ `application_attempt`: 4 references up in 4 items, default **contract**

Items: `application_attempt::effect_program::model::WorthQueryApplicationEffectProgram` (1), `application_attempt::read_set::ReadSetScopedOps` (1), `application_attempt::read_set::WorthQueryCompleteApplicationReadSet` (1), `application_attempt::read_set::source_facts::SourceFactsScopedOps` (1)

- `application_attempt::read_set::WorthQueryCompleteApplicationReadSet` held by `application_attempt::effect_program::model::WorthQueryApplicationEffectProgram` in `impl<Schema, Operation, Input, Scope> WorthQueryCompleteApplicationReadSet< Sche`
- `application_attempt::read_set::WorthQueryCompleteApplicationReadSet` held by `application_attempt::workflow::schema::layout::WorthQueryWorkflowLayout` in `impl<Schema, Operation, Input, Scope> WorthQueryCompleteApplicationReadSet< Sche`
- `application_attempt::read_set::WorthQueryCompleteApplicationReadSet` held by `application_attempt::workflow_transition_program::publication::PreparedWorkflowAdvance` in `impl<Schema, Operation, Input, Scope> WorthQueryCompleteApplicationReadSet< Sche`
- `application_attempt::read_set::WorthQueryCompleteApplicationReadSet` held by `application_attempt::workflow::instance::transition::admission::selection::SelectedWorkflowTransition` in `impl<Schema, Operation, Input, Scope> WorthQueryCompleteApplicationReadSet< Sche`
- `application_attempt::read_set::WorthQueryCompleteApplicationReadSet` held by `application_attempt::workflow::instance::transition::admission::admit_workflow_transition` in `impl<Schema, Operation, Input, Scope> WorthQueryCompleteApplicationReadSet< Sche`
- `application_attempt::read_set::WorthQueryCompleteApplicationReadSet` held by `application_attempt::workflow::definition::compilation::reconstruction::reconstruct_compiled_definition` in `impl<Schema, Operation, Input, Scope> WorthQueryCompleteApplicationReadSet< Sche`

### `root` ↔ `application_query`: 3 references up in 2 items, default **contract**

Items: `application_query::derived_view::registry::ManagedDerivedViewRegistry` (2), `application_query::managed_index::source::installation::ManagedSourceInstallation` (1)

- `application_query::managed_index::source::installation::ManagedSourceInstallation` held by `application_query::managed_index::source::service::SourceService` in `declaration`
- `application_query::managed_index::source::installation::ManagedSourceInstallation` held by `application_query::managed_index::configuration::ManagedIndexProviderCatalog` in `impl ManagedSourceInstallation`
- `application_query::managed_index::source::installation::ManagedSourceInstallation` held by `application_query::managed_index::source::service::SourceService` in `impl ManagedSourceInstallation`
- `application_query::derived_view::registry::ManagedDerivedViewRegistry` held by `application_query::derived_view::registry::RegistryState` in `declaration`
- `application_query::derived_view::registry::ManagedDerivedViewRegistry` held by `application_query::derived_view::registry::RegistryState` in `impl ManagedDerivedViewRegistry`
- `application_query::derived_view::registry::ManagedDerivedViewRegistry` held by `application_query::managed_index::retention::ManagedIndexState` in `impl ManagedDerivedViewRegistry`

### `output_lineage` ↔ `application_query`: 3 references up in 2 items, default **contract**

Items: `application_query::observed_source::source_identity::membership::WorthQueryObservedSourceSelection` (2), `application_query::derived_view::registry::ManagedDerivedViewRegistry` (1)

- `application_query::observed_source::source_identity::membership::WorthQueryObservedSourceSelection` held by `application_query::observed_source::source_identity::WorthQueryObservedSourceMeaning` in `declaration`
- `application_query::observed_source::source_identity::membership::WorthQueryObservedSourceSelection` held by `application_query::observed_source::source_identity::material_storage::admit_footprint_comparison` in `impl WorthQueryObservedSourceSelection`
- `application_query::observed_source::source_identity::membership::WorthQueryObservedSourceSelection` held by `application_query::observed_source::source_identity::membership::same_footprint_membership` in `impl WorthQueryObservedSourceSelection`
- `application_query::observed_source::source_identity::membership::WorthQueryObservedSourceSelection` held by `application_query::observed_source::source_identity::WorthQueryObservedSourceMeaning` in `impl WorthQueryObservedSourceSelection`
- `application_query::derived_view::registry::ManagedDerivedViewRegistry` held by `application_query::derived_view::registry::RegistryState` in `declaration`
- `application_query::derived_view::registry::ManagedDerivedViewRegistry` held by `application_query::derived_view::registry::RegistryState` in `impl ManagedDerivedViewRegistry`

### `product_operation` ↔ `provider`: 3 references up in 2 items, default **contract**

Items: `provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` (2), `provider::relational_execution_denial` (1)

- `provider::relational_execution_denial` held by `(module: not movable)`
- `provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` held by `provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyStore` in `declaration`
- `provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` held by `provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyStore` in `impl WorthQueryUnpublishedIdempotencyDisposition`
- `provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` held by `provider::unpublished_idempotency::release::WorthQueryUnpublishedReleaseReservation` in `impl WorthQueryUnpublishedIdempotencyDisposition`

### `program_occurrence` ↔ `application_installation`: 3 references up in 1 items, default **contract**

Items: `application_installation::program::supported_program::WorthQuerySupportedProgramRecord` (3)

- `application_installation::program::supported_program::WorthQuerySupportedProgramRecord` held by `product_operation::program_adoption::authorization::ProgramAdoptionAdmission` in `impl WorthQuerySupportedProgramRecord`
- `application_installation::program::supported_program::WorthQuerySupportedProgramRecord` held by `product_operation::program_adoption::authorization::admit_program` in `impl WorthQuerySupportedProgramRecord`

### `causal_inspection` ↔ `application_contribution`: 3 references up in 3 items, default **contract**

Items: `application_contribution::producer::output_evaluation::ProducerOutputEvaluation` (1), `application_contribution::producer::output_evaluation::ProducerOutputEvaluationState` (1), `application_contribution::producer::output_evaluation::ComparedProducerOutput` (1)

- `application_contribution::producer::output_evaluation::ProducerOutputEvaluation` held by `application_contribution::producer::output_equivalence_contract::InstalledProducerOutputEquivalence` in `declaration`
- `application_contribution::producer::output_evaluation::ProducerOutputEvaluation` held by `application_contribution::producer::output_evaluation::ProducerOutputEvaluationState` in `declaration`
- `application_contribution::producer::output_evaluation::ProducerOutputEvaluationState` held by `application_contribution::producer::output_evaluation::ComparedProducerOutput` in `declaration`
- `application_contribution::producer::output_evaluation::ComparedProducerOutput` held by `application_contribution::producer::output_equivalence_contract::InstalledProducerOutputEquivalence` in `declaration`

### `conditional_operation` ↔ `application_contribution`: 3 references up in 3 items, default **contract**

Items: `application_contribution::producer::registry::WorthQueryInstalledApplicationProducerRegistry` (1), `application_contribution::producer::WorthQueryApplicationProducerBinding` (1), `application_contribution::conditional::output_readiness_contract::WorthQueryOutputReadinessContractDenial` (1)

- `application_contribution::producer::registry::WorthQueryInstalledApplicationProducerRegistry` held by `application_contribution::producer::registry::InstalledProducerProvider` in `impl<Schema> WorthQueryInstalledApplicationProducerRegistry<Schema>`
- `application_contribution::producer::registry::WorthQueryInstalledApplicationProducerRegistry` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl<Schema> WorthQueryInstalledApplicationProducerRegistry<Schema>`
- `application_contribution::producer::registry::WorthQueryInstalledApplicationProducerRegistry` held by `application_contribution::producer::registry::InstalledProducerProvider` in `declaration`
- `application_contribution::producer::registry::WorthQueryInstalledApplicationProducerRegistry` held by `application_contribution::producer::WorthQueryApplicationProducerBinding` in `impl<Schema> super::WorthQueryInstalledApplicationProducerRegistry<Schema>`
- `application_contribution::producer::registry::WorthQueryInstalledApplicationProducerRegistry` held by `application_contribution::producer::WorthQueryApplicationProducerBinding` in `impl<Schema> WorthQueryInstalledApplicationProducerRegistry<Schema>`
- `application_contribution::producer::registry::WorthQueryInstalledApplicationProducerRegistry` held by `application_contribution::producer::scheduling::WorthQueryInstalledOutputProducerRoutes` in `impl<Schema> WorthQueryInstalledApplicationProducerRegistry<Schema>`

### `schema_layout` ↔ `application_attempt`: 3 references up in 2 items, default **contract**

Items: `application_attempt::workflow::schema::layout::WorthQueryWorkflowLayout` (2), `application_attempt::workflow::schema::lower_workflow` (1)

- `application_attempt::workflow::schema::layout::WorthQueryWorkflowLayout` held by `application_attempt::workflow::schema::layout::WorkflowLayoutFields` in `declaration`
- `application_attempt::workflow::schema::layout::WorthQueryWorkflowLayout` held by `application_attempt::workflow::schema::layout::WorkflowLayoutFields` in `impl WorthQueryWorkflowLayout`
- `application_attempt::workflow::schema::layout::WorthQueryWorkflowLayout` held by `application_attempt::workflow::schema::layout::WorkflowLayoutFields` in `impl std::ops::Deref for WorthQueryWorkflowLayout`
- `application_attempt::workflow::schema::lower_workflow` held by `application_attempt::workflow::schema::approval::lower_approval` in `body`
- `application_attempt::workflow::schema::lower_workflow` held by `application_attempt::workflow::schema::assessment::lower_assessment_evidence` in `body`
- `application_attempt::workflow::schema::lower_workflow` held by `application_attempt::workflow::schema::captured_input::lower` in `body`

### `application_query` ↔ `invariant_projection`: 2 references up in 2 items, default **contract**

Items: `invariant_projection::consumed_output::verification::work_budget::map_verification_stop` (1), `invariant_projection::operation_reader::output_availability::denial::WorthQueryOutputAvailabilityDenial` (1)

- `invariant_projection::operation_reader::output_availability::denial::WorthQueryOutputAvailabilityDenial` held by `invariant_projection::operation_reader::output_availability::denial::demand_stop::meter` in `impl WorthQueryOutputAvailabilityDenial`
- `invariant_projection::operation_reader::output_availability::denial::WorthQueryOutputAvailabilityDenial` held by `invariant_projection::operation_reader::output_availability::denial::demand_stop::query_stop` in `impl WorthQueryOutputAvailabilityDenial`

### `application_inbound` ↔ `provider`: 2 references up in 2 items, default **contract**

Items: `provider::host::ProviderHost` (1), `provider::committed_dispatch_outbox::CommittedDispatchOutboxScopedOps` (1)

- `provider::host::ProviderHost` held by `provider::WorthQueryPrimaryGraphProvider` in `declaration`
- `provider::committed_dispatch_outbox::CommittedDispatchOutboxScopedOps` held by `provider::host::ProviderHost` in `declaration`
- `provider::committed_dispatch_outbox::CommittedDispatchOutboxScopedOps` held by `provider::host::ProviderHost` in `impl< Schema, HostRuntime: crate::domain_computation::primary_graph::provider::h`

### `product_operation` ↔ `output_lineage`: 2 references up in 2 items, default **contract**

Items: `output_lineage::native_output_witness::PreparedNativeOutputWitness` (1), `output_lineage::invalidation::publication::register_republished` (1)

- `output_lineage::native_output_witness::PreparedNativeOutputWitness` held by `application_attempt::provider_binding::registration::provider_registration::WorthQueryPrimaryGraphApplicationAttempt` in `impl PreparedNativeOutputWitness`
- `output_lineage::invalidation::publication::register_republished` held by `output_lineage::invalidation::owner::SourceInvalidationOwner` in `body`
- `output_lineage::invalidation::publication::register_republished` held by `output_lineage::invalidation::publication::registration_reason` in `body`
- `output_lineage::invalidation::publication::register_republished` held by `output_lineage::invalidation::publication::stopped` in `body`

### `freshness` ↔ `application_attempt`: 2 references up in 2 items, default **contract**

Items: `application_attempt::retained_decision_facts::endpoints::AdmittedAdjacencyEndpoints` (1), `application_attempt::fact::consult_observation` (1)

- `application_attempt::retained_decision_facts::endpoints::AdmittedAdjacencyEndpoints` held by `application_attempt::retained_decision_facts::endpoints::heap_sort` in `impl AdmittedAdjacencyEndpoints`
- `application_attempt::fact::consult_observation` held by `(module: not movable)`

### `bootstrap` ↔ `handler`: 2 references up in 1 items, default **contract**

Items: `handler::registry::PendingMutationHandlerRegistry` (2)

- `handler::registry::PendingMutationHandlerRegistry` held by `operation_body::handler::OperationBodyHandler` in `impl<Schema: ApplicationSchema> PendingMutationHandlerRegistry<Schema>`

### `operation_body` ↔ `application_attempt`: 2 references up in 2 items, default **contract**

Items: `application_attempt::read_set::source_facts::SourceFactsScopedOps` (1), `application_attempt::host::ApplicationAttemptHost` (1)

- `application_attempt::read_set::source_facts::SourceFactsScopedOps` held by `application_attempt::host::ApplicationAttemptHost` in `declaration`
- `application_attempt::read_set::source_facts::SourceFactsScopedOps` held by `application_attempt::read_set::source_facts::compare_source_facts` in `declaration`
- `application_attempt::read_set::source_facts::SourceFactsScopedOps` held by `application_attempt::host::ApplicationAttemptHost` in `impl<Schema: worth_query_installation::facade::ApplicationSchema, HostRuntime: c`
- `application_attempt::host::ApplicationAttemptHost` held by `application_inbound::host::ApplicationInboundHost` in `declaration`
- `application_attempt::host::ApplicationAttemptHost` held by `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` in `declaration`
- `application_attempt::host::ApplicationAttemptHost` held by `provider::host::ProviderHost` in `declaration`

### `program_occurrence` ↔ `application_attempt`: 2 references up in 1 items, default **contract**

Items: `application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenial` (2)

- `application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenial` held by `application_invariant::refusal::decode` in `impl WorthQueryApplicationCommitDenial`

### `application_entry` ↔ `application_query`: 2 references up in 1 items, default **contract**

Items: `application_query::output_source_observation::SourceQueryReaderAccess` (2)

- `application_query::output_source_observation::SourceQueryReaderAccess` held by `application_query::output_source_observation::SourceQueryObservationStop` in `impl<Schema: ApplicationSchema> SourceQueryReaderAccess<'_, Schema>`
- `application_query::output_source_observation::SourceQueryReaderAccess` held by `application_query::one_shot::WorthQueryAdmittedOneShotStop` in `impl<Schema: ApplicationSchema> SourceQueryReaderAccess<'_, Schema>`
- `application_query::output_source_observation::SourceQueryReaderAccess` held by `application_query::output_source_observation::SourceQueryObservationStop` in `declaration`
- `application_query::output_source_observation::SourceQueryReaderAccess` held by `application_query::output_source_observation::current_control::HistorySourceQueryControl` in `declaration`
- `application_query::output_source_observation::SourceQueryReaderAccess` held by `application_query::one_shot::WorthQueryApplicationOneShotResult` in `impl<Schema: ApplicationSchema> SourceQueryReaderAccess<'_, Schema>`
- `application_query::output_source_observation::SourceQueryReaderAccess` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl<'a, S: ApplicationSchema> SourceQueryReaderAccess<'a, S>`

### `output_lineage` ↔ `invariant_projection`: 2 references up in 1 items, default **contract**

Items: `invariant_projection::output_availability_candidate::ObservedOutputCandidate` (2)

- `invariant_projection::output_availability_candidate::ObservedOutputCandidate` held by `invariant_projection::output_availability_candidate::dependencies::PostingDependencies` in `declaration`
- `invariant_projection::output_availability_candidate::ObservedOutputCandidate` held by `invariant_projection::output_availability_candidate::dependencies::PostingDependencies` in `impl ObservedOutputCandidate`

