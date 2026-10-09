use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_proof::TransitionOutcome;
use worth_relational::facade::publication::PatchStreamPosition;
use worth_runtime_bridge::facade::{
    BridgeConditionalSignalBasisBinding, BridgeSealedRuntimeAssembly,
    RelationalCommittedPatchRequest, TruthCommitIdentity,
};

use super::signal_decision_reentry::{
    reconsider_retained_wake, WorthQueryConditionalTruthBasis, WorthQueryRetainedConditionalWake,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::{
    CommitTouchInterest, CommitTouchesStop, TouchedCommits,
};

/// Commits selected from the canonical subscription's retained touches. Any
/// writer's commit on the branch is visible here, not only Query's own.
pub(super) struct WorthQueryRelevantAuthoritativeCommits {
    batch: TouchedCommits,
}

pub(super) struct WorthQueryDeliveredAuthoritativeCommits {
    pub(super) work_remaining: bool,
    pub(super) caught_up_to_latest: bool,
}

impl WorthQueryRelevantAuthoritativeCommits {
    pub(super) fn commit_count(&self) -> usize {
        self.batch.commits.len()
    }
}

pub(super) fn relevant_authoritative_commits<Schema>(
    runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
        Schema,
    >,
    branch: &worth_relational::facade::history::BranchId,
    commit_ceiling: Option<worth_relational::facade::history::CommitId>,
    cursor: Option<PatchStreamPosition>,
    maximum_commits: usize,
    interest: CommitTouchInterest<'_>,
) -> Result<WorthQueryRelevantAuthoritativeCommits, CommitTouchesStop> {
    runtime
        .primary_provider
        .graph
        .source_owner
        .invalidation_owner
        .touched_commits_after(branch, cursor, commit_ceiling, maximum_commits, interest)
        .map(|batch| WorthQueryRelevantAuthoritativeCommits { batch })
}

/// Delivers each relevant commit to Signal-hosted nodes and reconsiders the
/// wakes it touches. The cursor passes a commit only once its invalidations
/// are owed on `pending`, so they survive a later failure in this batch.
#[allow(clippy::too_many_arguments)]
pub(super) fn deliver_authoritative_commits(
    phase: &WorthQueryAdvancementPhase<'_>,

    bridge: &BridgeSealedRuntimeAssembly,
    signal_basis: &BridgeConditionalSignalBasisBinding,
    cursor: &mut Option<PatchStreamPosition>,
    commits: WorthQueryRelevantAuthoritativeCommits,
    wakes: &mut [WorthQueryRetainedConditionalWake],
    query_binding_identity: &str,
    query_capability_identity: u64,
    truth: &WorthQueryConditionalTruthBasis,
    pending: &mut super::lifecycle::WorthQueryPendingGranularInvalidations,
) -> Result<WorthQueryDeliveredAuthoritativeCommits, String> {
    let TouchedCommits {
        commits,
        next_cursor,
        work_remaining,
        caught_up_to_latest,
    } = commits.batch;
    for touched in &commits {
        // Bridge deliveries only feed Signal-hosted nodes. Which wakes are
        // reconsidered is decided by the owner's retained touches.
        let delivered = deliver_commit_dependencies(
            phase
                .request_for_owner(truth.owner_identity())
                .expect("private conditional progression uses its admitted runtime"),
            bridge,
            signal_basis,
            touched.commit,
            truth,
            pending.direct(),
        )?;
        for wake in wakes.iter_mut().filter(|wake| {
            touched.touches(&super::lifecycle::source_entity(
                wake.due.source_record_identity(),
            ))
        }) {
            let record = wake.due.source_record_identity();
            let triggering_correspondence = delivered
                .iter()
                .map(|delivery| delivery.correspondence_receipt())
                .find(|receipt| {
                    receipt
                        .change_set()
                        .changes()
                        .iter()
                        .any(|change| change.relational_record_identity() == Some(record))
                });
            reconsider_retained_wake(
                phase
                    .request_for_owner(truth.owner_identity())
                    .expect("private conditional progression uses its admitted runtime"),
                bridge,
                wake,
                signal_basis,
                query_binding_identity,
                query_capability_identity,
                truth,
                triggering_correspondence,
            );
        }
        pending.owe(promote_performed_signal_deliveries(delivered, wakes));
        *cursor = Some(touched.position);
    }
    *cursor = next_cursor;
    Ok(WorthQueryDeliveredAuthoritativeCommits {
        work_remaining,
        caught_up_to_latest,
    })
}

pub(super) fn promote_performed_signal_deliveries(
    deliveries: Vec<worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery>,
    wakes: &mut [WorthQueryRetainedConditionalWake],
) -> Vec<worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery> {
    deliveries
        .into_iter()
        .map(|delivery| {
            if delivery.performed_signal().is_some() {
                return delivery;
            }
            for wake in wakes.iter_mut() {
                let Some(evidence) = retained_decision_evidence_mut(&mut wake.decision) else {
                    continue;
                };
                match worth_runtime_bridge::facade::assemble_granular_invalidation_delivery(
                    delivery.correspondence_receipt(),
                    Some(evidence),
                ) {
                    Ok(performed) => return performed,
                    Err(_) => continue,
                }
            }
            delivery
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn reconsider_retained_wakes_for_deliveries(
    phase: &WorthQueryAdvancementPhase<'_>,

    bridge: &BridgeSealedRuntimeAssembly,
    deliveries: &[worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery],
    wakes: &mut [WorthQueryRetainedConditionalWake],
    signal_basis: &BridgeConditionalSignalBasisBinding,
    query_binding_identity: &str,
    query_capability_identity: u64,
    truth: &WorthQueryConditionalTruthBasis,
) {
    for delivery in deliveries {
        let receipt = delivery.correspondence_receipt();
        for wake in wakes.iter_mut().filter(|wake| {
            receipt.change_set().changes().iter().any(|change| {
                change.relational_record_identity() == Some(wake.due.source_record_identity())
            })
        }) {
            reconsider_retained_wake(
                phase
                    .request_for_owner(truth.owner_identity())
                    .expect("private conditional progression uses its admitted runtime"),
                bridge,
                wake,
                signal_basis,
                query_binding_identity,
                query_capability_identity,
                truth,
                Some(receipt),
            );
        }
    }
}

fn retained_decision_evidence_mut(
    decision: &mut super::signal_decision_reentry::WorthQueryRetainedConditionalDecision,
) -> Option<&mut worth_runtime_bridge::facade::BridgeConditionalDecisionEvidence> {
    use super::signal_decision_reentry::WorthQueryRetainedConditionalDecision as Decision;
    match decision {
        Decision::Eligible(evidence)
        | Decision::Suppressed(evidence)
        | Decision::Deferred(evidence)
        | Decision::OperationRetryable(evidence, _)
        | Decision::OperationBackpressured(evidence, _)
        | Decision::OperationControlStopped(evidence, _)
        | Decision::OperationTerminalFailure(evidence, _)
        | Decision::OperationSettlementDeferred(evidence, _)
        | Decision::OperationProductUnpublished(evidence, _)
        | Decision::OperationProductStale(evidence, _)
        | Decision::OperationNoEffect(evidence, _)
        | Decision::OperationIndeterminate(evidence, _)
        | Decision::OperationCommitted(evidence)
        | Decision::OperationAlreadyCommitted(evidence) => Some(evidence),
        Decision::OperationCommitRetryable(evidence, _)
        | Decision::OperationExecutionControlRetryable(evidence, _) => Some(evidence),
        Decision::OperationSettlementExecutionDenied(evidence, _, _) => Some(evidence),
        Decision::Failed(_) => None,
    }
}

fn deliver_commit_dependencies(
    execution: worth_execution::ExecutionRequest<'_, '_>,

    bridge: &BridgeSealedRuntimeAssembly,
    signal_basis: &BridgeConditionalSignalBasisBinding,
    commit: worth_relational::facade::history::CommitId,
    truth: &WorthQueryConditionalTruthBasis,
    preperformed_deliveries: &[worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery],
) -> Result<Vec<worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery>, String> {
    let mut granular_invalidations = Vec::new();
    let lowering = signal_basis.installed_lowering_ref();
    let commit_identity = TruthCommitIdentity::from_relational_commit_id(commit.0);
    for dependency_ordinal in 0..lowering.contract().dependency_count() {
        if preperformed_deliveries.iter().any(|delivery| {
            let change = delivery.correspondence_receipt().change_set();
            change.commit_identity() == &commit_identity
                && change.dependency().dependency_ordinal() == dependency_ordinal
        }) {
            continue;
        }
        let outcome = bridge
            .deliver_authoritative_change(
                execution,
                signal_basis,
                dependency_ordinal,
                RelationalCommittedPatchRequest::at_snapshot(
                    commit_identity.clone(),
                    truth.snapshot().clone(),
                ),
            )
            .map_err(|denial| denial.detail().to_string())?;
        let receipt = match outcome {
            TransitionOutcome::Success(receipt) => receipt,
            TransitionOutcome::Denied(denial) => {
                return Err(format!(
                    "Bridge denied conditional authoritative change: {denial:?}"
                ))
            }
            TransitionOutcome::Failed(failure) => {
                return Err(format!(
                    "Bridge failed conditional authoritative change: {failure:?}"
                ))
            }
            TransitionOutcome::Deferred(_) => {
                return Err("Bridge deferred conditional authoritative change".to_string())
            }
            TransitionOutcome::Stale(_) => {
                return Err("Bridge found stale conditional authoritative change".to_string())
            }
            TransitionOutcome::RebindRequired(posture) => {
                return Err(format!(
                    "Bridge requires conditional correspondence rebinding: {posture:?}"
                ))
            }
        };
        if !receipt.change_set().changes().is_empty() {
            granular_invalidations.push(
                worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery::direct(&receipt),
            );
        }
    }
    Ok(granular_invalidations)
}
