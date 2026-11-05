//! Current member requests lend fresh native result execution authority.
use super::*;
use worth_execution::{
    ExecutionRequest, NativeRetainedReduction, RetainedMapDenial, RetainedReduceDenial,
};

/// What a settled retained reduction reports to the computation that asked for it, so the
/// request never names that computation's causes.
pub trait WorthQueryRetainedReductionObserver {
    fn observe_retained_reduction<R, F, E, Tree>(
        self,
        outcome: &worth_execution::RetainedReduceOutcome<R, F, E, Tree>,
    );
}

impl QueryRequestExecution<'_> {
    /// Lends the original request's active descendant to native retained nodes.
    pub(in crate::domain_computation) fn with_native_lease<R>(
        &self,
        use_lease: impl FnOnce(&ExecutionResourceLease<'_>) -> R,
    ) -> Result<R, worth_execution::WorkCeilingDenial> {
        let lease = match &self.form {
            Form::Leased { lease, .. } | Form::Serial { lease, .. } => lease
                .as_ref()
                .map_err(|denial| worth_execution::WorkCeilingDenial::Admission(*denial))?,
        };
        ExecutionRequest::leased(lease).in_scope(|active| {
            use_lease(active.expect("a leased request retains its native backing"))
        })
    }
}

impl QueryDispatch<'_> {
    pub(in crate::domain_computation) fn advance_retained<T, R, E, F>(
        &self,
        ceiling: Option<u64>,
        prior: &worth_execution::ExecutionRetainedRun<R>,
        changes: std::collections::BTreeMap<worth_foundational::facade::PartitionIdentity, T>,
        removed: worth_proof::CanonicalUniqueVec<worth_foundational::facade::PartitionIdentity>,
        kernel: F,
    ) -> Result<worth_execution::RetainedRunOutcome<R, E>, worth_execution::RetainedRunDenial>
    where
        T: Sync,
        R: Send + Sync,
        E: Send,
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        let DispatchForm::Leased(lease) = &self.form;
        prior.advance_native(
            ExecutionRequest::leased(lease),
            ExecutionWorkCeiling::new(ceiling),
            changes,
            removed,
            kernel,
        )
    }

    pub(in crate::domain_computation) fn reduce_retained<
        T,
        R,
        E,
        Kernel,
        Diagnostic,
        Combine,
        Observer,
    >(
        &self,
        ceiling: Option<u64>,
        map: &ExecutionMap<T, (), worth_execution::NativePayload>,
        (kernel, diagnostic): (Kernel, Diagnostic),
        identity: R,
        combine: Combine,
        observer: Observer,
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
        Observer: WorthQueryRetainedReductionObserver,
    {
        let certification = self
            .certified_seed
            .map(|seed| (seed, diagnostic, identity.clone(), combine.clone()));
        let DispatchForm::Leased(lease) = &self.form;
        let outcome = map.run_reduce_retained_native(
            ExecutionRequest::leased(lease),
            ExecutionWorkCeiling::new(ceiling),
            kernel,
            identity,
            combine,
        );
        observer.observe_retained_reduction(&outcome);
        if let (Some((seed, kernel, identity, combine)), Ok(result)) =
            (certification, &outcome.result)
        {
            crate::domain_computation::request_execution::test_placement::certify_native(
                map,
                lease,
                seed,
                kernel,
                identity,
                combine,
                (&result.tree, &result.report),
            )
            .map_err(|cause| RetainedReduceDenial::Retention(RetainedMapDenial::Request(cause)))?;
        }
        outcome.result
    }
}

mod court_doors_for_test;
