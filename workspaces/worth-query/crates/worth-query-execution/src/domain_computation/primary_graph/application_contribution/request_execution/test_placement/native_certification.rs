//! The ordinary native payload path keeps the same serial/perturbed oracle.
use super::*;

pub(in super::super) fn certify_native<T, R, E, Kernel, Combine>(
    map: &ExecutionMap<T, (), worth_execution::NativePayload>,
    lease: &ExecutionResourceLease<'_>,
    seed: u64,
    kernel: Kernel,
    identity: R,
    combine: Combine,
    (run, report): (
        &worth_execution::NativeReductionTree<R, Combine>,
        &ExecutionReport,
    ),
) -> Result<(), worth_execution::WorkCeilingDenial>
where
    T: Sync,
    R: Send + Sync + Clone + CanonicalBits,
    E: Send,
    Kernel: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Clone + Sync,
    Combine: Fn(&R, &R) -> R + Clone + Sync,
{
    let (checked, _) = worth_execution::ExecutionWorkCeiling::new(None).run_detached_for_test(
        lease,
        |diagnostic| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let (certified, certified_report, _) = map
                    .certify_reduce_native(diagnostic, seed, kernel, identity, combine)
                    .map_err(|failure| certification_failure(diagnostic, seed, failure))?;
                let same_bits =
                    compare_canonical_values(diagnostic, run.result(), certified.result())
                        .map_err(|stop| {
                            if diagnostic.is_cancelled() {
                                worth_execution::WorkCeilingDenial::Stopped(
                                    worth_execution::MapKernelStop::Cancelled,
                                )
                            } else if diagnostic.deadline_elapsed() {
                                worth_execution::WorkCeilingDenial::Stopped(
                                    worth_execution::MapKernelStop::DeadlineElapsed,
                                )
                            } else {
                                panic!("the certified result could not be compared: {stop:?}")
                            }
                        })?;
                Ok((
                    same_bits,
                    certified_report.charged_work(),
                    certified_report.charged_span(),
                ))
            }))
        },
    )?;
    let (same_bits, work, span) = match checked {
        Ok(result) => result?,
        Err(payload) => std::panic::resume_unwind(payload),
    };
    assert!(
        same_bits,
        "the run's result differs from the certified one under seed {seed}"
    );
    assert_eq!(
        report.charged_work(),
        work,
        "native canonical work differs under seed {seed}"
    );
    assert_eq!(
        report.charged_span(),
        span,
        "native canonical span differs under seed {seed}"
    );
    Ok(())
}
