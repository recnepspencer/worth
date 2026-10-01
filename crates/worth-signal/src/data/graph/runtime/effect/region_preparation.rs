//! Budgeted conversion from provider regions to canonical output scopes.
use crate::data::error::SignalError;
use crate::data::output::{ChangedRegion, PartitionSubscription, ScopeCoverage};
use crate::data::proof::PartitionScopeSet;
use crate::logic::evaluation::EvaluationWork;
use crate::logic::invalidation::causality::normalize_changed_scopes;

pub(super) fn copy_region_scopes<'a>(
    regions: impl Iterator<Item = &'a ChangedRegion>,
    count: usize,
    work: &mut EvaluationWork<'_, '_>,
) -> Result<PartitionScopeSet, SignalError> {
    work.reserve(
        count
            .checked_mul(std::mem::size_of::<PartitionSubscription>() + 1)
            .filter(|bytes| *bytes <= isize::MAX as usize),
    )?;
    let mut scopes = Vec::with_capacity(count);
    for region in regions {
        work.reserve(region.path().checked_segment_bytes().and_then(|bytes| {
            bytes.checked_add(
                region
                    .path()
                    .depth()
                    .checked_mul(std::mem::size_of::<String>())?,
            )
        }))?;
        scopes.push(match region.coverage() {
            ScopeCoverage::Exact => PartitionSubscription::exact(region.path().clone()),
            ScopeCoverage::Subtree => PartitionSubscription::subtree(region.path().clone()),
        });
    }
    normalize_changed_scopes(scopes, work)
}
