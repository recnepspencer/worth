use worth_foundational::{ExecutionReport, PartitionIdentity};

use crate::{
    authority::{ExecutionMemoryReservation, ExecutionResourceLease, LeaseDenial},
    report::ChargedBytes,
};

use super::{
    admission::AdmittedBatch,
    input::{BorrowedInputs, OwnedInputs},
    meter::{KernelContext, KernelFailure},
    run::run_batch,
    PreparedBatchResources,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum BackendKind {
    Serial,
    Native,
    Perturbation(u64),
}

pub(crate) struct TaskOutcome<R, E> {
    pub(crate) result: Result<R, KernelFailure<E>>,
    pub(crate) work: u64,
    pub(crate) span: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BatchStop<E> {
    Failure {
        identity: PartitionIdentity,
        cause: KernelFailure<E>,
    },
    WorkExhausted {
        identity: PartitionIdentity,
    },
    Admission(LeaseDenial),
}

pub(crate) struct BatchOutcome<R, E> {
    pub(crate) values: Vec<R>,
    pub(crate) stop: Option<BatchStop<E>>,
    /// Exclusive identity boundary; every earlier partition settled.
    pub(crate) prefix_boundary: Option<PartitionIdentity>,
    pub(crate) report: ExecutionReport,
}

pub(crate) fn run_checked_batch<T, R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    batch: &AdmittedBatch<T>,
    backend: BackendKind,
    kernel: &F,
) -> BatchOutcome<R, E>
where
    T: ChargedBytes + Sync,
    R: ChargedBytes + Send,
    E: ChargedBytes + Send,
    F: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
{
    run_checked_batch_with_charge(lease, batch, backend, kernel, true)
}

pub(crate) fn run_checked_batch_with_charge<T, R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    batch: &AdmittedBatch<T>,
    backend: BackendKind,
    kernel: &F,
    charge_parent: bool,
) -> BatchOutcome<R, E>
where
    T: ChargedBytes + Sync,
    R: ChargedBytes + Send,
    E: ChargedBytes + Send,
    F: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
{
    run_checked_batch_inner(lease, batch, backend, kernel, charge_parent, None, None)
}

/// A run whose memory admission takes over `taken`, the caller's reservation
/// for the run's inputs: it becomes the run's own, at the run's exact bytes,
/// so the inputs are never unreserved between the two.
pub(crate) fn run_checked_batch_taking<T, R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    batch: &AdmittedBatch<T>,
    backend: BackendKind,
    taken: Option<ExecutionMemoryReservation>,
    kernel: &F,
) -> BatchOutcome<R, E>
where
    T: ChargedBytes + Sync,
    R: ChargedBytes + Send,
    E: ChargedBytes + Send,
    F: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
{
    run_checked_batch_inner(lease, batch, backend, kernel, true, None, taken)
}

pub(crate) fn run_checked_batch_prepared<T, R, E, F>(
    lease: &ExecutionResourceLease<'_>,
    batch: &AdmittedBatch<T>,
    backend: BackendKind,
    resources: PreparedBatchResources,
    kernel: &F,
) -> BatchOutcome<R, E>
where
    T: ChargedBytes + Sync,
    R: ChargedBytes + Send,
    E: ChargedBytes + Send,
    F: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
{
    run_checked_batch_inner(
        Some(lease),
        batch,
        backend,
        kernel,
        true,
        Some(resources),
        None,
    )
}

fn run_checked_batch_inner<T, R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    batch: &AdmittedBatch<T>,
    backend: BackendKind,
    kernel: &F,
    charge_parent: bool,
    prepared: Option<PreparedBatchResources>,
    taken: Option<ExecutionMemoryReservation>,
) -> BatchOutcome<R, E>
where
    T: ChargedBytes + Sync,
    R: ChargedBytes + Send,
    E: ChargedBytes + Send,
    F: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
{
    run_batch(
        lease,
        batch.declaration(),
        BorrowedInputs(batch.values()),
        backend,
        kernel,
        charge_parent,
        prepared,
        taken,
    )
}

pub(crate) fn run_owned_batch_taking<T, R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    batch: AdmittedBatch<T>,
    backend: BackendKind,
    taken: Option<ExecutionMemoryReservation>,
    kernel: &F,
) -> BatchOutcome<R, E>
where
    T: ChargedBytes + Send,
    R: ChargedBytes + Send,
    E: ChargedBytes + Send,
    F: Fn(T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
{
    let (declaration, values) = batch.into_parts();
    run_batch(
        lease,
        &declaration,
        OwnedInputs(values),
        backend,
        kernel,
        true,
        None,
        taken,
    )
}
