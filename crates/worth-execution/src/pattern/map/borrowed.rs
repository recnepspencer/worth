use crate::{
    authority::{ExecutionMemoryReservation, ExecutionResourceLease},
    backend::{run_checked_batch_taking, BackendKind},
    oracle::{self, CanonicalBits},
    report::ChargedBytes,
};

use super::{
    backend_for, prepared, ExecutionMap, MapKernelContext, MapKernelFailure, MapOutcome,
    OracleMismatch, PreparedExecutionMap,
};

impl<T: Sync + ChargedBytes, K> ExecutionMap<T, K> {
    /// Consume the checked map and retain its exact live memory admission
    /// before domain evaluators run. Dispatch later reserves only a worker.
    pub fn prepare_run<'authority, R, E>(
        self,
        lease: ExecutionResourceLease<'authority>,
    ) -> Result<PreparedExecutionMap<'authority, T, K, R, E>, crate::LeaseDenial>
    where
        R: Send + ChargedBytes,
        E: Send + ChargedBytes,
    {
        prepared::prepare_map(self, lease)
    }

    pub fn run<R, E, F>(
        &self,
        lease: Option<&ExecutionResourceLease<'_>>,
        kernel: F,
    ) -> MapOutcome<R, E>
    where
        R: Send + ChargedBytes,
        E: Send + ChargedBytes,
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        self.run_with_backend(lease, backend_for(lease), kernel)
    }

    /// [`Self::run`], with the run's memory admission taking over `inputs`,
    /// the caller's reservation for what the partitions hold. It becomes the
    /// run's own, at the run's exact bytes on the run's lease or serial
    /// budget, in one ledger step, so the inputs are never unreserved between
    /// the caller's hold and the run's. The run releases it when it settles.
    pub fn run_taking<R, E, F>(
        &self,
        lease: Option<&ExecutionResourceLease<'_>>,
        inputs: ExecutionMemoryReservation,
        kernel: F,
    ) -> MapOutcome<R, E>
    where
        R: Send + ChargedBytes,
        E: Send + ChargedBytes,
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        self.run_with_backend_taking(lease, backend_for(lease), Some(inputs), kernel)
    }

    pub(crate) fn run_with_backend<R, E, F>(
        &self,
        lease: Option<&ExecutionResourceLease<'_>>,
        backend: BackendKind,
        kernel: F,
    ) -> MapOutcome<R, E>
    where
        R: Send + ChargedBytes,
        E: Send + ChargedBytes,
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        self.run_with_backend_taking(lease, backend, None, kernel)
    }

    pub(crate) fn run_with_backend_taking<R, E, F>(
        &self,
        lease: Option<&ExecutionResourceLease<'_>>,
        backend: BackendKind,
        taken: Option<ExecutionMemoryReservation>,
        kernel: F,
    ) -> MapOutcome<R, E>
    where
        R: Send + ChargedBytes,
        E: Send + ChargedBytes,
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        run_checked_batch_taking(lease, &self.batch, backend, taken, &kernel).into()
    }

    /// Certification runs the exact admitted inputs and kernel on the serial
    /// oracle and a schedule-perturbed backend before comparing canonical cost.
    pub fn certify<R, E, F>(
        &self,
        lease: &ExecutionResourceLease<'_>,
        seed: u64,
        kernel: F,
    ) -> Result<MapOutcome<R, E>, OracleMismatch>
    where
        R: Send + ChargedBytes + CanonicalBits,
        E: Send + ChargedBytes + CanonicalBits,
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        oracle::certify(lease, &self.batch, BackendKind::Perturbation(seed), &kernel)
            .map(MapOutcome::from)
    }
}
