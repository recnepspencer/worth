use std::num::NonZeroUsize;

use worth_query_execution::facade::{
    application_contribution::WorthQueryAdvancementDenial,
    primary_graph::{
        WorthQueryConditionalClockObservationDenial, WorthQueryConditionalClockObservationFailure,
        WorthQueryGranularInvalidationDeliveryBatch,
    },
    product::WorthQueryProductBranchAdmissionDenial,
};
use worth_query_installation::facade::WorthQueryClockCoordinate;

use super::super::clock::WorthQueryDesktopClockAdvanceDenial;

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
    /// have settled, with no remaining due, deferred, failed, or authoritative
    /// catch-up work.
    CaughtUp,
    /// Reached `policy.max_steps()` while due work still remained; resume on
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
    /// Operation invocation, due-field verification, or domain/commit validation
    /// refused execution terminally (distinct from authorization refusal).
    OperationRefused,
    /// Owner-local snapshot, retention, or provider capacity backpressure
    /// deferred execution.
    Backpressured,
    /// Installed clock source returned `Stale`, `Reordered`, or `Closed`.
    ClockStopped(WorthQueryTemporalCatchUpClockStop),
    /// A retained indeterminate or non-progressing decision blocked further
    /// catch-up steps until inspected or discharged.
    BlockedByRetainedDecision,
    /// The underlying clock observation port failed.
    ObservationFailed(WorthQueryConditionalClockObservationFailure),
    /// A subsequent step after earlier progress or granular invalidation
    /// deliveries was refused before clock observation; earlier progress and
    /// owed granular invalidation batches are preserved on the returned report.
    StepRefused(Box<WorthQueryTemporalCatchUpDenial>),
}

/// Pre-loop or port-opening refusal before catch-up could observe the clock.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryTemporalCatchUpDenial {
    InvalidMaxSteps { requested: usize, maximum: usize },
    TemporalBindingNotInstalled,
    ForeignClockHandle,
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

/// Descriptive summary of one bounded caller-pumped catch-up run, carrying any
/// owed granular invalidation delivery batches produced by accepted/duplicate
/// clock observations when not consumed through a streaming callback.
///
/// Descriptive only; grants no execution, admission, or discharge authority.
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
    granular_invalidation_batches: Vec<WorthQueryGranularInvalidationDeliveryBatch>,
}

impl<Clock> std::fmt::Debug for WorthQueryTemporalCatchUpReport<Clock> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryTemporalCatchUpReport")
            .field("status", &self.status)
            .field("steps_executed", &self.steps_executed)
            .field("fired", &self.committed_occurrences)
            .field("skipped", &self.already_committed_occurrences)
            .field("failed", &self.failed_occurrences)
            .field("indeterminate", &self.indeterminate_occurrences)
            .field("remaining", &self.work_remaining)
            .field(
                "last_observed_time_ns",
                &self.last_observed_time.as_ref().map(|t| t.nanoseconds()),
            )
            .field("last_sequence", &self.last_sequence)
            .field(
                "granular_invalidation_batch_count",
                &self.granular_invalidation_batches.len(),
            )
            .finish()
    }
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
        granular_invalidation_batches: Vec<WorthQueryGranularInvalidationDeliveryBatch>,
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
            granular_invalidation_batches,
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

    /// Whether due, deferred, failed, or authoritative temporal work still
    /// remains for a subsequent pump.
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

    /// Inspects non-empty granular invalidation delivery batches retained from
    /// accepted/duplicate observations during this catch-up call.
    pub fn granular_invalidation_batches(&self) -> &[WorthQueryGranularInvalidationDeliveryBatch] {
        &self.granular_invalidation_batches
    }

    /// Takes ownership of all non-empty granular invalidation delivery batches
    /// produced during this catch-up call so downstream watches receive every
    /// owed delivery. Retained batches hold Home installation custody and must
    /// be consumed or dropped before closing a Directory Home (or use the
    /// `_observing` streaming variants to retain zero custody on the report).
    pub fn take_granular_invalidation_batches(
        &mut self,
    ) -> Vec<WorthQueryGranularInvalidationDeliveryBatch> {
        std::mem::take(&mut self.granular_invalidation_batches)
    }
}
