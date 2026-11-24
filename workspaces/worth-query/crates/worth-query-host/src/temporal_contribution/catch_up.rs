mod classification;

use std::num::NonZeroUsize;

use worth_execution::ExecutionRequest;
use worth_query_execution::facade::{
    application_contribution::WorthQueryAdvancementDenial,
    primary_graph::{
        WorthQueryConditionalClockHandle, WorthQueryConditionalClockObservationDenial,
        WorthQueryConditionalClockObservationFailure, WorthQueryConditionalClockObservationOutcome,
        WorthQueryPrimaryGraphApplicationRuntime,
    },
    product::{WorthQueryProductBranch, WorthQueryProductBranchAdmissionDenial},
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryClockCoordinate, WorthQueryNamedClock,
};

use super::clock::{WorthQueryDesktopClockAdvanceDenial, WorthQueryDesktopClockHandle};
use classification::{
    classify_advancement_stop, classify_branch_stop, classify_clock_port_denial,
    classify_observation_failure, classify_work_ceiling_denial, empty_report, CatchUpAccumulator,
};

/// Hard ceiling on per-call catch-up steps. Public Host APIs never accept
/// `usize::MAX` as a pump ceiling.
pub const MAX_TEMPORAL_CATCH_UP_STEPS: usize = 10_000;

/// Explicit caller-owned step policy for one catch-up pump call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryTemporalCatchUpPolicy {
    max_steps: NonZeroUsize,
}

impl WorthQueryTemporalCatchUpPolicy {
    /// Constructs a validated bounded catch-up policy (`1..=MAX_TEMPORAL_CATCH_UP_STEPS`).
    pub const fn bounded(max_steps: usize) -> Result<Self, WorthQueryTemporalCatchUpDenial> {
        let Some(non_zero) = NonZeroUsize::new(max_steps) else {
            return Err(WorthQueryTemporalCatchUpDenial::InvalidMaxSteps {
                requested: max_steps,
                maximum: MAX_TEMPORAL_CATCH_UP_STEPS,
            });
        };
        if max_steps > MAX_TEMPORAL_CATCH_UP_STEPS {
            return Err(WorthQueryTemporalCatchUpDenial::InvalidMaxSteps {
                requested: max_steps,
                maximum: MAX_TEMPORAL_CATCH_UP_STEPS,
            });
        }
        Ok(Self {
            max_steps: non_zero,
        })
    }

    pub const fn max_steps(self) -> NonZeroUsize {
        self.max_steps
    }
}

/// Non-advancing clock posture observed during a catch-up step.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryTemporalCatchUpClockStop {
    Stale,
    Reordered,
    Closed,
}

/// Typed terminal status of one bounded catch-up pump call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryTemporalCatchUpStatus {
    /// All due active occurrences at or before the observed clock coordinate
    /// have settled, with no remaining due or authoritative catch-up work.
    CaughtUp,
    /// Reached `bounds.max_steps()` while due work still remained; resume on
    /// the next caller pump without duplicating or skipping any occurrence.
    StepLimitReached,
    /// Caller's `ExecutionRequest` work or memory budget stopped catch-up;
    /// durable state remains consistent for resume under fresh budget.
    BudgetExhausted,
    /// Caller's `ExecutionRequest` or operation cancellation stopped catch-up.
    Cancelled,
    /// Fresh principal admission or operation authorization refused execution;
    /// durable state remains consistent until valid authority is restored.
    AuthorizationRefused,
    /// Owner-local snapshot or retention backpressure deferred execution.
    Backpressured,
    /// Installed clock source returned `Stale`, `Reordered`, or `Closed`.
    ClockStopped(WorthQueryTemporalCatchUpClockStop),
    /// A retained failure, indeterminate, or non-progressing decision blocked
    /// further catch-up steps until inspected or discharged.
    BlockedByRetainedDecision,
    /// The underlying clock observation port failed.
    ObservationFailed(WorthQueryConditionalClockObservationFailure),
}

/// Pre-loop or port-opening refusal before catch-up could observe the clock.
#[derive(Debug)]
pub enum WorthQueryTemporalCatchUpDenial {
    InvalidMaxSteps { requested: usize, maximum: usize },
    TemporalBindingNotInstalled,
    ClockAdvance(WorthQueryDesktopClockAdvanceDenial),
    Advancement(WorthQueryAdvancementDenial),
    BranchSelection(WorthQueryProductBranchAdmissionDenial),
    ClockPort(WorthQueryConditionalClockObservationDenial),
}

impl std::fmt::Display for WorthQueryTemporalCatchUpDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "temporal catch-up refused: {self:?}")
    }
}

impl std::error::Error for WorthQueryTemporalCatchUpDenial {}

/// O(1)-memory descriptive summary of one bounded caller-pumped catch-up run.
///
/// Descriptive only; grants no execution, admission, or discharge authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryTemporalCatchUpReport<Clock> {
    status: WorthQueryTemporalCatchUpStatus,
    steps_executed: usize,
    committed_occurrences: usize,
    already_committed_occurrences: usize,
    failed_occurrences: usize,
    indeterminate_occurrences: usize,
    work_remaining: bool,
    last_observed_time: Option<WorthQueryClockCoordinate<Clock>>,
    last_sequence: Option<u64>,
}

impl<Clock> WorthQueryTemporalCatchUpReport<Clock> {
    #[allow(clippy::too_many_arguments)]
    pub(super) const fn from_parts(
        status: WorthQueryTemporalCatchUpStatus,
        steps_executed: usize,
        committed_occurrences: usize,
        already_committed_occurrences: usize,
        failed_occurrences: usize,
        indeterminate_occurrences: usize,
        work_remaining: bool,
        last_observed_time: Option<WorthQueryClockCoordinate<Clock>>,
        last_sequence: Option<u64>,
    ) -> Self {
        Self {
            status,
            steps_executed,
            committed_occurrences,
            already_committed_occurrences,
            failed_occurrences,
            indeterminate_occurrences,
            work_remaining,
            last_observed_time,
            last_sequence,
        }
    }

    pub const fn status(&self) -> &WorthQueryTemporalCatchUpStatus {
        &self.status
    }

    pub const fn steps_executed(&self) -> usize {
        self.steps_executed
    }

    /// Number of due occurrences newly committed by this catch-up call.
    pub const fn fired(&self) -> usize {
        self.committed_occurrences
    }

    /// Number of already-committed occurrences idempotently skipped without re-firing.
    pub const fn skipped(&self) -> usize {
        self.already_committed_occurrences
    }

    /// Whether due or authoritative temporal work still remains for a subsequent pump.
    pub const fn remaining(&self) -> bool {
        self.work_remaining
    }

    pub const fn failed(&self) -> usize {
        self.failed_occurrences
    }

    pub const fn indeterminate(&self) -> usize {
        self.indeterminate_occurrences
    }

    pub fn last_observed_time(&self) -> Option<WorthQueryClockCoordinate<Clock>> {
        self.last_observed_time
            .as_ref()
            .map(|time| WorthQueryClockCoordinate::from_nanoseconds(time.nanoseconds()))
    }

    pub const fn last_sequence(&self) -> Option<u64> {
        self.last_sequence
    }
}

/// Advances a caller-owned [`WorthQueryDesktopClockHandle`] to `target_now` and
/// pumps due timed intents under `execution` and `policy`.
///
/// OS background wake while the desktop application is closed remains outside
/// WORTH; callers invoke this helper explicitly on reopen or foreground ticks
/// (or via [`crate::application_home::WorthQueryApplicationHome::catch_up_temporal`]).
/// Checks `execution.consult()` before advancing the clock or touching runtime
/// state so cancelled or budget-exhausted requests perform zero work.
#[allow(clippy::too_many_arguments)]
pub fn pump_temporal_catch_up_until<Schema, Node, Clock>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    branch: WorthQueryProductBranch,
    clock_handle: &WorthQueryConditionalClockHandle<Schema, Node, Clock>,
    desktop_clock: &WorthQueryDesktopClockHandle<Clock>,
    target_now: WorthQueryClockCoordinate<Clock>,
    execution: &ExecutionRequest<'_, '_>,
    policy: WorthQueryTemporalCatchUpPolicy,
) -> Result<WorthQueryTemporalCatchUpReport<Clock>, WorthQueryTemporalCatchUpDenial>
where
    Schema: ApplicationSchema + 'static,
    Clock: WorthQueryNamedClock,
{
    if let Err(denial) = execution.consult() {
        return Ok(empty_report(classify_work_ceiling_denial(&denial), true));
    }
    desktop_clock
        .advance_to(target_now)
        .map_err(WorthQueryTemporalCatchUpDenial::ClockAdvance)?;
    pump_temporal_catch_up(runtime, branch, clock_handle, execution, policy)
}

/// Pumps due timed intents from the currently installed named clock source
/// under bounded `execution` (`Work`/`Memory`/`Cancellation`) and `policy`
/// (`max_steps`).
///
/// Re-selects the fresh branch head on every step so multi-occurrence and
/// recurring catch-up observes committed successor revisions cleanly without
/// duplicating or skipping any occurrence.
pub fn pump_temporal_catch_up<Schema, Node, Clock>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    branch: WorthQueryProductBranch,
    clock_handle: &WorthQueryConditionalClockHandle<Schema, Node, Clock>,
    execution: &ExecutionRequest<'_, '_>,
    policy: WorthQueryTemporalCatchUpPolicy,
) -> Result<WorthQueryTemporalCatchUpReport<Clock>, WorthQueryTemporalCatchUpDenial>
where
    Schema: ApplicationSchema + 'static,
    Clock: WorthQueryNamedClock,
{
    let mut state = CatchUpAccumulator::<Clock>::new();
    if let Err(denial) = execution.consult() {
        return Ok(state.finish(classify_work_ceiling_denial(&denial), true));
    }

    for step in 0..policy.max_steps().get() {
        if let Err(denial) = execution.consult() {
            return Ok(state.finish(classify_work_ceiling_denial(&denial), true));
        }
        let selected =
            match runtime.with_host_advancement(|phase| runtime.on_branch(branch).select(&phase)) {
                Ok(Ok(selected)) => selected,
                Ok(Err(denial)) => {
                    if let Some(status) = classify_branch_stop(&denial) {
                        return Ok(state.finish(status, true));
                    }
                    return Err(WorthQueryTemporalCatchUpDenial::BranchSelection(denial));
                }
                Err(denial) => {
                    if let Some(status) = classify_advancement_stop(&denial) {
                        return Ok(state.finish(status, true));
                    }
                    return Err(WorthQueryTemporalCatchUpDenial::Advancement(denial));
                }
            };
        let mut port = match selected.conditional_clock(execution, clock_handle) {
            Ok(port) => port,
            Err(denial) => {
                if let Some(status) = classify_clock_port_denial(&denial) {
                    return Ok(state.finish(status, true));
                }
                return Err(WorthQueryTemporalCatchUpDenial::ClockPort(denial));
            }
        };
        match port.observe(execution) {
            WorthQueryConditionalClockObservationOutcome::Accepted(receipt)
            | WorthQueryConditionalClockObservationOutcome::Duplicate(receipt) => {
                let step_outcome = state.record_receipt(&receipt);
                let has_more_work = receipt.due_work_remaining()
                    || receipt.authoritative_work_remaining()
                    || receipt.retained_eligible_wake_count() > 0
                    || step_outcome.retry_product_stale;
                if let Some(stop_status) = step_outcome.stop_status {
                    return Ok(state.finish(stop_status, true));
                }
                if !has_more_work {
                    return Ok(state.finish(WorthQueryTemporalCatchUpStatus::CaughtUp, false));
                }
                if step + 1 >= policy.max_steps().get() {
                    return Ok(
                        state.finish(WorthQueryTemporalCatchUpStatus::StepLimitReached, true)
                    );
                }
            }
            WorthQueryConditionalClockObservationOutcome::Stale => {
                return Ok(state.finish(
                    WorthQueryTemporalCatchUpStatus::ClockStopped(
                        WorthQueryTemporalCatchUpClockStop::Stale,
                    ),
                    true,
                ));
            }
            WorthQueryConditionalClockObservationOutcome::Reordered => {
                return Ok(state.finish(
                    WorthQueryTemporalCatchUpStatus::ClockStopped(
                        WorthQueryTemporalCatchUpClockStop::Reordered,
                    ),
                    true,
                ));
            }
            WorthQueryConditionalClockObservationOutcome::Closed => {
                return Ok(state.finish(
                    WorthQueryTemporalCatchUpStatus::ClockStopped(
                        WorthQueryTemporalCatchUpClockStop::Closed,
                    ),
                    true,
                ));
            }
            WorthQueryConditionalClockObservationOutcome::Failed(failure) => {
                return Ok(state.finish(classify_observation_failure(failure), true));
            }
        }
    }

    Ok(state.finish(WorthQueryTemporalCatchUpStatus::StepLimitReached, true))
}
