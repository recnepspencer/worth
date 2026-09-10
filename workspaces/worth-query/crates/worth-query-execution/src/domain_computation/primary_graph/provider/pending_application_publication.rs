//! Move-only recovery state for Query publication after Relational movement.

pub(super) mod registry;
pub(in crate::domain_computation::primary_graph) use registry::WorthQueryApplicationPublicationRecoveryReservation;

use super::{
    session_commit::{provider_failure, snapshot_admission_failure},
    WorthQueryPrimaryGraphApplicationAttempt, WorthQueryPrimaryGraphCommittedApplication,
    WorthQueryPrimaryGraphProvider,
};
use crate::domain_computation::{
    WorthQueryProviderSessionFailure, WorthQueryProviderSessionProtocolStage,
    WorthQueryProviderSessionRecoveryPosture,
};

pub(in crate::domain_computation::primary_graph) struct WorthQueryPendingApplicationPublication {
    product_incarnation: worth_runtime_world::facade::ProductBranchIncarnation,
    attempt: Option<WorthQueryPrimaryGraphApplicationAttempt>,
    branch: worth_relational::facade::history::BranchId,
    before: Option<worth_relational::facade::snapshots::SnapshotHandle>,
    next_basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    committed: worth_relational::facade::transactions::CommitResult,
    application: Option<WorthQueryPrimaryGraphCommittedApplication>,
    emitted_effect_count: usize,
    outcome_identity: crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity,
    aggregate_published: bool,
    receipt_basis_lease:
        Option<worth_relational::facade::branch::RelationalBranchRetentionLease>,
}

impl WorthQueryPendingApplicationPublication {
    pub(in crate::domain_computation::primary_graph) fn new(
        attempt: WorthQueryPrimaryGraphApplicationAttempt,
        branch: worth_relational::facade::history::BranchId,
        before: worth_relational::facade::snapshots::SnapshotHandle,
        next_basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        committed: worth_relational::facade::transactions::CommitResult,
        application: WorthQueryPrimaryGraphCommittedApplication,
        emitted_effect_count: usize,
        outcome_identity: crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity,
    ) -> Self {
        let product_incarnation = application
            .committed_product_publication()
            .product_incarnation();
        Self {
            product_incarnation,
            attempt: Some(attempt),
            branch,
            before: Some(before),
            next_basis,
            committed,
            application: Some(application),
            emitted_effect_count,
            outcome_identity,
            aggregate_published: false,
            receipt_basis_lease: None,
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
}

impl WorthQueryPrimaryGraphProvider {
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
        if result.is_ok() {
            slot.complete();
            self.pending_application_publications
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .remove_exact(occurrence, &slot);
        }
        result
    }
}

fn resume(
    provider: &WorthQueryPrimaryGraphProvider,
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    pending: &mut WorthQueryPendingApplicationPublication,
) -> Result<(), WorthQueryProviderSessionFailure> {
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
) -> Result<(), WorthQueryProviderSessionFailure> {
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
    publish_indexes(provider, runtime, &pending.branch, commit_id)?;
    provider
        .graph
        .bind_truth_head_basis_in_runtime(runtime, &pending.next_basis)
        .map_err(bridge_head_failure)?;
    #[cfg(test)]
    if provider.take_panicked_pending_application_publication() {
        panic!("injected unwind after performed World publication reached terminal cutover");
    }
    let attempt = pending
        .attempt
        .take()
        .expect("pending application publication retains causality until final cutover");
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
        );
    pending.release_before(runtime);
    provider
        .completed_commit_evidence
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .record(
            pending
                .application
                .take()
                .expect("completed publication retains exact commit evidence until cutover"),
        );
    Ok(())
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

fn publish_indexes(
    provider: &WorthQueryPrimaryGraphProvider,
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    branch: &worth_relational::facade::history::BranchId,
    commit_id: worth_relational::facade::history::CommitId,
) -> Result<(), WorthQueryProviderSessionFailure> {
    if provider.take_failed_index_publication() {
        return Err(failure(
            "injected primary index publication failure after authoritative commit",
        ));
    }
    let indexes = runtime.index_authority().build_for_commit(
        worth_relational::facade::indexes::DerivedIndexBuildRequest {
            source_commit_id: commit_id,
            branch_id: branch.clone(),
            index_ids: provider.graph.primary_index_ids.to_vec(),
        },
    );
    if indexes.failed_indexes.is_empty() {
        Ok(())
    } else {
        Err(failure(
            "application commit succeeded but primary indexes did not refresh",
        ))
    }
}

fn failure(detail: &'static str) -> WorthQueryProviderSessionFailure {
    provider_failure(WorthQueryProviderSessionProtocolStage::Commit, detail)
        .with_recovery_posture(WorthQueryProviderSessionRecoveryPosture::RecoveryRequired)
}

fn snapshot_capacity_failure(detail: &'static str) -> WorthQueryProviderSessionFailure {
    WorthQueryProviderSessionFailure::new(
        crate::domain_computation::WorthQueryProviderSessionDenialKind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots: 0,
        },
        WorthQueryProviderSessionProtocolStage::Commit,
        detail,
        crate::domain_computation::WorthQueryProviderSessionProtocolCounters::default(),
    )
    .with_recovery_posture(WorthQueryProviderSessionRecoveryPosture::RecoveryRequired)
}

fn basis_retention_failure(
    denial: worth_relational::facade::branch::RelationalBranchBasisDenial,
) -> WorthQueryProviderSessionFailure {
    let kind = match denial {
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionCapacityExhausted => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::RetentionCapacityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionIdentityExhausted => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::RetentionIdentityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::SnapshotIdentityExhausted => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::SnapshotIdentityExhausted
        }
        _ => crate::domain_computation::WorthQueryProviderSessionDenialKind::ProviderRejected,
    };
    WorthQueryProviderSessionFailure::new(
        kind,
        WorthQueryProviderSessionProtocolStage::Commit,
        format!("application commit basis could not be retained: {denial:?}"),
        crate::domain_computation::WorthQueryProviderSessionProtocolCounters::default(),
    )
    .with_recovery_posture(WorthQueryProviderSessionRecoveryPosture::RecoveryRequired)
}

fn bridge_head_failure(
    denial: worth_relational::facade::branch::RelationalBranchBasisDenial,
) -> WorthQueryProviderSessionFailure {
    let kind = match denial {
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionCapacityExhausted => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::RetentionCapacityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionIdentityExhausted => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::RetentionIdentityExhausted
        }
        worth_relational::facade::branch::RelationalBranchBasisDenial::SnapshotIdentityExhausted => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::SnapshotIdentityExhausted
        }
        _ => crate::domain_computation::WorthQueryProviderSessionDenialKind::ProviderRejected,
    };
    WorthQueryProviderSessionFailure::new(
        kind,
        WorthQueryProviderSessionProtocolStage::Commit,
        format!("application commit succeeded but Bridge head binding failed: {denial:?}"),
        crate::domain_computation::WorthQueryProviderSessionProtocolCounters::default(),
    )
    .with_recovery_posture(WorthQueryProviderSessionRecoveryPosture::RecoveryRequired)
}
