use std::collections::BTreeSet;

use crate::identity::data::{EntityId, PartitionId};
use crate::transactions::data::{
    CreateIntent, EntityMutationIntent, EntityReference, MergedCommitPlan, MutationIntent,
    RelationMutationIntent,
};
use crate::validation::engine::state_view::{InvariantStateView, VisibleRelationMetadata};

use crate::validation::data::{
    PlannedEntityCreate, PlannedRelationCreate, PlannedRelationEndpointUpdate, TouchedStructuralSet,
};

pub(crate) fn collect_touched_structural_set(
    state_view: &InvariantStateView<'_>,
    merged_plan: Option<&MergedCommitPlan>,
    work: &super::CustomInvariantWorkMeter,
) -> TouchedStructuralSet {
    let mut visible_entities = BTreeSet::new();
    let mut visible_relations = BTreeSet::new();
    let mut touched_partitions = BTreeSet::new();
    let mut planned_entity_deletes = Vec::new();
    let mut planned_entity_creates = Vec::new();
    let mut planned_relation_creates = Vec::new();
    let mut planned_relation_deletes = Vec::new();
    let mut planned_relation_endpoint_updates = Vec::new();

    if let Some(ids) =
        state_view.touched_visible_entity_ids_with_budget(|units| work.try_charge(units))
    {
        visible_entities.extend(ids);
    }
    if let Some(ids) =
        state_view.touched_visible_relation_ids_with_budget(|units| work.try_charge(units))
    {
        visible_relations.extend(ids);
    }

    // A sparse relation overlay can materialize a touched relation without
    // materializing either endpoint's partition.  Seed the structural scope
    // from the relation metadata before walking adjacency so custom rules see
    // the complete selected relation boundary without enumerating the root.
    let touched_relation_ids = visible_relations.iter().copied().collect::<Vec<_>>();
    for relation_id in touched_relation_ids {
        if !work.try_charge(1) {
            break;
        }
        if let Some(metadata) = state_view.relation_metadata(relation_id) {
            include_relation_metadata(&mut visible_entities, &mut touched_partitions, metadata);
        }
    }

    if let Some(plan) = merged_plan {
        for intent in &plan.merged_intents {
            if !work.try_charge(1) {
                break;
            }
            intent.seed_touched_partitions(&mut touched_partitions);
            match intent {
                MutationIntent::Create(CreateIntent::Entity(spec)) => {
                    planned_entity_creates.push(PlannedEntityCreate::new(
                        spec.partition_id,
                        spec.kind_id,
                        spec.client_key.clone(),
                    ));
                }
                MutationIntent::Create(CreateIntent::EntityAspects(spec)) => {
                    planned_entity_creates.push(PlannedEntityCreate::new(
                        spec.partition_id,
                        spec.kind_id,
                        spec.client_key.clone(),
                    ));
                }
                MutationIntent::Create(CreateIntent::BulkEntities(spec)) => {
                    if !work.try_charge(spec.client_keys.len()) {
                        break;
                    }
                    for client_key in spec.client_keys.iter() {
                        planned_entity_creates.push(PlannedEntityCreate::new(
                            spec.partition_id,
                            spec.kind_id,
                            client_key.clone(),
                        ));
                    }
                }
                MutationIntent::Create(CreateIntent::Relation(spec)) => {
                    include_existing_entity_reference(&mut visible_entities, &spec.source);
                    include_existing_entity_reference(&mut visible_entities, &spec.target);
                    planned_relation_creates.push(PlannedRelationCreate::new(
                        spec.partition_id,
                        spec.kind_id,
                        spec.client_key.clone(),
                        spec.source.clone(),
                        spec.target.clone(),
                    ));
                }
                MutationIntent::Create(CreateIntent::RelationAspects(spec)) => {
                    include_existing_entity_reference(&mut visible_entities, &spec.source);
                    include_existing_entity_reference(&mut visible_entities, &spec.target);
                    planned_relation_creates.push(PlannedRelationCreate::new(
                        spec.partition_id,
                        spec.kind_id,
                        spec.client_key.clone(),
                        spec.source.clone(),
                        spec.target.clone(),
                    ));
                }
                MutationIntent::Create(CreateIntent::BulkRelations(spec)) => {
                    if !work.try_charge(spec.client_keys.len()) {
                        break;
                    }
                    for ((source, target), client_key) in
                        spec.endpoints.iter().zip(spec.client_keys.iter())
                    {
                        include_existing_entity_reference(&mut visible_entities, source);
                        include_existing_entity_reference(&mut visible_entities, target);
                        planned_relation_creates.push(PlannedRelationCreate::new(
                            spec.partition_id,
                            spec.kind_id,
                            client_key.clone(),
                            source.clone(),
                            target.clone(),
                        ));
                    }
                }
                MutationIntent::Entity(EntityMutationIntent::UpdateFields(spec)) => {
                    visible_entities.insert(spec.entity_id);
                }
                MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(spec)) => {
                    visible_entities.insert(spec.entity_id);
                }
                MutationIntent::Entity(EntityMutationIntent::Replace(spec)) => {
                    visible_entities.insert(spec.entity_id);
                }
                MutationIntent::Entity(EntityMutationIntent::Delete(spec)) => {
                    visible_entities.insert(spec.entity_id);
                    planned_entity_deletes.push(spec.entity_id);
                }
                MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(spec)) => {
                    visible_relations.insert(spec.relation_id);
                    include_existing_entity_reference(&mut visible_entities, &spec.source);
                    include_existing_entity_reference(&mut visible_entities, &spec.target);
                    planned_relation_endpoint_updates.push(PlannedRelationEndpointUpdate::new(
                        spec.relation_id,
                        spec.kind_id,
                        spec.source.clone(),
                        spec.target.clone(),
                    ));
                    if let Some(metadata) = state_view.relation_metadata(spec.relation_id) {
                        include_relation_metadata(
                            &mut visible_entities,
                            &mut touched_partitions,
                            metadata,
                        );
                    }
                }
                MutationIntent::Relation(RelationMutationIntent::ApplyAspectPatch(spec)) => {
                    visible_relations.insert(spec.relation_id);
                    if let Some(metadata) = state_view.relation_metadata(spec.relation_id) {
                        include_relation_metadata(
                            &mut visible_entities,
                            &mut touched_partitions,
                            metadata,
                        );
                    }
                }
                MutationIntent::Relation(RelationMutationIntent::Delete(spec)) => {
                    visible_relations.insert(spec.relation_id);
                    planned_relation_deletes.push(spec.relation_id);
                    if let Some(metadata) = state_view.relation_metadata(spec.relation_id) {
                        include_relation_metadata(
                            &mut visible_entities,
                            &mut touched_partitions,
                            metadata,
                        );
                    }
                }
            }
        }
    }

    let seed_entities = visible_entities.iter().copied().collect::<Vec<_>>();
    for entity_id in seed_entities {
        if !work.try_charge(1) {
            break;
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
                include_relation_metadata(&mut visible_entities, &mut touched_partitions, metadata);
            }
        }
    }

    TouchedStructuralSet::new(
        visible_entities.into_iter().collect::<Vec<_>>().into(),
        visible_relations.into_iter().collect::<Vec<_>>().into(),
        touched_partitions.into_iter().collect::<Vec<_>>().into(),
        planned_entity_deletes.into(),
        planned_entity_creates.into(),
        planned_relation_creates.into(),
        planned_relation_deletes.into(),
        planned_relation_endpoint_updates.into(),
    )
}

fn include_existing_entity_reference(
    visible_entities: &mut BTreeSet<EntityId>,
    entity_reference: &EntityReference,
) {
    if let EntityReference::Existing(entity_id) = entity_reference {
        visible_entities.insert(*entity_id);
    }
}

fn include_relation_metadata(
    visible_entities: &mut BTreeSet<EntityId>,
    touched_partitions: &mut BTreeSet<PartitionId>,
    metadata: VisibleRelationMetadata,
) {
    visible_entities.insert(metadata.source);
    visible_entities.insert(metadata.target);
    touched_partitions.insert(metadata.relation_id.partition_id);
    touched_partitions.insert(metadata.source.partition_id);
    touched_partitions.insert(metadata.target.partition_id);
}
