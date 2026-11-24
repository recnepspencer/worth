use worth_store_io_scheduler::QueueExecutionOutcome;

use crate::physical_runtime::work::{PhysicalExecutorDispatch, PhysicalWorkSettlement};
use crate::physical_runtime::{
    PhysicalSignalSettlementOutcome, PhysicalWorkSchedulerPosture, SettledPhysicalWork,
};

use super::PhysicalRecoveryCoordination;

#[cfg(feature = "certification-test-authority")]
mod certification;
#[cfg(feature = "certification-test-authority")]
pub(super) use certification::{
    settle_with_certification, PhysicalRecoverySettlementCertificationStage,
};

pub(super) fn settle(
    coordination: &PhysicalRecoveryCoordination,
    dispatch: PhysicalExecutorDispatch,
) -> PhysicalSignalSettlementOutcome {
    settle_with(coordination, dispatch, |coordination, settled| {
        coordination.signal.record_settlement(settled)
    })
}

fn settle_with(
    coordination: &PhysicalRecoveryCoordination,
    dispatch: PhysicalExecutorDispatch,
    record_signal: impl FnOnce(
        &PhysicalRecoveryCoordination,
        &SettledPhysicalWork,
    ) -> PhysicalSignalSettlementOutcome,
) -> PhysicalSignalSettlementOutcome {
    let settlement = PhysicalWorkSettlement::settle(dispatch);
    let (settled, _revocation, effect_activity, _residency) = settlement.into_parts();
    coordination.submission.record_settled_causality(&settled);
    let signal = record_signal(coordination, &settled);
    coordination
        .submission
        .record_derived_completion_causality(settled.intent().identity(), signal);
    if signal == PhysicalSignalSettlementOutcome::DerivedStateUnavailable {
        coordination
            .submission
            .record_derived_reconciliation_deferred(settled.intent().identity());
    }
    let awaits_abandonment = settled.retry_is_physically_safe();
    drop(settled);
    drop(effect_activity);
    if awaits_abandonment {
        coordination.submission.await_abandonment_idle();
    }
    signal
}

pub(super) const fn signal_completion_is_terminal(
    outcome: PhysicalSignalSettlementOutcome,
) -> bool {
    matches!(
        outcome,
        PhysicalSignalSettlementOutcome::Committed
            | PhysicalSignalSettlementOutcome::ReconciledFromPhysicalTruth
    )
}

pub(super) fn scheduler_posture(outcome: &QueueExecutionOutcome) -> PhysicalWorkSchedulerPosture {
    if matches!(outcome, QueueExecutionOutcome::Executed(_)) {
        PhysicalWorkSchedulerPosture::Executed
    } else {
        PhysicalWorkSchedulerPosture::RejectedAfterEffect
    }
}
