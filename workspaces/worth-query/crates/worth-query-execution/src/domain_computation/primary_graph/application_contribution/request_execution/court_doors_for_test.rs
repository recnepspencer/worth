//! Sealed test doors over this module's private members.
use super::*;

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

impl QueryDispatch<'_> {
    #[doc(hidden)]
    pub fn map_for_test<T, K, R, E, Kernel>(
        &self,
        ceiling: u64,
        map: &ExecutionMap<T, K>,
        inputs: QueryMemoryReservation,
        kernel: Kernel,
    ) -> Result<MapOutcome<R, E>, WorkCeilingDenial>
    where
        T: Sync + ChargedBytes,
        R: Send + ChargedBytes,
        E: Send + ChargedBytes,
        Kernel: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        Self::map::<T, K, R, E, Kernel>(self, ceiling, map, inputs, kernel)
    }
}

impl<'a> QueryRequestExecution<'a> {
    #[doc(hidden)]
    pub fn open_control_for_test(
        placement: RuntimeWorldExecutionPlacement<'a>,
        cancellation: worth_execution::CancellationToken,
        deadline: Option<worth_execution::MonotonicDeadline>,
    ) -> Self {
        Self::open_control(placement, cancellation, deadline)
    }
}
