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
    pub(super) fn advance_owned_async_retry_handoff(
        &mut self,
        node: ResourceNodeId,
        handle: ResourceRequestHandle,
        schedule: &SignalOwnedAsyncRetrySchedule,
        coordinate: u64,
    ) -> Result<SignalOwnedAsyncManagedHandoff, SignalConditionalServiceExecutionDenial> {
        self.admit_owned_async_source(node)?;
        self.require_owned_async_handle(node, handle)?;
        let scheduled = schedule
            .report()
            .scheduled_retry()
            .filter(|scheduled| scheduled.previous() == handle)
            .ok_or(SignalConditionalServiceExecutionDenial::DefinitionMismatch)?;
        let wake = scheduled.backoff_wake_id();
        if self.derived.resource.pending_retry_wake_for_handle(handle) != Some(wake) {
            return Err(SignalConditionalServiceExecutionDenial::DefinitionMismatch);
        }
        let ready = match self.advance_owned_async_wake_handoff(wake, coordinate)? {
            Ok(ready) => ready,
            Err(deferred) => return Ok(deferred),
        };
        let capture = self.captures_telemetry();
        let prepared = match self.derived.resource.prepare_scheduled_resource_retry(
            handle,
            &ready,
            capture.then_some(&mut self.derived.telemetry.resource),
        ) {
            Ok(prepared) => prepared,
            Err(report) => {
                return Ok(self.replacement_handoff(
                    SignalOwnedAsyncManagedProgress::Denied,
                    SignalOwnedAsyncManagedReport::Retry(report),
                    None,
                    Some(SignalOwnedAsyncManagedWakeCleanup::ReadyRetry {
                        previous: handle,
                        wake: ready,
                        reason: TemporalWakeRetirementReason::Consumed,
                    }),
                    None,
                ));
            }
        };
        let retired = match self.derived.temporal.retire_wake(
            wake,
            TemporalWakeRetirementReason::Consumed,
            capture.then_some(&mut self.derived.telemetry.temporal),
        ) {
            Ok(retired) => retired,
            Err(cause) => {
                return Ok(self.defer_owned_async_temporal_progress(
                    Some(SignalOwnedAsyncManagedWakeCleanup::ReadyRetry {
                        previous: handle,
                        wake: ready,
                        reason: TemporalWakeRetirementReason::Consumed,
                    }),
                    SignalConditionalServiceExecutionDenial::SlotAdmission(cause),
                ));
            }
        };
        let generation_started = prepared.previous().generation_started_tick();
        let tick = self.derived.temporal.clock_basis().current_tick();
        let timeout = match self.schedule_owned_async_timeout(node, generation_started) {
            Ok(timeout) => timeout,
            Err(cause) => {
                return Ok(self.defer_owned_async_temporal_progress(
                    Some(SignalOwnedAsyncManagedWakeCleanup::RetiredRetry {
                        previous: handle,
                        wake: retired,
                        reason: TemporalWakeRetirementReason::Consumed,
                    }),
                    cause,
                ));
            }
        };
        let report = self
            .derived
            .resource
            .admit_prepared_scheduled_resource_retry(
                prepared,
                ready,
                self.branch_id(),
                tick,
                timeout.map(|(_, admission)| admission),
                capture.then_some(&mut self.derived.telemetry.resource),
            );
        let admitted = report
            .admitted_retry()
            .expect("the prepared native retry kernel installs one admitted occurrence")
            .admitted_request();
        Ok(self.replacement_handoff(
            SignalOwnedAsyncManagedProgress::Replaced,
            SignalOwnedAsyncManagedReport::Retry(report),
            Some(admitted),
            None,
            None,
        ))
    }
}
