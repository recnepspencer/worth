//! Move-only recovery state for Query publication after Relational movement.
mod aggregate_projection;
mod custody;
use aggregate_projection::publish_aggregate_projection;
pub(in crate::domain_computation::primary_graph) use custody::WorthQueryPendingApplicationPublication;

use std::sync::Arc;

use crate::domain_computation::primary_graph::application_query::derived_view::{
    PredicatePreparationOwner, PredicatePreparationRetirement,
};
use worth_runtime_world::facade::CompositeCommitIdentity;

mod aftermath;
pub(crate) mod registry;
mod settlement;
mod stops;
pub(in crate::domain_computation::primary_graph) use registry::WorthQueryApplicationPublicationRecoveryReservation;
use registry::WorthQueryPendingApplicationIdempotency;
use stops::basis_retention_failure;
use stops::bridge_head_failure;
use stops::failure;
use stops::snapshot_capacity_failure;

use super::session_commit::snapshot_admission_failure;
use super::WorthQueryPrimaryGraphApplicationAttempt;
use super::WorthQueryPrimaryGraphCommittedApplication;
use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::WorthQueryProviderSessionFailure;
use crate::domain_computation::WorthQueryProviderSessionProtocolStage;
use crate::domain_computation::WorthQueryProviderSessionRecoveryPosture;

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

    pub fn has_pending_application_publication_for_test(&self) -> bool {
        self.pending_application_publication_count_for_test() != 0
    }

    pub fn pending_application_publication_count_for_test(&self) -> usize {
        self.pending_application_publications
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .active_slot_count()
    }

    pub(in crate::domain_computation::primary_graph) fn reserve_application_publication_recovery(
        &self,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        binding: Option<
            crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
        >,
    ) -> Result<WorthQueryApplicationPublicationRecoveryReservation, &'static str> {
        registry::WorthQueryPendingApplicationPublicationRegistry::reserve(
            &self.pending_application_publications,
            observation.lifecycle_incarnation(),
            binding,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn install_and_publish_application(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
        reservation: WorthQueryApplicationPublicationRecoveryReservation,
        pending: WorthQueryPendingApplicationPublication,
        predicate: &mut Option<PredicatePreparationOwner>,
        retired: &mut PredicatePreparationRetirement,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        let occurrence = pending.product_incarnation();
        reservation.install(pending, predicate);
        self.resume_pending_application_publication(execution, runtime, occurrence, retired)
    }

    pub(in crate::domain_computation::primary_graph) fn resume_pending_application_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        retired: &mut PredicatePreparationRetirement,
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
                let result = resume(execution, self, runtime, pending);
                if result.is_err() {
                    pending.release_before(runtime);
                }
                if pending.attempt.is_none() {
                    retired.retire(&mut pending.predicate);
                }
                result.map(|world| (world, Arc::clone(&pending.committed)))
            })
            .map_err(failure)?;
        let (_, committed) = result?;
        slot.complete();
        self.pending_application_publications
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove_exact(occurrence, &slot);
        self.retire_changed_history(runtime, &committed);
        Ok(())
    }
}

fn resume(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
    let result = publish_with_snapshot(execution, provider, runtime, pending, &after, commit_id);
    crate::relational_snapshot_release::release_query_snapshot(runtime, &after);
    result
}

fn publish_with_snapshot(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
    prepare_writer(execution, provider, runtime, pending, after)?;
    if pending.receipt_basis_lease.is_none() {
        pending.receipt_basis_lease = Some(
            runtime
                .retain_component_basis(&pending.next_basis)
                .map_err(basis_retention_failure)?,
        );
    }
    publish_aggregate_projection(execution, provider, runtime, pending, after)?;
    provider
        .graph
        .bind_truth_head_basis_in_runtime(runtime, &pending.next_basis)
        .map_err(bridge_head_failure)?;
    let dispatch_claims = usize::from(
        pending
            .attempt
            .as_ref()
            .expect("root writer")
            .dispatch_outbox()
            .is_some(),
    ) + pending
        .members
        .iter()
        .filter(|member| {
            member
                .pending
                .as_ref()
                .expect("member pending custody")
                .attempt
                .as_ref()
                .expect("member writer")
                .dispatch_outbox()
                .is_some()
        })
        .count();
    for member in &mut pending.members {
        prepare_writer(
            execution,
            provider,
            runtime,
            member.pending.as_mut().expect("performed writer"),
            after,
        )?;
    }
    if provider.take_panicked_pending_application_publication() {
        panic!("injected unwind after performed World publication reached terminal cutover");
    }
    provider
        .receipt_basis_retention
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .retain_group(
            commit_id,
            pending
                .receipt_basis_lease
                .take()
                .expect("one group receipt basis"),
            dispatch_claims,
        );
    for (ordinal, member) in pending.members.iter_mut().enumerate() {
        let leaf = member
            .pending
            .as_mut()
            .expect("group keeps member until shared cutover");
        let mut completed =
            settlement::finish_writer(execution, provider, runtime, leaf, after, commit_id, false);
        // A member has a real receipt, but the root owns the one product delivery
        // and history lifecycle. Native publication custody remains on its slot.
        completed.detach_coupled_delivery();
        pending
            .application
            .as_ref()
            .expect("root receipt")
            .complete_coupled_member(ordinal, completed);
    }
    let completed = settlement::finish_writer(
        execution, provider, runtime, pending, after, commit_id, true,
    );
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

fn prepare_writer(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    pending: &mut WorthQueryPendingApplicationPublication,
    after: &worth_relational::facade::snapshots::SnapshotHandle,
) -> Result<(), WorthQueryProviderSessionFailure> {
    // Custody is already registered in pending publication. Report the exact
    // read stop once; recovery keeps the evidence and can verify its source.
    if let Some(cause) = pending.source_read_stop.take() {
        return Err(WorthQueryProviderSessionFailure::checked_read_stopped(
            super::relational_execution_denial::relational_execution_kind(cause, None),
            WorthQueryProviderSessionProtocolStage::Commit,
            "postcommit source read stopped",
        )
        .with_recovery_posture(WorthQueryProviderSessionRecoveryPosture::RecoveryRequired));
    }
    aftermath::seal(execution, provider, runtime, pending, after)?;
    Ok(())
}

mod court_doors_for_test;
