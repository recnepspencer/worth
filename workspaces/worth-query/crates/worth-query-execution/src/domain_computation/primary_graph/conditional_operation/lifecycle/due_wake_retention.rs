use worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly;

use super::{
    authoritative_clock_progression::AuthoritativeClockProgress, ErasedClockObservationReceipt,
    WorthQueryConditionalTruthBasis,
};
use crate::domain_computation::primary_graph::conditional_operation::{
    application_operation_reentry::WorthQueryTemporalReentryCounts,
    signal_decision_reentry::{
        evaluate_due_wake, retained_decision_counts, WorthQueryRetainedConditionalDecision,
    },
};

pub(super) fn retain_due(
    accepted: worth_runtime_bridge::facade::BridgeManagedClockAcceptedObservation,
    bridge: &BridgeSealedRuntimeAssembly,
    truth: &WorthQueryConditionalTruthBasis,
    authoritative_deliveries: &[worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery],
    signal_basis: &worth_runtime_bridge::facade::BridgeConditionalSignalBasisBinding,
    runtime_binding_identity: &str,
    runtime_capability_identity: u64,
    retained_wakes: &mut Vec<
            crate::domain_computation::primary_graph::conditional_operation::signal_decision_reentry::WorthQueryRetainedConditionalWake,
        >,
) -> ErasedClockObservationReceipt {
    let sequence = accepted.sequence();
    let observed_coordinate = accepted.observed_coordinate();
    let due = accepted.into_due();
    let due_wake_count = due.wakes().len();
    let due_work_remaining = due.due_work_remaining();
    let evaluated = due
        .into_wakes()
        .into_iter()
        .map(|wake| {
            let triggering_correspondence = authoritative_deliveries
                .iter()
                .map(|delivery| delivery.correspondence_receipt())
                .find(|receipt| {
                    receipt.change_set().changes().iter().any(|change| {
                        change.relational_record_identity() == Some(wake.source_record_identity())
                    })
                });
            evaluate_due_wake(
                bridge,
                wake,
                signal_basis,
                runtime_binding_identity,
                runtime_capability_identity,
                truth,
                triggering_correspondence,
            )
        })
        .collect::<Vec<_>>();
    retained_wakes.extend(evaluated);
    let decisions = retained_decision_counts(retained_wakes);
    ErasedClockObservationReceipt {
        sequence,
        observed_coordinate,
        due_wake_count,
        due_work_remaining,
        authoritative_commit_count: 0,
        authoritative_work_remaining: false,
        retained_due_wake_count: retained_wakes.len(),
        retained_eligible_wake_count: decisions.eligible,
        retained_suppressed_wake_count: decisions.suppressed,
        retained_deferred_wake_count: decisions.deferred,
        retained_failed_wake_count: decisions.failed,
        committed_operation_count: 0,
        already_committed_operation_count: 0,
        failed_operation_count: 0,
        indeterminate_operation_count: 0,
        snapshot_capacity_backpressure: None,
        retention_capacity_backpressure: false,
        execution_provenance: Vec::new(),
        granular_invalidations: Vec::new(),
    }
}

pub(super) fn complete_clock_receipt(
    mut receipt: ErasedClockObservationReceipt,
    counts: WorthQueryTemporalReentryCounts,
    authoritative: AuthoritativeClockProgress,
    retained_wakes: &mut Vec<
            crate::domain_computation::primary_graph::conditional_operation::signal_decision_reentry::WorthQueryRetainedConditionalWake,
        >,
    totals: &mut super::operation_totals::WorthQueryTemporalOperationTotals,
) -> ErasedClockObservationReceipt {
    receipt.due_work_remaining |= authoritative.work_remaining;
    receipt.authoritative_commit_count = authoritative.commit_count;
    receipt.authoritative_work_remaining = authoritative.work_remaining;
    receipt.granular_invalidations = authoritative.granular_invalidations;
    totals.accumulate(counts);
    receipt.execution_provenance =
        super::super::execution_provenance::execution_provenance(retained_wakes);
    retained_wakes.retain(|wake| {
        !matches!(
            wake.decision,
            WorthQueryRetainedConditionalDecision::OperationCommitted(_)
                | WorthQueryRetainedConditionalDecision::OperationAlreadyCommitted(_)
        )
    });
    let decisions = retained_decision_counts(retained_wakes);
    receipt.retained_due_wake_count = retained_wakes.len();
    receipt.retained_eligible_wake_count = decisions.eligible;
    receipt.retained_suppressed_wake_count = decisions.suppressed;
    receipt.retained_deferred_wake_count = decisions.deferred;
    receipt.retained_failed_wake_count = decisions.failed;
    totals.apply_to(&mut receipt);
    receipt.snapshot_capacity_backpressure = counts.snapshot_capacity_backpressure;
    receipt.retention_capacity_backpressure = counts.retention_capacity_backpressure;
    receipt
}
