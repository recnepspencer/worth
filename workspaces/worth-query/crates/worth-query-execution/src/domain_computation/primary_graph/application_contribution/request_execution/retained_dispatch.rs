//! Current member requests lend fresh native result execution authority.
use super::*;
use worth_execution::{
    ExecutionRequest, ExecutionRetainedResult, RetainedMapDenial, RetainedMapInput,
    RetainedReduceDenial, RetainedReduction,
};

impl QueryDispatch<'_> {
    pub(in super::super) fn map_retained<T, K, R, E, F>(
        &self,
        ceiling: u64,
        map: &ExecutionMap<RetainedMapInput<T, R>, K>,
        inputs: QueryMemoryReservation,
        kernel: F,
    ) -> Result<MapOutcome<ExecutionRetainedResult<R>, E>, RetainedMapDenial>
    where
        T: ChargedBytes + Sync,
        R: ChargedBytes + Send + Sync,
        E: ChargedBytes + Send,
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        let DispatchForm::Leased(lease) = &self.form;
        map.run_retained_inputs(
            ExecutionRequest::leased(lease),
            ExecutionWorkCeiling::new(ceiling),
            inputs.held,
            kernel,
        )
    }

    pub(in super::super) fn reduce_retained<T, K, R, E, Kernel, Diagnostic, Combine>(
        &self,
        ceiling: u64,
        map: &ExecutionMap<T, K>,
        inputs: QueryMemoryReservation,
        tree: &mut QueryMemoryReservation,
        (kernel, diagnostic): (Kernel, Diagnostic),
        identity: R,
        combine: Combine,
        max_value_bytes: u64,
        cause: super::super::partitioned_computation::WorthQueryPartitionedComputationFullCause,
    ) -> Result<RetainedReduction<R, Combine>, RetainedReduceDenial<E>>
    where
        T: Sync + ChargedBytes,
        R: Send + Sync + ChargedBytes + Clone + CanonicalBits,
        E: Send + ChargedBytes,
        Kernel:
            Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Clone + Sync,
        Diagnostic:
            Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Clone + Sync,
        Combine: Fn(&R, &R) -> R + Clone + Sync,
    {
        let certification = self
            .certified_seed
            .map(|seed| (seed, diagnostic, identity.clone(), combine.clone()));
        let DispatchForm::Leased(lease) = &self.form;
        let outcome = map.run_reduce_retained_holding(
            ExecutionRequest::leased(lease),
            ExecutionWorkCeiling::new(ceiling),
            inputs.held,
            tree.held.as_mut(),
            kernel,
            identity,
            combine,
            max_value_bytes,
            0,
        );
        super::super::partitioned_computation::observe_retained_full_tree(cause, &outcome);
        tree.bytes = match (&tree.held, &outcome.result) {
            (Some(held), _) => held.bytes(),
            (None, Ok(result)) => kept_tree_bytes(&result.tree).unwrap_or(u64::MAX),
            (None, Err(_)) => 0,
        };
        if let (Some((seed, kernel, identity, combine)), Ok(result)) =
            (certification, &outcome.result)
        {
            test_placement::certify(
                map,
                lease,
                seed,
                kernel,
                identity,
                combine,
                max_value_bytes,
                (&result.tree, &result.report),
            )
            .map_err(|cause| RetainedReduceDenial::Retention(RetainedMapDenial::Request(cause)))?;
        }
        outcome.result
    }
}
