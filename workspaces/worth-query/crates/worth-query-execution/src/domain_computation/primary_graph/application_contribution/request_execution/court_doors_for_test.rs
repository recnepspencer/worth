//! Sealed test doors over this module's private members.
use super::*;

impl<'a> QueryRequestExecution<'a> {
    #[doc(hidden)]
    pub fn open_control_for_test(
        placement: RuntimeWorldExecutionPlacement<'a>,
        cancellation: worth_execution::CancellationToken,
        deadline: Option<std::time::Instant>,
    ) -> Self {
        Self::open_control(placement, cancellation, deadline)
    }
}

impl<'a> QueryRequestExecution<'a> {
    #[doc(hidden)]
    pub fn declared_work_mut_for_test(&mut self) -> &mut Option<u64> {
        &mut self.declared_work
    }
}

impl<'a> QueryRequestExecution<'a> {
    #[doc(hidden)]
    pub fn allocation_policy_for_test(
        &self,
    ) -> Result<worth_execution::ExecutionAllocationPolicy<'_, 'a>, Resource> {
        Self::allocation_policy(self)
    }
}

impl<'a> QueryRequestExecution<'a> {
    #[doc(hidden)]
    pub fn checkpoint_for_test(&self) -> Result<(), MapKernelStop> {
        Self::checkpoint(self)
    }
}

impl<'a> QueryRequestExecution<'a> {
    #[doc(hidden)]
    pub fn dispatch_for_test(&self) -> Result<QueryDispatch<'_>, Resource> {
        Self::dispatch(self)
    }
}

impl<'a> QueryRequestExecution<'a> {
    #[doc(hidden)]
    pub fn open_for_test(
        placement: RuntimeWorldExecutionPlacement<'a>,
        request: &'a WorthQueryRequestScope,
    ) -> Self {
        Self::open(placement, request)
    }
}
