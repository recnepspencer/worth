//! The actual canonical root transferred once from its consumed programme runtime.
use super::SignalOwner;
use crate::branch::owner_services::{
    lifecycle_state::SignalPreparedOwnerStopGate, SignalBranchBasisPort,
};
use crate::branch::SignalOwnerStopDenial;
use std::sync::{Arc, Weak};
use worth_execution::ExecutionRequest;
pub(crate) struct SignalSealedOwnerRoot<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) owner: Arc<SignalOwner<D, I, T>>,
}
impl<D, I, T> SignalSealedOwnerRoot<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn downgrade_owner(&self) -> Weak<SignalOwner<D, I, T>> {
        Arc::downgrade(&self.owner)
    }
    pub(crate) fn basis_port(&self) -> SignalBranchBasisPort<D, I, T> {
        SignalBranchBasisPort::new(self.downgrade_owner(), self.owner.runtime_instance_id)
    }
    pub(crate) fn prepare_home_stop(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalPreparedOwnerStopGate<'_>, SignalOwnerStopDenial> {
        self.owner.try_prepare_authored_home_stop(request)
    }
    pub(crate) fn commit_home_stop(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), SignalOwnerStopDenial> {
        self.owner.try_commit_owner_stop(request)
    }
}
impl<D, I, T> Drop for SignalSealedOwnerRoot<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn drop(&mut self) {
        let _ = self.owner.request_close();
    }
}
