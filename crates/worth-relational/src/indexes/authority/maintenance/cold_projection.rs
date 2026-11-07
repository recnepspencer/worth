//! Independent cold projection from the selected sealed head; no delta dispatcher or edits.
pub(super) mod funding;
use super::{
    changes::ChangedRecords, field_keys, join, ordering, record_metadata, root_view::IndexRootView,
    work::MaintenanceWork,
};
use crate::identity::data::{EntityId, RelationId};
use crate::indexes::data::*;
use std::collections::BTreeMap;
use DerivedIndexMaintenanceDenialKind as Denial;

pub(super) fn project(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    definition: &DerivedIndexDefinition,
    after: &IndexRootView,
    records: &ChangedRecords,
    work: &mut MaintenanceWork,
) -> Result<DerivedIndexEntries, Denial> {
    match &definition.kind {
        DerivedIndexKind::EntityField { field_locator } => {
            let mut rows = BTreeMap::new();
            for id in &records.entities {
                if let Some((key, id)) =
                    field_keys::entity(execution, Some(after), *id, field_locator, work)?
                {
                    let width = key.canonical_value_bytes().len() + 1;
                    push(&mut rows, key, id, width, work)?;
                }
            }
            seal(rows, |key| key.canonical_value_bytes().len() + 1, work)
                .map(DerivedIndexEntries::EntityField)
        }
        DerivedIndexKind::RelationField { field_locator } => {
            let mut rows = BTreeMap::new();
            for id in &records.relations {
                if let Some((key, id)) =
                    field_keys::relation(execution, Some(after), *id, field_locator, work)?
                {
                    let width = key.canonical_value_bytes().len() + 1;
                    push(&mut rows, key, id, width, work)?;
                }
            }
            seal(rows, |key| key.canonical_value_bytes().len() + 1, work)
                .map(DerivedIndexEntries::RelationField)
        }
        DerivedIndexKind::RelatedEntityOrdering {
            relation_kind,
            parent_endpoint,
            child_kind,
            ordering,
        } => {
            let mut rows = BTreeMap::new();
            for id in &records.relations {
                if let Some((parent, row)) = ordering::project_row(
                    execution,
                    after,
                    *id,
                    (*relation_kind, *parent_endpoint, *child_kind, ordering),
                    work,
                )? {
                    push(&mut rows, parent, row, 1, work)?;
                }
            }
            for rows in rows.values_mut() {
                let width = rows
                    .iter()
                    .map(|row| {
                        row.ordering_values()
                            .iter()
                            .map(|value| {
                                value.value().owned_allocation_capacity_bytes()
                                    + std::mem::size_of_val(value.value())
                            })
                            .sum::<usize>()
                    })
                    .max()
                    .unwrap_or(1)
                    .max(1);
                let comparisons = rows
                    .len()
                    .checked_mul((usize::BITS - rows.len().leading_zeros()) as usize + 1)
                    .and_then(|n| n.checked_mul(width))
                    .ok_or(Denial::WorkBudgetExceeded)?;
                work.prepare(comparisons as u64, (width * 4) as u64)?;
                rows.sort_unstable_by(|left, right| {
                    crate::indexes::projected_field_values::compare_related_entries(
                        left, right, ordering,
                    )
                });
            }
            seal(rows, |_| 1, work).map(DerivedIndexEntries::RelatedEntityOrdering)
        }
        DerivedIndexKind::RelationJoin(definition) => {
            collect_join(execution, after, records, *definition, work)
                .map(DerivedIndexEntries::RelationJoin)
        }
    }
}
fn push<K: Ord, R>(
    rows: &mut BTreeMap<K, Vec<R>>,
    key: K,
    row: R,
    width: usize,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    work.derive_row()?;
    work.ordered::<K, Vec<R>>(rows.len(), width, 0)?;
    let bucket = rows.entry(key).or_default();
    work.grow_vec(bucket)?;
    bucket.push(row);
    Ok(())
}
fn seal<K: Ord + Clone, R: Clone>(
    rows: BTreeMap<K, Vec<R>>,
    width: impl Fn(&K) -> usize,
    work: &mut MaintenanceWork,
) -> Result<DerivedIndexEntryMap<K, R>, Denial> {
    let mut bytes = funding::tree_bytes::<(std::sync::Arc<K>, DerivedIndexRows<R>)>(rows.len())?;
    for bucket in rows.values() {
        let (_, vector) = super::persistent_allocation::row_copy_bounds::<R>(bucket.len(), false)?;
        bytes = bytes
            .checked_add(vector as u64)
            .and_then(|n| {
                n.checked_add(
                    (bucket.len() * (std::mem::size_of::<R>() + 2 * std::mem::size_of::<usize>())
                        + std::mem::size_of::<K>()
                        + 2 * std::mem::size_of::<usize>()) as u64,
                )
            })
            .ok_or(Denial::WorkBudgetExceeded)?;
    }
    let maximum_width = rows.keys().map(width).max().unwrap_or(1);
    let comparisons = rows
        .len()
        .checked_mul((usize::BITS - rows.len().leading_zeros()) as usize + 1)
        .and_then(|n| n.checked_mul(11))
        .and_then(|n| n.checked_mul(maximum_width))
        .ok_or(Denial::WorkBudgetExceeded)?;
    work.prepare(comparisons as u64 + 1, bytes)?;
    Ok(rows.into())
}
fn collect_join(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    after: &IndexRootView,
    records: &ChangedRecords,
    definition: RelationJoinDefinition,
    work: &mut MaintenanceWork,
) -> Result<RelationJoinIndex, Denial> {
    let mut left = crate::indexes::data::ColdJoinLeg::new();
    let mut right = crate::indexes::data::ColdJoinLeg::new();
    for id in &records.relations {
        let metadata = record_metadata::relation(execution, Some(after), *id, work)?;
        for (leg, rows) in [
            (definition.left(), &mut left),
            (definition.right(), &mut right),
        ] {
            if let Some((shared, external)) = join::coordinate(
                execution,
                after,
                metadata,
                leg,
                definition.shared_entity_kind(),
                work,
            )? {
                work.ordered::<EntityId, BTreeMap<EntityId, std::collections::BTreeSet<RelationId>>>(rows.len(), 1, 0)?;
                let external_rows = rows.entry(shared).or_default();
                work.ordered::<EntityId, std::collections::BTreeSet<RelationId>>(
                    external_rows.len(),
                    1,
                    0,
                )?;
                let relations = external_rows.entry(external).or_default();
                work.ordered::<RelationId, ()>(relations.len(), 1, 0)?;
                relations.insert(*id);
            }
        }
    }
    let mut rows = BTreeMap::new();
    for (shared, left_rows) in &left {
        let Some(right_rows) = right.get(shared) else {
            continue;
        };
        for (left, left_relations) in left_rows {
            for (right, right_relations) in right_rows {
                push(
                    &mut rows,
                    RelationJoinKey::new(*left, *right),
                    RelationJoinEntry::new(
                        *shared,
                        *left_relations.first().ok_or(Denial::PriorEntryMismatch)?,
                        *right_relations.first().ok_or(Denial::PriorEntryMismatch)?,
                    ),
                    1,
                    work,
                )?;
            }
        }
    }
    funding::legs(&left, work)?;
    funding::legs(&right, work)?;
    work.prepare(
        1,
        (std::mem::size_of::<JoinLegsState>() + 2 * std::mem::size_of::<usize>()) as u64,
    )?;
    Ok(RelationJoinIndex::maintained(
        seal(rows, |_| 1, work)?,
        JoinLegsState::cold(left, right),
    ))
}
