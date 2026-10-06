//! Move-only recovery state for Query publication after Relational movement.

use std::sync::Arc;

use worth_runtime_world::facade::CompositeCommitIdentity;

mod aftermath;
pub(super) mod registry;
mod stops;
pub(in crate::domain_computation::primary_graph) use registry::WorthQueryApplicationPublicationRecoveryReservation;
use registry::WorthQueryPendingApplicationIdempotency;
use stops::{basis_retention_failure, bridge_head_failure, failure, snapshot_capacity_failure};

use super::{
    session_commit::snapshot_admission_failure, WorthQueryPrimaryGraphApplicationAttempt,
    WorthQueryPrimaryGraphCommittedApplication, WorthQueryPrimaryGraphProvider,
};
use crate::domain_computation::{
    WorthQueryProviderSessionFailure, WorthQueryProviderSessionProtocolStage,
    WorthQueryProviderSessionRecoveryPosture,
};

pub(in crate::domain_computation::primary_graph) struct WorthQueryPendingApplicationPublication {
    product_incarnation: worth_runtime_world::facade::ProductBranchIncarnation,
    idempotency: crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding,
    attempt: Option<WorthQueryPrimaryGraphApplicationAttempt>,
    before: Option<worth_relational::facade::snapshots::SnapshotHandle>,
    next_basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    committed: Arc<worth_relational::facade::transactions::CommitResult>,
    application: Option<WorthQueryPrimaryGraphCommittedApplication>,
    emitted_effect_count: usize,
    outcome_identity: crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity,
    aggregate_published: bool,
    receipt_basis_lease:
        Option<worth_relational::facade::branch::RelationalBranchRetentionLease>,
    required_prerequisites:
        Option<crate::domain_computation::primary_graph::PreparedPrerequisiteClaims>,
    prepared_lineage_slot:
        Option<crate::domain_computation::primary_graph::output_lineage::PreparedOutputLineageSlot>,
    publication_admission: crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
}

impl WorthQueryPendingApplicationPublication {
    pub(in crate::domain_computation::primary_graph) fn new(
        attempt: WorthQueryPrimaryGraphApplicationAttempt,
        before: worth_relational::facade::snapshots::SnapshotHandle,
        next_basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        committed: Arc<worth_relational::facade::transactions::CommitResult>,
        application: WorthQueryPrimaryGraphCommittedApplication,
        required_prerequisites: Option<
            crate::domain_computation::primary_graph::PreparedPrerequisiteClaims,
        >,
        prepared_lineage_slot: Option<
            crate::domain_computation::primary_graph::output_lineage::PreparedOutputLineageSlot,
        >,
        publication_admission: crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
    ) -> Self {
        let product_incarnation = application
            .committed_product_publication()
            .product_incarnation();
        let idempotency = application.commit_evidence().idempotency();
        let emitted_effect_count = attempt.emitted_effect_count();
        let outcome_identity = attempt.outcome_identity();
        Self {
            product_incarnation,
            idempotency,
            attempt: Some(attempt),
            before: Some(before),
            next_basis,
            committed,
            application: Some(application),
            emitted_effect_count,
            outcome_identity,
            aggregate_published: false,
            receipt_basis_lease: None,
            required_prerequisites,
            prepared_lineage_slot,
            publication_admission,
        }
    }

    fn release_before(
        &mut self,
        runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    ) {
        if let Some(before) = self.before.take() {
            crate::relational_snapshot_release::release_query_snapshot(runtime, &before);
        }
    }

    const fn product_incarnation(&self) -> worth_runtime_world::facade::ProductBranchIncarnation {
        self.product_incarnation
    }

    pub(super) const fn idempotency(
        &self,
    ) -> crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding{
        self.idempotency
    }
}

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn inspect_pending_application_idempotency(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) -> Option<Option<crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding>>{
        let pending = self
            .pending_application_publications
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .slot(occurrence)?
            .inspect_idempotency()?;
        Some(match pending {
            WorthQueryPendingApplicationIdempotency::Reserved => None,
            WorthQueryPendingApplicationIdempotency::Pending(binding) => Some(binding),
        })
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn has_pending_application_publication_for_test(
        &self,
    ) -> bool {
        self.pending_application_publication_count_for_test() != 0
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn pending_application_publication_count_for_test(
        &self,
    ) -> usize {
        self.pending_application_publications
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .active_slot_count()
    }

    pub(in crate::domain_computation::primary_graph) fn reserve_application_publication_recovery(
        &self,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Result<WorthQueryApplicationPublicationRecoveryReservation, &'static str> {
        registry::WorthQueryPendingApplicationPublicationRegistry::reserve(
            &self.pending_application_publications,
            observation.lifecycle_incarnation(),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn install_and_publish_application(
        &self,
        runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
        reservation: WorthQueryApplicationPublicationRecoveryReservation,
        pending: WorthQueryPendingApplicationPublication,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        let occurrence = pending.product_incarnation();
        reservation.install(pending);
        self.resume_pending_application_publication(runtime, occurrence)
    }

    pub(in crate::domain_computation::primary_graph) fn resume_pending_application_publication(
        &self,
        runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        let Some(slot) = self
            .pending_application_publications
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .slot(occurrence)
        else {
            return Ok(());
        };
        let result = slot
            .with_pending(|pending| {
                let result = resume(self, runtime, pending);
                if result.is_err() {
                    pending.release_before(runtime);
                }
                result
            })
            .map_err(failure)?;
        let settled = result?;
        slot.complete();
        self.pending_application_publications
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove_exact(occurrence, &slot);
        self.retire_history_behind(runtime, &settled);
        Ok(())
    }
}

fn resume(
    provider: &WorthQueryPrimaryGraphProvider,
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    pending: &mut WorthQueryPendingApplicationPublication,
) -> Result<CompositeCommitIdentity, WorthQueryProviderSessionFailure> {
    let commit_id = pending.committed.envelope().commit.commit_id;
    if provider.take_failed_post_commit_snapshot() {
        return Err(snapshot_capacity_failure(
            "injected post-commit snapshot admission failure",
        ));
    }
    let after = runtime
        .snapshots()
        .snapshot_for_observation(&pending.next_basis.observation())
        .map_err(|denial| {
            snapshot_admission_failure(
                WorthQueryProviderSessionProtocolStage::Commit,
                denial.into(),
                "application commit basis could not open its post-commit snapshot",
            )
            .with_recovery_posture(WorthQueryProviderSessionRecoveryPosture::RecoveryRequired)
        })?;
    let result = publish_with_snapshot(provider, runtime, pending, &after, commit_id);
    crate::relational_snapshot_release::release_query_snapshot(runtime, &after);
    result
}

fn publish_with_snapshot(
    provider: &WorthQueryPrimaryGraphProvider,
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    pending: &mut WorthQueryPendingApplicationPublication,
    after: &worth_relational::facade::snapshots::SnapshotHandle,
    commit_id: worth_relational::facade::history::CommitId,
) -> Result<CompositeCommitIdentity, WorthQueryProviderSessionFailure> {
    if pending.next_basis.observation().commit_id() != Some(commit_id) {
        return Err(failure(
            "application commit basis does not select the published commit",
        ));
    }
    let committed_product_publication = pending
        .application
        .as_ref()
        .expect("pending application retains its performed World publication")
        .committed_product_publication()
        .clone();
    if pending.receipt_basis_lease.is_none() {
        pending.receipt_basis_lease = Some(
            runtime
                .retain_component_basis(&pending.next_basis)
                .map_err(basis_retention_failure)?,
        );
    }
    publish_aggregate_projection(provider, runtime, pending, after);
    provider
        .graph
        .bind_truth_head_basis_in_runtime(runtime, &pending.next_basis)
        .map_err(bridge_head_failure)?;
    aftermath::seal(provider, runtime, pending, after)?;
    #[cfg(test)]
    if provider.take_panicked_pending_application_publication() {
        panic!("injected unwind after performed World publication reached terminal cutover");
    }
    let mut attempt = pending
        .attempt
        .take()
        .expect("pending application publication retains causality until final cutover");
    // The output's lineage record and its settlement row hold the consumed
    // outputs; the completed receipt evidence never does.
    let consumed_outputs = attempt.retain_consumed_outputs();
    let outstanding = attempt.take_outstanding_dispatch_reservation();
    let causality = attempt.publish_causality(provider, committed_product_publication);
    assert_eq!(
        causality.emitted_effect_count(),
        pending.emitted_effect_count
    );
    assert_eq!(causality.outcome_identity(), pending.outcome_identity);
    provider
        .receipt_basis_retention
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .retain(
            commit_id,
            pending
                .receipt_basis_lease
                .take()
                .expect("publication cutover retains its exact receipt basis"),
            outstanding.is_some(),
        );
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
        let admission = &mut pending.publication_admission;
        let registration_result = crate::domain_computation::primary_graph::output_lineage::invalidation::register_completed(
            owner,
            &completed,
            &consumed_outputs,
            runtime,
            after,
            Arc::clone(&identity),
            work_membership,
            recorded
                .as_ref()
                .and_then(|recorded| recorded.output_witness.as_ref())
                .and_then(|witness| witness.get()),
            admission,
        );
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
        (Some(superseded), displaced) => superseded.retire(displaced, owner),
        (None, Some(displaced)) => {
            drop(owner.retire_released([displaced], &mut owner.edit_admission()));
        }
        (None, None) => {}
    }
    let settled = completed
        .committed_product_publication()
        .composite_commit()
        .clone();
    provider
        .completed_commit_evidence
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .record(completed);
    Ok(settled)
}

fn publish_aggregate_projection(
    provider: &WorthQueryPrimaryGraphProvider,
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    pending: &mut WorthQueryPendingApplicationPublication,
    after: &worth_relational::facade::snapshots::SnapshotHandle,
) {
    if pending.aggregate_published {
        return;
    }
    let mut aggregates = provider
        .graph
        .aggregate_projections
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(before) = pending.before.as_ref() {
        aggregates.refresh_after_commit(runtime, before, after, pending.committed.patch());
    } else {
        aggregates.recover_after_commit(after.version_id());
    }
    pending.aggregate_published = true;
}
