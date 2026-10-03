//! Physical tree backing, comparison work, and prepaid lifecycle cleanup.

use super::{BranchCursors, ProductBranchIncarnation};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

pub(super) fn levels(entries: usize) -> Result<usize, WorthQueryOutputDemandDenial> {
    let n = entries.max(1);
    Ok(usize::BITS as usize - n.leading_zeros() as usize)
}

pub(super) fn charge_tree(
    admission: &mut InvalidationEditAdmission,
    entries: usize,
    comparison: usize,
    searches: usize,
) -> Result<(), WorthQueryOutputDemandDenial> {
    let comparisons = entries.min(
        11usize
            .checked_mul(levels(entries)?)
            .ok_or_else(work_denial)?,
    );
    let work = comparisons
        .checked_mul(comparison)
        .and_then(|n| n.checked_add(1))
        .and_then(|n| n.checked_mul(searches))
        .ok_or_else(work_denial)?;
    admission
        .charge_external_work(u64::try_from(work).map_err(|_| work_denial())?)
        .map_err(|_| work_denial())
}

pub(super) fn charge_split(
    admission: &mut InvalidationEditAdmission,
    entries: usize,
) -> Result<(), WorthQueryOutputDemandDenial> {
    let visits = levels(entries)?
        .checked_add(1)
        .and_then(|levels| levels.checked_mul(11))
        .ok_or_else(work_denial)?;
    admission
        .charge_external_work(u64::try_from(visits).map_err(|_| work_denial())?)
        .map_err(|_| work_denial())
}

pub(super) fn charge_outer_retirement(
    admission: &mut InvalidationEditAdmission,
    installed_required_bytes: usize,
) -> Result<(), WorthQueryOutputDemandDenial> {
    let minimum_outer = node_bytes::<ProductBranchIncarnation, BranchCursors>()?;
    let maximum_rows = installed_required_bytes
        .checked_div(minimum_outer)
        .ok_or_else(work_denial)?
        .max(1);
    let maximum_levels = levels(maximum_rows)?;
    let comparisons = maximum_rows.min(
        11usize
            .checked_mul(maximum_levels)
            .ok_or_else(work_denial)?,
    );
    // One exact lookup, removal search and up to one eleven-key rebalance
    // per possible level. The installed aggregate bounds future cardinality.
    let visits = comparisons
        .checked_mul(6)
        .and_then(|n| n.checked_add(11usize.checked_mul(maximum_levels.checked_add(1)?)?))
        .and_then(|n| n.checked_add(2))
        .ok_or_else(work_denial)?;
    admission
        .charge_external_work(u64::try_from(visits).map_err(|_| work_denial())?)
        .map_err(|_| work_denial())
}

pub(super) fn node_bytes<K, V>() -> Result<usize, WorthQueryOutputDemandDenial> {
    11usize
        .checked_mul(std::mem::size_of::<(K, V)>())
        .and_then(|n| n.checked_add(12 * std::mem::size_of::<usize>()))
        .and_then(|n| n.checked_add(64))
        .and_then(|n| n.checked_mul(2))
        .ok_or_else(capacity_denial)
}

pub(super) fn admission_denial(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> WorthQueryOutputDemandDenial {
    use worth_relational::facade::mvcc::CompanionPreflightStop;
    match stop {
        CompanionPreflightStop::WorkExhausted { .. }
        | CompanionPreflightStop::WorkCounterOverflow => work_denial(),
        _ => capacity_denial(),
    }
}

pub(super) fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "required discontinuity cursor exceeds request work",
    )
}

pub(super) fn capacity_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "required discontinuity cursor exceeds retained capacity",
    )
}
