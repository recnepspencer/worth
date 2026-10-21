use super::super::{
    backend_for, ExecutionMap, MapKernelContext, MapKernelFailure, RetainedMapDenial,
};
use super::{finish_native_outcome, RetainedRunDenial, RetainedRunMetrics, RetainedRunOutcome};
use crate::{
    backend::run_retained_batch, reduction::RunTree, ChargedBytes, ExecutionRequest,
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
        let result = request
            .run(ceiling, |lease| {
                if !self.has_keyless_retention() {
                    return Err(RetainedRunDenial::RequiresKeyless);
                }
                let outcome =
                    run_retained_batch(lease, &self.batch, backend_for(lease), None, &kernel)
                        .map_err(|cause| {
                            RetainedRunDenial::Native(RetainedMapDenial::storage(cause))
                        })?;
                finish_native_outcome(
                    RunTree::empty(),
                    outcome,
                    lease,
                    RetainedRunMetrics {
                        supplied_inputs: self.partition_count() as u64,
                        ..Default::default()
                    },
                )
                .map_err(RetainedRunDenial::from)
            })
            .map_err(|cause| RetainedRunDenial::Native(RetainedMapDenial::Request(cause)))?
            .0;
        request
            .check_live()
            .map_err(|stop| RetainedRunDenial::Native(RetainedMapDenial::interruption(stop)))?;
        result
    }
}

impl<T, K> ExecutionMap<T, K> {
    pub(in crate::pattern) fn has_keyless_retention(&self) -> bool {
        self._read_sets.iter().all(Vec::is_empty)
            && self
                ._write_sets
                .members()
                .iter()
                .all(|(_, keys)| keys.is_empty())
    }
}
