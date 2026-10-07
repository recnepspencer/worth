use std::mem::size_of;

use worth_foundational::{ExecutionPhysicalReport, ExecutionReport};

use crate::{
    authority::ExecutionResourceLease,
    backend::{enter_certification_activity, enter_retained_memory, run_scope, BackendKind},
    oracle::{compare_canonical_values, CanonicalBits, OracleMismatch},
    report::ChargedBytes,
};

use super::*;

/// A stage run stopped, or its result or charged cost diverged from the serial
/// oracle built from the same retained snapshot and changed partitions.
#[derive(Debug)]
pub enum DecomposeCertificationFailure<E> {
    Run(DecomposeRunFailure<E>),
    Mismatch(OracleMismatch),
}

impl<I, C, S, B, O, F> ExecutionDecompose<I, C, S, B, O, F>
where
    I: Clone + Send + Sync + ChargedBytes + CanonicalBits,
    C: Clone + Send + Sync + ChargedBytes + CanonicalBits,
    S: Clone + Send + ChargedBytes,
    B: Clone + Send + Sync + ChargedBytes + CanonicalBits,
    O: Clone + Send + ChargedBytes + CanonicalBits,
    F: Fn(&C, &C) -> C + Clone + Sync,
{
    /// Certify a seeded perturbed stage run against a serial run forked from
    /// the same retained state. Only the selected result updates this value.
    pub fn certify<T, K, E, Interior, Interface, Back>(
        &mut self,
        lease: &ExecutionResourceLease<'_>,
        seed: u64,
        changed: &ExecutionMap<T, K>,
        editions: DecomposeKernelEditions,
        interior: Interior,
        solve_interface: Interface,
        back_substitute: Back,
    ) -> Result<DecomposeComplete<O>, DecomposeCertificationFailure<E>>
    where
        T: Sync + ChargedBytes,
        E: Send + ChargedBytes,
        Interior: Fn(
                &T,
                &mut MapKernelContext<'_, '_>,
            ) -> Result<InteriorResult<I, C>, MapKernelFailure<E>>
            + Clone
            + Sync,
        Interface: Fn(
                &C,
                &mut MapKernelContext<'_, '_>,
            ) -> Result<InterfaceSolution<S, B>, MapKernelFailure<E>>
            + Clone
            + Sync,
        Back: Fn(&BackInput<I, B>, &mut MapKernelContext<'_, '_>) -> Result<O, MapKernelFailure<E>>
            + Clone
            + Sync,
    {
        let activity = enter_certification_activity();
        let snapshot_bytes = self
            .snapshot
            .as_ref()
            .map_or(0, ChargedBytes::additional_charged_bytes);
        let clone_bytes = u64::try_from(size_of::<Self>())
            .ok()
            .and_then(|bytes| {
                bytes.checked_add(
                    u64::try_from(
                        self.identities
                            .len()
                            .checked_mul(size_of::<worth_foundational::PartitionIdentity>())?,
                    )
                    .ok()?,
                )
            })
            .and_then(|bytes| bytes.checked_add(snapshot_bytes))
            .and_then(|bytes| bytes.checked_add(self.max_reduction_value_bytes))
            .and_then(|bytes| {
                bytes.checked_add(
                    self.reducer_storage_bytes
                        .checked_mul(if self.snapshot.is_some() { 2 } else { 1 })?,
                )
            })
            .ok_or(DecomposeCertificationFailure::Mismatch(
                OracleMismatch::Stop,
            ))?;
        let oracle_retention = lease
            .reserve_retained_memory(clone_bytes)
            .map_err(|_| DecomposeCertificationFailure::Mismatch(OracleMismatch::Stop))?;
        let oracle_physical = enter_retained_memory(clone_bytes);
        let mut oracle_clone = None;
        let preparation = run_scope(Some(lease), 0, 0, |context| {
            context.checkpoint(0)?;
            oracle_clone = Some(self.clone());
            Ok::<(), MapKernelFailure<()>>(())
        });
        preparation
            .result
            .map_err(|_| DecomposeCertificationFailure::Mismatch(OracleMismatch::Stop))?;
        let mut oracle = oracle_clone.expect("admitted clone completed");
        let mut oracle_result_retention = None;
        let mut oracle_result_bytes = 0;
        let expected = {
            let mut oracle_handoff = |bytes| {
                let reservation = lease.reserve_retained_memory(bytes)?;
                oracle_result_bytes = bytes;
                oracle_result_retention = Some(reservation);
                Ok(())
            };
            oracle
                .run_with_backend(
                    Some(lease),
                    BackendKind::Serial,
                    changed,
                    editions,
                    &interior,
                    &solve_interface,
                    &back_substitute,
                    false,
                    Some(&mut oracle_handoff),
                )
                .map_err(DecomposeCertificationFailure::Run)?
        };
        let oracle_result_physical = enter_retained_memory(oracle_result_bytes);
        let expected_bytes = expected.values.additional_charged_bytes();
        let expected_values_retention = lease
            .reserve_retained_memory(expected_bytes)
            .map_err(|_| DecomposeCertificationFailure::Mismatch(OracleMismatch::Stop))?;
        drop((
            oracle,
            oracle_retention,
            oracle_physical,
            oracle_result_retention,
            oracle_result_physical,
        ));
        let _expected_values = expected_values_retention;
        let _expected_physical = enter_retained_memory(expected_bytes);
        let _selected_retention = lease
            .reserve_retained_memory(clone_bytes)
            .map_err(|_| DecomposeCertificationFailure::Mismatch(OracleMismatch::Stop))?;
        let _selected_physical = enter_retained_memory(clone_bytes);
        let mut selected_clone = None;
        let preparation = run_scope(Some(lease), 0, 0, |context| {
            context.checkpoint(0)?;
            selected_clone = Some(self.clone());
            Ok::<(), MapKernelFailure<()>>(())
        });
        preparation
            .result
            .map_err(|_| DecomposeCertificationFailure::Mismatch(OracleMismatch::Stop))?;
        let mut selected = selected_clone.expect("admitted clone completed");
        let mut selected_result_retention = None;
        let mut selected_result_bytes = 0;
        let mut actual = {
            let mut selected_handoff = |bytes| {
                let reservation = lease.reserve_retained_memory(bytes)?;
                selected_result_bytes = bytes;
                selected_result_retention = Some(reservation);
                Ok(())
            };
            selected
                .run_with_backend(
                    Some(lease),
                    BackendKind::Perturbation(seed),
                    changed,
                    editions,
                    &interior,
                    &solve_interface,
                    &back_substitute,
                    true,
                    Some(&mut selected_handoff),
                )
                .map_err(DecomposeCertificationFailure::Run)?
        };
        let selected_result_physical = enter_retained_memory(selected_result_bytes);
        let _selected_result_retention = selected_result_retention;
        let _selected_result_physical = selected_result_physical;
        if expected.values.len() != actual.values.len() {
            return Err(DecomposeCertificationFailure::Mismatch(
                OracleMismatch::Values,
            ));
        }
        for (left, right) in expected.values.iter().zip(&actual.values) {
            if !compare_canonical_values(lease, left, right)
                .map_err(DecomposeCertificationFailure::Mismatch)?
            {
                return Err(DecomposeCertificationFailure::Mismatch(
                    OracleMismatch::Values,
                ));
            }
        }
        if expected.total_report.charged_work() != actual.total_report.charged_work() {
            return Err(DecomposeCertificationFailure::Mismatch(
                OracleMismatch::ChargedWork,
            ));
        }
        if expected.total_report.charged_span() != actual.total_report.charged_span() {
            return Err(DecomposeCertificationFailure::Mismatch(
                OracleMismatch::ChargedSpan,
            ));
        }
        if expected.reuse != actual.reuse || expected.reduction_metrics != actual.reduction_metrics
        {
            return Err(DecomposeCertificationFailure::Mismatch(
                OracleMismatch::Values,
            ));
        }
        let report = actual.total_report;
        let physical = report.physical();
        let physical = ExecutionPhysicalReport::new(
            physical.active_workers_high_watermark(),
            activity.peak_memory(),
            physical.steals(),
            physical.peak_queue_width(),
            physical.discarded_in_flight_work(),
        );
        actual.total_report = ExecutionReport::new(
            report.resolved_posture(),
            report.charged_work(),
            report.charged_span(),
            physical,
        );
        if let Some(fallback) = report.fallback() {
            actual.total_report = actual.total_report.with_fallback(fallback);
        }
        *self = selected;
        Ok(actual)
    }
}
