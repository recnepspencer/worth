# Helper-move simulation: `domain_computation::primary_graph` (80 modules)

Before: component 59, 1248 references against the current layering.
After: component 56, 406 against the current layering, 657 against a fresh layering of the result.
Moved: 438 helpers and 638 units over 5 rounds. Of 1214 helpers blocking coupled units, 263 could move at once.

Remaining seams in the largest component: 62 (62 contract).

## Helpers blocking coupled units

| helper | owner | destination | units blocked | verdict |
|---|---|---|---|---|
| `application_query::managed_index::provider_failure::equal_cause` | application_query | record_reservations | 78 | could move |
| `application_contribution::conditional::output_readiness_contract::denial` | application_contribution | record_reservations | 77 | blocked: application_contribution::conditional::output_readiness_contract::WorthQueryOutputReadinessContractDenial (struct) |
| `application_resource::provider::ingress::backing::CHUNK_BYTES` | application_resource | record_reservations | 72 | could move |
| `application_resource::provider::ingress::backing::lock` | application_resource | record_reservations | 72 | could move |
| `application_output_demand::denial::bridge_deferral::is_temporarily_deferred` | application_output_demand | record_reservations | 71 | blocked: application_output_demand::denial::bridge_deferral::signal_service_is_deferred (fn) |
| `application_resource::installation::denial` | application_resource | record_reservations | 71 | blocked: denial::WorthQueryPrimaryGraphInstallationDenial (struct) |
| `application_resource::storage::lock` | application_resource | record_reservations | 71 | could move |
| `application_contribution::producer::denial_value::mismatch_or` | application_contribution | record_reservations | 70 | could move |
| `product_operation::document_history::current_head::confirm_in_execution` | product_operation | record_reservations | 67 | blocked: product_operation::document_history::current_head::confirm_in_execution_with_effect (fn), product_operation::document_history::denial::WorthQueryDocumentHistoryDenial (enum), product_operation::host::ProductOperationHost (trait) |
| `product_operation::document_history::current_head::confirm_in_native` | product_operation | record_reservations | 67 | blocked: product_operation::document_history::current_head::confirm_in_native_with_effect (fn), product_operation::document_history::denial::WorthQueryDocumentHistoryDenial (enum), product_operation::host::ProductOperationHost (trait) |
| `application_contribution::conditional::output_readiness_contract::typed::field_denial` | application_contribution | record_reservations | 65 | blocked: application_contribution::conditional::output_readiness_contract::WorthQueryOutputReadinessContractDenial (struct) |
| `application_contribution::native_field_contract::field_denial` | application_contribution | record_reservations | 65 | could move |
| `application_runtime::operation_control::attempt_registration::arm` | application_runtime | record_reservations | 65 | could move |
| `application_runtime::operation_control::attempt_registration::clear` | application_runtime | record_reservations | 65 | could move |
| `application_runtime::operation_control::attempt_registration::reach` | application_runtime | record_reservations | 65 | could move |
| `application_runtime::operation_control::attempt_registration::source_publication::release` | application_runtime | record_reservations | 65 | blocked: application_runtime::operation_control::attempt_registration::clear (fn) |
| `application_runtime::operation_control::attempt_registration::source_publication::wait` | application_runtime | record_reservations | 65 | could move |
| `product_operation::document_history::step::NEXT_DOCUMENT` | product_operation | record_reservations | 65 | could move |
| `product_operation::document_history::step::mint_document_ordinal` | product_operation | record_reservations | 65 | could move |
| `conditional_operation::lifecycle_inventory::live_count` | conditional_operation | record_reservations | 64 | could move |
| `application_contribution::producer::denial_value::declaration::encode_declared` | application_contribution | record_reservations | 63 | blocked: application_contribution::producer::denial_value::WorthQueryProducerDenialValue (enum), application_invariant::refusal::canonical_wire_maximum (fn) |
| `application_contribution::producer::registry::semantic_edition::posture_code` | application_contribution | record_reservations | 63 | could move |
| `conditional_operation::lifecycle::commit_watch::capacity` | conditional_operation | record_reservations | 62 | blocked: conditional_operation::installation::denial::WorthQueryConditionalRuntimeInstallationDenial (struct), conditional_operation::installation::denial::WorthQueryConditionalRuntimeInstallationDenialKind (enum) |
| `conditional_operation::lifecycle::commit_watch::check_request` | conditional_operation | record_reservations | 62 | blocked: conditional_operation::installation::denial::WorthQueryConditionalRuntimeInstallationDenial (struct), conditional_operation::installation::denial::WorthQueryConditionalRuntimeInstallationDenialKind (enum) |
| `conditional_operation::lifecycle::commit_watch::source_entity` | conditional_operation | record_reservations | 62 | could move |
| `conditional_operation::lifecycle::pending_invalidations::same_invalidation` | conditional_operation | record_reservations | 62 | could move |
| `conditional_operation::lifecycle::pending_invalidations::snapshot_version` | conditional_operation | record_reservations | 62 | could move |
| `application_contribution::partitioned_computation::incremental::observed::observe_retained_full_tree` | application_contribution | record_reservations | 61 | blocked: application_contribution::partitioned_computation::incremental::observed::observe_tree (fn), application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause (enum), application_contribution::partitioned_computation::incremental::tree_report::WorthQueryPartitionedTreeRun (enum) |
| `conditional_operation::lifecycle::operation_cell::installation_denial_observation_kind` | conditional_operation | record_reservations | 61 | blocked: conditional_operation::clock_observation::WorthQueryConditionalClockObservationFailureKind (enum), conditional_operation::installation::denial::WorthQueryConditionalRuntimeInstallationDenialKind (enum) |
| `application_checkpoint::facts::payload::chunk::deep_clone_bytes` | application_checkpoint | record_reservations | 56 | blocked: application_attempt::fact::WorthQueryApplicationObservedFact (enum), application_checkpoint::facts::payload::chunk::bytes (fn) |
| `application_checkpoint::facts::payload::patch::take_published` | application_checkpoint | record_reservations | 56 | blocked: application_checkpoint::facts::payload::FactPayload (struct), application_checkpoint::facts::payload::patch::PreparedNode (enum) |
| `application_invariant::refusal::canonical_wire_maximum` | application_invariant | record_reservations | 56 | blocked: application_invariant::refusal::maximum (fn) |
| `application_query::observed_source::source_identity::material_storage::admit_footprint_comparison` | application_query | record_reservations | 56 | blocked: application_query::observed_source::WorthQueryObservedSourceFootprint (struct) |
| `application_query::projection::projected_tree::content_comparison::same_relation` | application_query | record_reservations | 56 | blocked: application_query::projection::projected_tree::content_comparison::same_relation_header (fn), application_query::projection::projected_tree::content_comparison::same_row (fn) |
| `application_query::projection::projected_tree::content_comparison::same_relation_header` | application_query | record_reservations | 56 | blocked: application_query::projection::projected_tree::content_comparison::same_text (fn) |
| `application_query::projection::projected_tree::content_comparison::same_row` | application_query | record_reservations | 56 | blocked: application_query::projection::projected_tree::content_comparison::same_text (fn), application_query::projection::projected_tree::content_comparison::same_value (fn) |
| `application_query::projection::projected_tree::content_comparison::traversal::frames` | application_query | record_reservations | 56 | could move |
| `invariant_projection::consumed_output::verification::work_budget::map_admission_stop` | invariant_projection | record_reservations | 56 | blocked: invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop (enum) |
| `invariant_projection::consumed_output::verification::work_budget::reserve_pending` | invariant_projection | record_reservations | 56 | blocked: invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop (enum) |
| `application_checkpoint::facts::payload::array_storage_bytes` | application_checkpoint | record_reservations | 55 | could move |
| `application_checkpoint::facts::payload::chunk::bytes` | application_checkpoint | record_reservations | 55 | could move |
| `application_checkpoint::facts::payload::concatenation::join` | application_checkpoint | record_reservations | 55 | blocked: application_checkpoint::facts::payload::FactPayload (struct), application_checkpoint::facts::payload::concatenation::balance (fn), application_checkpoint::facts::payload::concatenation::node (fn) |
| `application_checkpoint::facts::payload::original_storage_bytes` | application_checkpoint | record_reservations | 55 | blocked: application_attempt::fact::WorthQueryApplicationObservedFact (enum) |
| `application_checkpoint::facts::payload::patch::preparation::build` | application_checkpoint | record_reservations | 55 | blocked: application_checkpoint::facts::payload::FactPayload (struct), application_checkpoint::facts::payload::patch::PreparedNode (enum), application_checkpoint::facts::payload::patch::leaf::PreparedLeaf (struct) |
| `application_checkpoint::facts::payload::patch::preparation::count_nodes` | application_checkpoint | record_reservations | 55 | blocked: application_checkpoint::facts::payload::FactPayload (struct), application_checkpoint::facts::payload::patch::preparation::children (fn), application_checkpoint::facts::payload::patch::preparation::intersects (fn) |
| `application_checkpoint::facts::payload::resident_bytes` | application_checkpoint | record_reservations | 55 | blocked: application_attempt::fact::WorthQueryApplicationObservedFact (enum), fact_verification_observation::original_payload_size (fn) |
| `application_query::managed_index::nodes::pages::PAGE_BITS` | application_query | record_reservations | 55 | could move |
| `application_query::managed_index::nodes::pages::PAGE_SIZE` | application_query | record_reservations | 55 | blocked: application_query::managed_index::nodes::pages::PAGE_BITS (const) |
| `application_query::managed_index::nodes::pages::branch` | application_query | record_reservations | 55 | blocked: application_query::managed_index::nodes::pages::retain (fn) |
| `application_query::managed_index::nodes::pages::build` | application_query | record_reservations | 55 | blocked: application_query::managed_index::nodes::pages::PAGE_SIZE (const), application_query::managed_index::nodes::pages::branch (fn), application_query::managed_index::nodes::pages::page (fn) |
| `application_query::managed_index::nodes::pages::header_memory` | application_query | record_reservations | 55 | could move |
| `application_query::managed_index::nodes::pages::page` | application_query | record_reservations | 55 | blocked: application_query::managed_index::nodes::pages::PAGE_SIZE (const), application_query::managed_index::nodes::pages::retain (fn) |
| `application_query::managed_index::nodes::pages::retain` | application_query | record_reservations | 55 | blocked: application_query::managed_index::nodes::pages::header_memory (fn) |
| `application_entry::mutation::execution::decision_query::native_decision_cause` | application_entry | record_reservations | 54 | could move |
| `application_installation::program::program_owner::occurrence_commit::commit_program_action` | application_installation | record_reservations | 54 | blocked: application_attempt::compare_and_commit::commit_outcome::WorthQueryApplicationCommitOutcome (enum), application_attempt::compare_and_commit::commit_outcome::denial::WorthQueryApplicationCommitDenial (struct), application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding (struct) |
| `application_installation::program::program_owner::occurrence_commit::commit_program_action_retained` | application_installation | record_reservations | 54 | blocked: application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding (struct), application_attempt::retained_commit::WorthQueryApplicationRetainedCommitOutcome (enum), application_installation::program::program_owner::contract::WorthQueryProgramOwner (trait) |
| `invariant_projection::output_availability_candidate::dependencies::push_pending` | invariant_projection | record_reservations | 54 | blocked: invariant_projection::consumed_output::ConsumedOutputEvidence (struct), invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop (enum), invariant_projection::consumed_output::verification::EvidenceView (struct) |
| `output_lineage::native_output_witness::overflow` | output_lineage | record_reservations | 54 | could move |
| `output_lineage::native_output_witness::retirement::native_metadata` | output_lineage | record_reservations | 54 | could move |
| `access_strategy::ordered_storage::insert` | access_strategy | record_reservations | 53 | blocked: access_strategy::ordered_storage::lookup (fn) |

## Remaining seams

### `application_attempt` ↔ `application_checkpoint`: 29 references up in 5 items, default **contract**

Items: `application_checkpoint::facts::payload::FactPayload` (23), `application_checkpoint::facts` (2), `application_checkpoint::facts::payload::native::NativeFactPayloadBuilder` (2), `application_checkpoint::facts::stream::FactDecoder` (1), `output_lineage::input_cutoff::stable_cutover::PublishedStableLineage` (1)

- `application_checkpoint::facts::payload::FactPayload` held by `application_checkpoint::facts::payload::resident_bytes` in `impl FactPayload`
- `application_checkpoint::facts::payload::FactPayload` held by `application_checkpoint::facts::payload::resident_bytes` in `impl From<Arc<[Fact]>> for FactPayload`
- `application_checkpoint::facts::payload::FactPayload` held by `application_checkpoint::facts::payload::concatenation::join` in `impl FactPayload`
- `application_checkpoint::facts::payload::FactPayload` held by `application_checkpoint::facts::payload::patch::NativeFactPatchBuilder` in `impl FactPayload`
- `application_checkpoint::facts::payload::FactPayload` held by `application_checkpoint::facts::payload::patch::preparation::build` in `impl FactPayload`
- `application_checkpoint::facts::payload::FactPayload` held by `application_checkpoint::facts::payload::patch::preparation::count_nodes` in `impl FactPayload`

### `application_query` ↔ `product_operation`: 26 references up in 8 items, default **contract**

Items: `product_operation::document_history::WorthQueryDocumentHistory` (11), `product_operation::query::history_outputs::WorthQueryHistoryOutputReadDenial` (4), `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` (3), `product_operation::document_history::current_head::confirm` (3), `product_operation::document_history::navigation::WorthQueryDocumentHistoryNavigation` (2), `product_operation::document_history::navigation::output_profile::WorthQueryHistoryOutputNavigationAdvance` (1), `product_operation::document_history::navigation::output_authority::WorthQueryHistoryOutputAuthority` (1), `product_operation::document_history::selection::HistorySelectionCell` (1)

- `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` held by `product_operation::document_history::navigation::WorthQueryHistoryNavigationDenial` in `impl HistoryProducerInvocation<'_>`
- `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` held by `product_operation::document_history::navigation::WorthQueryHistoryNavigationDenial` in `impl<'invocation> HistoryProducerInvocation<'invocation>`
- `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` held by `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerCutover` in `impl<'invocation> HistoryProducerInvocation<'invocation>`
- `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` held by `product_operation::document_history::current_head::confirm_in_native` in `impl HistoryProducerInvocation<'_>`
- `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` held by `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryBorrowedReadStop` in `impl HistoryProducerInvocation<'_>`
- `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` held by `product_operation::document_history::current_head::confirm_in_execution` in `impl HistoryProducerInvocation<'_>`

### `output_lineage` ↔ `application_contribution`: 22 references up in 8 items, default **contract**

Items: `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` (5), `application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` (4), `application_contribution::producer::demand::progression::required_wave::resolved::matched_predecessors::MatchedRequiredPredecessors` (4), `application_contribution::producer::registry::semantic_edition::InstalledProducerEdition` (3), `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` (2), `application_contribution::partitioned_computation::incremental::retained::SealedComputationRun` (2), `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` (1), `application_contribution::partitioned_computation::incremental::published_state::observe_published` (1)

- `application_contribution::partitioned_computation::incremental::retained::SealedComputationRun` held by `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` in `impl SealedComputationRun`
- `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` held by `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` in `impl PriorAbsence`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::producer::registry::semantic_edition::InstalledProducerEdition` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` held by `application_contribution::partitioned_computation::incremental::retained::RetainedBasis` in `impl RetainedComputation`

### `application_attempt` ↔ `provider`: 18 references up in 8 items, default **contract**

Items: `provider::WorthQueryPrimaryGraphProvider` (10), `provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::PreparedSourceFactRebase` (2), `provider::relational_execution_denial::query_read_execution_kind` (1), `provider::host::ProviderHost` (1), `provider::committed_dispatch_outbox::ProviderCommittedDispatchOutboxScopedOps` (1), `provider::application_decision_fact::application_table::ApplicationFactTable` (1), `provider::application_decision_fact::application_facts::CompletedApplicationFacts` (1), `provider::pending_application_publication::registry::WorthQueryApplicationPublicationRecoveryReservation` (1)

- `provider::application_decision_fact::application_table::ApplicationFactTable` held by `provider::application_decision_fact::WorthQueryPrimaryGraphApplicationDecisionFact` in `impl ApplicationFactTable`
- `provider::WorthQueryPrimaryGraphProvider` held by `application_query::derived_view::registry::predicate::preparation::PredicatePreparationRetirement` in `impl WorthQueryPrimaryGraphProvider`
- `provider::WorthQueryPrimaryGraphProvider` held by `application_query::derived_view::registry::predicate::preparation::PredicatePreparationOwner` in `impl WorthQueryPrimaryGraphProvider`
- `provider::WorthQueryPrimaryGraphProvider` held by `provider::application_attempt_state::WorthQueryPrimaryGraphApplicationAttemptStore` in `impl WorthQueryPrimaryGraphProvider`
- `provider::WorthQueryPrimaryGraphProvider` held by `live_delivery::source::WorthQueryLiveDeliverySource` in `impl WorthQueryPrimaryGraphProvider`
- `provider::WorthQueryPrimaryGraphProvider` held by `provider::pending_application_publication::registry::WorthQueryPendingApplicationPublicationRegistry` in `impl WorthQueryPrimaryGraphProvider`

### `provider` ↔ `application_query`: 18 references up in 4 items, default **contract**

Items: `application_query::derived_view::registry::predicate::preparation::PredicatePreparationOwner` (10), `application_query::derived_view::registry::predicate::preparation::PredicatePreparationRetirement` (6), `application_query::derived_view::registry::predicate::preparation::PredicatePreparationFactory` (1), `application_query::derived_view::change_summary::changes_from_summary` (1)

- `application_query::derived_view::registry::predicate::preparation::PredicatePreparationOwner` held by `application_query::derived_view::registry::predicate::preparation::round::Round` in `declaration`
- `application_query::derived_view::registry::predicate::preparation::PredicatePreparationRetirement` held by `application_query::derived_view::registry::predicate::preparation::PredicatePreparationOwner` in `declaration`
- `application_query::derived_view::registry::predicate::preparation::PredicatePreparationRetirement` held by `application_query::derived_view::registry::predicate::preparation::PredicatePreparationOwner` in `impl PredicatePreparationRetirement`
- `application_query::derived_view::registry::predicate::preparation::PredicatePreparationFactory` held by `application_query::derived_view::registry::predicate::preparation::RoundLink` in `declaration`
- `application_query::derived_view::registry::predicate::preparation::PredicatePreparationFactory` held by `application_query::derived_view::registry::predicate::preparation::Child` in `impl PredicatePreparationFactory`
- `application_query::derived_view::registry::predicate::preparation::PredicatePreparationFactory` held by `application_query::derived_view::registry::predicate::preparation::PredicatePreparationOwner` in `impl PredicatePreparationFactory`

### `provider` ↔ `output_lineage`: 16 references up in 9 items, default **contract**

Items: `output_lineage::prepared_slot::PreparedOutputLineageSlot` (4), `output_lineage::invalidation::computation_changes::postcondition_carry::PostconditionCarry` (2), `output_lineage::invalidation::settlement::posting_continuation::PostingContinuation` (2), `output_lineage::prepared_slot::preparation::prepare` (2), `output_lineage::native_output_witness::PreparedNativeOutputWitness` (2), `output_lineage::invalidation::output_availability::ObservedSourcePublication` (1), `output_lineage::prepared_slot::recovery::PreparedLineageRecoveryMetadata` (1), `output_lineage::invalidation::settlement::posting_continuation::PostingRebaseBasis` (1)

- `output_lineage::prepared_slot::PreparedOutputLineageSlot` held by `output_lineage::retained_computation::PriorComputationRecord` in `impl PreparedOutputLineageSlot`
- `output_lineage::prepared_slot::PreparedOutputLineageSlot` held by `output_lineage::prepared_slot::recovery::PreparedLineageRecoveryMetadata` in `impl PreparedOutputLineageSlot`
- `output_lineage::invalidation::computation_changes::postcondition_carry::PostconditionCarry` held by `output_lineage::invalidation::computation_changes::postcondition_carry::SelectedPostconditionRebase` in `impl PostconditionCarry`
- `output_lineage::prepared_slot::PreparedOutputLineageSlot` held by `application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` in `impl PreparedOutputLineageSlot`
- `output_lineage::prepared_slot::PreparedOutputLineageSlot` held by `output_lineage::input_reuse_key::PreparedInputReuseKey` in `impl PreparedOutputLineageSlot`
- `output_lineage::prepared_slot::PreparedOutputLineageSlot` held by `output_lineage::native_output_witness::SealedNativeOutputWitness` in `impl PreparedOutputLineageSlot`

### `output_lineage` ↔ `application_output_demand`: 15 references up in 3 items, default **contract**

Items: `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` (11), `application_output_demand::registry::ready_backing::ReadyCompletion` (3), `application_output_demand::settlement::WorthQueryOutputDemandSettlement` (1)

- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::readiness_delivery::WorthQueryOutputReadinessDeliveryEvidence` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::registry::WorthQueryAcceptedOutputAuthority` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::settlement::receipt_custody::SettlementReceiptCustody` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::registry::restoration::WorthQueryRestoredAcceptedOutput` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `output_lineage::input_cutoff::stable_cutover::PublishedStableLineage` in `impl WorthQueryOutputDemandSettlement`

### `application_attempt` ↔ `application_output_demand`: 13 references up in 5 items, default **contract**

Items: `application_output_demand::registry::required_context::RequiredOutputDemandContext` (5), `application_output_demand::settlement::WorthQueryOutputDemandSettlement` (5), `application_output_demand::settlement::SettlementScopedOps` (1), `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` (1), `application_output_demand::registry::WorthQueryOutputDemandRegistry` (1)

- `application_output_demand::registry::WorthQueryOutputDemandRegistry` held by `application_output_demand::registry::WorthQueryOutputDemandInterest` in `impl WorthQueryOutputDemandRegistry`
- `application_output_demand::registry::WorthQueryOutputDemandRegistry` held by `application_output_demand::registry::DemandState` in `impl WorthQueryOutputDemandRegistry`
- `application_output_demand::registry::WorthQueryOutputDemandRegistry` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl WorthQueryOutputDemandRegistry`
- `application_output_demand::registry::WorthQueryOutputDemandRegistry` held by `application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint` in `impl WorthQueryOutputDemandRegistry`
- `application_output_demand::registry::WorthQueryOutputDemandRegistry` held by `application_output_demand::registry::demand_key::WorthQueryOutputDemandKey` in `impl WorthQueryOutputDemandRegistry`
- `application_output_demand::registry::WorthQueryOutputDemandRegistry` held by `application_output_demand::registry::required_work::selection::SelectedReadyReadmission` in `impl WorthQueryOutputDemandRegistry`

### `application_attempt` ↔ `application_contribution`: 12 references up in 5 items, default **contract**

Items: `application_contribution::partitioned_computation::incremental::prior_absence::CompletedComputationRetention` (4), `application_contribution::partitioned_computation::incremental::native_reads::NativeComputationReads` (3), `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` (3), `application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` (1), `application_contribution::execution_denial` (1)

- `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` held by `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` in `impl PriorAbsence`
- `application_contribution::partitioned_computation::incremental::prior_absence::CompletedComputationRetention` held by `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` in `declaration`
- `application_contribution::partitioned_computation::incremental::prior_absence::CompletedComputationRetention` held by `application_contribution::partitioned_computation::incremental::retained::CompletedComputationRun` in `declaration`
- `application_contribution::partitioned_computation::incremental::prior_absence::CompletedComputationRetention` held by `application_contribution::partitioned_computation::incremental::native_reads::NativeComputationReads` in `impl CompletedComputationRetention`
- `application_contribution::partitioned_computation::incremental::prior_absence::CompletedComputationRetention` held by `application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` in `impl CompletedComputationRetention`
- `application_contribution::partitioned_computation::incremental::native_reads::NativeComputationReads` held by `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` in `declaration`

### `provider` ↔ `application_checkpoint`: 11 references up in 3 items, default **contract**

Items: `application_checkpoint::facts::payload::FactPayload` (8), `application_checkpoint::facts::payload::native::NativeFactPayloadBuilder` (2), `application_checkpoint::facts::payload::patch::NativeFactPatchBuilder` (1)

- `application_checkpoint::facts::payload::FactPayload` held by `application_checkpoint::facts::payload::resident_bytes` in `impl FactPayload`
- `application_checkpoint::facts::payload::FactPayload` held by `application_checkpoint::facts::payload::resident_bytes` in `impl From<Arc<[Fact]>> for FactPayload`
- `application_checkpoint::facts::payload::FactPayload` held by `application_checkpoint::facts::payload::concatenation::join` in `impl FactPayload`
- `application_checkpoint::facts::payload::FactPayload` held by `application_checkpoint::facts::payload::patch::NativeFactPatchBuilder` in `impl FactPayload`
- `application_checkpoint::facts::payload::FactPayload` held by `application_checkpoint::facts::payload::patch::preparation::build` in `impl FactPayload`
- `application_checkpoint::facts::payload::FactPayload` held by `application_checkpoint::facts::payload::patch::preparation::count_nodes` in `impl FactPayload`

### `application_runtime` ↔ `conditional_operation`: 11 references up in 9 items, default **contract**

Items: `conditional_operation::lifecycle::registry::WorthQueryConditionalOperationRegistry` (3), `conditional_operation::installation::WorthQueryConditionalApplicationRuntimeInstallation` (1), `conditional_operation::reinstallation::ReinstallationOps` (1), `conditional_operation::publication::install_pending_bindings` (1), `conditional_operation::publication::require_complete_binding_inventory` (1), `conditional_operation::host::ConditionalOperationHost` (1), `conditional_operation::publication::publication_denial` (1), `conditional_operation::inspection::InspectionOps` (1)

- `conditional_operation::installation::WorthQueryConditionalApplicationRuntimeInstallation` held by `conditional_operation::installation::denial::foreign_binding_denial` in `impl<Schema> WorthQueryConditionalApplicationRuntimeInstallation<Schema>`
- `conditional_operation::lifecycle::registry::WorthQueryConditionalOperationRegistry` held by `conditional_operation::lifecycle::installed_operation::WorthQueryInstalledConditionalOperation` in `impl<Schema> WorthQueryConditionalOperationRegistry<Schema>`
- `conditional_operation::lifecycle::registry::WorthQueryConditionalOperationRegistry` held by `conditional_operation::lifecycle::operation_cell::WorthQueryConditionalOperationCell` in `impl<Schema> WorthQueryConditionalOperationRegistry<Schema>`
- `conditional_operation::installation::WorthQueryConditionalApplicationRuntimeInstallation` held by `application_runtime::installation::input::ApplicationRuntimePublication` in `declaration`
- `conditional_operation::installation::WorthQueryConditionalApplicationRuntimeInstallation` held by `conditional_operation::installation::pending_operation::WorthQueryPendingConditionalOperation` in `declaration`
- `conditional_operation::installation::WorthQueryConditionalApplicationRuntimeInstallation` held by `application_runtime::installation::publish_application_runtime_with_conditionals` in `impl<Schema> WorthQueryConditionalApplicationRuntimeInstallation<Schema>`

### `application_attempt` ↔ `invariant_projection`: 10 references up in 5 items, default **contract**

Items: `invariant_projection::operation_reader::decision_reads::expected_keys::ExpectedFactKeys` (6), `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionSnapshot` (1), `invariant_projection::realized_scope::WorthQueryRealizedProjectionScope` (1), `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` (1), `invariant_projection::operation_reader::borrowed::BorrowedOperationProjection` (1)

- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::WorthQueryApplicationInvariantProjectionSnapshot` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::locked_reader::WorthQueryApplicationInvariantProjectionReader` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::locked_reader::WorthQueryCompletedInvariantProjection` in `impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` in `impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionAuthorit`
- `invariant_projection::WorthQueryApplicationInvariantProjectionAuthority` held by `invariant_projection::operation_reader::admitted::SourceAccess` in `impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionAuthorit`

### `invariant_projection` ↔ `application_contribution`: 9 references up in 7 items, default **contract**

Items: `application_contribution::partitioned_computation::incremental::prior_absence::CompletedComputationRetention` (2), `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` (2), `application_contribution::partitioned_computation::incremental::native_reads::NativeComputationReads` (1), `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` (1), `application_contribution::producer::source_qualification::SourceQualificationScopedOps` (1), `application_contribution::partitioned_computation::incremental::deposit::ComputationDeposit` (1), `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` (1)

- `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` held by `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` in `impl PriorAbsence`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::producer::registry::semantic_edition::InstalledProducerEdition` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::prior_absence::CompletedComputationRetention` held by `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` in `declaration`
- `application_contribution::partitioned_computation::incremental::prior_absence::CompletedComputationRetention` held by `application_contribution::partitioned_computation::incremental::retained::CompletedComputationRun` in `declaration`

### `application_attempt` ↔ `product_operation`: 9 references up in 1 items, default **contract**

Items: `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` (9)

- `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` held by `product_operation::document_history::navigation::WorthQueryHistoryNavigationDenial` in `impl HistoryProducerInvocation<'_>`
- `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` held by `product_operation::document_history::navigation::WorthQueryHistoryNavigationDenial` in `impl<'invocation> HistoryProducerInvocation<'invocation>`
- `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` held by `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerCutover` in `impl<'invocation> HistoryProducerInvocation<'invocation>`
- `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` held by `product_operation::document_history::current_head::confirm_in_native` in `impl HistoryProducerInvocation<'_>`
- `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` held by `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryBorrowedReadStop` in `impl HistoryProducerInvocation<'_>`
- `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` held by `product_operation::document_history::current_head::confirm_in_execution` in `impl HistoryProducerInvocation<'_>`

### `application_attempt` ↔ `output_lineage`: 9 references up in 3 items, default **contract**

Items: `output_lineage::invalidation::computation_changes::postcondition_carry::PostconditionCarry` (4), `output_lineage::invalidation::output_availability::ObservedSourcePublication` (3), `output_lineage::source_facts::ComparableSourceFacts` (2)

- `output_lineage::source_facts::ComparableSourceFacts` held by `application_checkpoint::facts::payload::FactPayload` in `impl ComparableSourceFacts`
- `output_lineage::invalidation::computation_changes::postcondition_carry::PostconditionCarry` held by `output_lineage::invalidation::computation_changes::postcondition_carry::SelectedPostconditionRebase` in `impl PostconditionCarry`
- `output_lineage::invalidation::output_availability::ObservedSourcePublication` held by `application_query::output_source_observation::read_closure::SourceQueryReadClosure` in `declaration`
- `output_lineage::invalidation::output_availability::ObservedSourcePublication` held by `invariant_projection::output_availability_candidate::ObservedOutputCandidate` in `declaration`
- `output_lineage::invalidation::output_availability::ObservedSourcePublication` held by `application_checkpoint::facts::payload::FactPayload` in `impl ObservedSourcePublication`
- `output_lineage::source_facts::ComparableSourceFacts` held by `application_checkpoint::facts::payload::FactPayload` in `declaration`

### `application_output_demand` ↔ `application_contribution`: 8 references up in 5 items, default **contract**

Items: `application_contribution::producer::output_evaluation::ProducerOutputEvaluation` (3), `application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` (2), `application_contribution::producer::demand::bridge_denial::bridge_denial_kind` (1), `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` (1), `application_contribution::producer::execution::unpublished::PendingProducerPublication` (1)

- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::partitioned_computation::incremental::retained::RetainedComputation` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::partitioned_computation::incremental::retained::WorthQueryPartitionedComputationFullCause` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::carriage::ComputationPrior` held by `application_contribution::producer::registry::semantic_edition::InstalledProducerEdition` in `impl ComputationPrior`
- `application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` held by `application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence` in `declaration`
- `application_contribution::partitioned_computation::incremental::prior_absence::SealedComputationRetention` held by `application_contribution::partitioned_computation::incremental::retained::SealedComputationRun` in `declaration`
- `application_contribution::producer::demand::bridge_denial::bridge_denial_kind` held by `application_contribution::producer::demand::bridge_execution_denial::query_cause` in `body`

### `application_contribution` ↔ `application_entry`: 7 references up in 4 items, default **contract**

Items: `application_entry::mutation::HandlerResult` (2), `application_entry::mutation::OperationHandler` (2), `application_entry::mutation::decision_plan::WorthQueryDecisionPlan` (2), `application_entry::mutation::execution::preparation::PreparationScopedOps` (1)

- `application_entry::mutation::HandlerResult` held by `application_entry::mutation::failure::HandlerFailure` in `impl<Value, Domain> HandlerResult<Value, Domain>`
- `application_entry::mutation::OperationHandler` held by `application_entry::mutation::decision_plan::WorthQueryDecisionPlan` in `declaration`
- `application_entry::mutation::OperationHandler` held by `handler::invariant::DecisionReader` in `declaration`
- `application_entry::mutation::execution::preparation::PreparationScopedOps` held by `application_entry::mutation::execution::candidate::build_handler_candidate` in `declaration`
- `application_entry::mutation::execution::preparation::PreparationScopedOps` held by `application_entry::mutation::execution::completion::PreparedMutationHandler` in `declaration`
- `application_entry::mutation::execution::preparation::PreparationScopedOps` held by `handler::invariant::DecisionReader` in `declaration`

### `application_attempt` ↔ `application_installation`: 7 references up in 2 items, default **contract**

Items: `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` (5), `application_installation::program::workflow_runtime::authentication::workflow_approval_authentication_intent` (2)

- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_installation::host::ApplicationInstallationHost` in `declaration`
- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_contribution::producer::demand::WorthQueryAdmittedOutputDemand` in `impl<Schema, Spec> WorthQueryWorkflowVocabulary<'_, Schema, Spec>`
- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl<Schema, Spec> WorthQueryWorkflowVocabulary<'_, Schema, Spec>`
- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_installation::host::ApplicationInstallationHost` in `impl<'vocabulary, Schema, Spec> WorthQueryWorkflowVocabulary<'vocabulary, Schema`
- `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` held by `application_installation::program::workflow_runtime::WorthQueryWorkflowApplicationRuntime` in `impl<'vocabulary, Schema, Spec, Program> From<&'vocabulary WorthQueryWorkflowApp`
- `application_installation::program::workflow_runtime::authentication::workflow_approval_authentication_intent` held by `application_installation::program::workflow_runtime::authentication::append` in `body`

### `provider` ↔ `product_operation`: 6 references up in 2 items, default **contract**

Items: `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerCutover` (5), `product_operation::generated_materialization::restoration::invariant_admission::admit_required_invariants` (1)

- `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerCutover` held by `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` in `declaration`
- `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerCutover` held by `product_operation::document_history::navigation::WorthQueryHistoryNavigationDenial` in `impl HistoryProducerCutover<'_>`
- `product_operation::generated_materialization::restoration::invariant_admission::admit_required_invariants` held by `product_operation::generated_materialization::restoration::invariant_admission::admit_requirement` in `body`

### `application_query` ↔ `application_output_demand`: 6 references up in 2 items, default **contract**

Items: `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` (5), `application_output_demand::settlement::WorthQueryOutputDemandSettlement` (1)

- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::readiness_delivery::WorthQueryOutputReadinessDeliveryEvidence` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::registry::WorthQueryAcceptedOutputAuthority` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::settlement::receipt_custody::SettlementReceiptCustody` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `application_output_demand::registry::restoration::WorthQueryRestoredAcceptedOutput` in `impl WorthQueryOutputDemandSettlement`
- `application_output_demand::settlement::WorthQueryOutputDemandSettlement` held by `output_lineage::input_cutoff::stable_cutover::PublishedStableLineage` in `impl WorthQueryOutputDemandSettlement`

### `provider` ↔ `application_output_demand`: 6 references up in 1 items, default **contract**

Items: `application_output_demand::registry::prerequisite_claims::PreparedPrerequisiteClaims` (6)

- `application_output_demand::registry::prerequisite_claims::PreparedPrerequisiteClaims` held by `application_output_demand::registry::WorthQueryOutputDemandRegistry` in `impl PreparedPrerequisiteClaims`
- `application_output_demand::registry::prerequisite_claims::PreparedPrerequisiteClaims` held by `application_output_demand::registry::claim_readthrough::incoming::ClaimBatch` in `impl PreparedPrerequisiteClaims`
- `application_output_demand::registry::prerequisite_claims::PreparedPrerequisiteClaims` held by `application_output_demand::registry::demand_key::WorthQueryOutputDemandKey` in `impl PreparedPrerequisiteClaims`
- `application_output_demand::registry::prerequisite_claims::PreparedPrerequisiteClaims` held by `application_output_demand::registry::settlement_retirement::SupersededSettlements` in `impl PreparedPrerequisiteClaims`
- `application_output_demand::registry::prerequisite_claims::PreparedPrerequisiteClaims` held by `application_output_demand::registry::claim_readthrough::incoming::ClaimBatch` in `declaration`
- `application_output_demand::registry::prerequisite_claims::PreparedPrerequisiteClaims` held by `application_output_demand::registry::demand_key::WorthQueryOutputDemandKey` in `declaration`

### `application_query` ↔ `output_reuse`: 5 references up in 3 items, default **contract**

Items: `output_reuse::selection::retained_fact_denial` (2), `output_reuse::selection::first_moved_fact_in` (2), `output_reuse::verification_denial` (1)

- `output_reuse::selection::retained_fact_denial` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `body`
- `output_reuse::selection::retained_fact_denial` held by `output_reuse::selection::interruption_denial` in `body`
- `output_reuse::selection::retained_fact_denial` held by `output_reuse::selection::read_denial` in `body`
- `output_reuse::selection::retained_fact_denial` held by `output_reuse::selection::work_denial` in `body`
- `output_reuse::verification_denial` held by `(module: not movable)`

### `entity_resolution` ↔ `application_attempt`: 5 references up in 1 items, default **contract**

Items: `product_operation::context::WorthQuerySelectedProductOperation` (5)

- `product_operation::context::WorthQuerySelectedProductOperation` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema>`
- `product_operation::context::WorthQuerySelectedProductOperation` held by `application_installation::program::workflow_runtime::vocabulary::WorthQueryWorkflowVocabulary` in `impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema>`
- `product_operation::context::WorthQuerySelectedProductOperation` held by `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` in `impl<S: ApplicationSchema> WorthQuerySelectedProductOperation<'_, S>`
- `product_operation::context::WorthQuerySelectedProductOperation` held by `application_output_demand::settlement::WorthQueryOutputDemandSettlement` in `impl<S: ApplicationSchema> WorthQuerySelectedProductOperation<'_, S>`
- `product_operation::context::WorthQuerySelectedProductOperation` held by `application_query::observed_source::retained_dependencies::WorthQueryRetainedQueryDependencies` in `impl<S: ApplicationSchema> WorthQuerySelectedProductOperation<'_, S>`
- `product_operation::context::WorthQuerySelectedProductOperation` held by `application_output_demand::currentness::denial` in `impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema>`

### `application_contribution` ↔ `product_operation`: 5 references up in 2 items, default **contract**

Items: `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` (4), `product_operation::context::ProductOperationContextScopedOps` (1)

- `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` held by `product_operation::document_history::navigation::WorthQueryHistoryNavigationDenial` in `impl HistoryProducerInvocation<'_>`
- `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` held by `product_operation::document_history::navigation::WorthQueryHistoryNavigationDenial` in `impl<'invocation> HistoryProducerInvocation<'invocation>`
- `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` held by `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerCutover` in `impl<'invocation> HistoryProducerInvocation<'invocation>`
- `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` held by `product_operation::document_history::current_head::confirm_in_native` in `impl HistoryProducerInvocation<'_>`
- `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` held by `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryBorrowedReadStop` in `impl HistoryProducerInvocation<'_>`
- `product_operation::document_history::navigation::output_authority::producer_invocation::HistoryProducerInvocation` held by `product_operation::document_history::current_head::confirm_in_execution` in `impl HistoryProducerInvocation<'_>`

### `application_contribution` ↔ `handler`: 5 references up in 2 items, default **contract**

Items: `handler::invariant::DecisionReader` (3), `handler::registry::PendingMutationHandlerRegistry` (2)

- `handler::registry::PendingMutationHandlerRegistry` held by `handler::registry::denial` in `impl<Schema: ApplicationSchema> PendingMutationHandlerRegistry<Schema>`
- `handler::invariant::DecisionReader` held by `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` in `impl<'borrow, 'reader, 'runtime, Schema, Binding> DecisionReader<'borrow, 'reade`
- `handler::invariant::DecisionReader` held by `application_entry::mutation::HandlerResult` in `impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>> Dec`
- `handler::registry::PendingMutationHandlerRegistry` held by `handler::registry::denial` in `impl<Schema> PendingMutationHandlerRegistry<Schema>`
- `handler::registry::PendingMutationHandlerRegistry` held by `application_entry::mutation::OperationHandler` in `impl<Schema: ApplicationSchema> PendingMutationHandlerRegistry<Schema>`
- `handler::invariant::DecisionReader` held by `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` in `declaration`

### `access_strategy` ↔ `application_query`: 4 references up in 2 items, default **contract**

Items: `application_query::managed_index::source_structure::SourceTree` (3), `application_query::managed_index::source_rows::SourceImageView` (1)

- `application_query::managed_index::source_structure::SourceTree` held by `application_query::managed_index::source_rows::SourceImageView` in `impl SourceTree`
- `application_query::managed_index::source_rows::SourceImageView` held by `application_query::managed_index::source_rows::SourceImageRows` in `declaration`
- `application_query::managed_index::source_rows::SourceImageView` held by `application_query::managed_index::source_rows::SourceImageRows` in `impl SourceImageView<'_>`
- `application_query::managed_index::source_structure::SourceTree` held by `application_query::managed_index::source_structure::cold::acyclic` in `impl SourceTree`
- `application_query::managed_index::source_structure::SourceTree` held by `application_query::managed_index::source_structure::cold_postings::capture` in `impl SourceTree`

### `application_contribution` ↔ `conditional_operation`: 4 references up in 3 items, default **contract**

Items: `conditional_operation::signal_decision_reentry::classify_bridge_signal` (2), `conditional_operation::installation::WorthQueryConditionalApplicationRuntimeInstallation` (1), `conditional_operation::predicate_admission::output_readiness_installation::prepare_dependency_conditional_installation` (1)

- `conditional_operation::installation::WorthQueryConditionalApplicationRuntimeInstallation` held by `conditional_operation::installation::denial::foreign_binding_denial` in `impl<Schema> WorthQueryConditionalApplicationRuntimeInstallation<Schema>`
- `conditional_operation::installation::WorthQueryConditionalApplicationRuntimeInstallation` held by `application_runtime::installation::input::ApplicationRuntimePublication` in `declaration`
- `conditional_operation::installation::WorthQueryConditionalApplicationRuntimeInstallation` held by `conditional_operation::installation::pending_operation::WorthQueryPendingConditionalOperation` in `declaration`
- `conditional_operation::installation::WorthQueryConditionalApplicationRuntimeInstallation` held by `application_runtime::installation::publish_application_runtime_with_conditionals` in `impl<Schema> WorthQueryConditionalApplicationRuntimeInstallation<Schema>`
- `conditional_operation::installation::WorthQueryConditionalApplicationRuntimeInstallation` held by `application_runtime::installation::input::ApplicationRuntimePublication` in `impl<Schema> WorthQueryConditionalApplicationRuntimeInstallation<Schema>`
- `conditional_operation::installation::WorthQueryConditionalApplicationRuntimeInstallation` held by `conditional_operation::access_validation::validate_reconstruction_access` in `impl<Schema> WorthQueryConditionalApplicationRuntimeInstallation<Schema>`

### `entity_resolution` ↔ `application_query`: 4 references up in 1 items, default **contract**

Items: `application_query::native_access::ApplicationQueryNativeAccess` (4)

- `application_query::native_access::ApplicationQueryNativeAccess` held by `root::WorthQueryPrimaryGraph` in `impl<'a> ApplicationQueryNativeAccess<'a>`

### `root` ↔ `application_query`: 4 references up in 3 items, default **contract**

Items: `application_query::derived_view::registry::ManagedDerivedViewRegistry` (2), `application_query::managed_index::source::installation::ManagedSourceInstallation` (1), `output_lineage::invalidation::owner::SourceInvalidationOwner` (1)

- `output_lineage::invalidation::owner::SourceInvalidationOwner` held by `output_lineage::invalidation::source_alignment::SnapshotAlignedMarkState` in `impl SourceInvalidationOwner`
- `output_lineage::invalidation::owner::SourceInvalidationOwner` held by `output_lineage::invalidation::settlement::SettlementRegistration` in `impl SourceInvalidationOwner`
- `output_lineage::invalidation::owner::SourceInvalidationOwner` held by `output_lineage::source_facts::ComparableSourceFacts` in `impl SourceInvalidationOwner`
- `output_lineage::invalidation::owner::SourceInvalidationOwner` held by `output_lineage::invalidation::computation_changes::ComputationChanges` in `impl SourceInvalidationOwner`
- `output_lineage::invalidation::owner::SourceInvalidationOwner` held by `output_lineage::source_facts::RetainedSourceFacts` in `impl SourceInvalidationOwner`
- `output_lineage::invalidation::owner::SourceInvalidationOwner` held by `output_lineage::invalidation::full_verification::FullVerificationImage` in `impl SourceInvalidationOwner`

### `invariant_projection` ↔ `operation_body`: 3 references up in 2 items, default **contract**

Items: `operation_body::requirement::value::field_value` (2), `operation_body::requirement::evaluation::evaluate_requirements` (1)

- `operation_body::requirement::value::field_value` held by `operation_body::requirement::expression_denial` in `body`
- `operation_body::requirement::evaluation::evaluate_requirements` held by `operation_body::requirement::evaluation::evaluate_requirement` in `body`
- `operation_body::requirement::evaluation::evaluate_requirements` held by `operation_body::requirement::evaluation::finding_quote` in `body`

### `program_occurrence` ↔ `application_installation`: 3 references up in 1 items, default **contract**

Items: `application_installation::program::supported_program::WorthQuerySupportedProgramRecord` (3)

- `application_installation::program::supported_program::WorthQuerySupportedProgramRecord` held by `product_operation::program_adoption::authorization::ProgramAdoptionAdmission` in `impl WorthQuerySupportedProgramRecord`
- `application_installation::program::supported_program::WorthQuerySupportedProgramRecord` held by `product_operation::program_adoption::authorization::admit_program` in `impl WorthQuerySupportedProgramRecord`

### `operation_body` ↔ `handler`: 3 references up in 1 items, default **contract**

Items: `handler::invariant::DecisionReader` (3)

- `handler::invariant::DecisionReader` held by `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` in `impl<'borrow, 'reader, 'runtime, Schema, Binding> DecisionReader<'borrow, 'reade`
- `handler::invariant::DecisionReader` held by `application_entry::mutation::HandlerResult` in `impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>> Dec`
- `handler::invariant::DecisionReader` held by `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` in `declaration`
- `handler::invariant::DecisionReader` held by `application_contribution::conditional::WorthQueryApplicationConditionalBinding` in `impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>> Dec`
- `handler::invariant::DecisionReader` held by `operation_body::input::prepare_inputs` in `impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>> Dec`
- `handler::invariant::DecisionReader` held by `operation_body::requirement::evaluation::evaluate_requirements` in `impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>> Dec`

### `output_reuse` ↔ `application_output_demand`: 3 references up in 1 items, default **contract**

Items: `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` (3)

- `application_output_demand::denial::kind::WorthQueryOutputDemandDenialKind` held by `application_output_demand::denial::bridge_deferral::is_temporarily_deferred` in `impl WorthQueryOutputDemandDenialKind`

### `schema_layout` ↔ `application_attempt`: 3 references up in 2 items, default **contract**

Items: `application_attempt::workflow::schema::layout::WorthQueryWorkflowLayout` (2), `application_attempt::workflow::schema::lower_workflow` (1)

- `application_attempt::workflow::schema::layout::WorthQueryWorkflowLayout` held by `application_attempt::workflow::schema::layout::WorkflowLayoutFields` in `declaration`
- `application_attempt::workflow::schema::layout::WorthQueryWorkflowLayout` held by `application_attempt::workflow::schema::layout::WorkflowLayoutFields` in `impl WorthQueryWorkflowLayout`
- `application_attempt::workflow::schema::layout::WorthQueryWorkflowLayout` held by `application_attempt::workflow::schema::layout::WorkflowLayoutFields` in `impl std::ops::Deref for WorthQueryWorkflowLayout`
- `application_attempt::workflow::schema::lower_workflow` held by `application_attempt::workflow::schema::approval::lower_approval` in `body`
- `application_attempt::workflow::schema::lower_workflow` held by `application_attempt::workflow::schema::assessment::lower_assessment_evidence` in `body`
- `application_attempt::workflow::schema::lower_workflow` held by `application_attempt::workflow::schema::captured_input::lower` in `body`

### `application_checkpoint` ↔ `output_lineage`: 2 references up in 2 items, default **contract**

Items: `output_lineage::WorthQueryApplicationOutputLineage` (1), `output_lineage::computation_source::ComputationSourceEvidence` (1)

- `output_lineage::WorthQueryApplicationOutputLineage` held by `output_lineage::recorded_output::RecordedOutput` in `impl WorthQueryApplicationOutputLineage`
- `output_lineage::WorthQueryApplicationOutputLineage` held by `output_lineage::native_output_witness::SealedNativeOutputWitness` in `impl WorthQueryApplicationOutputLineage`
- `output_lineage::WorthQueryApplicationOutputLineage` held by `output_lineage::retained_computation::RecordedComputation` in `impl WorthQueryApplicationOutputLineage`
- `output_lineage::WorthQueryApplicationOutputLineage` held by `application_attempt::compare_and_commit::commit_receipt::WorthQueryApplicationCommitReceipt` in `impl WorthQueryApplicationOutputLineage`
- `output_lineage::WorthQueryApplicationOutputLineage` held by `output_lineage::source_facts::RetainedSourceFacts` in `impl WorthQueryApplicationOutputLineage`
- `output_lineage::WorthQueryApplicationOutputLineage` held by `output_lineage::RecordedGeneration` in `impl WorthQueryApplicationOutputLineage`

### `application_runtime` ↔ `application_installation`: 2 references up in 2 items, default **contract**

Items: `application_installation::program::program_owner::selected_owner::SelectedOwnerOps` (1), `application_installation::program::program_owner::WorthQuerySelectedProgramOwner` (1)

- `application_installation::program::program_owner::selected_owner::SelectedOwnerOps` held by `application_installation::program::program_owner::WorthQuerySelectedProgramOwner` in `declaration`
- `application_installation::program::program_owner::WorthQuerySelectedProgramOwner` held by `application_installation::program::program_owner::contract::WorthQueryProgramOwner` in `impl<Schema> WorthQueryProgramOwner<Schema> for WorthQuerySelectedProgramOwner<'`

### `application_inbound` ↔ `provider`: 2 references up in 2 items, default **contract**

Items: `provider::host::ProviderHost` (1), `provider::committed_dispatch_outbox::CommittedDispatchOutboxScopedOps` (1)

- `provider::host::ProviderHost` held by `provider::WorthQueryPrimaryGraphProvider` in `declaration`
- `provider::committed_dispatch_outbox::CommittedDispatchOutboxScopedOps` held by `provider::host::ProviderHost` in `declaration`
- `provider::committed_dispatch_outbox::CommittedDispatchOutboxScopedOps` held by `provider::host::ProviderHost` in `impl< Schema, HostRuntime: crate::domain_computation::primary_graph::provider::h`

### `application_attempt` ↔ `application_entry`: 2 references up in 2 items, default **contract**

Items: `application_entry::mutation::HandlerResult` (1), `application_entry::mutation::execution::candidate::build_handler_candidate` (1)

- `application_entry::mutation::HandlerResult` held by `application_entry::mutation::failure::HandlerFailure` in `impl<Value, Domain> HandlerResult<Value, Domain>`
- `application_entry::mutation::execution::candidate::build_handler_candidate` held by `application_entry::mutation::HandlerResult` in `body`
- `application_entry::mutation::execution::candidate::build_handler_candidate` held by `application_entry::mutation::OperationHandler` in `body`

### `output_lineage` ↔ `invariant_projection`: 2 references up in 1 items, default **contract**

Items: `invariant_projection::output_availability_candidate::ObservedOutputCandidate` (2)

- `invariant_projection::output_availability_candidate::ObservedOutputCandidate` held by `invariant_projection::output_availability_candidate::dependencies::PostingDependencies` in `declaration`
- `invariant_projection::output_availability_candidate::ObservedOutputCandidate` held by `invariant_projection::output_availability_candidate::dependencies::PostingDependencies` in `impl ObservedOutputCandidate`

### `application_entry` ↔ `handler`: 2 references up in 1 items, default **contract**

Items: `handler::invariant::DecisionReader` (2)

- `handler::invariant::DecisionReader` held by `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` in `impl<'borrow, 'reader, 'runtime, Schema, Binding> DecisionReader<'borrow, 'reade`
- `handler::invariant::DecisionReader` held by `application_entry::mutation::HandlerResult` in `impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>> Dec`
- `handler::invariant::DecisionReader` held by `invariant_projection::operation_reader::WorthQueryApplicationOperationInvariantProjectionReader` in `declaration`
- `handler::invariant::DecisionReader` held by `application_contribution::conditional::WorthQueryApplicationConditionalBinding` in `impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>> Dec`
- `handler::invariant::DecisionReader` held by `operation_body::input::prepare_inputs` in `impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>> Dec`
- `handler::invariant::DecisionReader` held by `operation_body::requirement::evaluation::evaluate_requirements` in `impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>> Dec`

