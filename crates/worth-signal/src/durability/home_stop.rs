//! Quiescent authored close fences the actual owner without claiming owner cleanup.
use crate::branch::owner_services::{
    durable_publication::SignalAuthoredStopPublicationGate,
    lifecycle_state::SignalPreparedOwnerStopGate,
};

#[derive(Debug)]
pub enum SignalAuthoredHomeStopDenial {
    Owner(crate::branch::SignalOwnerStopDenial),
    Request(worth_execution::WorkCeilingDenial),
    Publications(Box<crate::branch::owner_services::SignalDurableAdvanceDenial>),
    PendingOriginals {
        originals: worth_execution::ExecutionArray<
            worth_foundational::facade::SignalBranchAdvanceIdentity,
        >,
    },
}
/// Dropping a preparation rolls its same owner's gate back while still holding it.
pub struct SignalPreparedAuthoredHomeStop<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    pub(crate) gate: SignalPreparedOwnerStopGate<'a>,
    pub(crate) _publication: SignalAuthoredStopPublicationGate<'a, D, I, T>,
}
impl<D, I, T> SignalPreparedAuthoredHomeStop<'_, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    /// Infallible final handoff immediately before the composing World's begin.
    pub fn commit(self) {
        self.gate.commit();
    }
}

/// Original stopped allocation retained through the Home's deferred reclaim.
/// This is neither Signal cleanup completion nor a Signal Closed receipt.
pub struct SignalRetainedAuthoredHomeAllocation {
    _owner: Box<dyn std::any::Any + Send>,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl<D, I, E, Ctx, T> super::SignalDurableOwner<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + 'static,
    Ctx: Send + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn retain_stopped_home_allocation(
        self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SignalRetainedAuthoredHomeAllocation, (Self, SignalAuthoredHomeStopDenial)> {
        let bytes = (std::mem::size_of::<Self>()
            + std::mem::size_of::<SignalRetainedAuthoredHomeAllocation>())
            as u64;
        let funding = match request.reserve_retained_memory(bytes) {
            Ok(funding) => funding,
            Err(cause) => return Err((self, SignalAuthoredHomeStopDenial::Request(cause))),
        };
        if let Err(cause) = self.commit_home_stop(request) {
            return Err((self, SignalAuthoredHomeStopDenial::Owner(cause)));
        }
        Ok(SignalRetainedAuthoredHomeAllocation {
            _owner: Box::new(self),
            _funding: funding,
        })
    }
}
