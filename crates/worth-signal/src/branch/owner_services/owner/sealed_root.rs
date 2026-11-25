//! The actual canonical root transferred once from its consumed programme runtime.
use crate::branch::owner_services::owner_backing::{StrongSignalOwner, WeakSignalOwner};
use crate::branch::owner_services::{
    lifecycle_state::SignalPreparedOwnerStopGate, SignalBranchBasisPort,
};
use crate::branch::SignalOwnerStopDenial;
use worth_execution::ExecutionRequest;
pub(crate) struct SignalSealedOwnerRoot<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) owner: StrongSignalOwner<D, I, T>,
}
impl<D, I, T> SignalSealedOwnerRoot<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn downgrade_owner(&self) -> WeakSignalOwner<D, I, T> {
        self.owner.downgrade()
    }
    pub(crate) fn operation_control_for_test(
        &self,
    ) -> crate::branch::owner_services::SignalOwnerOperationControlForTest {
        self.owner.operation_control_for_test()
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
