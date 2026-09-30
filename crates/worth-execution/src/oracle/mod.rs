mod canonical_bits;

use canonical_bits::visit_exact;
pub use canonical_bits::CanonicalBits;

use crate::{
    authority::ExecutionResourceLease,
    backend::{
        enter_retained_memory, run_checked_batch, run_checked_batch_with_charge, AdmittedBatch,
        BackendKind, BatchOutcome, KernelContext, KernelFailure,
    },
    report::ChargedBytes,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OracleMismatch {
    Values,
    Stop,
    ChargedWork,
    ChargedSpan,
}

/// Evaluate the same admitted batch and kernel through serial and selected ports.
pub(crate) fn certify<T, R, E, F>(
    lease: &ExecutionResourceLease<'_>,
    batch: &AdmittedBatch<T>,
    selected: BackendKind,
    kernel: &F,
) -> Result<BatchOutcome<R, E>, OracleMismatch>
where
    T: Sync + ChargedBytes,
    R: Send + ChargedBytes + CanonicalBits,
    E: Send + ChargedBytes + CanonicalBits,
    F: Fn(&T, &mut KernelContext<'_, '_>) -> Result<R, KernelFailure<E>> + Sync,
{
    let retained_bytes = batch
        .retained_result_bytes::<R>()
        .ok_or(OracleMismatch::Stop)?;
    let expected_retained = lease
        .reserve_retained_memory(retained_bytes)
        .map_err(|_| OracleMismatch::Stop)?;
    let _expected_physical = enter_retained_memory(retained_bytes);
    let expected =
        run_checked_batch_with_charge(Some(lease), batch, BackendKind::Serial, kernel, false);
    let actual_retained = lease
        .reserve_retained_memory(retained_bytes)
        .map_err(|_| OracleMismatch::Stop)?;
    let _actual_physical = enter_retained_memory(retained_bytes);
    let actual = run_checked_batch(Some(lease), batch, selected, kernel);
    compare(lease, &expected, &actual)?;
    drop((expected_retained, actual_retained));
    Ok(actual)
}

fn compare<R: CanonicalBits, E: CanonicalBits>(
    lease: &ExecutionResourceLease<'_>,
    expected: &BatchOutcome<R, E>,
    actual: &BatchOutcome<R, E>,
) -> Result<(), OracleMismatch> {
    if expected.values.len() != actual.values.len() {
        return Err(OracleMismatch::Values);
    }
    for (left, right) in expected.values.iter().zip(&actual.values) {
        if !same_bits(lease, left, right)? {
            return Err(OracleMismatch::Values);
        }
    }
    if !same_stop(lease, &expected.stop, &actual.stop)?
        || expected.prefix_boundary != actual.prefix_boundary
    {
        return Err(OracleMismatch::Stop);
    }
    if expected.report.charged_work() != actual.report.charged_work() {
        return Err(OracleMismatch::ChargedWork);
    }
    if expected.report.charged_span() != actual.report.charged_span() {
        return Err(OracleMismatch::ChargedSpan);
    }
    Ok(())
}

/// Hold one reference encoding at a time. The selected encoding is checked
/// against that buffer as it streams, so it never needs a second allocation.
fn same_bits<T: CanonicalBits + ?Sized>(
    lease: &ExecutionResourceLease<'_>,
    expected: &T,
    actual: &T,
) -> Result<bool, OracleMismatch> {
    let length = expected.canonical_len().ok_or(OracleMismatch::Stop)?;
    let bytes = u64::try_from(length).map_err(|_| OracleMismatch::Stop)?;
    let _reservation = lease
        .reserve_retained_memory(bytes)
        .map_err(|_| OracleMismatch::Stop)?;
    let _physical = enter_retained_memory(bytes);
    let mut reference = Vec::new();
    reference
        .try_reserve_exact(length)
        .map_err(|_| OracleMismatch::Stop)?;
    if !visit_exact(expected, &mut |chunk| {
        reference.extend_from_slice(chunk);
        true
    }) {
        return Err(OracleMismatch::Stop);
    }
    if actual.canonical_len() != Some(length) {
        return Ok(false);
    }
    let mut position = 0_usize;
    let equal = visit_exact(actual, &mut |chunk| {
        let Some(end) = position.checked_add(chunk.len()) else {
            return false;
        };
        if reference.get(position..end) != Some(chunk) {
            return false;
        }
        position = end;
        true
    });
    Ok(equal && position == length)
}

fn same_stop<E: CanonicalBits>(
    lease: &ExecutionResourceLease<'_>,
    expected: &Option<crate::backend::BatchStop<E>>,
    actual: &Option<crate::backend::BatchStop<E>>,
) -> Result<bool, OracleMismatch> {
    use crate::backend::{BatchStop, KernelFailure};

    match (expected, actual) {
        (None, None) => Ok(true),
        (
            Some(BatchStop::Failure {
                identity: left_id,
                cause: left_cause,
            }),
            Some(BatchStop::Failure {
                identity: right_id,
                cause: right_cause,
            }),
        ) => {
            if left_id != right_id {
                return Ok(false);
            }
            match (left_cause, right_cause) {
                (KernelFailure::Stop(left), KernelFailure::Stop(right)) => Ok(left == right),
                (KernelFailure::Domain(left), KernelFailure::Domain(right)) => {
                    same_bits(lease, left, right)
                }
                (KernelFailure::Panic, KernelFailure::Panic) => Ok(true),
                (KernelFailure::ResultCapacityExceeded, KernelFailure::ResultCapacityExceeded) => {
                    Ok(true)
                }
                _ => Ok(false),
            }
        }
        (
            Some(BatchStop::WorkExhausted { identity: left }),
            Some(BatchStop::WorkExhausted { identity: right }),
        ) => Ok(left == right),
        // A denied dispatch never ran the kernel, even if both ports denied it.
        (Some(BatchStop::Admission(_)), Some(BatchStop::Admission(_))) => Ok(false),
        _ => Ok(false),
    }
}
