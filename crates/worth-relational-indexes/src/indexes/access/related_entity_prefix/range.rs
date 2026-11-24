//! Two paid binary bounds restrict the range before page materialization.
use super::*;
use std::cmp::Ordering;
use worth_foundational::facade::AspectValue;
use worth_relational_branch::internal_port::indexes::data::related_entity_prefix::BoundedRelatedEntityPrefixBoundaryFieldsPort as _;
type Rows = DerivedIndexRows<RelatedEntityOrderingEntry>;

pub(super) fn entry<'a>(
    rows: &'a Rows,
    index: usize,
    _meter: &mut MapKernelContext<'_, '_>,
) -> Result<&'a RelatedEntityOrderingEntry, Denial> {
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
    read: &PinnedIndexLookup<'_>,
    request: &BoundedRelatedEntityPrefixLookupRequest<'_>,
    prepared: &PreparedRelatedLookup<'_>,
    rows: Option<&Rows>,
    meter: &mut MapKernelContext<'_, '_>,
) -> Result<(usize, usize, usize), Denial> {
    let Some(rows) = rows else {
        return if (*request.owner_after()).is_some() {
            Err(Denial::ForeignBoundary)
        } else {
            Ok((0, 0, 0))
        };
    };
    let ordering = prepared.contract.ordering();
    let mut comparisons = 0;
    let start = if (*request.owner_prefix()).is_empty() {
        0
    } else {
        bound(
            rows,
            request.owner_prefix(),
            ordering,
            false,
            meter,
            &mut comparisons,
        )?
    };
    let end = if (*request.owner_prefix()).is_empty() {
        rows.len()
    } else {
        bound(
            rows,
            request.owner_prefix(),
            ordering,
            true,
            meter,
            &mut comparisons,
        )?
    };
    let Some(after) = *request.owner_after() else {
        return Ok((start, end, comparisons));
    };
    charge(meter, 1 + (*after.owner_basis()).branch_id().0.len() as u64)?;
    if (*after.owner_runtime()) != read.runtime_identity
        || (*after.owner_basis()).root_id() != read.basis().root_id()
        || (*after.owner_basis()).branch_id() != read.basis().branch_id()
        || after.owner_generation().generation_id != prepared.generation_id
        || after.owner_generation().index_id != (*request.owner_ordered()).index_id()
        || (*after.owner_parent()) != (*request.owner_ordered()).parent_entity_id()
        || (*after.owner_child_kind()) != (*request.owner_ordered()).child_kind()
        || (*after.owner_prefix_len()) != (*request.owner_prefix()).len()
    {
        return Err(Denial::ForeignBoundary);
    }
    if (*after.owner_last()) < start || (*after.owner_last()) >= end {
        return Err(Denial::ForeignBoundary);
    }
    // Same immutable generation and position; changed scope/prefix is nevertheless rejected.
    if compare_prefix(
        entry(rows, *after.owner_last(), meter)?,
        request.owner_prefix(),
        ordering,
        meter,
    )? != Ordering::Equal
    {
        return Err(Denial::ForeignBoundary);
    }
    Ok(((*after.owner_last()) + 1, end, comparisons))
}
