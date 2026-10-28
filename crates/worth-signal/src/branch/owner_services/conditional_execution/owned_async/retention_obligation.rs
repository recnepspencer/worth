use super::*;
use crate::data::resource::ScheduledResourceRetry;
use crate::data::temporal::TemporalWakeId;

/// One report's exact retired storage, never a growing predecessor chain.
pub(crate) struct SignalOwnedAsyncRetentionObligation {
    pub(crate) previous: Option<ResourceRequestHandle>,
    pub(crate) retry: Option<ScheduledResourceRetry>,
    pub(crate) wakes: [Option<TemporalWakeId>; 2],
}

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) fn release_pending_process_local_retention(
        &self,
        binding: &SignalOwnedAsyncSourceBinding,
        owner: &mut SignalOwnedAsyncManagedAdmission,
    ) -> Result<(), SignalOwnedAsyncQueueDenial> {
        if !owner.process_local {
            return Ok(());
        }
        if owner.cleanup.is_some() {
            return Err(SignalOwnedAsyncQueueDenial::CleanupPending);
        }
        let Some(retention) = &owner.retention else {
            return Ok(());
        };
        let released = self
            .with_owned_async_cell(&binding.basis, |cell, admission| {
                cell.release_conditional_process_local_predecessor(
                    admission,
                    &binding.basis,
                    &self.definition,
                    binding.node,
                    owner.admitted.handle(),
                    retention,
                )
            })
            .map_err(SignalOwnedAsyncQueueDenial::Service)?;
        owner.retention = None;
        if let Some(released) = released {
            owner.released_predecessor = Some(released);
        }
        Ok(())
    }
}
