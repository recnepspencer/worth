use super::*;
use crate::branch::owner_services::conditional_execution::SignalOwnedAsyncManagedWakeCleanup;

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) fn retire_optional_wake(
        &mut self,
        wake: Option<crate::data::temporal::TemporalWakeId>,
        reason: TemporalWakeRetirementReason,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        let Some(wake) = wake else {
            return Ok(());
        };
        let capture = self.captures_telemetry();
        self.derived
            .temporal
            .retire_wake(
                wake,
                reason,
                capture.then_some(&mut self.derived.telemetry.temporal),
            )
            .map(|_| ())
            .map_err(SignalConditionalServiceExecutionDenial::SlotAdmission)
    }

    pub(crate) fn finish_managed_owned_async_wake_cleanup(
        &mut self,
        node: ResourceNodeId,
        cleanup: &mut SignalOwnedAsyncManagedWakeCleanup,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        self.admit_owned_async_source(node)?;
        let capture = self.captures_telemetry();
        if let SignalOwnedAsyncManagedWakeCleanup::ReadyRetry {
            previous,
            wake,
            reason,
        } = cleanup
        {
            self.require_owned_async_handle(node, *previous)?;
            let previous = *previous;
            let reason = *reason;
            let retired = self
                .derived
                .temporal
                .retire_wake(
                    wake.id(),
                    reason,
                    capture.then_some(&mut self.derived.telemetry.temporal),
                )
                .map_err(SignalConditionalServiceExecutionDenial::SlotAdmission)?;
            // Retain the actual completed retirement before any later handoff.
            *cleanup = SignalOwnedAsyncManagedWakeCleanup::RetiredRetry {
                previous,
                wake: retired,
                reason,
            };
        }
        match cleanup {
            SignalOwnedAsyncManagedWakeCleanup::RetiredRetry {
                previous,
                wake,
                reason,
            } => {
                self.require_owned_async_handle(node, *previous)?;
                if wake.reason() != *reason {
                    return Err(SignalConditionalServiceExecutionDenial::DefinitionMismatch);
                }
                // An independently installed successor is never cleared here.
                if self
                    .derived
                    .resource
                    .pending_retry_wake_for_handle(*previous)
                    == Some(wake.id())
                    && self.derived.resource.pending_retry_wake_for_node(node) == Some(wake.id())
                {
                    self.derived.resource.clear_pending_retry_for_node(node);
                }
            }
            SignalOwnedAsyncManagedWakeCleanup::Retirement { wake, reason } => {
                self.retire_optional_wake(Some(*wake), *reason)?;
            }
            SignalOwnedAsyncManagedWakeCleanup::ReadyTimeout { wake, reason } => {
                self.retire_optional_wake(Some(wake.id()), *reason)?;
            }
            SignalOwnedAsyncManagedWakeCleanup::ReadyRetry { .. } => {
                unreachable!("a successful ready-wake cleanup retains its actual retirement")
            }
        }
        Ok(())
    }
}
