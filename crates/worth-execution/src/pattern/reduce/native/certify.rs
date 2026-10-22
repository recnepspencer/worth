use super::*;
use crate::{
    backend::enter_certification_activity,
    oracle::{compare_canonical_values, OracleMismatch},
};

impl<T: Sync, K> ExecutionMap<T, K, NativePayload> {
    /// The serial oracle and perturbed scheduler use the same native storage
    /// policy, canonical association and actual retained map completion.
    pub fn certify_reduce_native<R, E, Kernel, Combine>(
        &self,
        lease: &ExecutionResourceLease<'_>,
        seed: u64,
        kernel: Kernel,
        identity: R,
        combine: Combine,
    ) -> Result<
        (
            crate::NativeReductionTree<R, Combine>,
            ExecutionReport,
            ReductionMetrics,
        ),
        ReduceCertificationFailure<E>,
    >
    where
        R: Clone + Send + Sync + CanonicalBits,
        E: Send,
        Kernel: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
        Combine: Fn(&R, &R) -> R + Clone + Sync,
    {
        let activity = enter_certification_activity();
        let preparation =
            crate::backend::run_scope_native_within(Some(lease), true, None, None, |context| {
                context.poll()?;
                Ok::<_, KernelFailure<()>>((identity.clone(), combine.clone()))
            });
        let (oracle_identity, oracle_combine) = preparation
            .result
            .map_err(|_| ReduceCertificationFailure::Mismatch(OracleMismatch::Stop))?;
        let mut expected_capture = NativeCapture::default();
        let expected = self.run_reduce_native_leased(
            Some(lease),
            BackendKind::Serial,
            false,
            &kernel,
            oracle_identity,
            oracle_combine,
            &mut expected_capture,
        );
        if let Some(cause) = expected_capture.denial.take() {
            return Err(ReduceCertificationFailure::Retention(cause));
        }
        let (expected, expected_report, _) = expected.map_err(ReduceCertificationFailure::Run)?;
        let mut actual_capture = NativeCapture::default();
        let actual = self.run_reduce_native_leased(
            Some(lease),
            BackendKind::Perturbation(seed),
            true,
            &kernel,
            identity,
            combine,
            &mut actual_capture,
        );
        if let Some(cause) = actual_capture.denial.take() {
            return Err(ReduceCertificationFailure::Retention(cause));
        }
        let (actual, actual_report, metrics) = actual.map_err(ReduceCertificationFailure::Run)?;
        if !compare_canonical_values(lease, expected.result(), actual.result())
            .map_err(ReduceCertificationFailure::Mismatch)?
        {
            return Err(ReduceCertificationFailure::Mismatch(OracleMismatch::Values));
        }
        if expected_report.accounted_cost().map(|cost| cost.0)
            != actual_report.accounted_cost().map(|cost| cost.0)
        {
            return Err(ReduceCertificationFailure::Mismatch(
                OracleMismatch::ChargedWork,
            ));
        }
        if expected_report.accounted_cost().map(|cost| cost.1)
            != actual_report.accounted_cost().map(|cost| cost.1)
        {
            return Err(ReduceCertificationFailure::Mismatch(
                OracleMismatch::ChargedSpan,
            ));
        }
        let p = actual_report.physical();
        let physical = ExecutionPhysicalReport::new(
            p.active_workers_high_watermark(),
            activity.peak_memory(),
            p.steals(),
            p.peak_queue_width(),
            p.discarded_in_flight_work(),
        );
        let mut report = if let Some((work, span)) = actual_report.accounted_cost() {
            ExecutionReport::new(actual_report.resolved_posture(), work, span, physical)
        } else {
            ExecutionReport::native(actual_report.resolved_posture(), physical)
        };
        if let Some(cause) = actual_report.fallback() {
            report = report.with_fallback(cause);
        }
        Ok((actual, report, metrics))
    }
}
