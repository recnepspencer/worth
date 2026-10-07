//! Work admission for owned scope-set construction and duplication.
use super::PartitionScopeSet;
use crate::data::error::SignalError;
use crate::data::output::{ChangedRegion, PartitionSubscription, ScopePath};
use crate::logic::evaluation::EvaluationWork;

impl PartitionScopeSet {
    pub(crate) fn admit_scopes_copy_work(
        scopes: &[PartitionSubscription],
        copies: usize,
        normalizations: usize,
        work: &mut EvaluationWork<'_, '_>,
    ) -> Result<(), SignalError> {
        admit_paths(
            scopes.iter().map(PartitionSubscription::path),
            scopes.len(),
            copies,
            normalizations,
            work,
        )
    }

    pub(crate) fn admit_regions_copy_work(
        regions: &[ChangedRegion],
        copies: usize,
        normalizations: usize,
        work: &mut EvaluationWork<'_, '_>,
    ) -> Result<(), SignalError> {
        admit_paths(
            regions.iter().map(ChangedRegion::path),
            regions.len(),
            copies,
            normalizations,
            work,
        )
    }
}

fn admit_paths<'a>(
    paths: impl Iterator<Item = &'a ScopePath>,
    count: usize,
    copies: usize,
    normalizations: usize,
    work: &mut EvaluationWork<'_, '_>,
) -> Result<(), SignalError> {
    let mut comparison = 16usize;
    work.reserve(
        count
            .checked_mul(copies)
            .and_then(|slots| slots.checked_mul(std::mem::size_of::<PartitionSubscription>())),
    )?;
    for path in paths {
        comparison = comparison.max(
            path.depth()
                .checked_mul(std::mem::size_of::<String>())
                .and_then(|slots| slots.checked_add(path.total_segment_bytes()))
                .unwrap_or(usize::MAX),
        );
        work.reserve(
            path.depth()
                .checked_mul(std::mem::size_of::<String>())
                .and_then(|slots| slots.checked_add(path.total_segment_bytes()))
                .and_then(|bytes| bytes.checked_mul(copies)),
        )?;
    }
    if count > 1 {
        let height = usize::BITS as usize - (count - 1).leading_zeros() as usize;
        work.reserve(
            count
                .checked_mul(height)
                .and_then(|steps| steps.checked_mul(comparison))
                .and_then(|steps| steps.checked_mul(normalizations)),
        )?;
    }
    Ok(())
}
