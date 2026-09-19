use std::collections::BTreeSet;

use crate::identity::data::{EntityId, RelationId};
use crate::transactions::data::EntityReference;
use crate::validation::data::CustomInvariantAccessContract;
use crate::validation::engine::state_view::InvariantStateView;

pub(super) fn include_affected_reference(
    visible_entities: &mut BTreeSet<EntityId>,
    entity_reference: &EntityReference,
    state_view: &InvariantStateView<'_>,
    before_image_view: Option<&InvariantStateView<'_>>,
    access: &CustomInvariantAccessContract,
) {
    if let EntityReference::Existing(entity_id) = entity_reference {
        include_affected_entity(
            visible_entities,
            *entity_id,
            state_view,
            before_image_view,
            access,
        );
    }
}

pub(super) fn include_affected_entity(
    visible_entities: &mut BTreeSet<EntityId>,
    entity_id: EntityId,
    state_view: &InvariantStateView<'_>,
    before_image_view: Option<&InvariantStateView<'_>>,
    access: &CustomInvariantAccessContract,
) -> bool {
    let affected = state_view
        .entity_metadata(entity_id)
        .or_else(|| before_image_view.and_then(|view| view.entity_metadata(entity_id)))
        .is_some_and(|metadata| access.affects_entity(metadata.kind_id));
    if affected {
        visible_entities.insert(entity_id);
    }
    affected
}

pub(super) fn include_affected_relation(
    visible_entities: &mut BTreeSet<EntityId>,
    visible_relations: &mut BTreeSet<RelationId>,
    relation_id: RelationId,
    state_view: &InvariantStateView<'_>,
    before_image_view: Option<&InvariantStateView<'_>>,
    access: &CustomInvariantAccessContract,
) -> bool {
    let metadata = state_view
        .relation_metadata(relation_id)
        .or_else(|| before_image_view.and_then(|view| view.relation_metadata(relation_id)));
    let Some(metadata) = metadata else {
        return false;
    };
    include_affected_entity(
        visible_entities,
        metadata.source,
        state_view,
        before_image_view,
        access,
    );
    include_affected_entity(
        visible_entities,
        metadata.target,
        state_view,
        before_image_view,
        access,
    );
    if access.affects_relation(metadata.kind_id) {
        visible_relations.insert(relation_id);
        return true;
    }
    false
}
