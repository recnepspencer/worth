use super::*;
use crate::branch::owner_services::conditional_execution::{
    SignalOwnedAsyncManagedHandoff, SignalOwnedAsyncManagedProgress,
    SignalOwnedAsyncManagedQueueHandoff, SignalOwnedAsyncManagedReport,
    SignalOwnedAsyncManagedWakeCleanup, SignalOwnedAsyncQueueDenial,
};
use crate::data::resource::AdmittedResourceRequest;

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) fn replacement_handoff(
        &mut self,
        progress: SignalOwnedAsyncManagedProgress,
        report: SignalOwnedAsyncManagedReport,
        admitted: Option<AdmittedResourceRequest>,
        cleanup: Option<SignalOwnedAsyncManagedWakeCleanup>,
        mut refusal: Option<SignalConditionalServiceExecutionDenial>,
    ) -> SignalOwnedAsyncManagedHandoff {
        let in_flight =
            admitted.and_then(|admitted| match self.require_in_flight(admitted.handle()) {
                Ok(actual) => Some(actual),
                Err(cause) => {
                    if refusal.is_none() {
                        refusal = Some(cause);
                    }
                    None
                }
            });
        SignalOwnedAsyncManagedHandoff {
            progress,
            report,
            admitted,
            in_flight,
            queue: if progress == SignalOwnedAsyncManagedProgress::Replaced {
                SignalOwnedAsyncManagedQueueHandoff::Replace(None)
            } else {
                SignalOwnedAsyncManagedQueueHandoff::Preserve
            },
            cleanup,
            refusal: refusal.map(SignalOwnedAsyncQueueDenial::Service),
        }
    }

    pub(super) fn defer_owned_async_temporal_progress(
        &mut self,
        cleanup: Option<SignalOwnedAsyncManagedWakeCleanup>,
        cause: SignalConditionalServiceExecutionDenial,
    ) -> SignalOwnedAsyncManagedHandoff {
        self.replacement_handoff(
            SignalOwnedAsyncManagedProgress::TemporalDeferred,
            SignalOwnedAsyncManagedReport::TemporalDeferred(self.derived.temporal.clock_basis()),
            None,
            cleanup,
            Some(cause),
        )
    }
}

/// The old unmanaged result cannot carry a retained temporal/cleanup handoff.
/// Managed callers consume the complete handoff instead of this legacy mapping.
pub(super) fn legacy_replacement_refusal(
    refusal: Option<SignalOwnedAsyncQueueDenial>,
) -> Result<(), SignalConditionalServiceExecutionDenial> {
    match refusal {
        None => Ok(()),
        Some(SignalOwnedAsyncQueueDenial::Service(cause)) => Err(cause),
        Some(_) => unreachable!("the unmanaged algorithm performs no managed queue action"),
    }
}
