//! Seed both real endpoint images, then expand the admitted entity incidence.
use super::metadata::include_relation_metadata;
use crate::identity::data::{EntityId, PartitionId, RelationId};
use crate::validation::custom_rule::CustomInvariantWorkMeter;
use crate::validation::data::CustomInvariantAccessContract;
use crate::validation::engine::state_view::InvariantStateView;
use std::collections::BTreeSet;

pub(super) fn expand(
    visible_entities: &mut BTreeSet<EntityId>,
    visible_relations: &mut BTreeSet<RelationId>,
    touched_partitions: &mut BTreeSet<PartitionId>,
    state_view: &InvariantStateView<'_>,
    before_image_view: Option<&InvariantStateView<'_>>,
    access: &CustomInvariantAccessContract,
    work: &CustomInvariantWorkMeter,
) -> BTreeSet<EntityId> {
    // Sparse overlays may omit, replace or hide one endpoint image. Neither
    // root's population is a substitute for these actual relation identities.
    let touched_relation_ids = visible_relations.iter().copied().collect::<Vec<_>>();
    for relation_id in touched_relation_ids {
        if !work.try_charge(1) {
            break;
        }
        if let Some(metadata) = state_view.relation_metadata(relation_id) {
            include_relation_metadata(visible_entities, touched_partitions, metadata);
        }
        if let Some(before_image) = before_image_view {
            if !work.try_charge(1) {
                break;
            }
            if let Some(metadata) = before_image.relation_metadata(relation_id) {
                include_relation_metadata(visible_entities, touched_partitions, metadata);
            }
        }
    }
    let direct_visible_entities = visible_entities.clone();
    let seed_entities = visible_entities.iter().copied().collect::<Vec<_>>();
    for entity_id in seed_entities {
        if !work.try_charge(1) {
            break;
        }
        if state_view
            .entity_metadata(entity_id)
            .is_some_and(|metadata| !access.affects_entity(metadata.kind_id))
        {
            continue;
        }
        let raw = state_view
            .relation_candidate_count(entity_id, true)
            .saturating_add(state_view.relation_candidate_count(entity_id, false));
        if !work.try_charge(raw.saturating_mul(3)) {
            break;
        }
        for relation_id in state_view.all_relations_for_entity(entity_id) {
            visible_relations.insert(relation_id);
            if let Some(metadata) = state_view.relation_metadata(relation_id) {
                include_relation_metadata(visible_entities, touched_partitions, metadata);
            }
        }
    }
    direct_visible_entities
}
