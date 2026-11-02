//! Existing memory publication uses the shared one-time Native mechanics.
use super::*;
pub(super) fn commit(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    mut query: durable_custody::QueryPreparingApplicationCommit,
    mint: super::super::WorthQueryCommitProgressionMint,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<
    WorthQueryCommittedApplicationSession,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    let _ = mint;
    if let Err(cause) = native_work::prepare_native_candidate(&execution, provider, &mut query) {
        return Err(native_work::memory_stop(cause, &query.attempt));
    }
    if let Err(cause) =
        native_work::prepare_candidate_support(&execution, provider, &mut query, allocation_policy)
    {
        return Err(native_work::memory_stop(cause, &query.attempt));
    }
    let durable_custody::QueryPreparingApplicationCommit {
        mut attempt,
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
    let prepared_touched_records =
        prepared_touched_records.expect("candidate touched records prepared");
    let index_maintenance_work = index_maintenance_work.expect("candidate indexes prepared");
    let performed = product_publication::publish(
        execution,
        provider,
        &mut attempt,
        candidate,
        &mut required_prerequisites,
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
                _completion: None,
                publication_mode: if unpublished.retains_conditional_definition() {
                    managed_unpublished::ManagedUnpublishedPublicationMode::ConditionalDefinition
                } else {
                    managed_unpublished::ManagedUnpublishedPublicationMode::Ordinary
                },
                attempt,
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
    Ok(WorthQueryCommittedApplicationSession {
        attempt,
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
    })
}
