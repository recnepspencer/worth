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

mod consumed_output;
mod managed_source;
mod managed_unpublished;
mod managed_views;
pub(in crate::domain_computation::primary_graph::provider) use managed_unpublished::ManagedUnpublishedAttempt;
pub(crate) mod precommit_snapshot;
mod product_publication;
mod required_origin;
pub(crate) mod stops;
pub(crate) mod touched_records;
use stops::failure;
use stops::index_preparation_stop;
use stops::native_output_witness_stop;
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
}

#[derive(Clone, Copy)]
enum PublishedSnapshotCustody {
    LiveOwned,
    SettledReleased,
}

pub(super) fn commit(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    prepared: WorthQueryPreparedApplicationCommit,
    mint: super::WorthQueryCommitProgressionMint,
    history: Option<crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerCutover<'_>>,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<
    WorthQueryCommittedApplicationSession,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    // The actual weak owner validates the expected owner before selecting the
    // ordinary algorithm. Unavailability never selects memory publication.
    let query = native_work::prepare_before_native(&execution, provider, prepared)?;
    if provider.take_fault(crate::domain_computation::primary_graph::provider::fault_port::WorthQueryPrimaryGraphFault::DuplicateQueryPreparationSlot) {
        return preparation_reservation_control::refuse_duplicate_before_world_for_test(execution, provider, query, allocation_policy);
    }
    let product = query.attempt.affinity().product_publication().clone();
    match product
        .publication()
        .query_publication_home_binding(product.observation())
    {
        Ok(None) => memory::commit(execution, provider, query, mint, history, allocation_policy),
        Ok(Some(_)) => match history {
            None => durable::commit(execution, provider, query, allocation_policy),
            Some(_) => Err(crate::domain_computation::WorthQueryProviderSessionCommitStop::ProductPreparationRefused(
                crate::domain_computation::WorthQueryProductPreparationRefused::new(
                    crate::domain_computation::WorthQueryProductPreparationRefusal::CurrentReadDependency(
                        worth_runtime_world::facade::CurrentProductReadDependencyDenial::DurabilityUnsupported), query))),
        },
        Err(cause) => Err(durable::refused_before_open(query, cause)),
    }
}

fn publication_request_stop(
    cause: worth_execution::WorkCeilingDenial,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    use crate::domain_computation::WorthQueryProviderSessionCommitControlStopped;
    use crate::domain_computation::WorthQueryProviderSessionCommitStop as Stop;
    use crate::domain_computation::WorthQueryProviderSessionFailure;
    use crate::domain_computation::WorthQueryProviderSessionProtocolStage;
    match crate::domain_computation::primary_graph::application_contribution::publication_execution_kind(cause) {
        Ok(kind) => Stop::PreEffectDenied(WorthQueryProviderSessionFailure::checked_read_stopped(Ok(kind), WorthQueryProviderSessionProtocolStage::Commit, "World publication request stopped before effect")),
        Err(control) => Stop::ControlStopped(WorthQueryProviderSessionCommitControlStopped::new(control, "World publication request stopped before effect")),
    }
}

fn world_no_effect(
    no_effect: worth_runtime_world::facade::NoEffectCompositePublication,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    use crate::domain_computation::WorthQueryProviderSessionCommitControlStopped;
    use crate::domain_computation::WorthQueryProviderSessionCommitStop as Stop;
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind;
    use worth_runtime_world::facade::NoEffectCause;
    match no_effect.cause() {
        NoEffectCause::StaleExpectedProductHead => Stop::ProductStale(
            crate::domain_computation::WorthQueryProductStaleApplication::new(no_effect),
        ),
        NoEffectCause::CancelledBeforeEffect
        | NoEffectCause::ExecutionRequest(worth_execution::WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::Cancelled,
        )) => Stop::ControlStopped(WorthQueryProviderSessionCommitControlStopped::new(
            WorthQueryProviderSessionControlStopKind::Cancelled,
            "World publication cancelled before effect",
        )),
        NoEffectCause::DeadlineBeforeEffect
        | NoEffectCause::ExecutionRequest(worth_execution::WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::DeadlineElapsed,
        )) => Stop::ControlStopped(WorthQueryProviderSessionCommitControlStopped::new(
            WorthQueryProviderSessionControlStopKind::TimedOut,
            "World publication deadline elapsed before effect",
        )),
        NoEffectCause::DurableOpenRequired
        | NoEffectCause::ExecutionRequest(_)
        | NoEffectCause::OwnerDeniedBeforeEffect
        | NoEffectCause::CurrentReadDependency(_)
        | NoEffectCause::StaleIncarnation
        | NoEffectCause::CorrespondenceRebindRequired
        | NoEffectCause::ReferenceGenerationExhausted
        | NoEffectCause::CapacityExhausted
        | NoEffectCause::OwnerUnavailable
        | NoEffectCause::RelationalDeferred(_)
        | NoEffectCause::PreEffectFailure => Stop::NoEffect(no_effect),
    }
}
impl WorthQueryCommittedApplicationSession {
    pub(super) fn retain_recovered_producer_facts(
        &self,
        evidence: &super::WorthQueryPrimaryGraphCommitEvidence,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> std::sync::Arc<crate::domain_computation::primary_graph::application_output_demand::RecoveredProducerOutputFacts>{
        use crate::domain_computation::primary_graph::application_output_demand::RecoveredProducerOutputFacts;
        std::sync::Arc::new(RecoveredProducerOutputFacts {
            publication: crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication::from_receipt(self.product_publication.clone()),
            changes: evidence.committed_changes().clone(),
            outcome: self.attempt.outcome_identity(),
            correspondence: evidence.retain_output_correspondence(),
            authority: evidence.retain_original_authority(),
            inputs: evidence.performed_input_facts().cloned(),
            fields: evidence.performed_output_fields().cloned(),
            unrebased_own_effect: evidence.rebased_source_facts().err().map(|failure| failure.own_effect),
            settlement: self.prepared_lineage_slot.as_ref().map(|slot| slot.identity().clone()),
            _memory: memory,
        })
    }
    pub(super) fn take_predicate_preparation(&mut self) -> Option<crate::domain_computation::primary_graph::application_query::derived_view::PredicatePreparationOwner>{
        self.predicate.take()
    }
    pub(in crate::domain_computation::primary_graph::provider) fn take_recovered_output_source(
        &self,
    ) -> crate::domain_computation::primary_graph::application_installation::WorthQueryRecoveredProgramOutputSource{
        crate::domain_computation::primary_graph::application_installation::WorthQueryRecoveredProgramOutputSource::take_original(&self.product_publication)
    }
    pub(super) fn take_prepared_touched_records(&mut self) -> PreparedTouchedRecords {
        self.prepared_touched_records
            .take()
            .expect("pre-effect native touched-record receipt storage follows publication")
    }
    pub(super) fn take_source_fact_rebase(&mut self) -> super::PreparedSourceFactRebase {
        self.source_fact_rebase
            .take()
            .expect("pre-effect source-fact rebase capacity follows the committed candidate")
    }
    pub(super) const fn attempt(&self) -> &WorthQueryPrimaryGraphApplicationAttempt {
        &self.attempt
    }

    pub(super) const fn work(&self) -> WorthQueryPrimaryMutationWorkCounters {
        self.work
    }

    pub(super) const fn index_maintenance_work(
        &self,
    ) -> worth_relational::facade::indexes::DerivedIndexMaintenanceWork {
        self.index_maintenance_work
    }

    pub(super) const fn retained_preimage(
        &self,
    ) -> Option<&crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage> {
        self.retained_preimage.as_ref()
    }

    pub(super) const fn preimage_retention_work(&self) -> WorthQueryPreImageRetentionWork {
        self.preimage_retention_work
    }

    pub(super) fn committed(&self) -> &worth_relational::facade::transactions::CommitResult {
        self.committed.as_ref()
    }

    pub(super) fn publish_and_encode(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        provider: &WorthQueryPrimaryGraphProvider,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        evidence: super::WorthQueryPrimaryGraphCommitEvidence,
        predicate: &mut Option<crate::domain_computation::primary_graph::application_query::derived_view::PredicatePreparationOwner>,
        retired: &mut crate::domain_computation::primary_graph::application_query::derived_view::PredicatePreparationRetirement,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        WorthQueryProviderSessionFailure,
    > {
        let published = publication::publish(
            execution, provider, runtime, self, evidence, predicate, retired,
        )?;
        publication::encode(provider, published)
    }
}

#[cfg(test)]
mod index_preparation_tests;

// These three governed producer calls stay in this exact registered module.
fn prepare_query_open(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    control: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationRequestControl,
) -> Result<
    worth_runtime_world::facade::WorldPendingOpenAttempt,
    worth_runtime_world::facade::WorldOpenAttemptRefusal,
> {
    product
        .publication()
        .prepare_query_publication_without_signal(
            execution,
            product.observation().clone(),
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            control.cancellation(),
            control.deadline(),
        )
}
fn bind_query_proposal(
    opened: worth_runtime_world::facade::WorldOpenedPublicationAttempt,
    proposal: worth_relational::facade::mvcc::ValidatedRelationalProposal,
) -> Result<
    (
        worth_runtime_world::facade::WorldQueryNativePreparation,
        worth_relational::facade::mvcc::ValidatedRelationalProposal,
    ),
    worth_runtime_world::facade::WorldNativePreparationRefusal,
> {
    opened.bind_query_native_proposal(proposal)
}
fn complete_query_candidate(
    preparation: worth_runtime_world::facade::WorldQueryNativePreparation,
    candidate: worth_relational::facade::mvcc::PreparedRelationalCommitCandidate,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<
    worth_runtime_world::facade::WorldPreparedPublicationWithoutSignal,
    worth_runtime_world::facade::WorldNativePreparationRefusal,
> {
    preparation.complete_query_native_preparation(candidate, request)
}
#[cfg(test)]
#[path = "commit_execution_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
