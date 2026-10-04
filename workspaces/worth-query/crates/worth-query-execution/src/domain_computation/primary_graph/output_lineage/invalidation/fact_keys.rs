use worth_foundational::facade::{AspectFieldLocator, AspectKey, CanonicalFieldPath};
use worth_relational::facade::storage::AuthoritativeFieldComparisonKey;

use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAdjacencyDirection as Direction, WorthQueryApplicationObservedFact as Fact,
};

use super::{fact_key::FactPostingKey as Key, mark_state::FullVerificationReason};

/// A retained fact is either addressable by native touches or requires a
/// bounded full verification. Admission failure is never an empty key set.
#[derive(Debug)]
pub(super) enum FactKeyProjectionStop<E> {
    FullVerificationRequired(FullVerificationReason),
    Admission(E),
}

struct Projection<A, F> {
    admit: A,
    emit: F,
}

impl<E, A, F> Projection<A, F>
where
    A: FnMut(usize, usize) -> Result<(), E>,
    F: FnMut(Key) -> Result<(), E>,
{
    fn work(&mut self, units: usize) -> Result<(), FactKeyProjectionStop<E>> {
        (self.admit)(units, 0).map_err(FactKeyProjectionStop::Admission)
    }

    fn key(
        &mut self,
        work: usize,
        bytes: usize,
        build: impl FnOnce() -> Key,
    ) -> Result<(), FactKeyProjectionStop<E>> {
        (self.admit)(work, bytes).map_err(FactKeyProjectionStop::Admission)?;
        (self.emit)(build()).map_err(FactKeyProjectionStop::Admission)
    }
}

/// Visit all reverse addresses for one retained fact ordinal. `admit` receives
/// initialized traversal work and owned key payload bytes before any key
/// cloning or native comparison-key encoding. The posting index admits its
/// Arc/map cells separately before it consumes `emit`.
pub(super) fn visit<E>(
    fact: &Fact,
    admit: impl FnMut(usize, usize) -> Result<(), E>,
    emit: impl FnMut(Key) -> Result<(), E>,
) -> Result<(), FactKeyProjectionStop<E>> {
    let mut projection = Projection { admit, emit };
    match fact {
        Fact::SourceEntity { entity_id } => {
            projection.key(1, 0, || Key::EntityLifecycle(*entity_id))?;
        }
        Fact::SourceAspectRevision {
            entity_id, aspect, ..
        } => {
            projection.key(1, 0, || Key::EntityLifecycle(*entity_id))?;
            let bytes = aspect.owned_allocation_capacity_bytes();
            projection.key(bytes.saturating_add(1), bytes, || Key::AspectRevision {
                entity: *entity_id,
                aspect: aspect.clone(),
            })?;
        }
        Fact::SourceFieldRevision {
            entity_id, locator, ..
        } => {
            projection.key(1, 0, || Key::EntityLifecycle(*entity_id))?;
            field_addresses(*entity_id, locator, &mut projection)?;
        }
        Fact::SourceAdjacencyRevision {
            relation_kind,
            anchor,
            direction,
            endpoints,
            ..
        } => {
            projection.key(1, 0, || Key::EntityLifecycle(*anchor))?;
            projection.key(1, 0, || Key::Adjacency {
                kind: *relation_kind,
                anchor: *anchor,
                direction: native_direction(*direction),
            })?;
            projection.work(endpoints.len())?;
            for endpoint in endpoints {
                projection.key(1, 0, || Key::EntityLifecycle(*endpoint))?;
            }
        }
        Fact::Entity { entity_id, .. } => {
            projection.key(1, 0, || Key::EntityLifecycle(*entity_id))?;
        }
        Fact::Field {
            entity_id,
            kind: _,
            locator,
            ..
        }
        | Fact::AbsentField {
            entity_id,
            kind: _,
            locator,
        } => {
            projection.key(1, 0, || Key::EntityLifecycle(*entity_id))?;
            field_addresses(*entity_id, locator, &mut projection)?;
        }
        Fact::Relation {
            relation_kind,
            from,
            to,
            ..
        } => {
            projection.key(1, 0, || Key::EntityLifecycle(*from))?;
            projection.key(1, 0, || Key::EntityLifecycle(*to))?;
            projection.key(1, 0, || Key::RelationMembership {
                kind: *relation_kind,
                from: *from,
                to: *to,
            })?;
        }
        Fact::Adjacency {
            relation_kind,
            anchor,
            direction,
            relations,
            ..
        } => {
            projection.key(1, 0, || Key::EntityLifecycle(*anchor))?;
            projection.key(1, 0, || Key::Adjacency {
                kind: *relation_kind,
                anchor: *anchor,
                direction: *direction,
            })?;
            projection.work(relations.len())?;
            for relation in relations {
                projection.key(1, 0, || Key::EntityLifecycle(relation.from))?;
                projection.key(1, 0, || Key::EntityLifecycle(relation.to))?;
            }
        }
        Fact::IndexedEntitySelection {
            index_id,
            entity_kind,
            locator,
            value,
            candidates,
            ..
        } => {
            projection.key(1, 0, || Key::IndexDefinition(*index_id))?;
            let key_bytes = AuthoritativeFieldComparisonKey::required_encoded_capacity_bytes(value)
                .and_then(|bytes| usize::try_from(bytes).ok())
                .ok_or(FactKeyProjectionStop::FullVerificationRequired(
                    FullVerificationReason::UnsupportedFact,
                ))?;
            let (aspect, path, path_bytes) = locator_parts(locator, &mut projection)?;
            let owned = key_bytes
                .checked_add(path_bytes)
                .and_then(|bytes| bytes.checked_add(aspect.owned_allocation_capacity_bytes()))
                .ok_or(FactKeyProjectionStop::FullVerificationRequired(
                    FullVerificationReason::UnsupportedFact,
                ))?;
            // The codec is the native index owner. Its encoded length is
            // admitted before it allocates; the produced key is moved once.
            projection.key(owned.saturating_add(1), owned, || {
                let key = AuthoritativeFieldComparisonKey::from_aspect_value(value);
                Key::IndexMembership {
                    index: *index_id,
                    kind: *entity_kind,
                    aspect: aspect.clone(),
                    path: path.clone(),
                    key,
                }
            })?;
            projection.work(candidates.len())?;
            for candidate in candidates {
                projection.key(1, 0, || Key::EntityLifecycle(*candidate))?;
            }
        }
        // No output's decision reads workflow definition or capacity truth:
        // workflow attempts publish without an output binding. One that did
        // would have no posting to mark it, so it verifies in full.
        Fact::WorkflowDefinitionPredecessor { .. }
        | Fact::WorkflowDefinitionCurrent { .. }
        | Fact::WorkflowInstanceCapacity { .. } => {
            return Err(FactKeyProjectionStop::FullVerificationRequired(
                FullVerificationReason::UnsupportedFact,
            ));
        }
        // The basis names immutable native history. Its own retained lease is
        // checked at reuse and later commits cannot mutate that observation.
        Fact::WorkflowHistoryBasis { .. } => {}
    }
    Ok(())
}

fn native_direction(
    direction: worth_relational::facade::runtime::RelationalAdjacencyDirection,
) -> Direction {
    match direction {
        worth_relational::facade::runtime::RelationalAdjacencyDirection::Outgoing => {
            Direction::Outgoing
        }
        worth_relational::facade::runtime::RelationalAdjacencyDirection::Incoming => {
            Direction::Incoming
        }
    }
}

fn locator_parts<'a, E, A, F>(
    locator: &'a AspectFieldLocator,
    projection: &mut Projection<A, F>,
) -> Result<(&'a AspectKey, &'a CanonicalFieldPath, usize), FactKeyProjectionStop<E>>
where
    A: FnMut(usize, usize) -> Result<(), E>,
    F: FnMut(Key) -> Result<(), E>,
{
    let aspect = locator.aspect().aspect_key();
    let path = locator.field_path();
    projection.work(path.fields().len().saturating_add(1))?;
    Ok((aspect, path, path.owned_allocation_capacity_bytes()))
}

fn field_addresses<E, A, F>(
    entity: worth_relational::facade::identity::EntityId,
    locator: &AspectFieldLocator,
    projection: &mut Projection<A, F>,
) -> Result<(), FactKeyProjectionStop<E>>
where
    A: FnMut(usize, usize) -> Result<(), E>,
    F: FnMut(Key) -> Result<(), E>,
{
    let (aspect, path, path_bytes) = locator_parts(locator, projection)?;
    let bytes = aspect
        .owned_allocation_capacity_bytes()
        .checked_add(path_bytes)
        .ok_or(FactKeyProjectionStop::FullVerificationRequired(
            FullVerificationReason::UnsupportedFact,
        ))?;
    projection.key(bytes.saturating_add(1), bytes, || Key::FieldRevision {
        entity,
        aspect: aspect.clone(),
        path: path.clone(),
    })?;
    Ok(())
}

#[cfg(test)]
#[path = "fact_keys/tests.rs"]
mod tests;
