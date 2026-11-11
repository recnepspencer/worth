//! Original-runtime entry to reversible owner-local stop.
use super::SignalRuntime;
use crate::branch::{
    PreparedInitialEmptySignalStop, SignalInitialEmptyStopRefused, SignalOwnerStopDenial as Denial,
};
use worth_execution::ExecutionRequest;
impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    /// Try-only metadata preparation. No allocation, owner cleanup or user Drop.
    /// Requires an already sealed, producer-initial empty owner.
    pub fn try_prepare_initial_empty_stop(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        PreparedInitialEmptySignalStop<D, I, E, Ctx, T>,
        SignalInitialEmptyStopRefused<D, I, E, Ctx, T>,
    > {
        match self.owner_services.try_prepare_initial_empty_stop(request) {
            Ok(()) => Ok(PreparedInitialEmptySignalStop::new(self)),
            Err(reason) => Err(SignalInitialEmptyStopRefused::new(self, reason)),
        }
    }
    pub(crate) fn try_resume_initial_empty_owner(&self) -> Result<(), Denial> {
        self.owner_services.try_resume_initial_empty_stop()
    }
    pub(crate) fn try_commit_initial_empty_owner_stop(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        self.owner_services.try_commit_initial_empty_stop(request)
    }
}
