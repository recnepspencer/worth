//! Two paid binary bounds restrict the range before page materialization.
use super::*;
use std::cmp::Ordering;
use worth_foundational::facade::AspectValue;
type Rows = DerivedIndexRows<RelatedEntityOrderingEntry>;

pub(super) fn entry<'a>(
    rows: &'a Rows,
    index: usize,
    meter: &mut MapKernelContext<'_, '_>,
) -> Result<&'a RelatedEntityOrderingEntry, Denial> {
    charge_navigation(meter, rows.len(), true)?;
    rows.get(index).ok_or(Denial::ForeignBoundary)
}
fn compare_prefix(
    row: &RelatedEntityOrderingEntry,
    prefix: &[AspectValue],
    ordering: &[RelatedEntityOrderingField],
    meter: &mut MapKernelContext<'_, '_>,
) -> Result<Ordering, Denial> {
    if row.ordering_values().len() != ordering.len() {
        return Err(Denial::InvalidPrefix);
    }
    for ((actual, expected), field) in row.ordering_values().iter().zip(prefix).zip(ordering) {
        charge(meter, 1)?;
        let bytes = actual
            .value()
            .semantic_byte_width()
            .max(expected.semantic_byte_width());
        charge(
            meter,
            u64::try_from(bytes)
                .map_err(|_| Denial::AccountingOverflow)?
                .checked_add(1)
                .ok_or(Denial::AccountingOverflow)?,
        )?;
        let cmp = crate::indexes::projected_field_values::compare_ordering_value(
            actual.value(),
            expected,
            field,
        );
        if cmp != Ordering::Equal {
            return Ok(cmp);
        }
    }
    Ok(Ordering::Equal)
}
fn bound(
    rows: &Rows,
    prefix: &[AspectValue],
    ordering: &[RelatedEntityOrderingField],
    upper: bool,
    meter: &mut MapKernelContext<'_, '_>,
    comparisons: &mut usize,
) -> Result<usize, Denial> {
    let (mut lo, mut hi) = (0, rows.len());
    while lo < hi {
        charge(meter, 1)?;
        let mid = lo + (hi - lo) / 2;
        let cmp = compare_prefix(entry(rows, mid, meter)?, prefix, ordering, meter)?;
        *comparisons += 1;
        if cmp == Ordering::Less || upper && cmp == Ordering::Equal {
            lo = mid + 1
        } else {
            hi = mid
        }
    }
    Ok(lo)
}
pub(super) fn select(
    read: &RelationalPinnedRead,
    request: &BoundedRelatedEntityPrefixLookupRequest<'_>,
    prepared: &PreparedRelatedLookup<'_>,
    rows: Option<&Rows>,
    meter: &mut MapKernelContext<'_, '_>,
) -> Result<(usize, usize, usize), Denial> {
    let Some(rows) = rows else {
        return if request.after.is_some() {
            Err(Denial::ForeignBoundary)
        } else {
            Ok((0, 0, 0))
        };
    };
    let ordering = prepared.contract.ordering();
    let mut comparisons = 0;
    let start = if request.prefix.is_empty() {
        0
    } else {
        bound(
            rows,
            request.prefix,
            ordering,
            false,
            meter,
            &mut comparisons,
        )?
    };
    let end = if request.prefix.is_empty() {
        rows.len()
    } else {
        bound(
            rows,
            request.prefix,
            ordering,
            true,
            meter,
            &mut comparisons,
        )?
    };
    let Some(after) = request.after else {
        return Ok((start, end, comparisons));
    };
    charge(meter, 1 + after.basis.branch_id().0.len() as u64)?;
    if after.runtime != read.query_context().runtime_instance_id
        || after.basis.root_id() != read.pinned_basis().root_id()
        || after.basis.branch_id() != read.pinned_basis().branch_id()
        || after.generation.generation_id != prepared.generation_id
        || after.generation.index_id != request.ordered.index_id()
        || after.parent != request.ordered.parent_entity_id()
        || after.child_kind != request.ordered.child_kind()
        || after.prefix_len != request.prefix.len()
    {
        return Err(Denial::ForeignBoundary);
    }
    if after.last < start || after.last >= end {
        return Err(Denial::ForeignBoundary);
    }
    // Same immutable generation and position; changed scope/prefix is nevertheless rejected.
    if compare_prefix(
        entry(rows, after.last, meter)?,
        request.prefix,
        ordering,
        meter,
    )? != Ordering::Equal
    {
        return Err(Denial::ForeignBoundary);
    }
    Ok((after.last + 1, end, comparisons))
}
