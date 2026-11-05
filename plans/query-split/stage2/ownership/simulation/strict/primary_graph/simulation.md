# Helper-move simulation: `domain_computation::primary_graph` (80 modules)

Before: component 59, 1506 references against the current layering.
After: component 59, 653 against the current layering, 739 against a fresh layering of the result.
Moved: 199 helpers and 326 units over 4 rounds. Of 1224 helpers blocking coupled units, 166 could move at once.

Remaining seams in the largest component: 89 (89 contract).

## Helpers blocking coupled units

| helper | owner | destination | units blocked | verdict |
|---|---|---|---|---|
| `application_query::managed_index::provider_failure::equal_cause` | application_query | application_program | 145 | blocked: application_query::managed_index::provider_failure::PaidProviderDenial (struct) |
| `application_contribution::conditional::output_readiness_contract::denial` | application_contribution | denial | 123 | blocked: application_contribution::conditional::output_readiness_contract::WorthQueryOutputReadinessContractDenial (struct) |
| `application_contribution::native_field_contract::field_denial` | application_contribution | application_program | 121 | blocked: application_contribution::native_field_contract::NativeFieldContractDenial (struct) |
| `application_contribution::conditional::output_readiness_contract::typed::field_denial` | application_contribution | application_program | 120 | blocked: application_contribution::conditional::output_readiness_contract::WorthQueryOutputReadinessContractDenial (struct) |
| `application_contribution::producer::denial_value::mismatch_or` | application_contribution | application_program | 120 | blocked: application_contribution::producer::denial_value::WorthQueryProducerDenialDecodeDenial (enum) |
| `application_contribution::producer::denial_value::declaration::encode_declared` | application_contribution | application_program | 117 | blocked: application_contribution::producer::denial_value::WorthQueryProducerDenialEncodingRefusal (enum), application_contribution::producer::denial_value::WorthQueryProducerDenialValue (enum), application_invariant::refusal::canonical_wire_maximum (fn) |
| `application_contribution::producer::registry::semantic_edition::posture_code` | application_contribution | application_program | 115 | could move |
| `application_query::resource_lifecycle::lifecycle_count::release` | application_query | application_program | 109 | blocked: application_query::resource_lifecycle::lifecycle_count::LifecycleCountUnderflow (struct) |
| `application_query::observed_source::source_identity::material_storage::admit_footprint_comparison` | application_query | application_program | 108 | blocked: application_query::observed_source::WorthQueryObservedSourceFootprint (struct) |
| `application_contribution::producer::demand::progression::required_wave::capacity_denial` | application_contribution | application_program | 105 | blocked: application_contribution::producer::demand::progression::required_wave::denial (fn), application_output_demand::denial::WorthQueryOutputDemandDenial (struct), application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind (enum) |
| `application_query::one_shot::denial::authorization_denial` | application_query | application_program | 102 | blocked: application_query::one_shot::denial::OneShotDenialPayload (struct), application_query::one_shot::denial::WorthQueryApplicationOneShotDenial (struct), application_query::one_shot::denial::WorthQueryApplicationOneShotDenialKind (enum) |
| `application_query::one_shot::denial::denial` | application_query | application_program | 102 | blocked: application_query::one_shot::denial::OneShotDenialPayload (struct), application_query::one_shot::denial::WorthQueryApplicationOneShotDenial (struct), application_query::one_shot::denial::WorthQueryApplicationOneShotDenialKind (enum) |
| `invariant_projection::consumed_output::verification::work_budget::map_admission_stop` | invariant_projection | application_program | 100 | blocked: invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop (enum), invariant_projection::consumed_output::verification::capacity_stop::VerificationCapacityStop (enum) |
| `application_query::component_result::inspect` | application_query | application_program | 99 | blocked: application_query::component_result::WorthQueryApplicationQueryComponents (struct), application_query::observed_source::result_set::WorthQueryObservedResultSet (struct) |
| `application_query::observed_source::source_identity::checkpoint_commitment::checkpoint_source_identity` | application_query | application_program | 99 | blocked: application_query::observed_source::WorthQueryObservedSourceFootprint (struct), application_query::observed_source::source_identity::checkpoint_commitment::checkpoint_source_identity_admitted (fn) |
| `application_query::orbit_result::inspect` | application_query | application_program | 99 | blocked: application_query::observed_source::result_set::WorthQueryObservedResultSet (struct), application_query::orbit_result::WorthQueryApplicationQueryOrbit (struct) |
| `application_query::projection::projected_tree::content_comparison::same_relation` | application_query | application_program | 99 | blocked: application_query::projection::projected_tree::WorthQueryApplicationProjectedRelation (struct), application_query::projection::projected_tree::content_comparison::ProjectionContentComparisonFault (enum), application_query::projection::projected_tree::content_comparison::same_relation_header (fn) |
| `application_query::projection::projected_tree::content_comparison::same_relation_header` | application_query | application_program | 99 | blocked: application_query::projection::projected_tree::WorthQueryApplicationProjectedRelation (struct), application_query::projection::projected_tree::content_comparison::same_text (fn) |
| `application_query::projection::projected_tree::content_comparison::same_row` | application_query | application_program | 99 | blocked: application_query::projection::projected_tree::WorthQueryApplicationProjectionNode (struct), application_query::projection::projected_tree::content_comparison::ProjectionContentComparisonFault (enum), application_query::projection::projected_tree::content_comparison::same_text (fn) |
| `application_query::projection::projected_tree::content_comparison::traversal::frames` | application_query | application_program | 99 | blocked: application_query::projection::projected_tree::content_comparison::ProjectionContentComparisonFault (enum), application_query::projection::projected_tree::content_comparison::traversal::Frame (struct), application_query::projection::projected_tree::content_comparison::traversal::Frames (struct) |
| `application_contribution::producer::demand::disclosure::denial` | application_contribution | application_program | 98 | blocked: application_output_demand::denial::WorthQueryOutputDemandDenial (struct), application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind (enum) |
| `application_contribution::producer::demand::disclosure::required_successor::validate_required_successor` | application_contribution | application_program | 98 | blocked: application_contribution::producer::WorthQueryProducerLifecyclePosture (enum), application_contribution::producer::WorthQueryProducerOutputFamily (trait), application_contribution::producer::demand::FamilySourceQuery (type) |
| `application_contribution::producer::demand::disclosure::validate_disclosure` | application_contribution | application_program | 98 | blocked: application_contribution::host::ApplicationContributionHost (trait), application_contribution::producer::WorthQueryProducerOutputFamily (trait), application_contribution::producer::demand::FamilySourceQuery (type) |
| `application_contribution::producer::demand::own_effect_observation::take` | application_contribution | application_program | 98 | blocked: application_contribution::producer::demand::own_effect_observation::Stage (enum) |
| `application_contribution::producer::demand::progression::denial` | application_contribution | application_program | 98 | blocked: application_output_demand::denial::WorthQueryOutputDemandDenial (struct), application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind (enum) |
| `application_contribution::producer::demand::progression::required_wave::denial` | application_contribution | application_program | 98 | blocked: application_output_demand::denial::WorthQueryOutputDemandDenial (struct), application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind (enum) |
| `application_contribution::producer::demand::progression::required_wave::performed::member_keys::same_key` | application_contribution | application_program | 98 | blocked: application_output_demand::registry::demand_key::WorthQueryOutputDemandKey (struct) |
| `application_contribution::producer::demand::progression::required_wave::required_settlement_denial` | application_contribution | application_program | 98 | blocked: application_contribution::producer::demand::progression::required_wave::admission_denial (fn), application_contribution::producer::demand::progression::required_wave::foreign_denial (fn), application_output_demand::denial::WorthQueryOutputDemandDenial (struct) |
| `application_contribution::producer::demand::progression::required_wave::selection::select_required_basis` | application_contribution | application_program | 98 | blocked: application_contribution::host::ApplicationContributionHost (trait), application_contribution::producer::demand::progression::required_wave::admission_denial (fn), application_contribution::producer::demand::progression::required_wave::position_denial (fn) |
| `application_contribution::producer::demand::progression::retained_read::authorize_history_source` | application_contribution | application_program | 98 | blocked: application_contribution::host::ApplicationContributionHost (trait), application_contribution::producer::WorthQueryProducerOutputFamily (trait), application_contribution::producer::demand::FamilySourceQuery (type) |
| `application_contribution::producer::demand::required_continuations::capacity_denial` | application_contribution | application_program | 98 | blocked: application_output_demand::denial::WorthQueryOutputDemandDenial (struct), application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind (enum) |
| `application_contribution::producer::demand::required_continuations::held::held_key` | application_contribution | application_program | 98 | blocked: application_contribution::producer::demand::required_continuations::held::HeldFrameSuccessor (struct), application_output_demand::registry::demand_key::WorthQueryOutputDemandKey (struct), application_output_demand::registry::held_successor::HeldRequiredSuccessor (type) |
| `application_contribution::producer::demand::required_continuations::progress::refused_custody` | application_contribution | application_program | 98 | blocked: application_output_demand::denial::WorthQueryOutputDemandDenial (struct), application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind (enum), application_output_demand::denial::kind::WorthQueryOutputDemandRecoveryPosture (enum) |
| `application_contribution::producer::demand::required_provenance::validation::EDITION_SUBJECT` | application_contribution | application_program | 98 | could move |
| `application_contribution::producer::demand::required_provenance::validation::MODE_SUBJECT` | application_contribution | application_program | 98 | could move |
| `application_contribution::producer::execution::denial::denial` | application_contribution | application_program | 98 | blocked: application_output_demand::denial::WorthQueryOutputDemandDenial (struct), application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind (enum) |
| `application_contribution::producer::execution::source_readmission::ready::ready_currentness_denial` | application_contribution | application_program | 98 | blocked: application_contribution::producer::execution::denial::ProducerExecutionStop (enum), application_output_demand::denial::WorthQueryOutputDemandDenial (struct), application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind (enum) |
| `application_contribution::producer::invocation::confirm_history_source` | application_contribution | application_program | 98 | blocked: application_contribution::host::ApplicationContributionHost (trait), application_contribution::producer::invocation::history_source_denial (fn), application_output_demand::denial::WorthQueryOutputDemandDenial (struct) |
| `application_query::projection::projection_denial` | application_query | application_program | 98 | blocked: application_query::projection::denial::WorthQueryApplicationProjectionDenial (struct), application_query::projection::denial::WorthQueryApplicationProjectionDenialKind (enum) |
| `application_query::projection::relation_cardinality_denial` | application_query | application_program | 98 | blocked: application_query::projection::denial::WorthQueryApplicationProjectionDenial (struct), application_query::projection::denial::WorthQueryApplicationProjectionDenialKind (enum), application_query::projection::projected_tree::WorthQueryApplicationProjectedRelation (struct) |
| `application_contribution::partitioned_computation::incremental::observed::observe_retained_full_tree` | application_contribution | application_program | 97 | blocked: application_contribution::partitioned_computation::incremental::observed::observe_tree (fn), application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause (enum), application_contribution::partitioned_computation::incremental::tree_report::WorthQueryPartitionedTreeAccounting (struct) |
| `application_contribution::producer::demand::bridge_denial::bridge_denial` | application_contribution | application_program | 96 | blocked: application_contribution::producer::demand::bridge_denial::bridge_denial_kind (fn), application_output_demand::denial::WorthQueryOutputDemandDenial (struct) |
| `application_contribution::producer::demand::progression::required_wave::admission_denial` | application_contribution | application_program | 96 | blocked: application_contribution::producer::demand::progression::required_wave::capacity_denial (fn), application_contribution::producer::demand::progression::required_wave::work_denial (fn), application_output_demand::denial::WorthQueryOutputDemandDenial (struct) |
| `application_contribution::producer::demand::progression::required_wave::resolved::match_one` | application_contribution | application_program | 96 | blocked: application_contribution::host::ApplicationContributionHost (trait), application_contribution::producer::demand::progression::required_wave::admission_denial (fn), application_contribution::producer::demand::progression::required_wave::required_settlement_denial (fn) |
| `application_contribution::producer::demand::required_continuations::promotion::prepare_typed_extraction` | application_contribution | application_program | 96 | blocked: application_contribution::producer::WorthQueryProducerOutputFamily (trait), application_contribution::producer::demand::required_continuations::RequiredFreshProgress (struct), application_contribution::producer::demand::required_continuations::TypedRequiredSuccessor (struct) |
| `application_contribution::producer::output_evaluation::consult` | application_contribution | application_program | 96 | blocked: application_output_demand::denial::WorthQueryOutputDemandDenial (struct), output_reuse::selection::RetainedFactStop (enum), output_reuse::selection::retained_fact_denial (fn) |
| `application_contribution::producer::output_evaluation::native::prepare` | application_contribution | application_program | 96 | blocked: application_attempt::fact::WorthQueryApplicationObservedFact (enum), application_contribution::producer::output_equivalence_contract::InstalledProducerOutputEquivalence (struct), application_contribution::producer::output_evaluation::NativeOutputBasis (struct) |
| `application_contribution::producer::output_evaluation::next_comparison_attempt` | application_contribution | application_program | 96 | could move |
| `application_query::observed_source::source_identity::membership::same_footprint_membership` | application_query | application_program | 96 | blocked: application_query::observed_source::WorthQueryObservedSourceFootprint (struct), application_query::observed_source::source_identity::membership::same_adjacencies (fn), application_query::observed_source::source_identity::membership::same_entities (fn) |
| `invariant_projection::consumed_output::verification::work_budget::reserve_pending` | invariant_projection | application_program | 96 | blocked: invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop (enum), invariant_projection::consumed_output::verification::capacity_stop::VerificationCapacityStop (enum) |
| `application_contribution::producer::execution::admitted::execute_fresh` | application_contribution | application_program | 95 | blocked: application_contribution::host::ApplicationContributionHost (trait), application_contribution::producer::ProducerSourceQuery (type), application_contribution::producer::ProducerSourceValue (type) |
| `application_contribution::producer::execution::current_control::select_invocation_control` | application_contribution | application_program | 95 | blocked: application_contribution::host::ApplicationContributionHost (trait), application_contribution::producer::execution::current_control::select_history_control (fn), application_contribution::producer::execution::denial::ProducerExecutionStop (enum) |
| `application_contribution::producer::execution::denial::failed` | application_contribution | application_program | 95 | blocked: application_contribution::producer::execution::denial::denial (fn), application_output_demand::denial::WorthQueryOutputDemandDenial (struct), application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind (enum) |
| `application_contribution::producer::execution::input_reuse::require_selected_program` | application_contribution | application_program | 95 | blocked: application_contribution::producer::WorthQueryProducerCommitAuthority (enum), application_output_demand::denial::WorthQueryOutputDemandDenial (struct), application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind (enum) |
| `application_contribution::producer::execution::selected::execution_plan::prepare_selected_execution` | application_contribution | application_program | 95 | blocked: application_contribution::host::ApplicationContributionHost (trait), application_contribution::producer::ProducerSourceQuery (type), application_contribution::producer::ProducerSourceValue (type) |
| `application_contribution::producer::execution::selected_public::denial::preparation_stop` | application_contribution | application_program | 95 | blocked: application_contribution::producer::WorthQueryApplicationProducerBinding (trait), application_contribution::producer::execution::denial::ProducerExecutionStop (enum), application_contribution::producer::execution::denial::principal_rejected (fn) |
| `application_contribution::producer::execution::selected_public::prepare_recovery_public_mutation` | application_contribution | application_program | 95 | blocked: application_contribution::host::ApplicationContributionHost (trait), application_contribution::producer::WorthQueryApplicationProducerBinding (trait), application_contribution::producer::execution::installed::TypedInstalledProducer (struct) |
| `application_contribution::producer::execution::source_readmission::readmit_source` | application_contribution | application_program | 95 | blocked: application_contribution::host::ApplicationContributionHost (trait), application_contribution::producer::ProducerSourceQuery (type), application_contribution::producer::ProducerSourceValue (type) |
| `application_contribution::producer::execution::source_readmission::ready::ready_resource_denial` | application_contribution | application_program | 95 | blocked: application_contribution::producer::execution::denial::ProducerExecutionStop (enum), application_output_demand::denial::WorthQueryOutputDemandDenial (struct), application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind (enum) |
| `application_contribution::producer::execution::source_readmission::ready_on_selected::certify_ready_on_selected_with_disclosure` | application_contribution | application_program | 95 | blocked: application_contribution::host::ApplicationContributionHost (trait), application_contribution::producer::ProducerSourceBinding (type), application_contribution::producer::ProducerSourceQuery (type) |

## Remaining seams

### `product_operation` ↔ `application_query`: 39 references up in 19 items, default **contract**

Items: `application_query::resource_lifecycle::basis_registry::lease::WorthQueryApplicationBasisLease` (6), `application_query::admission::AdmissionScopedOps` (6), `application_query::controls::WorthQueryApplicationQueryControls` (6), `application_query::observed_source::WorthQueryObservedSource` (2), `application_query::projection::WorthQueryApplicationProjection` (2), `application_query::managed_index::denial::WorthQueryManagedIndexDenial` (2), `application_query::basis::execution_custody::WorthQueryApplicationQueryBasisCustody` (2), `application_query::one_shot::WorthQueryApplicationOneShotResult` (2)

- `application_query::observed_source::WorthQueryObservedSource` held by `schema_layout::WorthQueryPrimaryGraphLayout` in `impl<Query> WorthQueryObservedSource<Query>`
- `application_query::observed_source::WorthQueryObservedSource` held by `application_query::observed_source::WorthQueryObservedSourceFootprint` in `impl<Query> WorthQueryObservedSource<Query>`
- `application_query::observed_source::WorthQueryObservedSource` held by `application_attempt::fact::WorthQueryApplicationObservedFact` in `impl<Query> WorthQueryObservedSource<Query>`
- `application_query::document_history::WorthQueryDocumentHistoryRead` held by `application_query::projection::WorthQueryApplicationProjection` in `impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> WorthQueryDocumentHi`
- `application_query::observed_source::WorthQueryObservedSource` held by `application_query::observed_source::fact_conversion::ValidatedSourceField` in `impl<Query> WorthQueryObservedSource<Query>`
- `application_query::observed_source::WorthQueryObservedSource` held by `application_query::observed_source::fact_conversion::source_fact_count` in `impl<Query> WorthQueryObservedSource<Query>`

### `application_output_demand` ↔ `output_lineage`: 31 references up in 10 items, default **contract**

Items: `output_lineage::invalidation::owner::SourceInvalidationOwner` (11), `invariant_projection::consumed_output::ConsumedOutputEvidence` (5), `output_lineage::native_output_witness::SealedNativeOutputWitness` (3), `output_lineage::current_output::RetainedOutputCurrentnessRead` (3), `output_lineage::input_reuse_key::PreparedInputReuseKey` (2), `output_lineage::retained_computation::PriorComputationRecord` (2), `output_lineage::required_settlement::current_accepted::BoundCurrentAcceptedOutput` (2), `output_lineage::invalidation::mark_state::FullVerificationReason` (1)

- `invariant_projection::consumed_output::ConsumedOutputEvidence` held by `invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop` in `impl ConsumedOutputEvidence`
- `output_lineage::invalidation::owner::SourceInvalidationOwner` held by `output_lineage::invalidation::derived::SettlementRegistrationStop` in `impl SourceInvalidationOwner`
- `invariant_projection::consumed_output::ConsumedOutputEvidence` held by `output_lineage::invalidation::owner::SourceInvalidationOwner` in `impl ConsumedOutputEvidence`
- `invariant_projection::consumed_output::ConsumedOutputEvidence` held by `invariant_projection::consumed_output::verification::ConsumedOutputVerification` in `impl ConsumedOutputEvidence`
- `output_lineage::invalidation::owner::SourceInvalidationOwner` held by `output_lineage::invalidation::source_alignment::SnapshotAlignedMarkState` in `impl SourceInvalidationOwner`
- `invariant_projection::consumed_output::ConsumedOutputEvidence` held by `output_lineage::invalidation::mark_state::FullVerificationReason` in `impl ConsumedOutputEvidence`

### `application_checkpoint` ↔ `product_operation`: 29 references up in 3 items, default **contract**

Items: `application_attempt::retained_decision_facts::StoreDenial` (14), `application_attempt::fact::WorthQueryApplicationObservedFact` (14), `output_lineage::computation_source::ComputationSourceEvidence` (1)

- `application_attempt::fact::WorthQueryApplicationObservedFact` held by `application_attempt::fact::source_currentness::WorthQuerySourceCurrentnessFailure` in `impl WorthQueryApplicationObservedFact`
- `output_lineage::computation_source::ComputationSourceEvidence` held by `output_lineage::source_facts::RetainedSourceFacts` in `impl ComputationSourceEvidence`
- `application_attempt::retained_decision_facts::StoreDenial` held by `application_attempt::denial::WorthQueryApplicationAttemptDenial` in `impl StoreDenial`
- `application_attempt::fact::WorthQueryApplicationObservedFact` held by `application_attempt::denial::WorthQueryApplicationAttemptDenial` in `impl WorthQueryApplicationObservedFact`
- `application_attempt::fact::WorthQueryApplicationObservedFact` held by `application_attempt::fact::workflow_history_basis::remains_equal` in `impl WorthQueryApplicationObservedFact`
- `application_attempt::fact::WorthQueryApplicationObservedFact` held by `source_read_verification::SourceReadVerification` in `impl WorthQueryApplicationObservedFact`

### `product_operation` ↔ `application_attempt`: 29 references up in 23 items, default **contract**

Items: `application_attempt::workflow::adoption::request::WorthQueryWorkflowDispositions` (5), `application_attempt::workflow::correction::PreparedWorkflowCorrection` (2), `application_attempt::workflow::adoption::occurrence::WorthQueryWorkflowAdoptionInventory` (2), `application_attempt::retained_decision_facts::control::StorageControl` (1), `application_attempt::effect_program::model::WorthQueryApplicationEffectProgram` (1), `application_attempt::host::ApplicationAttemptHost` (1), `application_attempt::workflow::adoption::inventory::WorkflowAdoptionInventoryRequest` (1), `application_attempt::compare_and_commit::commit_receipt::committed_changes::WorthQueryApplicationCommittedChanges` (1)

- `application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenial` held by `application_attempt::compare_and_commit::commit_outcome::denial::denial_cause::DenialCause` in `impl WorthQueryApplicationCommitDenial`
- `application_attempt::effect_program::model::WorthQueryApplicationEffectProgram` held by `application_attempt::denial::WorthQueryApplicationAttemptDenial` in `impl<Schema, Operation, Input, Scope> WorthQueryApplicationEffectProgram<Schema,`
- `application_attempt::effect_program::model::WorthQueryApplicationEffectProgram` held by `application_attempt::read_set::WorthQueryCompleteApplicationReadSet` in `impl<Schema, Operation, Input, Scope> WorthQueryApplicationEffectProgram<Schema,`
- `application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenial` held by `application_attempt::denial::WorthQueryApplicationAttemptDenial` in `impl WorthQueryApplicationCommitDenial`
- `application_attempt::effect_program::model::WorthQueryApplicationEffectProgram` held by `application_attempt::workflow::actor::record::WorkflowAdmittedActor` in `impl<Schema, Operation, Input, Scope> WorthQueryApplicationEffectProgram<Schema,`
- `application_attempt::effect_program::model::WorthQueryApplicationEffectProgram` held by `application_attempt::workflow::instance::start_provenance::admit` in `impl<Schema, Operation, Input, Scope> WorthQueryApplicationEffectProgram<Schema,`

### `output_lineage` ↔ `application_contribution`: 23 references up in 9 items, default **contract**

Items: `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` (5), `application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` (4), `application_contribution::producer::demand::progression::required_wave::resolved::matched_predecessors::MatchedRequiredPredecessors` (4), `application_contribution::producer::registry::semantic_edition::InstalledProducerEdition` (3), `application_contribution::partitioned_computation::incremental::retained::SealedComputationRun` (2), `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` (2), `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` (1), `application_contribution::producer::source_qualification::QualifiedOutputSource` (1)

- `application_contribution::partitioned_computation::incremental::retained::SealedComputationRun` held by `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` in `impl SealedComputationRun`
- `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` held by `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` in `impl PriorAbsence`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::producer::registry::semantic_edition::InstalledProducerEdition` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `output_lineage::retained_computation::PriorComputationRecord` in `impl ComputationPrior`

### `application_attempt` ↔ `provider`: 20 references up in 9 items, default **contract**

Items: `provider::WorthQueryPrimaryGraphProvider` (10), `provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::PreparedSourceFactRebase` (2), `provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::OwnEffectOnReads` (2), `provider::host::ProviderHost` (1), `provider::application_decision_fact::application_table::ApplicationFactTable` (1), `provider::pending_application_publication::registry::WorthQueryApplicationPublicationRecoveryReservation` (1), `provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::diagnostic::RebaseVerificationReason` (1), `provider::committed_dispatch_outbox::ProviderCommittedDispatchOutboxScopedOps` (1)

- `provider::application_decision_fact::application_table::ApplicationFactTable` held by `provider::application_decision_fact::WorthQueryPrimaryGraphApplicationDecisionFact` in `impl ApplicationFactTable`
- `provider::WorthQueryPrimaryGraphProvider` held by `application_query::derived_view::registry::predicate::preparation::PredicatePreparationRetirement` in `impl WorthQueryPrimaryGraphProvider`
- `provider::WorthQueryPrimaryGraphProvider` held by `application_query::derived_view::registry::predicate::preparation::PredicatePreparationOwner` in `impl WorthQueryPrimaryGraphProvider`
- `provider::WorthQueryPrimaryGraphProvider` held by `provider::application_attempt_state::WorthQueryPrimaryGraphApplicationAttemptStore` in `impl WorthQueryPrimaryGraphProvider`
- `provider::WorthQueryPrimaryGraphProvider` held by `provider::pending_application_publication::registry::WorthQueryPendingApplicationPublicationRegistry` in `impl WorthQueryPrimaryGraphProvider`
- `provider::WorthQueryPrimaryGraphProvider` held by `provider::invariant_execution_failure::map_validation_failure` in `impl WorthQueryPrimaryGraphProvider`

### `output_lineage` ↔ `invariant_projection`: 19 references up in 7 items, default **contract**

Items: `invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop` (8), `invariant_projection::consumed_output::verification::ConsumedOutputVerification` (5), `invariant_projection::output_availability_candidate::ObservedOutputCandidate` (2), `invariant_projection::consumed_output::pending_dependency::SelectedPendingConsumedOutput` (1), `invariant_projection::consumed_output::verification::work_budget::map_admission_stop` (1), `invariant_projection::consumed_output::verification::work_budget::reserve_pending` (1), `invariant_projection::consumed_output::publication_recovery::PublicationRecoveryStop` (1)

- `invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop` held by `application_query::managed_index::invocation::WorthQueryManagedIndexWorkDenial` in `declaration`
- `invariant_projection::consumed_output::pending_dependency::SelectedPendingConsumedOutput` held by `invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop` in `impl<'selected> SelectedPendingConsumedOutput<'selected>`
- `invariant_projection::consumed_output::pending_dependency::SelectedPendingConsumedOutput` held by `output_lineage::native_output_witness::SealedNativeOutputWitness` in `impl<'selected> SelectedPendingConsumedOutput<'selected>`
- `invariant_projection::consumed_output::verification::ConsumedOutputVerification` held by `invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop` in `impl ConsumedOutputVerification`
- `invariant_projection::consumed_output::verification::work_budget::map_admission_stop` held by `invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop` in `body`
- `invariant_projection::consumed_output::verification::work_budget::map_admission_stop` held by `invariant_projection::consumed_output::verification::capacity_stop::VerificationCapacityStop` in `body`

### `application_discovery` ↔ `application_attempt`: 17 references up in 10 items, default **contract**

Items: `application_attempt::fact::adjacency::observe_adjacency` (4), `application_attempt::workflow::schema::layout::WorthQueryWorkflowLayout` (3), `application_attempt::workflow_instance_observation::governed_record` (2), `application_attempt::workflow_instance_observation::read_only::ReadOnlyWorkflowHistory` (2), `application_attempt::observation::observe_field_value` (1), `application_attempt::workflow_instance_observation::read_fields::ReadFields` (1), `application_attempt::workflow::schema::version::WORKFLOW_INSTANCE_FACT_PROTOCOL_VERSION` (1), `application_attempt::workflow_instance_observation::history_projection` (1)

- `application_attempt::observation::observe_field_value` held by `application_attempt::observation::WorthQueryApplicationFieldObservation` in `body`
- `application_attempt::observation::observe_field_value` held by `application_attempt::observation::observe_field` in `body`
- `application_attempt::fact::adjacency::observe_adjacency` held by `application_attempt::denial::WorthQueryApplicationAttemptDenial` in `body`
- `application_attempt::fact::adjacency::observe_adjacency` held by `application_attempt::fact::WorthQueryApplicationObservedRelation` in `body`
- `application_attempt::fact::adjacency::observe_adjacency` held by `application_attempt::fact::adjacency::observe_adjacency_checked` in `body`
- `application_attempt::fact::adjacency::observe_adjacency` held by `application_attempt::fact::adjacency::resolve_observation` in `body`

### `product_operation` ↔ `application_contribution`: 16 references up in 3 items, default **contract**

Items: `application_contribution::producer::WorthQueryApplicationProducerBinding` (12), `application_contribution::producer::WorthQueryApplicationProducerProvider` (3), `application_contribution::producer::registry::WorthQueryInstalledApplicationProducerRegistry` (1)

- `application_contribution::producer::registry::WorthQueryInstalledApplicationProducerRegistry` held by `application_contribution::producer::registry::InstalledProducerProvider` in `impl<Schema> WorthQueryInstalledApplicationProducerRegistry<Schema>`
- `application_contribution::producer::registry::WorthQueryInstalledApplicationProducerRegistry` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl<Schema> WorthQueryInstalledApplicationProducerRegistry<Schema>`
- `application_contribution::producer::registry::WorthQueryInstalledApplicationProducerRegistry` held by `application_contribution::producer::registry::InstalledProducerProvider` in `declaration`
- `application_contribution::producer::registry::WorthQueryInstalledApplicationProducerRegistry` held by `application_contribution::producer::WorthQueryApplicationProducerBinding` in `impl<Schema> super::WorthQueryInstalledApplicationProducerRegistry<Schema>`
- `application_contribution::producer::registry::WorthQueryInstalledApplicationProducerRegistry` held by `application_contribution::producer::WorthQueryApplicationProducerBinding` in `impl<Schema> WorthQueryInstalledApplicationProducerRegistry<Schema>`
- `application_contribution::producer::registry::WorthQueryInstalledApplicationProducerRegistry` held by `application_contribution::producer::scheduling::WorthQueryInstalledOutputProducerRoutes` in `impl<Schema> WorthQueryInstalledApplicationProducerRegistry<Schema>`

### `application_checkpoint` ↔ `application_attempt`: 15 references up in 2 items, default **contract**

Items: `application_attempt::retained_decision_facts::control::StorageControl` (14), `application_attempt::retained_decision_facts::endpoints::AdmittedAdjacencyEndpoints` (1)

- `application_attempt::retained_decision_facts::control::StorageControl` held by `application_attempt::retained_decision_facts::StoreDenial` in `impl<'scope, 'authority> StorageControl<'scope, 'authority>`
- `application_attempt::retained_decision_facts::endpoints::AdmittedAdjacencyEndpoints` held by `application_attempt::denial::WorthQueryApplicationAttemptDenial` in `impl AdmittedAdjacencyEndpoints`
- `application_attempt::retained_decision_facts::endpoints::AdmittedAdjacencyEndpoints` held by `application_attempt::retained_decision_facts::endpoints::heap_sort` in `impl AdmittedAdjacencyEndpoints`
- `application_attempt::retained_decision_facts::endpoints::AdmittedAdjacencyEndpoints` held by `application_attempt::retained_decision_facts::endpoints::union_visit` in `impl AdmittedAdjacencyEndpoints`

### `application_attempt` ↔ `invariant_projection`: 15 references up in 6 items, default **contract**

Items: `invariant_projection::operation_reader::decision_reads::expected_keys::ExpectedFactKeys` (6), `invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop` (5), `invariant_projection::operation_reader::borrowed::BorrowedOperationProjection` (1), `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` (1), `invariant_projection::realized_scope::WorthQueryRealizedProjectionScope` (1), `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionSnapshot` (1)

- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::WorthQueryApplicationInvariantProjectionSnapshot` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::admission_denial::from_branch_basis_denial` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::admission_denial::from_snapshot_admission_denial` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::locked_reader::WorthQueryApplicationInvariantProjectionReader` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::locked_reader::WorthQueryCompletedInvariantProjection` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`

### `application_output_demand` ↔ `application_query`: 14 references up in 6 items, default **contract**

Items: `application_query::observed_source::retained_dependencies::WorthQueryRetainedQueryDependencies` (4), `application_query::managed_index::invocation::WorthQueryManagedIndexWorkDenial` (3), `application_query::observed_source::source_identity::WorthQueryObservedSourceEpoch` (3), `application_query::managed_index::denial::WorthQueryManagedIndexDenial` (2), `application_query::observed_source::WorthQueryObservedSource` (1), `application_query::managed_index::denial::causes::WorthQueryManagedIndexDenialCause` (1)

- `application_query::observed_source::WorthQueryObservedSource` held by `schema_layout::WorthQueryPrimaryGraphLayout` in `impl<Query> WorthQueryObservedSource<Query>`
- `application_query::observed_source::WorthQueryObservedSource` held by `application_query::observed_source::WorthQueryObservedSourceFootprint` in `impl<Query> WorthQueryObservedSource<Query>`
- `application_query::observed_source::WorthQueryObservedSource` held by `application_attempt::fact::WorthQueryApplicationObservedFact` in `impl<Query> WorthQueryObservedSource<Query>`
- `application_query::observed_source::source_identity::WorthQueryObservedSourceEpoch` held by `application_query::observed_source::source_identity::WorthQueryObservedSourceMeaning` in `impl WorthQueryObservedSourceEpoch`
- `application_query::observed_source::WorthQueryObservedSource` held by `application_query::observed_source::fact_conversion::ValidatedSourceField` in `impl<Query> WorthQueryObservedSource<Query>`
- `application_query::observed_source::WorthQueryObservedSource` held by `application_query::observed_source::fact_conversion::source_fact_count` in `impl<Query> WorthQueryObservedSource<Query>`

### `application_attempt` ↔ `output_lineage`: 14 references up in 3 items, default **contract**

Items: `invariant_projection::consumed_output::ConsumedOutputEvidence` (7), `output_lineage::invalidation::computation_changes::postcondition_carry::PostconditionCarry` (4), `output_lineage::invalidation::output_availability::ObservedSourcePublication` (3)

- `invariant_projection::consumed_output::ConsumedOutputEvidence` held by `invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop` in `impl ConsumedOutputEvidence`
- `invariant_projection::consumed_output::ConsumedOutputEvidence` held by `output_lineage::invalidation::owner::SourceInvalidationOwner` in `impl ConsumedOutputEvidence`
- `invariant_projection::consumed_output::ConsumedOutputEvidence` held by `invariant_projection::consumed_output::verification::ConsumedOutputVerification` in `impl ConsumedOutputEvidence`
- `invariant_projection::consumed_output::ConsumedOutputEvidence` held by `output_lineage::invalidation::mark_state::FullVerificationReason` in `impl ConsumedOutputEvidence`
- `invariant_projection::consumed_output::ConsumedOutputEvidence` held by `output_lineage::native_output_witness::SealedNativeOutputWitness` in `impl ConsumedOutputEvidence`
- `invariant_projection::consumed_output::ConsumedOutputEvidence` held by `invariant_projection::consumed_output::verification::work_budget::map_admission_stop` in `impl ConsumedOutputEvidence`

### `application_attempt` ↔ `application_output_demand`: 12 references up in 4 items, default **contract**

Items: `application_output_demand::settlement::WorthQueryOutputDemandSettlement` (5), `application_output_demand::registry::required_context::RequiredOutputDemandContext` (5), `application_output_demand::settlement::SettlementScopedOps` (1), `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` (1)

- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::readiness_delivery::WorthQueryOutputReadinessDeliveryEvidence` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::registry::WorthQueryAcceptedOutputAuthority` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::settlement::receipt_custody::SettlementReceiptCustody` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::registry::restoration::WorthQueryRestoredAcceptedOutput` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::registry::required_context::RequiredOutputDemandContext` held by `application_output_demand::registry::prerequisite_claims::PreparedPrerequisiteClaims` in `impl RequiredOutputDemandContext`

### `application_attempt` ↔ `application_contribution`: 12 references up in 5 items, default **contract**

Items: `application_contribution::partitioned_computation::incremental::prior_absence::CompletedComputationRetention` (4), `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` (3), `application_contribution::partitioned_computation::incremental::native_reads::NativeComputationReads` (3), `application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` (1), `application_contribution::execution_denial` (1)

- `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` held by `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` in `impl PriorAbsence`
- `application_contribution::partitioned_computation::incremental::prior_absence::CompletedComputationRetention` held by `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` in `declaration`
- `application_contribution::partitioned_computation::incremental::prior_absence::CompletedComputationRetention` held by `application_contribution::partitioned_computation::incremental::retained::CompletedComputationRun` in `declaration`
- `application_contribution::partitioned_computation::incremental::prior_absence::CompletedComputationRetention` held by `application_contribution::partitioned_computation::incremental::native_reads::NativeComputationReads` in `impl CompletedComputationRetention`
- `application_contribution::partitioned_computation::incremental::prior_absence::CompletedComputationRetention` held by `application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` in `impl CompletedComputationRetention`
- `application_contribution::partitioned_computation::incremental::native_reads::NativeComputationReads` held by `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` in `declaration`

### `application_attempt` ↔ `application_query`: 12 references up in 5 items, default **contract**

Items: `application_query::observed_source::WorthQueryObservedSource` (6), `application_query::managed_index::invocation::WorthQueryManagedIndexWorkDenial` (3), `application_query::observed_source::result_set::WorthQueryObservedResultSet` (1), `application_query::resource_lifecycle::basis_registry::lease::WorthQueryApplicationBasisLease` (1), `application_query::observed_source::query_context_binding::ObservedQueryExpressionContext` (1)

- `application_query::observed_source::WorthQueryObservedSource` held by `schema_layout::WorthQueryPrimaryGraphLayout` in `impl<Query> WorthQueryObservedSource<Query>`
- `application_query::observed_source::WorthQueryObservedSource` held by `application_query::observed_source::WorthQueryObservedSourceFootprint` in `impl<Query> WorthQueryObservedSource<Query>`
- `application_query::observed_source::WorthQueryObservedSource` held by `application_attempt::fact::WorthQueryApplicationObservedFact` in `impl<Query> WorthQueryObservedSource<Query>`
- `application_query::observed_source::WorthQueryObservedSource` held by `application_query::observed_source::fact_conversion::ValidatedSourceField` in `impl<Query> WorthQueryObservedSource<Query>`
- `application_query::observed_source::WorthQueryObservedSource` held by `application_query::observed_source::fact_conversion::source_fact_count` in `impl<Query> WorthQueryObservedSource<Query>`
- `application_query::observed_source::WorthQueryObservedSource` held by `application_output_demand::settlement::observation::OriginalSettledProducerObservation` in `impl<Query> WorthQueryObservedSource<Query>`

### `application_output_demand` ↔ `application_contribution`: 11 references up in 6 items, default **contract**

Items: `application_contribution::producer::output_evaluation::ProducerOutputEvaluation` (3), `application_contribution::producer::denial_value::WorthQueryProducerDenialValue` (3), `application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` (2), `application_contribution::producer::demand::bridge_denial::bridge_denial_kind` (1), `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` (1), `application_contribution::producer::execution::unpublished::PendingProducerPublication` (1)

- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::producer::registry::semantic_edition::InstalledProducerEdition` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `output_lineage::retained_computation::PriorComputationRecord` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` held by `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` in `declaration`
- `application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` held by `application_contribution::partitioned_computation::incremental::retained::SealedComputationRun` in `declaration`

### `causal_inspection` ↔ `application_query`: 10 references up in 7 items, default **contract**

Items: `application_query::admission::permission::fresh::access::AccessScopedOps` (3), `application_query::observed_source::WorthQueryObservedSource` (2), `application_query::admitted_result::WorthQueryApplicationOutputDemandSource` (1), `application_query::projection::retained_projection::comparison::compare_original_projections` (1), `application_query::admission::permission::read_authorization::ReadAuthorizationScopedOps` (1), `application_query::admission::permission::selected_access::SelectedIssuedAccessRoot` (1), `application_query::projection::retained_projection::materialization::materialize_original_projection` (1)

- `application_query::observed_source::WorthQueryObservedSource` held by `schema_layout::WorthQueryPrimaryGraphLayout` in `impl<Query> WorthQueryObservedSource<Query>`
- `application_query::observed_source::WorthQueryObservedSource` held by `application_query::observed_source::WorthQueryObservedSourceFootprint` in `impl<Query> WorthQueryObservedSource<Query>`
- `application_query::observed_source::WorthQueryObservedSource` held by `application_attempt::fact::WorthQueryApplicationObservedFact` in `impl<Query> WorthQueryObservedSource<Query>`
- `application_query::observed_source::WorthQueryObservedSource` held by `application_query::observed_source::fact_conversion::ValidatedSourceField` in `impl<Query> WorthQueryObservedSource<Query>`
- `application_query::observed_source::WorthQueryObservedSource` held by `application_query::observed_source::fact_conversion::source_fact_count` in `impl<Query> WorthQueryObservedSource<Query>`
- `application_query::observed_source::WorthQueryObservedSource` held by `application_output_demand::settlement::observation::OriginalSettledProducerObservation` in `impl<Query> WorthQueryObservedSource<Query>`

### `invariant_projection` ↔ `application_contribution`: 9 references up in 7 items, default **contract**

Items: `application_contribution::partitioned_computation::incremental::prior_absence::CompletedComputationRetention` (2), `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` (2), `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` (1), `application_contribution::partitioned_computation::incremental::native_reads::NativeComputationReads` (1), `application_contribution::partitioned_computation::incremental::deposit::ComputationDeposit` (1), `application_contribution::producer::source_qualification::SourceQualificationScopedOps` (1), `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` (1)

- `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` held by `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` in `impl PriorAbsence`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::producer::registry::semantic_edition::InstalledProducerEdition` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `output_lineage::retained_computation::PriorComputationRecord` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::prior_absence::CompletedComputationRetention` held by `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` in `declaration`

### `product_operation` ↔ `application_installation`: 9 references up in 6 items, default **contract**

Items: `application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime` (3), `application_installation::program::program_owner::contract::WorthQueryProgramOwner` (2), `application_installation::program::demand::root_completion::OriginalProgramRootCompletion` (1), `application_installation::program::demand::WorthQueryAdmittedProgramOutput` (1), `application_installation::program::specialized_action::WorthQueryAdmittedProgramOperation` (1), `application_installation::program::demand::WorthQuerySettledProgramOutput` (1)

- `application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>`
- `application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>> WorthQu`
- `application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime` held by `application_installation::program::demand::WorthQueryAdmittedProgramOutput` in `impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>`
- `application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime` held by `application_query::admitted_result::WorthQueryApplicationOutputDemandSource` in `impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>`
- `application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime` held by `application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenial` in `impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>`
- `application_installation::program::program_runtime::WorthQueryProgramApplicationRuntime` held by `application_installation::program::program_owner::WorthQuerySelectedProgramOwner` in `impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>`

### `access_strategy` ↔ `application_query`: 9 references up in 3 items, default **contract**

Items: `application_query::managed_index::invocation::WorthQueryManagedIndexWorkDenial` (5), `application_query::managed_index::source_structure::SourceTree` (3), `application_query::managed_index::source_rows::SourceImageView` (1)

- `application_query::managed_index::source_structure::SourceTree` held by `application_query::managed_index::source_rows::SourceImageView` in `impl SourceTree`
- `application_query::managed_index::invocation::WorthQueryManagedIndexWorkDenial` held by `application_query::managed_index::provider_failure::WorthQueryManagedIndexProviderFailure` in `declaration`
- `application_query::managed_index::source_structure::SourceTree` held by `application_query::managed_index::nodes::pages::NodePages` in `declaration`
- `application_query::managed_index::source_structure::SourceTree` held by `application_query::managed_index::source_structure::cold::acyclic` in `impl SourceTree`
- `application_query::managed_index::source_structure::SourceTree` held by `application_query::managed_index::nodes::pages::NodePages` in `impl SourceTree`
- `application_query::managed_index::source_structure::SourceTree` held by `application_query::managed_index::source_structure::cold_columns::column` in `impl SourceTree`

### `operation_body` ↔ `handler`: 8 references up in 2 items, default **contract**

Items: `handler::candidate::CandidateWriter` (5), `handler::invariant::DecisionReader` (3)

- `handler::invariant::DecisionReader` held by `application_attempt::denial::WorthQueryApplicationAttemptDenial` in `impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> DecisionReader<'_, `
- `handler::candidate::CandidateWriter` held by `application_attempt::denial::WorthQueryApplicationAttemptDenial` in `impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>> Can`
- `handler::invariant::DecisionReader` held by `handler::decision_reads::indexed_selection::resolution_denial` in `impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>> Dec`
- `handler::invariant::DecisionReader` held by `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` in `impl<'borrow, 'reader, 'runtime, Schema, Binding> DecisionReader<'borrow, 'reade`
- `handler::candidate::CandidateWriter` held by `application_attempt::denial::WorthQueryApplicationAttemptDenial` in `impl<Schema, Binding> CandidateWriter<'_, Schema, Binding>`
- `handler::invariant::DecisionReader` held by `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` in `declaration`

### `application_attempt` ↔ `application_installation`: 7 references up in 2 items, default **contract**

Items: `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` (5), `application_installation::program::workflow_runtime::authentication::workflow_approval_authentication_intent` (2)

- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_installation::host::ApplicationInstallationHost` in `declaration`
- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_contribution::producer::demand::WorthQueryAdmittedOutputDemand` in `impl<Schema, Spec> WorthQueryWorkflowVocabulary<'_, Schema, Spec>`
- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_output_demand::denial::WorthQueryOutputDemandDenial` in `impl<Schema, Spec> WorthQueryWorkflowVocabulary<'_, Schema, Spec>`
- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl<Schema, Spec> WorthQueryWorkflowVocabulary<'_, Schema, Spec>`
- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_query::admitted_result::WorthQueryApplicationOutputDemandSource` in `impl<Schema, Spec> WorthQueryWorkflowVocabulary<'_, Schema, Spec>`
- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_installation::host::ApplicationInstallationHost` in `impl<'vocabulary, Schema, Spec> WorthQueryWorkflowVocabulary<'vocabulary, Schema`

### `product_operation` ↔ `application_output_demand`: 6 references up in 2 items, default **contract**

Items: `application_output_demand::settlement::WorthQueryOutputDemandSettlement` (3), `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` (3)

- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::readiness_delivery::WorthQueryOutputReadinessDeliveryEvidence` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::registry::WorthQueryAcceptedOutputAuthority` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::settlement::receipt_custody::SettlementReceiptCustody` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::registry::restoration::WorthQueryRestoredAcceptedOutput` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::readiness_delivery::WorthQueryOutputReadinessDeliveryEvidence` in `declaration`

### `application_output_demand` ↔ `invariant_projection`: 5 references up in 5 items, default **contract**

Items: `invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop` (1), `invariant_projection::operation_reader::prepared_inputs::PreparedCurrentInputs` (1), `invariant_projection::consumed_output::pending_dependency::SelectedPendingConsumedOutput` (1), `invariant_projection::consumed_output::verification::ConsumedOutputVerification` (1), `invariant_projection::consumed_output::verification::work_budget::map_verification_stop` (1)

- `invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop` held by `application_query::managed_index::invocation::WorthQueryManagedIndexWorkDenial` in `declaration`
- `invariant_projection::operation_reader::prepared_inputs::PreparedCurrentInputs` held by `invariant_projection::consumed_output::ConsumedOutputEvidence` in `declaration`
- `invariant_projection::operation_reader::prepared_inputs::PreparedCurrentInputs` held by `invariant_projection::consumed_output::ConsumedOutputEvidence` in `impl PreparedCurrentInputs<'_>`
- `invariant_projection::consumed_output::pending_dependency::SelectedPendingConsumedOutput` held by `invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop` in `impl<'selected> SelectedPendingConsumedOutput<'selected>`
- `invariant_projection::consumed_output::pending_dependency::SelectedPendingConsumedOutput` held by `output_lineage::native_output_witness::SealedNativeOutputWitness` in `impl<'selected> SelectedPendingConsumedOutput<'selected>`
- `invariant_projection::consumed_output::verification::ConsumedOutputVerification` held by `invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop` in `impl ConsumedOutputVerification`

### `operation_body` ↔ `invariant_projection`: 5 references up in 2 items, default **contract**

Items: `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` (4), `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` (1)

- `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` held by `invariant_projection::operation_reader::current_output::WorthQueryCurrentOutputDenial` in `impl<'reader, 'runtime, Schema, Operation> WorthQueryApplicationOperationInvaria`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::WorthQueryApplicationInvariantProjectionSnapshot` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::admission_denial::from_branch_basis_denial` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::admission_denial::from_snapshot_admission_denial` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::locked_reader::WorthQueryApplicationInvariantProjectionReader` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`

### `application_discovery` ↔ `application_installation`: 5 references up in 1 items, default **contract**

Items: `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` (5)

- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_installation::host::ApplicationInstallationHost` in `declaration`
- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_contribution::producer::demand::WorthQueryAdmittedOutputDemand` in `impl<Schema, Spec> WorthQueryWorkflowVocabulary<'_, Schema, Spec>`
- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_output_demand::denial::WorthQueryOutputDemandDenial` in `impl<Schema, Spec> WorthQueryWorkflowVocabulary<'_, Schema, Spec>`
- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl<Schema, Spec> WorthQueryWorkflowVocabulary<'_, Schema, Spec>`
- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_query::admitted_result::WorthQueryApplicationOutputDemandSource` in `impl<Schema, Spec> WorthQueryWorkflowVocabulary<'_, Schema, Spec>`
- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_installation::host::ApplicationInstallationHost` in `impl<'vocabulary, Schema, Spec> WorthQueryWorkflowVocabulary<'vocabulary, Schema`

### `product_operation` ↔ `provider`: 5 references up in 4 items, default **contract**

Items: `provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` (2), `provider::aftermath_causality::correction_history::WorthQueryCorrectionHistoryProgress` (1), `provider::aftermath_causality::correction_history::page_allocation::WorthQueryCorrectionHistoryPageAllocation` (1), `provider::relational_execution_denial` (1)

- `provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` held by `provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyStore` in `declaration`
- `provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` held by `provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyStore` in `impl WorthQueryUnpublishedIdempotencyDisposition`
- `provider::unpublished_idempotency::WorthQueryUnpublishedIdempotencyDisposition` held by `provider::unpublished_idempotency::release::WorthQueryUnpublishedReleaseReservation` in `impl WorthQueryUnpublishedIdempotencyDisposition`
- `provider::aftermath_causality::correction_history::WorthQueryCorrectionHistoryProgress` held by `provider::aftermath_causality::correction_history::page_allocation::WorthQueryCorrectionHistoryPageAllocation` in `declaration`
- `provider::aftermath_causality::correction_history::page_allocation::WorthQueryCorrectionHistoryPageAllocation` held by `provider::aftermath_causality::correction_history::page_allocation::allocation_denial` in `impl WorthQueryCorrectionHistoryPageAllocation`
- `provider::aftermath_causality::correction_history::page_allocation::WorthQueryCorrectionHistoryPageAllocation` held by `provider::aftermath_causality::correction_history::page_allocation::allocation_failed` in `impl WorthQueryCorrectionHistoryPageAllocation`

### `output_reuse` ↔ `product_operation`: 5 references up in 2 items, default **contract**

Items: `application_output_demand::denial::WorthQueryOutputDemandDenial` (3), `application_attempt::fact::WorthQueryApplicationObservedFact` (2)

- `application_output_demand::denial::WorthQueryOutputDemandDenial` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl WorthQueryOutputDemandDenial`
- `application_attempt::fact::WorthQueryApplicationObservedFact` held by `application_attempt::fact::source_currentness::WorthQuerySourceCurrentnessFailure` in `impl WorthQueryApplicationObservedFact`
- `application_output_demand::denial::WorthQueryOutputDemandDenial` held by `application_contribution::producer::denial_value::WorthQueryProducerDenialValue` in `impl WorthQueryOutputDemandDenial`
- `application_attempt::fact::WorthQueryApplicationObservedFact` held by `application_attempt::denial::WorthQueryApplicationAttemptDenial` in `impl WorthQueryApplicationObservedFact`
- `application_attempt::fact::WorthQueryApplicationObservedFact` held by `application_attempt::fact::workflow_history_basis::remains_equal` in `impl WorthQueryApplicationObservedFact`
- `application_attempt::fact::WorthQueryApplicationObservedFact` held by `source_read_verification::SourceReadVerification` in `impl WorthQueryApplicationObservedFact`

### `application_entry` ↔ `product_operation`: 4 references up in 4 items, default **contract**

Items: `application_attempt::denial::WorthQueryApplicationAttemptDenial` (1), `application_query::host::ApplicationQueryHost` (1), `application_output_demand::denial::WorthQueryOutputDemandDenial` (1), `product_operation::context::ProductOperationContextScopedOps` (1)

- `application_attempt::denial::WorthQueryApplicationAttemptDenial` held by `application_attempt::denial::AttemptDenialCause` in `impl WorthQueryApplicationAttemptDenial`
- `application_output_demand::denial::WorthQueryOutputDemandDenial` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl WorthQueryOutputDemandDenial`
- `application_attempt::denial::WorthQueryApplicationAttemptDenial` held by `application_attempt::denial::AttemptDenialMeaning` in `impl WorthQueryApplicationAttemptDenial`
- `application_output_demand::denial::WorthQueryOutputDemandDenial` held by `application_contribution::producer::denial_value::WorthQueryProducerDenialValue` in `impl WorthQueryOutputDemandDenial`
- `application_output_demand::denial::WorthQueryOutputDemandDenial` held by `application_query::managed_index::invocation::WorthQueryManagedIndexWorkDenial` in `impl WorthQueryOutputDemandDenial`
- `application_attempt::denial::WorthQueryApplicationAttemptDenial` held by `application_attempt::denial::AttemptDenialMeaning` in `declaration`

### `application_entry` ↔ `handler`: 4 references up in 2 items, default **contract**

Items: `handler::invariant::DecisionReader` (2), `handler::candidate::CandidateWriter` (2)

- `handler::invariant::DecisionReader` held by `application_attempt::denial::WorthQueryApplicationAttemptDenial` in `impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> DecisionReader<'_, `
- `handler::candidate::CandidateWriter` held by `application_attempt::denial::WorthQueryApplicationAttemptDenial` in `impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>> Can`
- `handler::invariant::DecisionReader` held by `handler::decision_reads::indexed_selection::resolution_denial` in `impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>> Dec`
- `handler::invariant::DecisionReader` held by `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` in `impl<'borrow, 'reader, 'runtime, Schema, Binding> DecisionReader<'borrow, 'reade`
- `handler::candidate::CandidateWriter` held by `application_attempt::denial::WorthQueryApplicationAttemptDenial` in `impl<Schema, Binding> CandidateWriter<'_, Schema, Binding>`
- `handler::invariant::DecisionReader` held by `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` in `declaration`

### `output_lineage` ↔ `application_query`: 4 references up in 3 items, default **contract**

Items: `application_query::observed_source::source_identity::membership::WorthQueryObservedSourceSelection` (2), `application_query::output_source_observation::read_closure::SourceQueryReadClosure` (1), `application_query::derived_view::registry::ManagedDerivedViewRegistry` (1)

- `application_query::derived_view::registry::ManagedDerivedViewRegistry` held by `application_query::one_shot::outcome::material::MaterialCompletion` in `impl ManagedDerivedViewRegistry`
- `application_query::output_source_observation::read_closure::SourceQueryReadClosure` held by `application_runtime::operation_control::attempt_registration::WorthQueryApplicationAttemptOperationControl` in `declaration`
- `application_query::output_source_observation::read_closure::SourceQueryReadClosure` held by `application_query::observed_source::WorthQueryObservedSource` in `impl SourceQueryReadClosure`
- `application_query::output_source_observation::read_closure::SourceQueryReadClosure` held by `invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop` in `impl SourceQueryReadClosure`
- `application_query::output_source_observation::read_closure::SourceQueryReadClosure` held by `invariant_projection::consumed_output::verification::work_budget::map_admission_stop` in `impl SourceQueryReadClosure`
- `application_query::observed_source::source_identity::membership::WorthQueryObservedSourceSelection` held by `application_query::observed_source::source_identity::WorthQueryObservedSourceMeaning` in `declaration`

### `application_query` ↔ `invariant_projection`: 4 references up in 4 items, default **contract**

Items: `invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop` (1), `invariant_projection::consumed_output::verification::work_budget::map_verification_stop` (1), `invariant_projection::operation_reader::output_availability::denial::WorthQueryOutputAvailabilityDenial` (1), `invariant_projection::consumed_output::verification::work_budget::map_admission_stop` (1)

- `invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop` held by `application_query::managed_index::invocation::WorthQueryManagedIndexWorkDenial` in `declaration`
- `invariant_projection::consumed_output::verification::work_budget::map_verification_stop` held by `invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop` in `body`
- `invariant_projection::consumed_output::verification::work_budget::map_verification_stop` held by `invariant_projection::consumed_output::verification::work_budget::map_admission_stop` in `body`
- `invariant_projection::consumed_output::verification::work_budget::map_verification_stop` held by `output_lineage::invalidation::verification::SettlementVerificationStop` in `body`
- `invariant_projection::operation_reader::output_availability::denial::WorthQueryOutputAvailabilityDenial` held by `invariant_projection::operation_reader::output_availability::denial::demand_stop::meter` in `impl WorthQueryOutputAvailabilityDenial`
- `invariant_projection::operation_reader::output_availability::denial::WorthQueryOutputAvailabilityDenial` held by `invariant_projection::operation_reader::output_availability::denial::demand_stop::query_stop` in `impl WorthQueryOutputAvailabilityDenial`

### `application_entry` ↔ `application_attempt`: 4 references up in 4 items, default **contract**

Items: `application_attempt::read_set::WorthQueryCompleteApplicationReadSet` (1), `application_attempt::read_set::source_facts::SourceFactsScopedOps` (1), `application_attempt::effect_program::model::WorthQueryApplicationEffectProgram` (1), `application_attempt::read_set::ReadSetScopedOps` (1)

- `application_attempt::read_set::WorthQueryCompleteApplicationReadSet` held by `application_attempt::denial::WorthQueryApplicationAttemptDenial` in `impl<Schema, Operation, Input, Scope> WorthQueryCompleteApplicationReadSet< Sche`
- `application_attempt::read_set::WorthQueryCompleteApplicationReadSet` held by `application_attempt::read_set::request_liveness::check_request_live` in `impl<Schema, Operation, Input, Scope> WorthQueryCompleteApplicationReadSet< Sche`
- `application_attempt::read_set::WorthQueryCompleteApplicationReadSet` held by `application_attempt::workflow::schema::layout::WorthQueryWorkflowLayout` in `impl<Schema, Operation, Input, Scope> WorthQueryCompleteApplicationReadSet< Sche`
- `application_attempt::read_set::WorthQueryCompleteApplicationReadSet` held by `application_attempt::workflow_transition_program::publication::PreparedWorkflowAdvance` in `impl<Schema, Operation, Input, Scope> WorthQueryCompleteApplicationReadSet< Sche`
- `application_attempt::read_set::WorthQueryCompleteApplicationReadSet` held by `application_attempt::workflow::definition::compilation::plan::CompiledWorkflowDefinition` in `impl<Schema, Operation, Input, Scope> WorthQueryCompleteApplicationReadSet< Sche`
- `application_attempt::read_set::WorthQueryCompleteApplicationReadSet` held by `application_attempt::workflow::instance::transition::admission::selection::SelectedWorkflowTransition` in `impl<Schema, Operation, Input, Scope> WorthQueryCompleteApplicationReadSet< Sche`

### `output_reuse` ↔ `output_lineage`: 4 references up in 4 items, default **contract**

Items: `output_lineage::native_output_witness::SealedNativeOutputWitness` (1), `output_lineage::invalidation::owner::SourceInvalidationOwner` (1), `output_lineage::invalidation::mark_state::FullVerificationReason` (1), `output_lineage::input_cutoff::verification::cutoff_declines` (1)

- `output_lineage::invalidation::owner::SourceInvalidationOwner` held by `output_lineage::invalidation::derived::SettlementRegistrationStop` in `impl SourceInvalidationOwner`
- `output_lineage::invalidation::owner::SourceInvalidationOwner` held by `output_lineage::invalidation::source_alignment::SnapshotAlignedMarkState` in `impl SourceInvalidationOwner`
- `output_lineage::invalidation::owner::SourceInvalidationOwner` held by `output_lineage::invalidation::settlement::SettlementRegistration` in `impl SourceInvalidationOwner`
- `output_lineage::invalidation::owner::SourceInvalidationOwner` held by `output_lineage::invalidation::source_alignment::BranchMarkRoot` in `impl SourceInvalidationOwner`
- `output_lineage::invalidation::owner::SourceInvalidationOwner` held by `output_lineage::invalidation::verification::SettlementVerificationStop` in `impl SourceInvalidationOwner`
- `output_lineage::invalidation::owner::SourceInvalidationOwner` held by `output_lineage::invalidation::derived::prepared::PreparedSettlementRegistration` in `impl SourceInvalidationOwner`

### `root` ↔ `application_attempt`: 4 references up in 4 items, default **contract**

Items: `application_attempt::workflow::instance::progression::retention::default_progress_retention_shards` (1), `application_attempt::workflow::instance::progression::retention::WorkflowInstanceProgressRetention` (1), `application_attempt::workflow::definition::compilation::reuse::WorkflowDefinitionCompilationReuse` (1), `application_attempt::workflow::schema::indexes::register_indexes` (1)

- `application_attempt::workflow::instance::progression::retention::default_progress_retention_shards` held by `application_attempt::workflow::instance::progression::retention::WorkflowInstanceProgressRetention` in `body`
- `application_attempt::workflow::instance::progression::retention::WorkflowInstanceProgressRetention` held by `application_attempt::workflow::instance::progression::retention::RetainedWorkflowInstanceProgress` in `declaration`
- `application_attempt::workflow::instance::progression::retention::WorkflowInstanceProgressRetention` held by `application_attempt::workflow::instance::progression::WorkflowInstanceProgress` in `impl WorkflowInstanceProgressRetention`
- `application_attempt::workflow::instance::progression::retention::WorkflowInstanceProgressRetention` held by `application_attempt::workflow::instance::progression::retention::RetainedWorkflowInstanceProgress` in `impl WorkflowInstanceProgressRetention`
- `application_attempt::workflow::instance::progression::retention::WorkflowInstanceProgressRetention` held by `application_attempt::workflow::instance::progression::retention::RetainedWorkflowInstanceProgressProjection` in `impl WorkflowInstanceProgressRetention`
- `application_attempt::workflow::instance::progression::retention::WorkflowInstanceProgressRetention` held by `application_attempt::workflow::instance::progression::retention::retained_charge_bytes` in `impl WorkflowInstanceProgressRetention`

### `component_selection` ↔ `application_attempt`: 3 references up in 2 items, default **contract**

Items: `application_attempt::fact::source_currentness::WorthQuerySourceCurrentnessFailure` (2), `application_attempt::retained_decision_facts::endpoints::AdmittedAdjacencyEndpoints` (1)

- `application_attempt::fact::source_currentness::WorthQuerySourceCurrentnessFailure` held by `application_query::managed_index::invocation::WorthQueryManagedIndexWorkDenial` in `declaration`
- `application_attempt::fact::source_currentness::WorthQuerySourceCurrentnessFailure` held by `invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop` in `declaration`
- `application_attempt::retained_decision_facts::endpoints::AdmittedAdjacencyEndpoints` held by `application_attempt::denial::WorthQueryApplicationAttemptDenial` in `impl AdmittedAdjacencyEndpoints`
- `application_attempt::retained_decision_facts::endpoints::AdmittedAdjacencyEndpoints` held by `application_attempt::retained_decision_facts::endpoints::heap_sort` in `impl AdmittedAdjacencyEndpoints`
- `application_attempt::retained_decision_facts::endpoints::AdmittedAdjacencyEndpoints` held by `application_attempt::retained_decision_facts::endpoints::union_visit` in `impl AdmittedAdjacencyEndpoints`

### `causal_inspection` ↔ `application_contribution`: 3 references up in 3 items, default **contract**

Items: `application_contribution::producer::output_evaluation::ProducerOutputEvaluation` (1), `application_contribution::producer::output_evaluation::ProducerOutputEvaluationState` (1), `application_contribution::producer::output_evaluation::ComparedProducerOutput` (1)

- `application_contribution::producer::output_evaluation::ProducerOutputEvaluation` held by `application_contribution::producer::output_equivalence_contract::InstalledProducerOutputEquivalence` in `declaration`
- `application_contribution::producer::output_evaluation::ProducerOutputEvaluation` held by `application_contribution::producer::output_evaluation::ProducerOutputEvaluationState` in `declaration`
- `application_contribution::producer::output_evaluation::ProducerOutputEvaluationState` held by `application_contribution::producer::output_evaluation::ComparedProducerOutput` in `declaration`
- `application_contribution::producer::output_evaluation::ComparedProducerOutput` held by `application_contribution::producer::output_equivalence_contract::InstalledProducerOutputEquivalence` in `declaration`

### `aggregate_observation` ↔ `application_attempt`: 3 references up in 1 items, default **contract**

Items: `application_attempt::fact::source_currentness::WorthQuerySourceCurrentnessFailure` (3)

- `application_attempt::fact::source_currentness::WorthQuerySourceCurrentnessFailure` held by `application_query::managed_index::invocation::WorthQueryManagedIndexWorkDenial` in `declaration`
- `application_attempt::fact::source_currentness::WorthQuerySourceCurrentnessFailure` held by `invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop` in `declaration`

### `live_delivery` ↔ `application_attempt`: 3 references up in 2 items, default **contract**

Items: `application_attempt::effect_program::model::emission::WorthQueryApplicationEmission` (2), `application_attempt::effect_program::model::emission::WorthQueryAdmittedApplicationEmissionBatch` (1)

- `application_attempt::effect_program::model::emission::WorthQueryApplicationEmission` held by `application_query::observed_source::query_context_binding::ObservedQueryExpressionContext` in `declaration`
- `application_attempt::effect_program::model::emission::WorthQueryApplicationEmission` held by `application_query::observed_source::query_context_binding::ObservedQueryExpressionContext` in `impl WorthQueryApplicationEmission`

