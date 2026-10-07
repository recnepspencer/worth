use worth_foundational::{ExecutionPhysicalReport, ExecutionReport};

use crate::{
    authority::ExecutionResourceLease,
    backend::{
        enter_certification_activity, enter_retained_memory, run_scope, BackendKind, KernelContext,
        KernelFailure,
    },
    oracle::{compare_canonical_values, CanonicalBits, OracleMismatch},
    reduction::{ReductionMetrics, ReductionTree},
    report::ChargedBytes,
};

use super::{ExecutionMap, ReduceInputDenial};

/// The selected run or its serial oracle could not complete, or their
/// canonical result and charged cost disagreed.
#[derive(Debug)]
pub enum ReduceCertificationFailure<E> {
    Run(ReduceInputDenial<E>),
    Mismatch(OracleMismatch),
}

impl<T: Sync + ChargedBytes, K> ExecutionMap<T, K> {
    /// Run a reduction through the serial oracle and a seeded perturbed backend.
    /// Both runs retain the same admitted partition identities and reducer.
    pub fn certify_reduce<R, E, Kernel, Combine>(
        &self,
        lease: &ExecutionResourceLease<'_>,
        seed: u64,
        kernel: Kernel,
        identity: R,
        combine: Combine,
        max_value_bytes: u64,
        reducer_storage_bytes: u64,
    ) -> Result<
        (ReductionTree<R, Combine>, ExecutionReport, ReductionMetrics),
        ReduceCertificationFailure<E>,
    >
    where
        R: Send + Sync + ChargedBytes + Clone + CanonicalBits,
        E: Send + ChargedBytes,
        Kernel: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Clone + Sync,
        Combine: Fn(&R, &R) -> R + Clone + Sync,
    {
        let activity = enter_certification_activity();
        let clone_bytes = max_value_bytes
            .checked_add(reducer_storage_bytes)
            .ok_or(ReduceCertificationFailure::Mismatch(OracleMismatch::Stop))?;
        let _clone_reservation = lease
            .reserve_retained_memory(clone_bytes)
            .map_err(|_| ReduceCertificationFailure::Mismatch(OracleMismatch::Stop))?;
        let _clone_physical = enter_retained_memory(clone_bytes);
        let mut oracle_parts = None;
        let preparation = run_scope(Some(lease), 0, 0, |context| {
            context.checkpoint(0)?;
            oracle_parts = Some((identity.clone(), combine.clone()));
            Ok::<(), KernelFailure<()>>(())
        });
        preparation
            .result
            .map_err(|_| ReduceCertificationFailure::Mismatch(OracleMismatch::Stop))?;
        let (oracle_identity, oracle_combine) = oracle_parts.expect("admitted clone completed");
        let mut expected_retention = None;
        let mut expected_bytes = 0;
        let (expected, expected_report, _) = {
            let mut expected_handoff = |bytes| {
                let reservation = lease.reserve_retained_memory(bytes)?;
                expected_retention = Some(reservation);
                expected_bytes = bytes;
                Ok(())
            };
            self.run_reduce_leased(
                lease,
                &kernel,
                oracle_identity,
                oracle_combine,
                max_value_bytes,
                reducer_storage_bytes,
                BackendKind::Serial,
                false,
                Some(&mut expected_handoff),
            )
            .map_err(ReduceCertificationFailure::Run)?
        };
        drop((_clone_reservation, _clone_physical));
        let _expected_retention = expected_retention;
        let _expected_physical = enter_retained_memory(expected_bytes);
        let mut actual_retention = None;
        let mut actual_bytes = 0;
        let (actual, actual_report, metrics) = {
            let mut actual_handoff = |bytes| {
                let reservation = lease.reserve_retained_memory(bytes)?;
                actual_retention = Some(reservation);
                actual_bytes = bytes;
                Ok(())
            };
            self.run_reduce_leased(
                lease,
                &kernel,
                identity,
                combine,
                max_value_bytes,
                reducer_storage_bytes,
                BackendKind::Perturbation(seed),
                true,
                Some(&mut actual_handoff),
            )
            .map_err(ReduceCertificationFailure::Run)?
        };
        let _actual_retention = actual_retention;
        let _actual_physical = enter_retained_memory(actual_bytes);
        if !compare_canonical_values(lease, expected.result(), actual.result())
            .map_err(ReduceCertificationFailure::Mismatch)?
        {
            return Err(ReduceCertificationFailure::Mismatch(OracleMismatch::Values));
        }
        if expected_report.charged_work() != actual_report.charged_work() {
            return Err(ReduceCertificationFailure::Mismatch(
                OracleMismatch::ChargedWork,
            ));
        }
        if expected_report.charged_span() != actual_report.charged_span() {
            return Err(ReduceCertificationFailure::Mismatch(
                OracleMismatch::ChargedSpan,
            ));
        }
        let physical = actual_report.physical();
        let physical = ExecutionPhysicalReport::new(
            physical.active_workers_high_watermark(),
            activity.peak_memory(),
            physical.steals(),
            physical.peak_queue_width(),
            physical.discarded_in_flight_work(),
        );
        let mut report = ExecutionReport::new(
            actual_report.resolved_posture(),
            actual_report.charged_work(),
            actual_report.charged_span(),
            physical,
        );
        if let Some(fallback) = actual_report.fallback() {
            report = report.with_fallback(fallback);
        }
        Ok((actual, report, metrics))
    }
}
