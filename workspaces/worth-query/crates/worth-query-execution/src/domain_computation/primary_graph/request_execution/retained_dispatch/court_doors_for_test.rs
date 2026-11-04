//! Sealed test doors over this module's private members.
use super::*;

impl QueryRequestExecution<'_> {
    #[doc(hidden)]
    pub fn with_native_lease_for_test<R>(
        &self,
        use_lease: impl FnOnce(&ExecutionResourceLease<'_>) -> R,
    ) -> Result<R, worth_execution::WorkCeilingDenial> {
        Self::with_native_lease(self, use_lease)
    }
}

impl QueryDispatch<'_> {
    #[doc(hidden)]
    pub fn reduce_retained_for_test<T, R, E, Kernel, Diagnostic, Combine>(
        &self,
        ceiling: Option<u64>,
        map: &ExecutionMap<T, (), worth_execution::NativePayload>,
        _arg3: (Kernel, Diagnostic),
        identity: R,
        combine: Combine,
        cause: crate::domain_computation::primary_graph::application_contribution::partitioned_computation::WorthQueryPartitionedComputationFullCause,
    ) -> Result<NativeRetainedReduction<R, Combine>, RetainedReduceDenial<E>>
    where
        T: Sync,
        R: Send + Sync + Clone + CanonicalBits,
        E: Send,
        Kernel:
            Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Clone + Sync,
        Diagnostic:
            Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Clone + Sync,
        Combine: Fn(&R, &R) -> R + Clone + Sync,
    {
        Self::reduce_retained::<T, R, E, Kernel, Diagnostic, Combine>(
            self, ceiling, map, _arg3, identity, combine, cause,
        )
    }
}
