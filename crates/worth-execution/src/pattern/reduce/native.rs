//! Native storage on the same canonical reduction and original map custody.
use super::*;
use crate::{ExecutionRequest, ExecutionWorkCeiling, NativePayload, RetainedMapDenial};
mod certify;
mod leased;
use leased::NativeCapture;
#[cfg(test)]
mod tests;

impl<T: Sync, K> ExecutionMap<T, K, NativePayload> {
    /// Maps and canonically reduces without an application heap invoice or
    /// result forecast. Only a completed native map can issue retained custody.
    pub fn run_reduce_retained_native<R, E, Kernel, Combine>(
        &self,
        request: ExecutionRequest<'_, '_>,
        ceiling: ExecutionWorkCeiling,
        kernel: Kernel,
        identity: R,
        combine: Combine,
    ) -> NativeRetainedReduceOutcome<R, Combine, E>
    where
        R: Clone + Send + Sync + CanonicalBits,
        E: Send,
        Kernel: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
        Combine: Fn(&R, &R) -> R + Sync,
    {
        let mut capture = NativeCapture::default();
        let run = request.run(ceiling, |lease| {
            self.run_reduce_native_leased(
                lease,
                super::super::map::backend_for(lease),
                true,
                kernel,
                identity,
                combine,
                &mut capture,
            )
        });
        let reduction_metrics = run.as_ref().ok().and_then(|(result, _)| match result {
            Ok((_, _, metrics)) => Some(*metrics),
            Err(ReduceInputDenial::ReductionStopped { failure, .. }) => Some(failure.metrics),
            _ => None,
        });
        let result = (|| {
            let result = run
                .map_err(|cause| {
                    RetainedReduceDenial::Retention(RetainedMapDenial::Request(cause))
                })?
                .0;
            request.check_live().map_err(|stop| {
                RetainedReduceDenial::Retention(RetainedMapDenial::interruption(stop))
            })?;
            if let Some(cause) = capture.denial.take() {
                return Err(RetainedReduceDenial::Retention(cause));
            }
            let (tree, report, metrics) = result.map_err(RetainedReduceDenial::Reduction)?;
            Ok(RetainedReduction {
                tree,
                run: capture.run.take(),
                report,
                metrics,
                _reducer: std::marker::PhantomData,
            })
        })();
        RetainedReduceOutcome {
            result,
            mapped_report: capture.report,
            reduction_metrics,
        }
    }
}
