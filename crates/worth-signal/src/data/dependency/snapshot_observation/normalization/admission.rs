//! Admission for the concrete snapshot normalization algorithm.
use crate::data::dependency::DependencySnapshotEntry;
use crate::data::error::SignalError;
use crate::logic::evaluation::EvaluationWork;

pub(super) fn comparison_bound(
    entries: &[DependencySnapshotEntry],
    work: &mut EvaluationWork<'_>,
) -> Result<usize, SignalError> {
    work.reserve(Some(entries.len()))?;
    let mut largest = 0;
    for entry in entries {
        let bytes = entry.scope.as_ref().map_or(Some(0), |s| {
            s.partition
                .0
                .len()
                .checked_add(s.detail.as_ref().map_or(0, String::len))
        });
        work.reserve(bytes.map(|_| 0))?;
        largest = largest.max(bytes.expect("checked scope size"));
    }
    let comparison = largest.checked_mul(2).and_then(|n| n.checked_add(32));
    work.reserve(comparison.and_then(|n| n.checked_mul(entries.len())))?;
    Ok(comparison.expect("admitted comparison size"))
}

pub(super) fn mutation_bound(
    entries: &[DependencySnapshotEntry],
    comparison: usize,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    // Arc::make_mut may copy every entry and both scope strings. Charge even
    // exclusive input conservatively; uniqueness is not an admission proof.
    work.reserve(
        entries
            .len()
            .checked_mul(std::mem::size_of::<DependencySnapshotEntry>() + 1)
            .filter(|n| *n <= isize::MAX as usize),
    )?;
    for entry in entries {
        let bytes = entry.scope.as_ref().map_or(Some(0), |s| {
            s.partition
                .0
                .len()
                .checked_add(s.detail.as_ref().map_or(0, String::len))
        });
        work.reserve(bytes)?;
    }
    let count = entries.len();
    let height = usize::BITS as usize - count.leading_zeros() as usize;
    work.reserve(
        count
            .checked_add(count / 2)
            .and_then(|n| n.checked_mul(height))
            .and_then(|n| n.checked_mul(comparison.checked_mul(2)?.checked_add(8)?))
            .and_then(|n| n.checked_add(count.checked_mul(comparison.checked_add(8)?)?)),
    )
}
