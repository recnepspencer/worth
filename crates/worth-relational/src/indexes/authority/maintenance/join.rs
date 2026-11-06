//! Delta joins use persistent leg memberships, including unpaired relations.
use super::{
    change_routing::ChangeRouting, entry_edits::edit, persistent_allocation::UngroupedAllocation,
    reads, record_metadata, work::MaintenanceWork,
};
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::indexes::data::{
    DerivedIndexMaintenanceDenialKind as Denial, JoinLegState, JoinLegsState,
    RelationJoinDefinition, RelationJoinEntry, RelationJoinIndex, RelationJoinKey, RelationJoinLeg,
    RelationJoinSharedEndpoint,
};
use crate::runtime::VisibilityProjectionView;
use std::collections::BTreeSet;

type Coordinate = (EntityId, EntityId);
type Pair = (EntityId, EntityId, EntityId);

/// Cost: changed leg memberships times their reached opposite postings, plus log lookups.
pub(super) fn refresh(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    index: &mut RelationJoinIndex,
    definition: RelationJoinDefinition,
    changes: &ChangeRouting,
    before: Option<&VisibilityProjectionView>,
    after: &VisibilityProjectionView,
    work: &mut MaintenanceWork,
) -> Result<bool, Denial> {
    let previous = match index.legs() {
        Some(legs) => legs.as_ref().clone(),
        None if before.is_none() && !work.prohibits_population_reconstruction() => {
            JoinLegsState::default()
        }
        None => return Err(Denial::ColdReconstructionRequired),
    };
    let mut next = previous.clone();
    let affected = affected_relations(execution, definition, changes, before, after, work)?;
    let mut changed_left = BTreeSet::new();
    let mut changed_right = BTreeSet::new();
    for relation in affected {
        let metadata = record_metadata::relation(execution, Some(after), relation, work)?;
        let left = coordinate(
            execution,
            after,
            metadata,
            definition.left(),
            definition.shared_entity_kind(),
            work,
        )?;
        let right = coordinate(
            execution,
            after,
            metadata,
            definition.right(),
            definition.shared_entity_kind(),
            work,
        )?;
        apply_leg(&mut next.left, relation, left, &mut changed_left, work)?;
        apply_leg(&mut next.right, relation, right, &mut changed_right, work)?;
    }
    if before.is_some() && changed_left.is_empty() && changed_right.is_empty() {
        return Ok(false);
    }
    let mut pairs = BTreeSet::new();
    reached_pairs(
        &changed_left,
        &previous.right,
        &next.right,
        true,
        &mut pairs,
        work,
    )?;
    reached_pairs(
        &changed_right,
        &previous.left,
        &next.left,
        false,
        &mut pairs,
        work,
    )?;
    let mut allocation = UngroupedAllocation::new();
    for (shared, left, right) in pairs {
        let old = row(&previous, shared, left, right, work)?;
        let new = row(&next, shared, left, right, work)?;
        if old == new {
            continue;
        }
        work.derive_row()?;
        let key = RelationJoinKey::new(left, right);
        if let Some(old) = old {
            edit(index, key, old, false, Ord::cmp, &mut allocation, work)?;
        }
        if let Some(new) = new {
            edit(index, key, new, true, Ord::cmp, &mut allocation, work)?;
        }
    }
    work.prepare(
        1,
        (std::mem::size_of::<JoinLegsState>() + 2 * std::mem::size_of::<usize>()) as u64,
    )?;
    index.install_legs(next);
    Ok(true)
}

fn apply_leg(
    leg: &mut JoinLegState,
    relation: RelationId,
    coordinate: Option<Coordinate>,
    changed: &mut BTreeSet<Coordinate>,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    let old = leg.coordinate(relation);
    if old == coordinate {
        return Ok(());
    }
    for key in old.into_iter().chain(coordinate) {
        work.ordered::<Coordinate, ()>(changed.len(), 1, 0)?;
        changed.insert(key);
    }
    let levels = u64::from(usize::BITS - leg.relation_count().leading_zeros()) + 1;
    let node_bytes = (std::mem::size_of::<(RelationId, Coordinate)>() * 64 * 6) as u64;
    work.prepare(
        levels * 64 * 6,
        levels
            .checked_mul(node_bytes)
            .ok_or(Denial::WorkBudgetExceeded)?,
    )?;
    leg.set(relation, coordinate);
    Ok(())
}

fn reached_pairs(
    changed: &BTreeSet<Coordinate>,
    old: &JoinLegState,
    new: &JoinLegState,
    left: bool,
    pairs: &mut BTreeSet<Pair>,
    work: &mut MaintenanceWork,
) -> Result<(), Denial> {
    for (shared, external) in changed {
        // Seek only the reached hub. Unchanged members on this side are never enumerated.
        work.lookup(old.relation_count(), 1)?;
        work.lookup(new.relation_count(), 1)?;
        for (opposite, _) in old.members(*shared).chain(new.members(*shared)) {
            work.charge(1)?;
            work.ordered::<Pair, ()>(pairs.len(), 1, 0)?;
            pairs.insert(if left {
                (*shared, *external, opposite)
            } else {
                (*shared, opposite, *external)
            });
        }
    }
    Ok(())
}

fn row(
    legs: &JoinLegsState,
    shared: EntityId,
    left: EntityId,
    right: EntityId,
    work: &mut MaintenanceWork,
) -> Result<Option<RelationJoinEntry>, Denial> {
    work.lookup(legs.left.relation_count(), 1)?;
    work.lookup(legs.right.relation_count(), 1)?;
    Ok(legs
        .left
        .minimum(shared, left)
        .zip(legs.right.minimum(shared, right))
        .map(|(left, right)| RelationJoinEntry::new(shared, left, right)))
}

fn coordinate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    after: &VisibilityProjectionView,
    metadata: Option<record_metadata::RelationMetadata>,
    leg: RelationJoinLeg,
    shared_kind: KindId,
    work: &mut MaintenanceWork,
) -> Result<Option<Coordinate>, Denial> {
    let Some(relation) = metadata.filter(|r| r.kind == leg.relation_kind()) else {
        return Ok(None);
    };
    let (shared, external) = match leg.shared_endpoint() {
        RelationJoinSharedEndpoint::Source => (relation.source, relation.target),
        RelationJoinSharedEndpoint::Target => (relation.target, relation.source),
    };
    if record_metadata::entity_kind(execution, Some(after), shared, work)? != Some(shared_kind)
        || record_metadata::entity_kind(execution, Some(after), external, work)?
            != Some(leg.external_entity_kind())
    {
        return Ok(None);
    }
    Ok(Some((shared, external)))
}

fn affected_relations(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    definition: RelationJoinDefinition,
    changes: &ChangeRouting,
    before: Option<&VisibilityProjectionView>,
    after: &VisibilityProjectionView,
    work: &mut MaintenanceWork,
) -> Result<BTreeSet<RelationId>, Denial> {
    let mut relations = BTreeSet::new();
    let legs = [definition.left(), definition.right()];
    let kinds = BTreeSet::from([
        definition.shared_entity_kind(),
        legs[0].external_entity_kind(),
        legs[1].external_entity_kind(),
    ]);
    for kind in BTreeSet::from([legs[0].relation_kind(), legs[1].relation_kind()]) {
        for relation in changes.relations(kind, work)? {
            work.ordered::<RelationId, ()>(relations.len(), 1, 0)?;
            relations.insert(relation.id);
        }
    }
    for kind in kinds {
        for entity in changes.entities(kind, work)? {
            if entity.old == entity.new {
                continue;
            }
            let mut directions = BTreeSet::new();
            for leg in legs {
                if kind == definition.shared_entity_kind() {
                    directions.insert((
                        leg.relation_kind(),
                        leg.shared_endpoint() == RelationJoinSharedEndpoint::Source,
                    ));
                }
                if kind == leg.external_entity_kind() {
                    directions.insert((
                        leg.relation_kind(),
                        leg.shared_endpoint() == RelationJoinSharedEndpoint::Target,
                    ));
                }
            }
            for (kind, outgoing) in directions {
                for view in before.into_iter().chain(std::iter::once(after)) {
                    for relation in
                        reads::adjacency(execution, view, entity.id, kind, outgoing, work)?
                    {
                        work.ordered::<RelationId, ()>(relations.len(), 1, 0)?;
                        relations.insert(relation.relation_id);
                    }
                }
            }
        }
    }
    Ok(relations)
}
