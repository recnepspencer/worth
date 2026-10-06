use worth_relational::facade::{
    history::{RelationalDescriptiveTouch, RelationalTouchAdjacencyDirection},
    transactions::RecordRef,
};

use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAdjacencyDirection;

use super::fact_key::FactPostingKey;

/// Payload capacity for exactly the keys emitted below. No selector is built
/// while measuring it. The caller admits path traversal before this forecast.
pub(super) fn payload_capacity(touch: &RelationalDescriptiveTouch) -> Option<u64> {
    use RelationalDescriptiveTouch as Touch;
    let bytes = match touch {
        Touch::AspectRevision {
            record: RecordRef::Entity(_),
            aspect,
        } => aspect.as_str().len(),
        Touch::FieldRevision {
            record: RecordRef::Entity(_),
            aspect,
            path,
            ..
        } => aspect
            .as_str()
            .len()
            .checked_add(path.owned_allocation_capacity_bytes())?
            .checked_mul(2)?,
        Touch::IndexMembership {
            aspect,
            path,
            old_key,
            new_key,
            ..
        } => {
            let common = aspect
                .as_str()
                .len()
                .checked_add(path.owned_allocation_capacity_bytes())?;
            let mut bytes = 0usize;
            for key in old_key.iter().chain(new_key.iter()) {
                bytes = bytes
                    .checked_add(common)?
                    .checked_add(key.canonical_value_bytes().len())?;
            }
            bytes
        }
        _ => 0,
    };
    u64::try_from(bytes).ok()
}

/// Lower one native change into reverse-index addresses. The visitor admits
/// each address before it is materialized, including its owned key payload.
pub(super) fn visit<E>(
    touch: &RelationalDescriptiveTouch,
    mut admit: impl FnMut(usize) -> Result<(), E>,
    mut emit: impl FnMut(FactPostingKey) -> Result<(), E>,
) -> Result<(), E> {
    use FactPostingKey as Key;
    use RelationalDescriptiveTouch as Touch;
    match touch {
        Touch::EntityLifecycle { entity, kind } => {
            admit(0)?;
            emit(Key::EntityLifecycle(*entity))?;
            admit(0)?;
            emit(Key::EntityKind(*kind))?;
        }
        Touch::RelationLifecycle { kind, .. } => {
            admit(0)?;
            emit(Key::RelationKind(*kind))?;
        }
        Touch::AspectRevision { record, aspect } => {
            if let RecordRef::Entity(entity) = record {
                admit(aspect.as_str().len())?;
                emit(Key::AspectRevision {
                    entity: *entity,
                    aspect: aspect.clone(),
                })?;
            }
        }
        Touch::FieldRevision {
            record,
            kind,
            aspect,
            path,
            ..
        } => {
            if let RecordRef::Entity(entity) = record {
                let bytes = aspect.as_str().len() + path.owned_allocation_capacity_bytes();
                admit(bytes)?;
                emit(Key::FieldRevision {
                    entity: *entity,
                    aspect: aspect.clone(),
                    path: path.clone(),
                })?;
                admit(bytes)?;
                emit(Key::PredicateField {
                    kind: *kind,
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
            admit(0)?;
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
            admit(0)?;
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
                admit(
                    aspect.as_str().len()
                        + path.owned_allocation_capacity_bytes()
                        + key.canonical_value_bytes().len(),
                )?;
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
            admit(0)?;
            emit(Key::IndexDefinition(*index))?;
        }
    }
    Ok(())
}
