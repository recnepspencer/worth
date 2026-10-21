use worth_foundational::{ExecutionReport, PartitionIdentity};

use crate::{
    authority::{ExecutionMemoryReservation, ExecutionResourceLease, LeaseDenial},
    report::ChargedBytes,
};

use super::{
    admission::AdmittedBatch,
    input::{BorrowedInputs, OwnedInputs},
    meter::{KernelContext, KernelFailure, NestedStopOwner},
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
    pub(crate) allocation_denial: Option<LeaseDenial>,
    pub(crate) result: super::task_result::TaskResult<R, E>,
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
    pub(crate) stop_owner: Option<NestedStopOwner>,
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

pub(crate) fn run_checked_batch_recording<T, R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    batch: &AdmittedBatch<T>,
    backend: BackendKind,
    records: &super::settlement::records::SettlementRecords,
    inputs: Option<ExecutionMemoryReservation>,
    kernel: &F,
    retained: Option<&mut super::settlement::RetainedSettlement<'_, R>>,
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
        true,
        None,
        inputs,
        Some(records),
        retained,
    )
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
        None,
        None,
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
        None,
        None,
    )
}

/// The receiver owns this recorder for exactly this controlled dispatch.
pub(crate) fn run_owned_batch_perturbed_for_test<T, R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    batch: AdmittedBatch<T>,
    seed: u64,
    kernel: &F,
) -> (BatchOutcome<R, E>, Vec<usize>)
where
    T: ChargedBytes + Send,
    R: ChargedBytes + Send,
    E: ChargedBytes + Send,
    F: Fn(T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
{
    let (declaration, values) = batch.into_parts();
    let order = std::sync::Mutex::new(Vec::new());
    let inputs = super::input::TracedOwnedInputsForTest {
        values: OwnedInputs(values),
        order: &order,
    };
    let outcome = run_batch(
        lease,
        &declaration,
        inputs,
        BackendKind::Perturbation(seed),
        kernel,
        true,
        None,
        None,
        None,
        None,
    );
    (outcome, order.into_inner().unwrap())
}
