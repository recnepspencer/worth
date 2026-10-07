use std::collections::BTreeMap;
use worth_execution::MapKernelFailure;

use crate::history::data::{
    RelationalDescriptiveTouch as Touch, RelationalTouchAdjacencyDirection as Direction,
};
use crate::identity::data::{EntityId, PartitionId, RelationId};
use crate::storage::substrate::{EntityRecordKind, RecordArena, RecordKind, RelationRecordKind};
use crate::transactions::data::RecordRef;

use super::super::{Failure, TouchBudget};
use super::push;

pub(super) fn entity(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    selected: &crate::branch::SelectedRelationalBranchState,
    working: &crate::runtime::WorkingState,
    partition: PartitionId,
    slot: usize,
    old: Option<&RecordArena<EntityRecordKind>>,
    new: &RecordArena<EntityRecordKind>,
    touches: &mut Vec<Touch>,
    budget: &mut TouchBudget<'_, '_, '_, '_>,
) -> Result<bool, Failure> {
    record(
        runtime,
        selected,
        working,
        partition,
        slot,
        old,
        new,
        |p, s, g| RecordRef::Entity(EntityId::new(p, s as u64, g)),
        touches,
        budget,
    )
}

pub(super) fn relation(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    selected: &crate::branch::SelectedRelationalBranchState,
    working: &crate::runtime::WorkingState,
    partition: PartitionId,
    slot: usize,
    old: Option<&RecordArena<RelationRecordKind>>,
    new: &RecordArena<RelationRecordKind>,
    touches: &mut Vec<Touch>,
    budget: &mut TouchBudget<'_, '_, '_, '_>,
) -> Result<bool, Failure> {
    let old_view = old.and_then(|arena| arena.get_slot(slot));
    let new_view = new.get_slot(slot);
    if old_view
        .as_ref()
        .is_some_and(|view| view.is_materialization_unavailable())
        || new_view
            .as_ref()
            .is_some_and(|view| view.is_materialization_unavailable())
    {
        return Ok(false);
    }
    let old_live = old_view.as_ref().filter(|view| view.is_live());
    let new_live = new_view.as_ref().filter(|view| view.is_live());
    let old_identity =
        old_live.map(|view| RelationId::new(partition, slot as u64, view.generation()));
    let new_identity =
        new_live.map(|view| RelationId::new(partition, slot as u64, view.generation()));
    let old_endpoints = old_live.and_then(|view| view.extra().endpoints.as_ref());
    let new_endpoints = new_live.and_then(|view| view.extra().endpoints.as_ref());
    let old_kind = old_live.and_then(|view| view.kind_id());
    let new_kind = new_live.and_then(|view| view.kind_id());
    if (old_live.is_some() && (old_endpoints.is_none() || old_kind.is_none()))
        || (new_live.is_some() && (new_endpoints.is_none() || new_kind.is_none()))
    {
        return Ok(false);
    }
    if old_identity != new_identity || old_endpoints != new_endpoints || old_kind != new_kind {
        for (id, kind, endpoints) in [
            old_identity
                .zip(old_kind)
                .zip(old_endpoints)
                .map(|((id, kind), endpoints)| (id, kind, endpoints)),
            new_identity
                .zip(new_kind)
                .zip(new_endpoints)
                .map(|((id, kind), endpoints)| (id, kind, endpoints)),
        ]
        .into_iter()
        .flatten()
        {
            budget.checkpoint(1)?;
            push(
                touches,
                Touch::RelationMembership {
                    relation: id,
                    kind,
                    source: endpoints.source,
                    target: endpoints.target,
                },
                budget,
            )?;
            push(
                touches,
                Touch::AdjacencyRevision {
                    kind,
                    anchor: endpoints.source,
                    direction: Direction::Outgoing,
                },
                budget,
            )?;
            push(
                touches,
                Touch::AdjacencyRevision {
                    kind,
                    anchor: endpoints.target,
                    direction: Direction::Incoming,
                },
                budget,
            )?;
        }
    }
    record(
        runtime,
        selected,
        working,
        partition,
        slot,
        old,
        new,
        |p, s, g| RecordRef::Relation(RelationId::new(p, s as u64, g)),
        touches,
        budget,
    )
}

fn record<K: RecordKind>(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    selected: &crate::branch::SelectedRelationalBranchState,
    working: &crate::runtime::WorkingState,
    partition: PartitionId,
    slot: usize,
    old: Option<&RecordArena<K>>,
    new: &RecordArena<K>,
    target: impl Fn(PartitionId, usize, u32) -> RecordRef,
    touches: &mut Vec<Touch>,
    budget: &mut TouchBudget<'_, '_, '_, '_>,
) -> Result<bool, Failure> {
    budget.checkpoint(2)?;
    let old_view = old.and_then(|arena| arena.get_slot(slot));
    let new_view = new.get_slot(slot);
    if old_view
        .as_ref()
        .is_some_and(|view| view.is_materialization_unavailable())
        || new_view
            .as_ref()
            .is_some_and(|view| view.is_materialization_unavailable())
    {
        return Ok(false);
    }
    let old_live = old_view.as_ref().filter(|view| view.is_live());
    let new_live = new_view.as_ref().filter(|view| view.is_live());
    let old_ref = old_live.map(|view| target(partition, slot, view.generation()));
    let new_ref = new_live.map(|view| target(partition, slot, view.generation()));
    let old_kind = old_live.and_then(|view| view.kind_id());
    let new_kind = new_live.and_then(|view| view.kind_id());
    if (old_live.is_some() && old_kind.is_none()) || (new_live.is_some() && new_kind.is_none()) {
        return Ok(false);
    }
    if old_ref != new_ref || old_kind != new_kind {
        for (record, kind) in [
            old_ref.as_ref().zip(old_kind),
            new_ref.as_ref().zip(new_kind),
        ]
        .into_iter()
        .flatten()
        {
            budget.checkpoint(1)?;
            let touch = match record {
                RecordRef::Entity(entity) => Touch::EntityLifecycle {
                    entity: *entity,
                    kind,
                },
                RecordRef::Relation(relation) => Touch::RelationLifecycle {
                    relation: *relation,
                    kind,
                },
            };
            push(touches, touch, budget)?;
        }
    }
    let empty_versions = BTreeMap::new();
    let empty_fields = BTreeMap::new();
    let old_versions = old_live
        .and_then(|_| old.and_then(|arena| arena.aspect_versions_at(slot)))
        .unwrap_or(&empty_versions);
    let new_versions = new_live
        .and_then(|_| new.aspect_versions_at(slot))
        .unwrap_or(&empty_versions);
    let old_fields = old_live
        .and_then(|_| old.and_then(|arena| arena.field_revisions_at(slot)))
        .unwrap_or(&empty_fields);
    let new_fields = new_live
        .and_then(|_| new.field_revisions_at(slot))
        .unwrap_or(&empty_fields);
    if (old_live.is_some()
        && old
            .and_then(|arena| arena.aspect_versions_at(slot))
            .is_none())
        || (new_live.is_some() && new.aspect_versions_at(slot).is_none())
        || (old_live.is_some()
            && old
                .and_then(|arena| arena.field_revisions_at(slot))
                .is_none())
        || (new_live.is_some() && new.field_revisions_at(slot).is_none())
    {
        return Ok(false);
    }
    let Some(record) = new_ref.as_ref().or(old_ref.as_ref()) else {
        return Ok(true);
    };
    let walk = lookup_work(old_versions.len(), new_versions.len())?
        .checked_add(lookup_work(old_fields.len(), new_fields.len())?)
        .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
    budget.checkpoint(walk)?;
    for (&symbol, old_revision) in old_versions {
        if Some(old_revision) != new_versions.get(&symbol)
            && !append_aspect(
                runtime,
                symbol,
                if new_versions.contains_key(&symbol) {
                    new_ref.as_ref().unwrap_or(record)
                } else {
                    old_ref.as_ref().unwrap_or(record)
                },
                touches,
                budget,
            )?
        {
            return Ok(false);
        }
    }
    for &symbol in new_versions.keys() {
        if !old_versions.contains_key(&symbol)
            && !append_aspect(runtime, symbol, record, touches, budget)?
        {
            return Ok(false);
        }
    }
    for (&symbols, old_revision) in old_fields {
        if Some(old_revision) != new_fields.get(&symbols)
            && !append_field(
                runtime,
                FieldDifference {
                    selected,
                    working,
                    symbols,
                    old_record: old_ref.as_ref(),
                    new_record: new_ref.as_ref(),
                    old_kind,
                    new_kind,
                    new_revision: new_fields.get(&symbols),
                },
                touches,
                budget,
            )?
        {
            return Ok(false);
        }
    }
    for (&symbols, new_revision) in new_fields {
        if !old_fields.contains_key(&symbols)
            && !append_field(
                runtime,
                FieldDifference {
                    selected,
                    working,
                    symbols,
                    old_record: old_ref.as_ref(),
                    new_record: new_ref.as_ref(),
                    old_kind,
                    new_kind,
                    new_revision: Some(new_revision),
                },
                touches,
                budget,
            )?
        {
            return Ok(false);
        }
    }
    Ok(true)
}

mod revisions;
use revisions::{append_aspect, append_field, FieldDifference};

fn lookup_work(old: usize, new: usize) -> Result<u64, Failure> {
    let old = u64::try_from(old).map_err(|_| MapKernelFailure::ResultCapacityExceeded)?;
    let new = u64::try_from(new).map_err(|_| MapKernelFailure::ResultCapacityExceeded)?;
    let old_search = u64::from(usize::BITS - (new as usize).saturating_add(1).leading_zeros());
    let new_search = u64::from(usize::BITS - (old as usize).saturating_add(1).leading_zeros());
    old.checked_mul(old_search.max(1))
        .and_then(|count| count.checked_add(new.checked_mul(new_search.max(1))?))
        .ok_or(MapKernelFailure::ResultCapacityExceeded)
}
