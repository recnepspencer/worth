//! Exact per-writer lineage and receipt settlement after the one World movement.
use super::*;
pub(super) fn finish_writer(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    pending: &mut WorthQueryPendingApplicationPublication,
    after: &worth_relational::facade::snapshots::SnapshotHandle,
    commit_id: worth_relational::facade::history::CommitId,
    publish_causality: bool,
) -> (
    WorthQueryPrimaryGraphCommittedApplication,
    super::super::original_attempt_completion::OriginalCompletionPublisher,
) {
    let committed_product_publication = pending
        .application
        .as_ref()
        .expect("performed group publication")
        .committed_product_publication()
        .clone();
    let mut attempt = pending
        .attempt
        .take()
        .expect("pending application publication retains causality until final cutover");
    // The output's lineage record and its settlement row hold the consumed
    // outputs; the completed receipt evidence never does.
    let original_completion = attempt.affinity().take_original_completion_publisher();
    let consumed_outputs = attempt.retain_consumed_outputs();
    let outstanding = attempt.take_outstanding_dispatch_reservation();
    if publish_causality {
        let predicate = pending.predicate.as_ref().map(|owner| {
            owner.publication_for(
                attempt.affinity().product_publication().observation(),
                &committed_product_publication,
            )
        });
        let causality = attempt.publish_group_causality(
            provider,
            committed_product_publication,
            predicate,
            pending.group_emissions.take(),
        );
        assert_eq!(
            causality.emitted_effect_count(),
            pending.emitted_effect_count
        );
        assert_eq!(causality.outcome_identity(), pending.outcome_identity);
    }
    if let Some(outstanding) = outstanding {
        outstanding.commit(
            commit_id,
            pending
                .application
                .as_ref()
                .expect("performed World publication remains in pending evidence")
                .committed_product_publication(),
        );
    }
    pending.release_before(runtime);
    let mut completed = pending
        .application
        .take()
        .expect("completed publication retains exact commit evidence until cutover");
    assert!(
        pending.required_prerequisites.is_none() || pending.prepared_lineage_slot.is_some(),
        "managed publication must carry a prepared output lineage slot"
    );
    let recorded = pending
        .prepared_lineage_slot
        .take()
        .map(|slot| slot.record(&completed, Arc::clone(&consumed_outputs)));
    let settlement_identity = recorded
        .as_ref()
        .map(|recorded| Arc::clone(&recorded.identity));
    if let Some(identity) = settlement_identity.as_ref() {
        completed.retain_exact_output_settlement(Arc::clone(identity));
    }
    let work_membership = pending
        .required_prerequisites
        .as_ref()
        .and_then(|prerequisites| prerequisites.work_membership());
    let superseded = pending.required_prerequisites.take().map(|prerequisites| {
        prerequisites.publish(Arc::clone(
            settlement_identity
                .as_ref()
                .expect("managed producer publication records its exact output settlement"),
        ))
    });
    if let Some(identity) = settlement_identity {
        let failure_membership = work_membership.clone();
        let owner = &provider.graph.source_owner.invalidation_owner;

        let registration_result = crate::domain_computation::primary_graph::output_lineage::invalidation::register_completed(execution, owner, &completed, recorded.as_ref().expect("performed lineage was recorded").computation_source, recorded.as_ref().expect("performed lineage was recorded").computation_registration, &consumed_outputs, runtime, after, Arc::clone(&identity), work_membership, recorded
                .as_ref()
                .and_then(|recorded| recorded.output_witness.as_ref())
                .and_then(|witness| witness.get()));
        if let Err(reason) = registration_result {
            provider
                .graph
                .output_lineage
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .require_settlement_verification(&identity, reason);
            if let Some(membership) = failure_membership {
                let prior = membership.mark_local_required(Arc::clone(&identity));
                drop(prior);
            }
        }
    }
    // The newest row is registered: the generation it displaced retires,
    // with whatever its demand supersedes when a demand manages it.
    let owner = &provider.graph.source_owner.invalidation_owner;
    let displaced = recorded.and_then(|recorded| recorded.displaced);
    match (superseded, displaced) {
        (Some(superseded), displaced) => {
            // Committed discovery cannot refuse and orphan the displaced settlement.
            superseded.retire(displaced, owner);
        }
        (None, Some(displaced)) => {
            drop(owner.retire_released([displaced]));
        }
        (None, None) => {}
    }
    (completed, original_completion)
}
