use super::super::{
    backend_for, ExecutionMap, MapKernelContext, MapKernelFailure, NativePayload, RetainedMapDenial,
};
use super::{finish_native_outcome, RetainedRunDenial, RetainedRunMetrics, RetainedRunOutcome};
use crate::{
    backend::{run_retained_batch, run_retained_native_batch, BatchOutcome, ResultStorageDenial},
    reduction::RunTree,
    ChargedBytes, ExecutionRequest, ExecutionResourceLease, ExecutionRetainedResult,
    ExecutionWorkCeiling,
};

impl<T: ChargedBytes + Sync, K> ExecutionMap<T, K> {
    /// Cold issue from the ordinary native map; only access-key-free maps can
    /// later accept keyless changes. Arbitrary declared maps remain full runs.
    /// Caller-owned input equality/currentness is never established here.
    pub fn run_retained_run<R, E, F>(
        &self,
        request: ExecutionRequest<'_, '_>,
        ceiling: ExecutionWorkCeiling,
        kernel: F,
    ) -> Result<RetainedRunOutcome<R, E>, RetainedRunDenial>
    where
        R: ChargedBytes + Send + Sync,
        E: ChargedBytes + Send,
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        issue(self, request, ceiling, false, |lease| {
            run_retained_batch(lease, &self.batch, backend_for(lease), None, &kernel)
        })
    }
}

impl<T: Sync, K> ExecutionMap<T, K, NativePayload> {
    /// Issue native retained results without a payload forecast. Every accepted
    /// result keeps its original cell and receipt; callers still qualify reuse.
    pub fn run_retained_run<R, E, F>(
        &self,
        request: ExecutionRequest<'_, '_>,
        ceiling: ExecutionWorkCeiling,
        kernel: F,
    ) -> Result<RetainedRunOutcome<R, E>, RetainedRunDenial>
    where
        R: Send + Sync,
        E: Send,
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        issue(self, request, ceiling, true, |lease| {
            run_retained_native_batch(lease, &self.batch, backend_for(lease), &kernel)
        })
    }
}

fn issue<T, K, Payload, R, E>(
    map: &ExecutionMap<T, K, Payload>,
    request: ExecutionRequest<'_, '_>,
    ceiling: ExecutionWorkCeiling,
    native: bool,
    run: impl FnOnce(
        Option<&ExecutionResourceLease<'_>>,
    ) -> Result<BatchOutcome<ExecutionRetainedResult<R>, E>, ResultStorageDenial>,
) -> Result<RetainedRunOutcome<R, E>, RetainedRunDenial> {
    let operation = |lease: Option<&ExecutionResourceLease<'_>>| {
        if !map.has_keyless_retention() {
            return Err(RetainedRunDenial::RequiresKeyless);
        }
        let outcome = run(lease)
            .map_err(|cause| RetainedRunDenial::Native(RetainedMapDenial::storage(cause)))?;
        finish_native_outcome(
            RunTree::empty(),
            outcome,
            lease,
            RetainedRunMetrics {
                supplied_inputs: map.partition_count() as u64,
                ..Default::default()
            },
        )
        .map_err(RetainedRunDenial::from)
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

impl<T, K, Payload> ExecutionMap<T, K, Payload> {
    pub(in crate::pattern) fn has_keyless_retention(&self) -> bool {
        self._read_sets.iter().all(Vec::is_empty)
            && self
                ._write_sets
                .members()
                .iter()
                .all(|(_, keys)| keys.is_empty())
    }
}
