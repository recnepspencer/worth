use super::super::{
    backend_for, ExecutionMap, KeylessPartition, MapKernelContext, MapKernelFailure,
    RetainedMapDenial,
};
use super::{
    finish_native_outcome, ExecutionRetainedRun, RetainedRunDenial, RetainedRunMetrics,
    RetainedRunOutcome,
};
use crate::{
    backend::{run_retained_native_sparse_batch, run_retained_sparse_batch, BatchOutcome},
    ChargedBytes, ExecutionRequest, ExecutionResourceLease, ExecutionRetainedResult,
    ExecutionWorkCeiling,
};
use std::collections::BTreeMap;
use worth_foundational::PartitionIdentity;
use worth_proof::CanonicalUniqueVec;

impl<R: ChargedBytes + Send + Sync> ExecutionRetainedRun<R> {
    /// Advance with actual changed inputs and removals only. Unmentioned slots
    /// retain their exact successful native result cells. The caller must
    /// independently qualify permission to omit those kernel invocations.
    /// All request, cancellation, deadline and native work limits are fresh.
    /// Refusal borrows and preserves this run, including its original funding.
    pub fn advance<T, E, F>(
        &self,
        request: ExecutionRequest<'_, '_>,
        ceiling: ExecutionWorkCeiling,
        changes: BTreeMap<PartitionIdentity, KeylessPartition<T>>,
        removed: CanonicalUniqueVec<PartitionIdentity>,
        kernel: F,
    ) -> Result<RetainedRunOutcome<R, E>, RetainedRunDenial>
    where
        T: ChargedBytes + Sync,
        E: ChargedBytes + Send,
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        self.advance_with(
            request,
            ceiling,
            false,
            changes,
            removed,
            |lease, changes, tree, metrics| {
                let map = ExecutionMap::<T, ()>::from_keyless_partitions(changes)
                    .map_err(RetainedRunDenial::Map)?;
                run_retained_sparse_batch(
                    lease,
                    &map.batch,
                    backend_for(lease),
                    tree,
                    metrics,
                    &kernel,
                )
                .map_err(|cause| RetainedRunDenial::Native(RetainedMapDenial::storage(cause)))
            },
        )
    }
}

impl<R: Send + Sync> ExecutionRetainedRun<R> {
    /// Advance the existing native run using changed values with no payload
    /// invoices. Retained cells keep their original grants and exact receipts.
    /// This receives new inputs, never reinterprets a declared partition batch.
    pub fn advance_native<T, E, F>(
        &self,
        request: ExecutionRequest<'_, '_>,
        ceiling: ExecutionWorkCeiling,
        changes: BTreeMap<PartitionIdentity, T>,
        removed: CanonicalUniqueVec<PartitionIdentity>,
        kernel: F,
    ) -> Result<RetainedRunOutcome<R, E>, RetainedRunDenial>
    where
        T: Sync,
        E: Send,
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        self.advance_with(
            request,
            ceiling,
            true,
            changes,
            removed,
            |lease, changes, tree, metrics| {
                let map = ExecutionMap::<T, ()>::from_native_partitions(changes)
                    .map_err(RetainedRunDenial::NativeMap)?;
                run_retained_native_sparse_batch(
                    lease,
                    &map.batch,
                    backend_for(lease),
                    tree,
                    metrics,
                    &kernel,
                )
                .map_err(|cause| RetainedRunDenial::Native(RetainedMapDenial::storage(cause)))
            },
        )
    }

    fn advance_with<C, E>(
        &self,
        request: ExecutionRequest<'_, '_>,
        ceiling: ExecutionWorkCeiling,
        native: bool,
        changes: BTreeMap<PartitionIdentity, C>,
        removed: CanonicalUniqueVec<PartitionIdentity>,
        dispatch: impl FnOnce(
            Option<&ExecutionResourceLease<'_>>,
            BTreeMap<PartitionIdentity, C>,
            &crate::reduction::RunTree<R>,
            &mut RetainedRunMetrics,
        )
            -> Result<BatchOutcome<ExecutionRetainedResult<R>, E>, RetainedRunDenial>,
    ) -> Result<RetainedRunOutcome<R, E>, RetainedRunDenial> {
        let operation = |lease: Option<&ExecutionResourceLease<'_>>| {
            // Re-admit cached incoming backing under the fresh request;
            // original node/result grants remain alive independently.
            let accounted = !native || crate::backend::RunLimits::native_accounting(lease);
            if accounted && !self.accounted {
                if let Some(id) = self.tree.summary().first {
                    return Err(RetainedRunDenial::Native(
                        RetainedMapDenial::RequiresExecution(id),
                    ));
                }
            }
            let _prior_hold = if accounted {
                Some(crate::ExecutionMemoryReservation::reserve_in_scope(
                    lease,
                    self.tree.bytes(),
                )?)
            } else {
                None
            };
            let _prior_activity =
                accounted.then(|| crate::backend::enter_retained_memory(self.tree.bytes()));
            let mut tree = self.tree.clone();
            let mut metrics = RetainedRunMetrics {
                supplied_inputs: changes.len() as u64,
                ..Default::default()
            };
            for &id in removed.as_slice() {
                if changes.contains_key(&id) {
                    return Err(RetainedRunDenial::ChangedAndRemoved(id));
                }
                if !tree.contains(id, &mut metrics) {
                    return Err(RetainedRunDenial::UnknownRemoval(id));
                }
                tree.remove(id, lease, &mut metrics)?;
            }
            // Replacement slots are not carried, even if their new content
            // or costs happen to equal the old result. New cells come only
            // from actual current evaluation and native settlement.
            for &id in changes.keys() {
                if tree.contains(id, &mut metrics) {
                    tree.remove(id, lease, &mut metrics)?;
                }
            }
            if let Some(id) = tree.summary().unqualified {
                return Err(RetainedRunDenial::Native(
                    RetainedMapDenial::RequiresExecution(id),
                ));
            }
            let outcome = dispatch(lease, changes, &tree, &mut metrics)?;
            finish_native_outcome(tree, outcome, lease, metrics).map_err(RetainedRunDenial::from)
        };
        let result = if native {
            super::run_native_retained(request, ceiling, operation)
        } else {
            request.run(ceiling, operation)
        }
        .map_err(|cause| RetainedRunDenial::Native(RetainedMapDenial::Request(cause)))?
        .0;
        request
            .check_live()
            .map_err(|stop| RetainedRunDenial::Native(RetainedMapDenial::interruption(stop)))?;
        result
    }
}
