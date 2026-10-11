//! Law 1, reach: this issuance door is sealed with the owner control files.
use super::SignalRuntime;
impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    /// Issue deterministic control over the real owner progression through its owned handle.
    pub fn owner_operation_control_for_test(
        &self,
    ) -> Result<
        crate::branch::owner_services::SignalOwnerOperationControlForTest,
        crate::branch::owner_services::SignalOwnerUnavailable,
    > {
        self.owner_services.operation_control_for_test()
    }
}
