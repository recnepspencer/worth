use super::*;

pub(crate) fn run_native_batch_with_charge<T, R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    batch: &AdmittedBatch<T>,
    backend: BackendKind,
    kernel: &F,
    charge_parent: bool,
) -> BatchOutcome<R, E>
where
    T: Sync,
    R: Send,
    E: Send,
    F: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
{
    run_batch(
        lease,
        batch.declaration(),
        BorrowedInputs(batch.values()),
        backend,
        kernel,
        charge_parent,
        None,
        None,
        None,
        None,
        PayloadStorage::Native,
    )
}

pub(crate) fn run_owned_native_batch<T, R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    batch: AdmittedBatch<T>,
    backend: BackendKind,
    kernel: &F,
) -> BatchOutcome<R, E>
where
    T: Send,
    R: Send,
    E: Send,
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
        None,
        None,
        None,
        PayloadStorage::Native,
    )
}

pub(crate) fn run_owned_native_batch_perturbed_for_test<T, R, E, F>(
    lease: Option<&ExecutionResourceLease<'_>>,
    batch: AdmittedBatch<T>,
    seed: u64,
    kernel: &F,
) -> (BatchOutcome<R, E>, Vec<usize>)
where
    T: Send,
    R: Send,
    E: Send,
    F: Fn(T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
{
    let (declaration, values) = batch.into_parts();
    let order = std::sync::Mutex::new(Vec::new());
    let inputs = super::super::input::TracedOwnedInputsForTest {
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
        PayloadStorage::Native,
    );
    (outcome, order.into_inner().unwrap())
}
