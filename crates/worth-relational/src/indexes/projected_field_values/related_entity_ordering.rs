use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::identity::data::KindId;
use crate::indexes::data::{
    RelatedEntityEndpoint, RelatedEntityOrderingDirection, RelatedEntityOrderingEntry,
    RelatedEntityOrderingField,
};
use crate::visibility::materialization::read_records::entity_query_locus_value;

use super::checked_entry_map::{checked_push, IndexKernelStop};
use super::field_projection_scope::source_entity_index_projection_scope_for_kind;
use super::{entity_aspect_field_ordering_value, IndexProjectionSource};

pub(in crate::indexes) struct RelatedEntityOrderingProjection<'definition> {
    relation_kind: KindId,
    parent_endpoint: RelatedEntityEndpoint,
    child_kind: KindId,
    ordering: &'definition [RelatedEntityOrderingField],
}

impl<'definition> RelatedEntityOrderingProjection<'definition> {
    pub(in crate::indexes) const fn new(
        relation_kind: KindId,
        parent_endpoint: RelatedEntityEndpoint,
        child_kind: KindId,
        ordering: &'definition [RelatedEntityOrderingField],
    ) -> Self {
        Self {
            relation_kind,
            parent_endpoint,
            child_kind,
            ordering,
        }
    }
}

pub(in crate::indexes) fn build_related_entity_ordering_index_checked(
    projection: &IndexProjectionSource<'_, '_>,
    contract: &RelatedEntityOrderingProjection<'_>,
    context: &mut crate::execution::PacketKernelContext<'_, '_, '_>,
) -> Result<
    BTreeMap<crate::identity::data::EntityId, Vec<RelatedEntityOrderingEntry>>,
    IndexKernelStop,
> {
    let mut entries = BTreeMap::new();
    let budget = std::cell::RefCell::new(context);
    projection.try_for_each_relation(
        contract.relation_kind,
        |bytes| {
            budget.borrow_mut().checkpoint(1)?;
            budget.borrow().check_scratch_peak(bytes)
        },
        |relation| {
            let (parent, child) = match contract.parent_endpoint {
                RelatedEntityEndpoint::SourceParent => (relation.source, relation.target),
                RelatedEntityEndpoint::TargetParent => (relation.target, relation.source),
            };
            budget.borrow_mut().checkpoint(1)?;
            budget
                .borrow()
                .check_scratch_peak(projection.candidate_entity_bytes(child))?;
            let values = projection
                .with_entity(
                    child,
                    |record| -> Result<
                        Option<(Vec<worth_foundational::facade::AspectValue>, u64)>,
                        IndexKernelStop,
                    > {
                        if record.kind.kind_id != contract.child_kind {
                            return Ok(None);
                        }
                        let mut values = Vec::new();
                        let mut owned = 0_u64;
                        for field in contract.ordering {
                            budget.borrow_mut().checkpoint(1)?;
                            if source_entity_index_projection_scope_for_kind(
                                projection,
                                record.kind.kind_id,
                                field.locator(),
                            )
                            .is_none()
                            {
                                return Ok(None);
                            }
                            let Some(value) = entity_query_locus_value(record, field.locator())
                            else {
                                return Ok(None);
                            };
                            owned = owned
                                .saturating_add(value.owned_allocation_capacity_bytes() as u64);
                            budget.borrow().check_scratch_peak(owned.saturating_add(
                                ((values.len() + 1)
                                    * std::mem::size_of::<worth_foundational::facade::AspectValue>(
                                    )) as u64,
                            ))?;
                            budget.borrow().check_result_peak(owned.saturating_add(
                                ((values.len() + 1)
                                    * std::mem::size_of::<
                                        crate::indexes::data::RelatedEntityOrderingValue,
                                    >()) as u64,
                            ))?;
                            values.try_reserve_exact(1).map_err(|_| {
                                worth_execution::MapKernelFailure::ResultCapacityExceeded
                            })?;
                            values.push(value.clone());
                        }
                        Ok(Some((values, owned)))
                    },
                )
                .transpose()?
                .flatten();
            if let Some((values, owned)) = values {
                let row = RelatedEntityOrderingEntry::new(values, child, relation.relation_id);
                checked_push(
                    &mut entries,
                    parent,
                    row,
                    0,
                    owned.saturating_add(
                        (contract.ordering.len()
                            * std::mem::size_of::<crate::indexes::data::RelatedEntityOrderingValue>(
                            )) as u64,
                    ),
                    &mut budget.borrow_mut(),
                )?;
            }
            Ok(())
        },
    )?;
    for rows in entries.values_mut() {
        budget.borrow_mut().checkpoint(rows.len() as u64)?;
        rows.sort_by(|left, right| compare_related_entries(left, right, contract.ordering));
    }
    Ok(entries)
}

pub(in crate::indexes) fn build_related_entity_ordering_index(
    projection: &IndexProjectionSource<'_, '_>,
    contract: &RelatedEntityOrderingProjection<'_>,
) -> BTreeMap<crate::identity::data::EntityId, Vec<RelatedEntityOrderingEntry>> {
    let mut entries =
        BTreeMap::<crate::identity::data::EntityId, Vec<RelatedEntityOrderingEntry>>::new();
    projection.for_each_relation(contract.relation_kind, |relation| {
        let (parent, child) = match contract.parent_endpoint {
            RelatedEntityEndpoint::SourceParent => (relation.source, relation.target),
            RelatedEntityEndpoint::TargetParent => (relation.target, relation.source),
        };
        let values = projection
            .with_entity(child, |record| {
                if record.kind.kind_id != contract.child_kind {
                    return None;
                }
                contract
                    .ordering
                    .iter()
                    .map(|field| {
                        entity_aspect_field_ordering_value(projection, child, field.locator())
                    })
                    .collect::<Option<Vec<_>>>()
            })
            .flatten();
        if let Some(values) = values {
            entries
                .entry(parent)
                .or_default()
                .push(RelatedEntityOrderingEntry::new(
                    values,
                    child,
                    relation.relation_id,
                ));
        }
    });
    for rows in entries.values_mut() {
        rows.sort_by(|left, right| compare_related_entries(left, right, contract.ordering));
    }
    entries
}

pub(in crate::indexes) fn compare_related_entries(
    left: &RelatedEntityOrderingEntry,
    right: &RelatedEntityOrderingEntry,
    ordering: &[RelatedEntityOrderingField],
) -> Ordering {
    for ((left, right), field) in left
        .ordering_values()
        .iter()
        .zip(right.ordering_values())
        .zip(ordering)
    {
        let comparison = match field.direction() {
            RelatedEntityOrderingDirection::Ascending => left.value().cmp(right.value()),
            RelatedEntityOrderingDirection::Descending => right.value().cmp(left.value()),
        };
        if comparison != Ordering::Equal {
            return comparison;
        }
    }
    left.child_entity_id()
        .cmp(&right.child_entity_id())
        .then_with(|| left.relation_id().cmp(&right.relation_id()))
}
