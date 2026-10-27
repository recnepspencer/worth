use super::*;
use crate::branch::owner_services::conditional_execution::SignalOwnedAsyncManagedHandoff;
use crate::data::temporal::{
    ClockAdvanceRequest, ClockDomain, ReadyTemporalWake, TemporalClockAdvanceSummary,
};

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn advance_owned_async_clock(
        &mut self,
        node: ResourceNodeId,
        coordinate: u64,
    ) -> Result<TemporalClockAdvanceSummary, SignalConditionalServiceExecutionDenial> {
        self.admit_owned_async_source(node)?;
        let before = self.derived.temporal.frontier_snapshot();
        let validated = self
            .derived
            .temporal
            .validate_clock_advance(ClockAdvanceRequest::new(
                ClockDomain::MonotonicExecution,
                ClockTick::new(coordinate),
            ))
            .map_err(SignalConditionalServiceExecutionDenial::SlotAdmission)?;
        self.derived.temporal.apply_clock_advance(validated);
        let after = self.derived.temporal.frontier_snapshot();
        Ok(TemporalClockAdvanceSummary::new(validated, before, after))
    }

    pub(super) fn advance_owned_async_wake_handoff(
        &mut self,
        wake: crate::data::temporal::TemporalWakeId,
        coordinate: u64,
    ) -> Result<
        Result<ReadyTemporalWake, SignalOwnedAsyncManagedHandoff>,
        SignalConditionalServiceExecutionDenial,
    > {
        let validated = self
            .derived
            .temporal
            .validate_clock_advance(ClockAdvanceRequest::new(
                ClockDomain::MonotonicExecution,
                ClockTick::new(coordinate),
            ))
            .map_err(SignalConditionalServiceExecutionDenial::SlotAdmission)?;
        self.derived.temporal.apply_clock_advance(validated);
        let capture = self.captures_telemetry();
        match self.derived.temporal.promote_wake_ready(
            wake,
            capture.then_some(&mut self.derived.telemetry.temporal),
        ) {
            Ok(ready) => Ok(Ok(ready)),
            Err(cause) => Ok(Err(self.defer_owned_async_temporal_progress(
                None,
                SignalConditionalServiceExecutionDenial::SlotAdmission(cause),
            ))),
        }
    }
}
