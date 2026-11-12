//! Publish an already prepared memory candidate and retain its exact terminal custody.
use super::*;
pub(super) fn publish(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    query: durable_custody::QueryPreparingApplicationCommit,
    history: Option<crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerCutover<'_>>,
) -> Result<
    ApplicationCommitExecution,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    let durable_custody::QueryPreparingApplicationCommit {
        mut attempt,
        mut members,
        prepared_group_publications,
        group_evidence,
        group_emissions,
        predicate,
        native,
        work,
        retained_preimage,
        preimage_retention_work,
        completion: _completion,
        before,
        mut required_prerequisites,
        prepared_output_witness,
        source_fact_rebase,
        prepared_touched_records,
        index_maintenance_work,
        managed_views,
        ..
    } = query;
    let durable_custody::QueryNativeCustody::Candidate(candidate) = native else {
        unreachable!("Native preparation completed once")
    };
    let source_fact_rebase = source_fact_rebase.expect("candidate source facts prepared");
    let adoption = executed_adoption::prepare(
        &execution,
        provider,
        &attempt,
        &candidate,
        members.is_empty() && history.is_none() && prepared_output_witness.is_some(),
        source_fact_rebase,
        required_prerequisites,
    )?;
    let (source_fact_rebase, restored_required) = match adoption {
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
                    prepared_output_witness.expect("adoption retains its real prepared witness"),
                ),
            )));
        }
    };
    required_prerequisites = restored_required;
    let prepared_touched_records =
        prepared_touched_records.expect("candidate touched records prepared");
    let index_maintenance_work = index_maintenance_work.expect("candidate indexes prepared");
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
                _completion,
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
                predicate,
                work,
                index_maintenance_work,
                retained_preimage,
                preimage_retention_work,
                before,
                managed_views,
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
            predicate,
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
            source_fact_rebase: Some(source_fact_rebase),

            required_prerequisites,
            prepared_lineage_slot,
            prepared_output_witness,
            coupled_deletions: Default::default(),
        },
    ))
}
