use worth_query_execution::facade::{
    application_contribution::WorthQueryManagedComputationResourceDenial,
    domain_computation::{
        WorthQueryProviderSessionControlStopKind, WorthQueryProviderSessionDenialKind,
    },
    primary_graph::{
        WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationCommitDenialKind,
        WorthQueryConditionalClockObservationReceipt, WorthQueryConditionalExecutionCause,
        WorthQueryConditionalExecutionTerminal, WorthQueryConditionalReentryFailure,
        WorthQueryGranularInvalidationDeliveryBatch,
    },
};
use worth_query_installation::facade::WorthQueryClockCoordinate;
use worth_runtime_bridge::facade::{BridgeConditionalDenialKind, BridgeExecutionDenial};

use super::{WorthQueryTemporalCatchUpReport, WorthQueryTemporalCatchUpStatus};

mod denials;
pub(super) use denials::{
    classify_advancement_stop, classify_branch_stop, classify_clock_port_denial,
    classify_observation_failure, classify_work_ceiling_denial,
};

pub(super) struct CatchUpAccumulator<Clock> {
    steps_executed: usize,
    committed_occurrences: usize,
    already_committed_occurrences: usize,
    failed_occurrences: usize,
    indeterminate_occurrences: usize,
    last_observed_time: Option<WorthQueryClockCoordinate<Clock>>,
    last_sequence: Option<u64>,
    granular_invalidation_batches: Vec<WorthQueryGranularInvalidationDeliveryBatch>,
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
            granular_invalidation_batches: Vec::new(),
        }
    }

    pub(super) fn retain_invalidation_batch(
        &mut self,
        batch: WorthQueryGranularInvalidationDeliveryBatch,
    ) {
        if !batch.is_empty() {
            self.granular_invalidation_batches.push(batch);
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
        let mut saw_operation_refused = false;
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
                        Some(WorthQueryConditionalExecutionCause::AuthorizationRefused) => {
                            saw_auth = true;
                        }
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
                        &mut saw_operation_refused,
                        &mut saw_backpressure,
                        &mut saw_blocked,
                    );
                }
                WorthQueryConditionalExecutionTerminal::DeferredRetained => {
                    if is_backpressure_cause(provenance.cause().as_ref()) {
                        saw_backpressure = true;
                    } else {
                        saw_blocked = true;
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

        if receipt.retained_deferred_wake_count() > 0 && !saw_backpressure {
            saw_blocked = true;
        }
        if receipt.retained_failed_wake_count() > 0
            && !saw_cancelled
            && !saw_budget
            && !saw_auth
            && !saw_operation_refused
            && !saw_backpressure
        {
            saw_blocked = true;
        }

        let stop_status = if saw_cancelled {
            Some(WorthQueryTemporalCatchUpStatus::Cancelled)
        } else if saw_budget {
            Some(WorthQueryTemporalCatchUpStatus::BudgetExhausted)
        } else if saw_auth {
            Some(WorthQueryTemporalCatchUpStatus::AuthorizationRefused)
        } else if saw_operation_refused {
            Some(WorthQueryTemporalCatchUpStatus::OperationRefused)
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
            self.granular_invalidation_batches,
        )
    }
}

pub(super) fn empty_report<Clock>(
    status: WorthQueryTemporalCatchUpStatus,
    work_remaining: bool,
) -> WorthQueryTemporalCatchUpReport<Clock> {
    CatchUpAccumulator::new().finish(status, work_remaining)
}

fn is_backpressure_cause(cause: Option<&WorthQueryConditionalExecutionCause>) -> bool {
    matches!(
        cause,
        Some(
            WorthQueryConditionalExecutionCause::ActiveSnapshotCapacityExhausted { .. }
                | WorthQueryConditionalExecutionCause::RetentionCapacityExhausted
                | WorthQueryConditionalExecutionCause::CandidateCapacityExhausted { .. }
                | WorthQueryConditionalExecutionCause::PublishedSnapshotCapacityExhausted { .. }
                | WorthQueryConditionalExecutionCause::PatchPositionReservationContended
                | WorthQueryConditionalExecutionCause::RelationalDeferred(_)
                | WorthQueryConditionalExecutionCause::SourceCurrentnessRaced(_)
                | WorthQueryConditionalExecutionCause::RequiredPrerequisitePending(_)
        )
    )
}

fn classify_failure_cause(
    cause: Option<&WorthQueryConditionalExecutionCause>,
    saw_cancelled: &mut bool,
    saw_budget: &mut bool,
    saw_auth: &mut bool,
    saw_operation_refused: &mut bool,
    saw_backpressure: &mut bool,
    saw_blocked: &mut bool,
) {
    match cause {
        Some(WorthQueryConditionalExecutionCause::Cancelled) => *saw_cancelled = true,
        Some(WorthQueryConditionalExecutionCause::TimedOut) => *saw_budget = true,
        Some(WorthQueryConditionalExecutionCause::AuthorizationRefused)
        | Some(WorthQueryConditionalExecutionCause::Reentry(
            WorthQueryConditionalReentryFailure::Principal(_),
        )) => *saw_auth = true,
        Some(WorthQueryConditionalExecutionCause::TerminalFailure)
        | Some(WorthQueryConditionalExecutionCause::CustomInvariant(_))
        | Some(WorthQueryConditionalExecutionCause::Reentry(
            WorthQueryConditionalReentryFailure::Invocation(_),
        )) => *saw_operation_refused = true,
        Some(WorthQueryConditionalExecutionCause::Reentry(
            WorthQueryConditionalReentryFailure::ProductSelection(denial),
        )) => match classify_branch_stop(denial) {
            Some(WorthQueryTemporalCatchUpStatus::Cancelled) => *saw_cancelled = true,
            Some(WorthQueryTemporalCatchUpStatus::BudgetExhausted) => *saw_budget = true,
            Some(WorthQueryTemporalCatchUpStatus::Backpressured) => *saw_backpressure = true,
            _ => *saw_blocked = true,
        },
        Some(WorthQueryConditionalExecutionCause::Reentry(
            WorthQueryConditionalReentryFailure::ApplicationAttempt(kind),
        )) => match kind {
            WorthQueryApplicationAttemptDenialKind::Cancelled => *saw_cancelled = true,
            WorthQueryApplicationAttemptDenialKind::DeadlineExceeded
            | WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded
            | WorthQueryApplicationAttemptDenialKind::AllocationDenied
            | WorthQueryApplicationAttemptDenialKind::ExecutionRequest(
                WorthQueryProviderSessionDenialKind::ExecutionResource { .. },
            ) => *saw_budget = true,
            WorthQueryApplicationAttemptDenialKind::EffectExecution(denial) => {
                match classify_work_ceiling_denial(denial) {
                    WorthQueryTemporalCatchUpStatus::Cancelled => *saw_cancelled = true,
                    _ => *saw_budget = true,
                }
            }
            _ => *saw_blocked = true,
        },
        Some(WorthQueryConditionalExecutionCause::ApplicationCommitDenied(
            WorthQueryApplicationCommitDenialKind::ExecutionResource { .. },
        )) => *saw_budget = true,
        Some(WorthQueryConditionalExecutionCause::ApplicationCommitDenied(_)) => {
            *saw_operation_refused = true;
        }
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
        Some(WorthQueryConditionalExecutionCause::BridgeConditional(
            BridgeConditionalDenialKind::ExecutionDenied(BridgeExecutionDenial::Cancelled),
        )) => *saw_cancelled = true,
        Some(WorthQueryConditionalExecutionCause::BridgeConditional(
            BridgeConditionalDenialKind::ExecutionDenied(
                BridgeExecutionDenial::DeadlineElapsed
                | BridgeExecutionDenial::WorkCeiling
                | BridgeExecutionDenial::WorkCounterOverflow
                | BridgeExecutionDenial::MemoryExhausted(_)
                | BridgeExecutionDenial::ChargedBytesOverflow
                | BridgeExecutionDenial::AccountingUnavailable
                | BridgeExecutionDenial::WorkLimitExceedsParent
                | BridgeExecutionDenial::MemoryLimitExceedsParent
                | BridgeExecutionDenial::WorkerLimitExceedsParent,
            ),
        )) => *saw_budget = true,
        Some(WorthQueryConditionalExecutionCause::BridgeConditional(
            BridgeConditionalDenialKind::ConditionalRetentionCapacity,
        )) => *saw_backpressure = true,
        other if is_backpressure_cause(other) => *saw_backpressure = true,
        _ => *saw_blocked = true,
    }
}

#[cfg(test)]
mod tests;
