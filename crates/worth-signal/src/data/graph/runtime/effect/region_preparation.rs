//! Budgeted conversion from provider regions to canonical output scopes.
use crate::data::error::SignalError;
use crate::data::output::{ChangedRegion, PartitionMatchMode, PartitionSubscription};
use crate::data::proof::PartitionScopeSet;
use crate::logic::evaluation::EvaluationWork;
use crate::logic::invalidation::causality::normalize_changed_scopes;

pub(super) fn copy_region_scopes<'a>(
    regions: impl Iterator<Item = &'a ChangedRegion>,
    count: usize,
    work: &mut EvaluationWork<'_>,
) -> Result<PartitionScopeSet, SignalError> {
    work.reserve(
        count
            .checked_mul(std::mem::size_of::<PartitionSubscription>() + 1)
            .filter(|bytes| *bytes <= isize::MAX as usize),
    )?;
    let mut scopes = Vec::with_capacity(count);
    for region in regions {
        work.reserve(
            region
                .partition
                .0
                .len()
                .checked_add(region.detail.as_ref().map_or(0, String::len))
                .and_then(|bytes| bytes.checked_add(4)),
        )?;
        scopes.push(PartitionSubscription {
            partition: region.partition.clone(),
            detail: region.detail.clone(),
            match_mode: if region.detail.is_some() {
                PartitionMatchMode::PartitionAndDetail
            } else {
                PartitionMatchMode::WholePartition
            },
        });
    }
    normalize_changed_scopes(scopes, work)
}
