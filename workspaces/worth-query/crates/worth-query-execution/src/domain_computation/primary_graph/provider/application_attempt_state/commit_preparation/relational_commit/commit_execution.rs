//! The single authoritative Relational commit transition.

mod availability;
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
    let WorthQueryPreparedApplicationCommit {
        mut attempt,
        mut members,
        candidate,
        predicate,
        work,
        retained_preimage,
        preimage_retention_work,
        source_fact_rebase,
        _completion,
    } = prepared;
    let _ = mint;
    let group_evidence = coupled::prepare_evidence_slots(&execution, members.len())?;
    let group_emissions = coupled::prepare_emissions(&execution, &attempt, &members)?;
    let product = attempt.affinity().product_publication().clone();
    let before = precommit_snapshot::WorthQueryPrecommitSnapshot::acquire(
        &execution,
        provider.graph.clone(),
        product.observation().basis().relational_basis(),
    )
    .map_err(|denial| {
        snapshot_admission_failure(
            WorthQueryProviderSessionProtocolStage::Commit,
            denial.into(),
            "application publication could not retain its exact pre-commit basis",
        )
    })
    .map_err(crate::domain_computation::WorthQueryProviderSessionCommitStop::from)?;
    let consumed_output::VerifiedConsumedOutputPublication {
        mut required_prerequisites,
    } = consumed_output::verify(&execution, provider, &mut attempt, before.as_snapshot())?;
    let availability_fence =
        availability::prepare(&execution, provider, &attempt, before.as_snapshot())?;
    let candidate = required_origin::bind(
        candidate,
        &execution,
        before.as_snapshot(),
        provider,
        &attempt,
        required_prerequisites.as_ref(),
        history.is_some_and(|cutover| cutover.uses_owned_model_graph()),
    )?;
    let predecessor = match (
        required_prerequisites.as_ref(),
        attempt.idempotency().producer_dependency_identity(),
    ) {
        (Some(required), Some(dependency)) => {
            let (producer, _) = required.native_prior_checkpoint_input();
            candidate
                .output_observation_predecessor(&execution, producer, &dependency)
                .map_err(transaction_commit_stop)?
        }
        _ => None,
    };
    let source_fact_rebase = source_fact_rebase.with_predecessor(predecessor);
    provider.fault_port.candidate_ready_for_native_preparation();
    let mut candidate = provider
        .graph
        .with_runtime_mut(&execution, |runtime| {
            runtime.prepare_validated_proposal(&execution, candidate)
        })
        .map_err(transaction_commit_stop)?;
    if let Some(fence) = availability_fence {
        fence
            .bind(&mut candidate)
            .map_err(native_output_witness_stop)?;
    }
    managed_source::bind(
        &execution,
        provider,
        before.as_snapshot(),
        &attempt,
        &mut candidate,
    )?;
    coupled::prepare_members(
        &execution,
        provider,
        &attempt,
        &mut members,
        &mut candidate,
        before.as_snapshot(),
        allocation_policy,
    )?;
    let prepared_group_publications = if members.is_empty() {
        None
    } else {
        Some(
            candidate
                .prepare_required_output_publications(
                    &execution,
                    &provider
                        .graph
                        .application_publication
                        .as_ref()
                        .expect("Native admitted the installed group issuer")
                        .native,
                )
                .map_err(transaction_commit_stop)?,
        )
    };
    correspondence::admit(
        &execution,
        provider,
        &mut attempt,
        &mut members,
        &candidate,
        prepared_group_publications.as_ref(),
    )?;
    // Native admission has separated canonical roles from closure evidence.
    // Prepare exactly those roles before publication seals their content proof.
    let prepared_output_witness =
        crate::domain_computation::primary_graph::output_lineage::PreparedNativeOutputWitness::prepare(
            &attempt,
            &provider.graph.layout,
        )
        .map_err(native_output_witness_stop)?;
    let prepared_touched_records =
        touched_records::PreparedTouchedRecords::prepare(&candidate, allocation_policy)?;
    let source_fact_rebase = provider.graph.with_runtime(&execution, |runtime| source_fact_rebase.prepare_candidate(&candidate, runtime, before.as_snapshot(),
        crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(allocation_policy, Some(attempt.affinity().publication_request())),
    )).map_err(stops::source_rebase_preparation_stop)?;
    let index_maintenance_work = candidate.prepared_index_maintenance_work().ok_or_else(|| {
        index_preparation_stop(worth_relational::facade::indexes::DerivedIndexMaintenanceDenial {
            kind: worth_relational::facade::indexes::DerivedIndexMaintenanceDenialKind::CandidateUnavailable,
            work: Default::default(),
        })
    })?;
    // The native preparation already paid the phase. The existing caller's
    // tighter admission policy can refuse that actual work before publication.
    if provider.take_tight_index_maintenance_budget() && index_maintenance_work.work_units > 1 {
        return Err(index_preparation_stop(worth_relational::facade::indexes::DerivedIndexMaintenanceDenial {
            kind: worth_relational::facade::indexes::DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded,
            work: index_maintenance_work,
        }));
    }
    let managed_views = managed_views::prepare(
        &execution,
        provider,
        &product,
        before.as_snapshot(),
        &candidate,
    );
    provider.after_application_candidate_preparation_for_test();
    let adoption = executed_adoption::prepare(
        &execution,
        provider,
        &attempt,
        &candidate,
        members.is_empty() && history.is_none() && prepared_output_witness.is_some(),
        source_fact_rebase,
        required_prerequisites,
    )?;
    let (source_fact_rebase, mut required_prerequisites) = match adoption {
        executed_adoption::AdoptionPreparation::Declined { rebase, required } => (rebase, required),
        executed_adoption::AdoptionPreparation::Selected {
            selected,
            adoption,
            association,
            rebase,
            required,
        } => {
            return Ok(ApplicationCommitExecution::AdoptionPrepared(Box::new(
                executed_adoption::PreparedExecutedOutputAdoption::new(
                    attempt,
                    candidate,
                    selected,
                    adoption,
                    association,
                    before,
                    rebase,
                    required,
                    prepared_output_witness.expect("adoption retained its real prepared witness"),
                ),
            )));
        }
    };
    let performed = product_publication::publish(
        execution,
        provider,
        &mut attempt,
        candidate,
        &mut required_prerequisites,
        &mut members,
        history,
    )?;
    let performed = match performed {
        product_publication::WorthQueryApplicationProductPublicationOutcome::Performed(
            performed,
        ) => performed,
        product_publication::WorthQueryApplicationProductPublicationOutcome::Unpublished {
            unpublished,
            reserved_terminal,
            prepared_lineage_slot,
            reservation,
        } => {
            let retained = ManagedUnpublishedAttempt {
                publication_mode: if unpublished.retains_conditional_definition() {
                    managed_unpublished::ManagedUnpublishedPublicationMode::ConditionalDefinition
                } else {
                    managed_unpublished::ManagedUnpublishedPublicationMode::Ordinary
                },
                attempt,
                members,
                prepared_group_publications,
                group_evidence,
                group_emissions,
                work,
                index_maintenance_work,
                retained_preimage,
                preimage_retention_work,
                before,
                managed_views,
                predicate,
                source_fact_rebase: Some(source_fact_rebase),
                required_prerequisites,
                prepared_lineage_slot,
                prepared_output_witness,
                prepared_touched_records,
                lineage_metadata: None,

                reserved_terminal,
            };
            reservation.retain_managed(unpublished.recovery_handle(), retained);
            return Err(
                crate::domain_computation::WorthQueryProviderSessionCommitStop::ProductUnpublished(
                    unpublished,
                ),
            );
        }
    };
    let prepared_lineage_slot = performed.prepared_lineage_slot;
    let performed = performed.publication;
    let next_basis = performed
        .publication()
        .commit()
        .basis()
        .relational_basis()
        .clone();
    let committed = performed
        .publication()
        .component_results()
        .retain_relational_commit_result()
        .expect("World performed a prepared Relational application candidate");
    Ok(ApplicationCommitExecution::Committed(
        WorthQueryCommittedApplicationSession {
            attempt,
            members,
            prepared_group_publications,
            group_evidence,
            group_emissions,
            own_publication: None,
            own_records: None,
            work,
            index_maintenance_work,
            retained_preimage,
            preimage_retention_work,
            before: before.into_publication(),
            next_basis,
            committed,
            published_snapshot_custody: PublishedSnapshotCustody::LiveOwned,
            prepared_touched_records: Some(prepared_touched_records),
            product_publication: performed,
            managed_views,
            predicate,
            source_fact_rebase: Some(source_fact_rebase),

            required_prerequisites,
            prepared_lineage_slot,
            prepared_output_witness,
            coupled_deletions: Default::default(),
        },
    ))
}

#[cfg(test)]
mod index_preparation_tests;
