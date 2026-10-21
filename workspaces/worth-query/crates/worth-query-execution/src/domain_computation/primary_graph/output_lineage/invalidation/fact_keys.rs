use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::AspectKey;
use worth_foundational::facade::CanonicalFieldPath;
use worth_relational::facade::storage::AuthoritativeFieldComparisonKey;

use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAdjacencyDirection as Direction;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact as Fact;

use super::fact_key::FactPostingKey as Key;
use super::mark_state::FullVerificationReason;

/// A retained fact is either addressable by native touches or requires a
/// bounded full verification. Admission failure is never an empty key set.
#[derive(Debug)]
pub(super) enum FactKeyProjectionStop<E> {
    FullVerificationRequired(FullVerificationReason),
    Admission(E),
}

struct Projection<F> {
    emit: F,
}

impl<E, F> Projection<F>
where
    F: FnMut(Key) -> Result<(), E>,
{
    fn key(&mut self, build: impl FnOnce() -> Key) -> Result<(), FactKeyProjectionStop<E>> {
        (self.emit)(build()).map_err(FactKeyProjectionStop::Admission)
    }
}

/// Visit all reverse addresses for one retained fact ordinal. `admit` receives
/// initialized traversal work and owned key payload bytes before any key
/// cloning or native comparison-key encoding. The posting index admits its
/// Arc/map cells separately before it consumes `emit`.
pub(super) fn visit<E>(
    fact: &Fact,
    emit: impl FnMut(Key) -> Result<(), E>,
) -> Result<(), FactKeyProjectionStop<E>> {
    visit_projected(fact, &mut Projection { emit })
}

fn visit_projected<E, F>(
    fact: &Fact,
    projection: &mut Projection<F>,
) -> Result<(), FactKeyProjectionStop<E>>
where
    F: FnMut(Key) -> Result<(), E>,
{
    match fact {
        Fact::SourcePublication { observation } => {
            projection.key(|| Key::SourcePublication(observation.address()))?;
            for fact in observation.posting_facts() {
                visit_projected(fact, projection)?;
            }
            // Completion and source-change wake are indexed. General observation
            // continuity is deliberately not certified by physical postings.
            return Err(FactKeyProjectionStop::FullVerificationRequired(
                FullVerificationReason::UnsupportedFact,
            ));
        }
        Fact::SourceEntity { entity_id } => {
            projection.key(|| Key::EntityLifecycle(*entity_id))?;
        }
        Fact::SourceAspectRevision {
            entity_id, aspect, ..
        }
        | Fact::AspectPresence {
            entity_id, aspect, ..
        } => {
            projection.key(|| Key::EntityLifecycle(*entity_id))?;
            projection.key(|| Key::AspectRevision {
                entity: *entity_id,
                aspect: aspect.clone(),
            })?;
        }
        Fact::SourceFieldRevision {
            entity_id, locator, ..
        } => {
            projection.key(|| Key::EntityLifecycle(*entity_id))?;
            field_addresses(*entity_id, locator, projection)?;
        }
        Fact::SourceAdjacencyRevision {
            relation_kind,
            anchor,
            direction,
            endpoints,
            ..
        } => {
            projection.key(|| Key::EntityLifecycle(*anchor))?;
            projection.key(|| Key::Adjacency {
                kind: *relation_kind,
                anchor: *anchor,
                direction: native_direction(*direction),
            })?;

            for endpoint in endpoints {
                projection.key(|| Key::EntityLifecycle(*endpoint))?;
            }
        }
        Fact::Entity { entity_id, .. } => {
            projection.key(|| Key::EntityLifecycle(*entity_id))?;
        }
        Fact::OptionalAspectAbsent {
            entity_id, locator, ..
        } => {
            projection.key(|| Key::EntityLifecycle(*entity_id))?;
            let aspect = locator.aspect().aspect_key();
            projection.key(|| Key::AspectRevision {
                entity: *entity_id,
                aspect: aspect.clone(),
            })?;
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
            projection.key(|| Key::EntityLifecycle(*entity_id))?;
            field_addresses(*entity_id, locator, projection)?;
        }
        Fact::Relation {
            relation_kind,
            from,
            to,
            ..
        } => {
            projection.key(|| Key::EntityLifecycle(*from))?;
            projection.key(|| Key::EntityLifecycle(*to))?;
            projection.key(|| Key::RelationMembership {
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
            projection.key(|| Key::EntityLifecycle(*anchor))?;
            projection.key(|| Key::Adjacency {
                kind: *relation_kind,
                anchor: *anchor,
                direction: *direction,
            })?;

            for relation in relations {
                projection.key(|| Key::EntityLifecycle(relation.from))?;
                projection.key(|| Key::EntityLifecycle(relation.to))?;
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
            projection.key(|| Key::IndexDefinition(*index_id))?;
            let (aspect, path) = locator_parts(locator);
            projection.key(|| {
                let key = AuthoritativeFieldComparisonKey::from_aspect_value(value);
                Key::IndexMembership {
                    index: *index_id,
                    kind: *entity_kind,
                    aspect: aspect.clone(),
                    path: path.clone(),
                    key,
                }
            })?;

            for candidate in candidates {
                projection.key(|| Key::EntityLifecycle(*candidate))?;
            }
        }
        // A retirement stops holding when its entity slot is reused, and the
        // commit that reuses it names only the new generation. No posting
        // marks that, so the output verifies in full.
        Fact::RetiredOutputEntity { .. } => {
            return Err(FactKeyProjectionStop::FullVerificationRequired(
                FullVerificationReason::UnsupportedFact,
            ));
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

fn locator_parts(locator: &AspectFieldLocator) -> (&AspectKey, &CanonicalFieldPath) {
    (locator.aspect().aspect_key(), locator.field_path())
}

fn field_addresses<E, F>(
    entity: worth_relational::facade::identity::EntityId,
    locator: &AspectFieldLocator,
    projection: &mut Projection<F>,
) -> Result<(), FactKeyProjectionStop<E>>
where
    F: FnMut(Key) -> Result<(), E>,
{
    let (aspect, path) = locator_parts(locator);
    projection.key(|| Key::FieldRevision {
        entity,
        aspect: aspect.clone(),
        path: path.clone(),
    })?;
    Ok(())
}

#[cfg(test)]
#[path = "fact_keys/tests.rs"]
mod tests;
