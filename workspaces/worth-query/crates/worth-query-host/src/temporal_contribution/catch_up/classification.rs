use worth_execution::{MapKernelStop, WorkCeilingDenial};
use worth_query_execution::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
        WorthQueryManagedComputationResourceDenial,
    },
    domain_computation::{
        WorthQueryProviderSessionControlStopKind, WorthQueryProviderSessionDenialKind,
    },
    primary_graph::{
        WorthQueryApplicationCommitDenialKind, WorthQueryConditionalClockObservationDenial,
        WorthQueryConditionalClockObservationDenialKind,
        WorthQueryConditionalClockObservationFailure,
        WorthQueryConditionalClockObservationFailureKind,
        WorthQueryConditionalClockObservationReceipt, WorthQueryConditionalExecutionCause,
        WorthQueryConditionalExecutionTerminal, WorthQueryConditionalReentryFailure,
        WorthQueryConditionalRuntimeInstallationDenialKind,
    },
    product::WorthQueryProductBranchAdmissionDenial,
};
use worth_query_installation::facade::WorthQueryClockCoordinate;

use super::{WorthQueryTemporalCatchUpReport, WorthQueryTemporalCatchUpStatus};

pub(super) struct CatchUpAccumulator<Clock> {
    steps_executed: usize,
    committed_occurrences: usize,
    already_committed_occurrences: usize,
    failed_occurrences: usize,
    indeterminate_occurrences: usize,
    last_observed_time: Option<WorthQueryClockCoordinate<Clock>>,
    last_sequence: Option<u64>,
}

pub(super) struct StepClassification {
    pub(super) stop_status: Option<WorthQueryTemporalCatchUpStatus>,
    pub(super) retry_product_stale: bool,
}

impl<Clock> CatchUpAccumulator<Clock> {
    pub(super) const fn new() -> Self {
        Self {
            steps_executed: 0,
            committed_occurrences: 0,
            already_committed_occurrences: 0,
            failed_occurrences: 0,
            indeterminate_occurrences: 0,
            last_observed_time: None,
            last_sequence: None,
        }
    }

    pub(super) fn record_receipt(
        &mut self,
        receipt: &WorthQueryConditionalClockObservationReceipt<Clock>,
    ) -> StepClassification {
        self.steps_executed = self.steps_executed.saturating_add(1);
        self.last_observed_time = Some(receipt.observed_time());
        self.last_sequence = Some(receipt.sequence());

        let mut step_committed = 0usize;
        let mut step_product_stale = 0usize;
        let mut saw_cancelled = false;
        let mut saw_budget = false;
        let mut saw_auth = false;
        let mut saw_backpressure = receipt.snapshot_capacity_backpressure().is_some()
            || receipt.retention_capacity_backpressure();
        let mut saw_blocked = false;

        for provenance in receipt.execution_provenance() {
            match provenance.terminal() {
                WorthQueryConditionalExecutionTerminal::Committed => {
                    step_committed = step_committed.saturating_add(1);
                    self.committed_occurrences = self.committed_occurrences.saturating_add(1);
                }
                WorthQueryConditionalExecutionTerminal::AlreadyCommitted => {
                    self.already_committed_occurrences =
                        self.already_committed_occurrences.saturating_add(1);
                }
                WorthQueryConditionalExecutionTerminal::ProductStale => {
                    step_product_stale = step_product_stale.saturating_add(1);
                }
                WorthQueryConditionalExecutionTerminal::Indeterminate => {
                    self.indeterminate_occurrences =
                        self.indeterminate_occurrences.saturating_add(1);
                    saw_blocked = true;
                }
                WorthQueryConditionalExecutionTerminal::ControlStopped => {
                    self.failed_occurrences = self.failed_occurrences.saturating_add(1);
                    match provenance.cause() {
                        Some(WorthQueryConditionalExecutionCause::TimedOut) => saw_budget = true,
                        _ => saw_cancelled = true,
                    }
                }
                WorthQueryConditionalExecutionTerminal::Retryable
                | WorthQueryConditionalExecutionTerminal::Failed => {
                    self.failed_occurrences = self.failed_occurrences.saturating_add(1);
                    classify_failure_cause(
                        provenance.cause().as_ref(),
                        &mut saw_cancelled,
                        &mut saw_budget,
                        &mut saw_auth,
                        &mut saw_backpressure,
                        &mut saw_blocked,
                    );
                }
                WorthQueryConditionalExecutionTerminal::DeferredRetained => {
                    if matches!(
                        provenance.cause(),
                        Some(
                            WorthQueryConditionalExecutionCause::ActiveSnapshotCapacityExhausted { .. }
                                | WorthQueryConditionalExecutionCause::RetentionCapacityExhausted
                        )
                    ) {
                        saw_backpressure = true;
                    }
                }
                WorthQueryConditionalExecutionTerminal::ProductUnpublished
                | WorthQueryConditionalExecutionTerminal::NoEffect => {
                    self.failed_occurrences = self.failed_occurrences.saturating_add(1);
                    saw_blocked = true;
                }
                WorthQueryConditionalExecutionTerminal::EligibleRetained
                | WorthQueryConditionalExecutionTerminal::SuppressedRetained => {}
            }
        }

        let stop_status = if saw_cancelled {
            Some(WorthQueryTemporalCatchUpStatus::Cancelled)
        } else if saw_budget {
            Some(WorthQueryTemporalCatchUpStatus::BudgetExhausted)
        } else if saw_auth {
            Some(WorthQueryTemporalCatchUpStatus::AuthorizationRefused)
        } else if saw_backpressure {
            Some(WorthQueryTemporalCatchUpStatus::Backpressured)
        } else if saw_blocked || (step_product_stale > 0 && step_committed == 0) {
            Some(WorthQueryTemporalCatchUpStatus::BlockedByRetainedDecision)
        } else {
            None
        };

        StepClassification {
            stop_status,
            retry_product_stale: step_product_stale > 0 && step_committed > 0,
        }
    }

    pub(super) fn finish(
        self,
        status: WorthQueryTemporalCatchUpStatus,
        work_remaining: bool,
    ) -> WorthQueryTemporalCatchUpReport<Clock> {
        WorthQueryTemporalCatchUpReport::from_parts(
            status,
            self.steps_executed,
            self.committed_occurrences,
            self.already_committed_occurrences,
            self.failed_occurrences,
            self.indeterminate_occurrences,
            work_remaining,
            self.last_observed_time,
            self.last_sequence,
        )
    }
}

pub(super) fn empty_report<Clock>(
    status: WorthQueryTemporalCatchUpStatus,
    work_remaining: bool,
) -> WorthQueryTemporalCatchUpReport<Clock> {
    CatchUpAccumulator::new().finish(status, work_remaining)
}

fn classify_failure_cause(
    cause: Option<&WorthQueryConditionalExecutionCause>,
    saw_cancelled: &mut bool,
    saw_budget: &mut bool,
    saw_auth: &mut bool,
    saw_backpressure: &mut bool,
    saw_blocked: &mut bool,
) {
    match cause {
        Some(WorthQueryConditionalExecutionCause::Cancelled) => *saw_cancelled = true,
        Some(WorthQueryConditionalExecutionCause::TimedOut) => *saw_budget = true,
        Some(WorthQueryConditionalExecutionCause::Reentry(
            WorthQueryConditionalReentryFailure::Principal(_),
        ))
        | Some(WorthQueryConditionalExecutionCause::TerminalFailure) => {
            *saw_auth = true;
        }
        Some(WorthQueryConditionalExecutionCause::ApplicationCommitDenied(
            WorthQueryApplicationCommitDenialKind::ExecutionResource { .. },
        )) => *saw_budget = true,
        Some(WorthQueryConditionalExecutionCause::ApplicationExecutionDenied {
            cause: Err(WorthQueryProviderSessionControlStopKind::Cancelled),
            ..
        }) => *saw_cancelled = true,
        Some(WorthQueryConditionalExecutionCause::ApplicationExecutionDenied {
            cause: Err(WorthQueryProviderSessionControlStopKind::TimedOut),
            ..
        }) => *saw_budget = true,
        Some(WorthQueryConditionalExecutionCause::ApplicationExecutionDenied {
            cause:
                Ok(WorthQueryProviderSessionDenialKind::ExecutionResource {
                    denial:
                        WorthQueryManagedComputationResourceDenial::WorkExhausted
                        | WorthQueryManagedComputationResourceDenial::MemoryLimit { .. }
                        | WorthQueryManagedComputationResourceDenial::PolicyMemoryLimit
                        | WorthQueryManagedComputationResourceDenial::WorkLimit
                        | WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted
                        | WorthQueryManagedComputationResourceDenial::ScratchCapacityExceeded,
                    ..
                }),
            ..
        }) => *saw_budget = true,
        Some(
            WorthQueryConditionalExecutionCause::ActiveSnapshotCapacityExhausted { .. }
            | WorthQueryConditionalExecutionCause::RetentionCapacityExhausted,
        ) => *saw_backpressure = true,
        _ => *saw_blocked = true,
    }
}

pub(super) fn classify_work_ceiling_denial(
    denial: &WorkCeilingDenial,
) -> WorthQueryTemporalCatchUpStatus {
    match denial {
        WorkCeilingDenial::Stopped(MapKernelStop::Cancelled) => {
            WorthQueryTemporalCatchUpStatus::Cancelled
        }
        WorkCeilingDenial::Stopped(
            MapKernelStop::WorkCeiling
            | MapKernelStop::WorkCounterOverflow
            | MapKernelStop::DeadlineElapsed
            | MapKernelStop::NestedStopped
            | MapKernelStop::AccountingUnavailable,
        )
        | WorkCeilingDenial::Admission(_)
        | WorkCeilingDenial::Panicked => WorthQueryTemporalCatchUpStatus::BudgetExhausted,
    }
}

pub(super) fn classify_advancement_stop(
    denial: &WorthQueryAdvancementDenial,
) -> Option<WorthQueryTemporalCatchUpStatus> {
    match denial {
        WorthQueryAdvancementDenial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled,
        ) => Some(WorthQueryTemporalCatchUpStatus::Cancelled),
        WorthQueryAdvancementDenial::Interrupted(
            WorthQueryManagedComputationInterruption::DeadlineExceeded,
        )
        | WorthQueryAdvancementDenial::Resource(_) => {
            Some(WorthQueryTemporalCatchUpStatus::BudgetExhausted)
        }
        WorthQueryAdvancementDenial::NestedOpening
        | WorthQueryAdvancementDenial::NestedStopped
        | WorthQueryAdvancementDenial::Panicked => None,
    }
}

pub(super) fn classify_branch_stop(
    denial: &WorthQueryProductBranchAdmissionDenial,
) -> Option<WorthQueryTemporalCatchUpStatus> {
    match denial {
        WorthQueryProductBranchAdmissionDenial::Stopped(stop) => match stop {
            Ok(_) => Some(WorthQueryTemporalCatchUpStatus::BudgetExhausted),
            Err(WorthQueryProviderSessionControlStopKind::Cancelled) => {
                Some(WorthQueryTemporalCatchUpStatus::Cancelled)
            }
            Err(WorthQueryProviderSessionControlStopKind::TimedOut) => {
                Some(WorthQueryTemporalCatchUpStatus::BudgetExhausted)
            }
        },
        WorthQueryProductBranchAdmissionDenial::Refused(_) => None,
    }
}

pub(super) fn classify_clock_port_denial(
    denial: &WorthQueryConditionalClockObservationDenial,
) -> Option<WorthQueryTemporalCatchUpStatus> {
    let WorthQueryConditionalClockObservationDenialKind::ProductAdmission(kind) = denial.kind()
    else {
        return None;
    };
    match kind {
        WorthQueryConditionalRuntimeInstallationDenialKind::ReconstructionPrincipal => {
            Some(WorthQueryTemporalCatchUpStatus::AuthorizationRefused)
        }
        WorthQueryConditionalRuntimeInstallationDenialKind::ReadExecution(Err(
            WorthQueryProviderSessionControlStopKind::Cancelled,
        )) => Some(WorthQueryTemporalCatchUpStatus::Cancelled),
        WorthQueryConditionalRuntimeInstallationDenialKind::ReadExecution(Err(
            WorthQueryProviderSessionControlStopKind::TimedOut,
        ))
        | WorthQueryConditionalRuntimeInstallationDenialKind::ReadExecution(Ok(
            WorthQueryProviderSessionDenialKind::ExecutionResource { .. },
        )) => Some(WorthQueryTemporalCatchUpStatus::BudgetExhausted),
        WorthQueryConditionalRuntimeInstallationDenialKind::ActiveSnapshotCapacityExhausted {
            ..
        }
        | WorthQueryConditionalRuntimeInstallationDenialKind::RetentionCapacityExhausted => {
            Some(WorthQueryTemporalCatchUpStatus::Backpressured)
        }
        _ => None,
    }
}

pub(super) fn classify_observation_failure(
    failure: WorthQueryConditionalClockObservationFailure,
) -> WorthQueryTemporalCatchUpStatus {
    match failure.kind() {
        WorthQueryConditionalClockObservationFailureKind::ExecutionRequest(ref denial) => {
            classify_advancement_stop(denial)
                .unwrap_or(WorthQueryTemporalCatchUpStatus::ObservationFailed(failure))
        }
        WorthQueryConditionalClockObservationFailureKind::ActiveSnapshotCapacityExhausted {
            ..
        }
        | WorthQueryConditionalClockObservationFailureKind::RetentionCapacityExhausted => {
            WorthQueryTemporalCatchUpStatus::Backpressured
        }
        _ => WorthQueryTemporalCatchUpStatus::ObservationFailed(failure),
    }
}
