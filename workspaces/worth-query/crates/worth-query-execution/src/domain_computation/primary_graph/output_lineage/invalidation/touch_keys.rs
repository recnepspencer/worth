use worth_relational::facade::history::RelationalDescriptiveTouch;
use worth_relational::facade::history::RelationalTouchAdjacencyDirection;
use worth_relational::facade::transactions::RecordRef;

use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAdjacencyDirection;

use super::fact_key::FactPostingKey;

/// Lower one native change into reverse-index addresses. The visitor admits
/// each address before it is materialized, including its owned key payload.
pub(super) fn visit<E>(
    touch: &RelationalDescriptiveTouch,
    mut emit: impl FnMut(FactPostingKey) -> Result<(), E>,
) -> Result<(), E> {
    use FactPostingKey as Key;
    use RelationalDescriptiveTouch as Touch;
    match touch {
        Touch::EntityLifecycle { entity, .. } => {
            emit(Key::EntityLifecycle(*entity))?;
        }
        Touch::RelationLifecycle { .. } => {}
        Touch::AspectRevision { record, aspect } => {
            if let RecordRef::Entity(entity) = record {
                emit(Key::AspectRevision {
                    entity: *entity,
                    aspect: aspect.clone(),
                })?;
            }
        }
        Touch::FieldRevision {
            record,
            aspect,
            path,
            ..
        } => {
            if let RecordRef::Entity(entity) = record {
                emit(Key::FieldRevision {
                    entity: *entity,
                    aspect: aspect.clone(),
                    path: path.clone(),
                })?;
            }
        }
        Touch::RelationMembership {
            kind,
            source,
            target,
            ..
        } => {
            emit(Key::RelationMembership {
                kind: *kind,
                from: *source,
                to: *target,
            })?;
        }
        Touch::AdjacencyRevision {
            kind,
            anchor,
            direction,
        } => {
            emit(Key::Adjacency {
                kind: *kind,
                anchor: *anchor,
                direction: match direction {
                    RelationalTouchAdjacencyDirection::Outgoing => {
                        WorthQueryApplicationAdjacencyDirection::Outgoing
                    }
                    RelationalTouchAdjacencyDirection::Incoming => {
                        WorthQueryApplicationAdjacencyDirection::Incoming
                    }
                },
            })?;
        }
        Touch::IndexMembership {
            index,
            kind,
            aspect,
            path,
            old_key,
            new_key,
        } => {
            for key in old_key.iter().chain(new_key.iter()) {
                emit(Key::IndexMembership {
                    index: *index,
                    kind: *kind,
                    aspect: aspect.clone(),
                    path: path.clone(),
                    key: key.clone(),
                })?;
            }
        }
        Touch::IndexDefinition { index } => {
            emit(Key::IndexDefinition(*index))?;
        }
    }
    Ok(())
}
