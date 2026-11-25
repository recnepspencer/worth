use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_runtime_bridge::facade::BridgeConditionalExecutionView;

use super::{
    authoritative_clock_progression::AuthoritativeClockProgress, ErasedClockObservationReceipt,
    WorthQueryConditionalTruthBasis,
};
use crate::domain_computation::primary_graph::conditional_operation::{
    application_operation_reentry::WorthQueryTemporalReentryCounts,
    signal_decision_reentry::{
        evaluate_due_wake, reconsider_execution_denied_wake, retained_decision_counts,
        WorthQueryRetainedConditionalDecision,
    },
};

pub(super) fn retain_due(
    phase: &WorthQueryAdvancementPhase<'_>,

    accepted: worth_runtime_bridge::facade::BridgeManagedClockAcceptedObservation,
    bridge: &BridgeConditionalExecutionView<'_>,
    truth: &WorthQueryConditionalTruthBasis,
    authoritative_deliveries: &[worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery],
    signal_basis: &worth_runtime_bridge::facade::BridgeConditionalSignalBasisBinding,
    runtime_binding_identity: &str,
    runtime_capability_identity: u64,
    retained_wakes: &mut Vec<
            crate::domain_computation::primary_graph::conditional_operation::signal_decision_reentry::WorthQueryRetainedConditionalWake,
        >,
) -> ErasedClockObservationReceipt {
    for wake in retained_wakes.iter_mut() {
        let triggering_correspondence = authoritative_deliveries
            .iter()
            .map(|delivery| delivery.correspondence_receipt())
            .find(|receipt| {
                receipt.change_set().changes().iter().any(|change| {
                    change.relational_record_identity() == Some(wake.due.source_record_identity())
                })
            });
        reconsider_execution_denied_wake(
            phase
                .request_for_owner(truth.owner_identity())
                .expect("private conditional progression uses its admitted runtime"),
            bridge,
            wake,
            signal_basis,
            runtime_binding_identity,
            runtime_capability_identity,
            truth,
            triggering_correspondence,
        );
    }
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
                phase
                    .request_for_owner(truth.owner_identity())
                    .expect("private conditional progression uses its admitted runtime"),
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
        granular_invalidation_coverage:
            crate::domain_computation::primary_graph::WorthQueryGranularInvalidationCoverage::Exact,
    }
}

pub(super) fn complete_clock_receipt(
    mut receipt: ErasedClockObservationReceipt,
    counts: WorthQueryTemporalReentryCounts,
    authoritative: AuthoritativeClockProgress,
    released: super::pending_invalidations::WorthQueryReleasedGranularInvalidations,
    retained_wakes: &mut Vec<
            crate::domain_computation::primary_graph::conditional_operation::signal_decision_reentry::WorthQueryRetainedConditionalWake,
        >,
    totals: &mut super::operation_totals::WorthQueryTemporalOperationTotals,
) -> ErasedClockObservationReceipt {
    receipt.due_work_remaining |= authoritative.work_remaining || counts.rescheduled_due_now;
    receipt.authoritative_commit_count = authoritative.commit_count;
    receipt.authoritative_work_remaining = authoritative.work_remaining;
    receipt.granular_invalidations = released.deliveries;
    receipt.granular_invalidation_coverage = released.coverage;
    totals.accumulate(counts);
    receipt.execution_provenance =
        super::super::execution_provenance::execution_provenance(retained_wakes);
    retained_wakes.retain(|wake| {
        wake.continuation_identity.is_some()
            || !matches!(
                wake.decision,
                WorthQueryRetainedConditionalDecision::OperationCommitted(_, _)
                    | WorthQueryRetainedConditionalDecision::OperationAlreadyCommitted(_, _)
                    | WorthQueryRetainedConditionalDecision::OperationRetiredNoEffect(_)
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

pub(super) fn reset_retained_wakes_for_replay_for_test(
    retained_wakes: &mut [crate::domain_computation::primary_graph::conditional_operation::signal_decision_reentry::WorthQueryRetainedConditionalWake],
    totals: &mut super::operation_totals::WorthQueryTemporalOperationTotals,
) {
    use WorthQueryRetainedConditionalDecision as D;
    for wake in retained_wakes {
        let prev = std::mem::replace(&mut wake.decision, D::InterruptedDuringReentry);
        wake.decision = match prev {
            D::Eligible(e)
            | D::OperationRetryable(e, _)
            | D::OperationCommitRetryable(e, _)
            | D::OperationExecutionControlRetryable(e, _)
            | D::OperationBackpressured(e, _)
            | D::OperationControlStopped(e, _)
            | D::OperationTerminalFailure(e, _)
            | D::OperationNoEffect(e, _)
            | D::OperationProductStale(e, _)
            | D::OperationPreparationPending(e, _)
            | D::OperationPreparationRefused(e, _)
            | D::OperationIndeterminate(e, _) => D::Eligible(e),
            other => other,
        };
        wake.attempt = 0;
        wake.application_attempted = false;
        wake.continuation_identity = None;
    }
    *totals = Default::default();
}

pub(super) fn complete_continuation_receipt_for_test(
    counts: WorthQueryTemporalReentryCounts,
    retained_wakes: &mut Vec<crate::domain_computation::primary_graph::conditional_operation::signal_decision_reentry::WorthQueryRetainedConditionalWake>,
    totals: &mut super::operation_totals::WorthQueryTemporalOperationTotals,
) -> ErasedClockObservationReceipt {
    let decisions = retained_decision_counts(retained_wakes);
    let shell = ErasedClockObservationReceipt {
        sequence: 1,
        observed_coordinate: 5,
        due_wake_count: retained_wakes.len(),
        due_work_remaining: false,
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
        granular_invalidation_coverage:
            crate::domain_computation::primary_graph::WorthQueryGranularInvalidationCoverage::Exact,
    };
    complete_clock_receipt(
        shell,
        counts,
        AuthoritativeClockProgress {
            commit_count: 0,
            work_remaining: false,
            caught_up_to_latest: true,
        },
        super::pending_invalidations::WorthQueryReleasedGranularInvalidations {
            deliveries: Vec::new(),
            coverage: crate::domain_computation::primary_graph::WorthQueryGranularInvalidationCoverage::Exact,
        },
        retained_wakes,
        totals,
    )
}

pub(in crate::domain_computation::primary_graph::conditional_operation) fn prime_global_plateaus_for_test(
) {
    let _ = worth_query_execution_foundation::facade::domain_computation::application_outcome_identity::WorthQueryApplicationOutcomeIdentity::restore(100_000);
    crate::domain_computation::provider_session::protocol::prime_protocol_session_plateau_for_test(
    );
    crate::domain_computation::provider_session::execution_attempt_identity::prime_execution_attempt_plateau_for_test();
    crate::domain_computation::managed_run::prime_managed_logical_run_plateau_for_test();
    crate::domain_computation::artifact_owner::prime_artifact_occurrence_plateau_for_test();
    crate::domain_computation::provider_session::graph_provider::bounded_step::prime_provider_checkpoint_plateau_for_test();
    crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity::prime_plateau_for_test();
}
