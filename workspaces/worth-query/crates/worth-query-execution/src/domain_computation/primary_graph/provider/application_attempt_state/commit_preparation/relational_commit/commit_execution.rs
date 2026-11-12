//! The single authoritative Relational commit transition.

mod availability;
mod before_world;
mod durable;
mod durable_custody;
mod memory;
mod native_work;
mod preparation_reservation_control;
pub(in crate::domain_computation) use durable_custody::QueryPreparingApplicationCommit;
pub(in crate::domain_computation::primary_graph::provider) use durable_custody::RetainedQueryDurablePreparation;
mod committed_session;
mod correspondence;
mod coupled;
pub(crate) mod executed_adoption;

mod consumed_output;
mod managed_source;
mod managed_unpublished;
mod managed_views;
pub(in crate::domain_computation::primary_graph::provider) use managed_unpublished::ManagedUnpublishedAttempt;
pub(crate) mod precommit_snapshot;
mod product_publication;
mod world_stop;
use world_stop::{publication_request_stop, world_no_effect};
mod required_origin;
pub(crate) mod stops;
pub(crate) mod touched_records;
use stops::failure;
use stops::index_preparation_stop;
use stops::native_output_witness_stop;
use stops::readmission_allocation;
use stops::transaction_commit_stop;
pub(in crate::domain_computation::primary_graph::provider) use touched_records::PreparedTouchedRecords;
pub(crate) mod publication;
pub(in crate::domain_computation::primary_graph) use publication::WorthQueryPrimaryGraphCommittedApplication;

use super::super::WorthQueryPreparedApplicationCommit;
use crate::domain_computation::primary_graph::provider::mutation_work::WorthQueryPrimaryMutationWorkCounters;
use crate::domain_computation::primary_graph::provider::session_commit::provider_failure;
use crate::domain_computation::primary_graph::provider::session_commit::snapshot_admission_failure;
use crate::domain_computation::primary_graph::provider::session_commit::WorthQueryPreImageRetentionWork;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphApplicationAttempt;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::WorthQueryProviderSessionFailure;
use crate::domain_computation::WorthQueryProviderSessionProtocolStage;

pub(in crate::domain_computation::primary_graph::provider) struct WorthQueryCommittedApplicationSession {
    attempt: WorthQueryPrimaryGraphApplicationAttempt,
    members: Vec<crate::domain_computation::primary_graph::provider::CoupledApplicationMember>,
    group_emissions: Option<crate::domain_computation::primary_graph::application_attempt::WorthQueryAdmittedApplicationEmissionBatch>,
    group_evidence: Option<coupled::CoupledEvidenceSlots>,
    own_publication: Option<worth_relational::facade::mvcc::RelationalRequiredOutputPublication>,
    own_records: Option<worth_execution::ExecutionArray<worth_relational::facade::transactions::RecordRef>>,
    prepared_group_publications: Option<worth_relational::facade::mvcc::RelationalPreparedOutputPublications>,
    work: WorthQueryPrimaryMutationWorkCounters,
    predicate: Option<crate::domain_computation::primary_graph::application_query::derived_view::PredicatePreparationOwner>,
    index_maintenance_work: worth_relational::facade::indexes::DerivedIndexMaintenanceWork,
    retained_preimage:
        Option<crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage>,
    preimage_retention_work: WorthQueryPreImageRetentionWork,
    before: worth_relational::facade::snapshots::SnapshotHandle,
    next_basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    committed: std::sync::Arc<worth_relational::facade::transactions::CommitResult>,
    published_snapshot_custody: PublishedSnapshotCustody,
    prepared_touched_records: Option<PreparedTouchedRecords>,
    product_publication: crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationReceipt,
    managed_views: Option<managed_views::PreparedViewPublication>,
    source_fact_rebase: Option<super::PreparedSourceFactRebase>,
    required_prerequisites:
        Option<crate::domain_computation::primary_graph::PreparedPrerequisiteClaims>,
    prepared_lineage_slot:
        Option<crate::domain_computation::primary_graph::output_lineage::PreparedOutputLineageSlot>,
    prepared_output_witness:
        Option<crate::domain_computation::primary_graph::output_lineage::PreparedNativeOutputWitness>,
    /// One coupled commit's deletions, shared by its root's and members' sessions.
    coupled_deletions: super::evidence_seal::postcommit_currentness::coupled_deletions::CoupledCommitDeletions,
}

#[derive(Clone, Copy)]
enum PublishedSnapshotCustody {
    LiveOwned,
    SettledReleased,
}

pub(super) enum ApplicationCommitExecution {
    Committed(WorthQueryCommittedApplicationSession),
    AdoptionPrepared(Box<executed_adoption::PreparedExecutedOutputAdoption>),
}

pub(super) fn commit(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    prepared: WorthQueryPreparedApplicationCommit,
    mint: super::WorthQueryCommitProgressionMint,
    history: Option<crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerCutover<'_>>,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<
    ApplicationCommitExecution,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    // The actual weak owner validates the expected owner before selecting the
    // ordinary algorithm. Unavailability never selects memory publication.
    let owned_history = history
        .as_ref()
        .is_some_and(|cutover| cutover.uses_owned_model_graph());
    let query = native_work::prepare_before_native(&execution, provider, prepared, owned_history)?;
    if provider.take_fault(crate::domain_computation::primary_graph::provider::fault_port::WorthQueryPrimaryGraphFault::DuplicateQueryPreparationSlot) {
        return preparation_reservation_control::refuse_duplicate_before_world_for_test(execution, provider, query, allocation_policy).map(ApplicationCommitExecution::Committed);
    }
    let product = query.attempt.affinity().product_publication().clone();
    match product
        .publication()
        .query_publication_home_binding(product.observation())
    {
        Ok(None) => memory::commit(execution, provider, query, mint, history, allocation_policy),
        Ok(Some(_)) => match history {
            None => durable::commit(execution, provider, query, allocation_policy).map(ApplicationCommitExecution::Committed),
            Some(_) => Err(crate::domain_computation::WorthQueryProviderSessionCommitStop::ProductPreparationRefused(
                crate::domain_computation::WorthQueryProductPreparationRefused::new(
                    crate::domain_computation::WorthQueryProductPreparationRefusal::CurrentReadDependency(
                        worth_runtime_world::facade::CurrentProductReadDependencyDenial::DurabilityUnsupported), query))),
        },
        Err(cause) => Err(durable::refused_before_open(query, cause)),
    }
}

#[cfg(test)]
mod index_preparation_tests;

// These three governed producer calls stay in this exact registered module.
mod durable_components;
use durable_components::{QueryBoundWorldPreparation, QueryPreparedWorldPublication};
fn prepare_query_open(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    control: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationRequestControl,
    with_signal: bool,
) -> Result<
    worth_runtime_world::facade::WorldPendingOpenAttempt,
    worth_runtime_world::facade::WorldOpenAttemptRefusal,
> {
    if with_signal {
        return product.publication().prepare_query_publication_with_signal(
            execution,
            product.observation().clone(),
            product.admitted_program_source()?,
            Some(worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary()),
            control.cancellation(),
            control.deadline(),
        );
    }
    product
        .publication()
        .prepare_query_publication_without_signal(
            execution,
            product.observation().clone(),
            product.admitted_program_source()?,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            control.cancellation(),
            control.deadline(),
        )
}
fn bind_query_proposal(
    opened: worth_runtime_world::facade::WorldOpenedPublicationAttempt,
    proposal: worth_relational::facade::mvcc::ValidatedRelationalProposal,
    with_signal: bool,
) -> Result<
    (
        QueryBoundWorldPreparation,
        worth_relational::facade::mvcc::ValidatedRelationalProposal,
    ),
    worth_runtime_world::facade::WorldNativePreparationRefusal,
> {
    if with_signal {
        opened
            .bind_query_signal_proposal(proposal)
            .map(|(prepared, proposal)| {
                (QueryBoundWorldPreparation::WithSignal(prepared), proposal)
            })
    } else {
        opened
            .bind_query_native_proposal(proposal)
            .map(|(prepared, proposal)| {
                (QueryBoundWorldPreparation::Relational(prepared), proposal)
            })
    }
}
fn complete_query_candidate(
    preparation: QueryBoundWorldPreparation,
    candidate: worth_relational::facade::mvcc::PreparedRelationalCommitCandidate,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<QueryPreparedWorldPublication, worth_runtime_world::facade::WorldNativePreparationRefusal>
{
    match preparation {
        QueryBoundWorldPreparation::Relational(preparation) => preparation
            .complete_query_native_preparation(candidate, request)
            .map(QueryPreparedWorldPublication::Relational),
        QueryBoundWorldPreparation::WithSignal(preparation) => preparation
            .complete_query_native_preparation(candidate, request)
            .map(QueryPreparedWorldPublication::WithSignal),
    }
}
#[cfg(test)]
#[path = "commit_execution_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
