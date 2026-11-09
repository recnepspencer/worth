//! Quiescent authored close fences the actual owner without claiming owner cleanup.
use crate::branch::owner_services::{
    durable_publication::SignalAuthoredStopPublicationGate,
    lifecycle_state::SignalPreparedOwnerStopGate,
};

#[derive(Debug)]
pub enum SignalAuthoredHomeStopDenial {
    Owner(crate::branch::SignalOwnerStopDenial),
    Medium(Box<super::SignalDurableMediumDenial>),
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
    _owner: std::sync::Mutex<Box<dyn std::any::Any + Send>>,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
pub(super) struct SignalHomeRetirementBacking<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    slot: Box<Option<super::SignalDurableOwner<D, I, E, Ctx, T>>>,
    funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl<D, I, E, Ctx, T> super::SignalDurableOwner<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + 'static,
    Ctx: Send + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Fund the actual original allocation before the composing World seals.
    pub fn preflight_home_retirement(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), SignalAuthoredHomeStopDenial> {
        let mut prepared = self.home_retirement.try_lock().map_err(|cause| {
            SignalAuthoredHomeStopDenial::Owner(match cause {
                std::sync::TryLockError::WouldBlock => crate::branch::SignalOwnerStopDenial::Busy,
                std::sync::TryLockError::Poisoned(_) => {
                    crate::branch::SignalOwnerStopDenial::Poisoned
                }
            })
        })?;
        if prepared.is_none() {
            let funding = request
                .reserve_retained_memory(std::mem::size_of::<Option<Self>>() as u64)
                .map_err(SignalAuthoredHomeStopDenial::Request)?;
            *prepared = Some(SignalHomeRetirementBacking {
                slot: Box::new(None),
                funding,
            });
        }
        Ok(())
    }
    pub fn retain_stopped_home_allocation(
        mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SignalRetainedAuthoredHomeAllocation, (Self, SignalAuthoredHomeStopDenial)> {
        if let Err(cause) = self.preflight_home_retirement(request) {
            return Err((self, cause));
        }
        let mut prepared = self
            .home_retirement
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .expect("this original owner's retirement allocation is prepared");
        if let Err(cause) = self.commit_home_stop(request) {
            *self
                .home_retirement
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(prepared);
            return Err((self, SignalAuthoredHomeStopDenial::Owner(cause)));
        }
        if let Err(cause) = self.publication.retire_medium_for_home(request) {
            *self
                .home_retirement
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(prepared);
            return Err((self, SignalAuthoredHomeStopDenial::Medium(Box::new(cause))));
        }
        *prepared.slot = Some(self);
        Ok(SignalRetainedAuthoredHomeAllocation {
            _owner: std::sync::Mutex::new(prepared.slot),
            _funding: prepared.funding,
        })
    }
}
