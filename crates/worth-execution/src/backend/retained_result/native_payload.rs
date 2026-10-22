//! Native payload posture uses the same captures, slots and canonical settlement.
use super::*;
use crate::backend::{
    input::BorrowedInputs, payload_storage::PayloadStorage, run::run_batch,
    settlement::RetainedSettlement,
};

pub(crate) fn run_retained_native_batch<T, R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    batch: &AdmittedBatch<T>,
    backend: BackendKind,
    kernel: &F,
) -> Result<BatchOutcome<ExecutionRetainedResult<R>, E>, ResultStorageDenial>
where
    T: Sync,
    R: Send + Sync,
    E: Send,
    F: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
{
    run_retained(lease, batch, backend, kernel, None)
}

pub(crate) fn run_retained_native_sparse_batch<T, R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    batch: &AdmittedBatch<T>,
    backend: BackendKind,
    tree: &crate::reduction::RunTree<R>,
    metrics: &mut crate::RetainedRunMetrics,
    kernel: &F,
) -> Result<BatchOutcome<ExecutionRetainedResult<R>, E>, ResultStorageDenial>
where
    T: Sync,
    R: Send + Sync,
    E: Send,
    F: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
{
    let mut retained = RetainedSettlement::new(tree, metrics);
    run_retained(lease, batch, backend, kernel, Some(&mut retained))
}

fn run_retained<T, R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    batch: &AdmittedBatch<T>,
    backend: BackendKind,
    kernel: &F,
    retained: Option<&mut RetainedSettlement<'_, R>>,
) -> Result<BatchOutcome<ExecutionRetainedResult<R>, E>, ResultStorageDenial>
where
    T: Sync,
    R: Send + Sync,
    E: Send,
    F: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
{
    let records = SettlementRecords::admit_native::<R>(batch.declaration(), lease)?;
    let outcome = run_batch(
        lease,
        batch.declaration(),
        BorrowedInputs(batch.values()),
        backend,
        kernel,
        true,
        None,
        None,
        Some(&records),
        retained,
        PayloadStorage::Native,
    );
    Ok(finish(batch.identities(), outcome, records, store))
}
