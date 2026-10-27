use super::*;
use crate::branch::owner_services::conditional_execution::{
    SignalOwnedAsyncManagedHandoff, SignalOwnedAsyncManagedProgress, SignalOwnedAsyncManagedReport,
    SignalOwnedAsyncManagedWakeCleanup,
};

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn schedule_owned_async_retry_handoff(
        &mut self,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
    ) -> Result<SignalOwnedAsyncManagedHandoff, SignalConditionalServiceExecutionDenial> {
        self.admit_owned_async_source(node)?;
        self.require_owned_async_handle(node, handle)?;
        let capture = self.captures_telemetry();
        let tick = self.derived.temporal.clock_basis().current_tick();
        let (delay, next_attempt, digest, budget) =
            match self.derived.resource.retry_backoff_delay_for_handle(
                handle,
                tick,
                capture.then_some(&mut self.derived.telemetry.resource),
            ) {
                Ok(decision) => decision,
                Err(class) => {
                    let report = self.derived.resource.deny_resource_retry_schedule(
                        handle,
                        class,
                        capture.then_some(&mut self.derived.telemetry.resource),
                    );
                    return Ok(self.replacement_handoff(
                        SignalOwnedAsyncManagedProgress::Denied,
                        SignalOwnedAsyncManagedReport::RetrySchedule(report),
                        None,
                        None,
                        None,
                    ));
                }
            };
        let wake = self
            .derived
            .temporal
            .schedule_owned_wake(
                TemporalWakeOwner::ResourceNode(node.node()),
                TemporalCondition::after(delay.get())
                    .map_err(SignalConditionalServiceExecutionDenial::SlotAdmission)?,
                ClockTick::new(tick.get().saturating_add(delay.get())),
                capture.then_some(&mut self.derived.telemetry.temporal),
            )
            .map_err(SignalConditionalServiceExecutionDenial::SlotAdmission)?;
        let report = self.derived.resource.schedule_resource_retry(
            handle,
            ResourceRetryReason::TimedOut,
            wake.id(),
            next_attempt,
            delay,
            digest,
            budget,
            capture.then_some(&mut self.derived.telemetry.resource),
        );
        let admitted = report.scheduled_retry().is_some();
        let (cleanup, refusal) = if admitted {
            (None, None)
        } else {
            match self.retire_optional_wake(Some(wake.id()), TemporalWakeRetirementReason::Disposed)
            {
                Ok(()) => (None, None),
                Err(cause) => (
                    Some(SignalOwnedAsyncManagedWakeCleanup::Retirement {
                        wake: wake.id(),
                        reason: TemporalWakeRetirementReason::Disposed,
                    }),
                    Some(cause),
                ),
            }
        };
        Ok(self.replacement_handoff(
            if admitted {
                SignalOwnedAsyncManagedProgress::Scheduled
            } else {
                SignalOwnedAsyncManagedProgress::Denied
            },
            SignalOwnedAsyncManagedReport::RetrySchedule(report),
            None,
            cleanup,
            refusal,
        ))
    }
}
