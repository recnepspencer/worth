use super::*;
use crate::data::resource::ScheduledResourceRetry;
use crate::data::temporal::RetiredTemporalWake;

/// Actual explicit abort evidence. It does not claim a retry admission, timeout
/// rollback or resource cancellation, and carries no new cleanup capability.
pub struct SignalOwnedAsyncReleasedRetry {
    scheduled: ScheduledResourceRetry,
    retired: RetiredTemporalWake,
}

impl SignalOwnedAsyncReleasedRetry {
    pub fn scheduled(&self) -> &ScheduledResourceRetry {
        &self.scheduled
    }
    pub fn retired(&self) -> &RetiredTemporalWake {
        &self.retired
    }
}

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn abort_process_local_owned_async_retry(
        &self,
        owner: &mut SignalOwnedAsyncManagedAdmission,
    ) -> Result<(), SignalOwnedAsyncQueueDenial> {
        use SignalOwnedAsyncQueueDenial as Denial;
        let binding = &owner.origin;
        self.require_owned_async_binding(binding)
            .map_err(Denial::Service)?;
        if !owner.process_local || !owner.requires_queue || !owner.admitted_current_occurrence {
            return Err(Denial::Service(
                SignalConditionalServiceExecutionDenial::DefinitionMismatch,
            ));
        }
        if owner.cleanup.is_some() {
            return Err(Denial::CleanupPending);
        }
        let scheduled = owner.pending_retry.as_ref().ok_or(Denial::Service(
            SignalConditionalServiceExecutionDenial::DefinitionMismatch,
        ))?;
        if owner.retention.as_ref().is_some_and(|retention| {
            retention.previous.is_some()
                || retention.retry.as_ref() != Some(scheduled)
                || retention
                    .wakes
                    .iter()
                    .flatten()
                    .any(|wake| *wake != scheduled.backoff_wake_id())
        }) {
            return Err(Denial::CleanupPending);
        }
        let retired = self
            .with_owned_async_cell(&binding.basis, |cell, admission| {
                cell.abort_conditional_process_local_owned_async_retry(
                    admission,
                    &binding.basis,
                    &self.definition,
                    binding.node,
                    owner.admitted.handle(),
                    scheduled,
                )
            })
            .map_err(Denial::Service)?;
        let scheduled = owner
            .pending_retry
            .take()
            .expect("actual abort retained its schedule");
        owner.retention = None;
        owner.released_retry = Some(SignalOwnedAsyncReleasedRetry { scheduled, retired });
        Ok(())
    }
}
