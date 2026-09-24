use std::collections::BTreeSet;

use crate::identity::data::{EntityId, PartitionId};
use crate::transactions::data::{
    CreateIntent, EntityMutationIntent, MergedCommitPlan, MutationIntent, RelationMutationIntent,
};
use crate::validation::data::{
    PlannedEntityCreate, PlannedRelationCreate, PlannedRelationEndpointUpdate, TouchedStructuralSet,
};
use crate::validation::engine::state_view::{InvariantStateView, VisibleRelationMetadata};

use super::affected_record_filter::{
    include_affected_entity, include_affected_reference, include_affected_relation,
};

pub(crate) fn collect_touched_structural_set(
    state_view: &InvariantStateView<'_>,
    before_image_view: Option<&InvariantStateView<'_>>,
    merged_plan: Option<&MergedCommitPlan>,
    access: &crate::validation::data::CustomInvariantAccessContract,
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
        for entity_id in ids.iter().copied() {
            include_affected_entity(
                &mut visible_entities,
                entity_id,
                state_view,
                before_image_view,
                access,
            );
        }
    }
    if let Some(ids) =
        state_view.touched_visible_relation_ids_with_budget(|units| work.try_charge(units))
    {
        for relation_id in ids.iter().copied() {
            include_affected_relation(
                &mut visible_entities,
                &mut visible_relations,
                relation_id,
                state_view,
                before_image_view,
                access,
            );
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
                    if access.affects_entity(spec.kind_id) {
                        planned_entity_creates.push(PlannedEntityCreate::new(
                            spec.partition_id,
                            spec.kind_id,
                            spec.client_key.clone(),
                        ));
                    }
                }
                MutationIntent::Create(CreateIntent::EntityAspects(spec)) => {
                    if access.affects_entity(spec.kind_id) {
                        planned_entity_creates.push(PlannedEntityCreate::new(
                            spec.partition_id,
                            spec.kind_id,
                            spec.client_key.clone(),
                        ));
                    }
                }
                MutationIntent::Create(CreateIntent::BulkEntities(spec)) => {
                    if access.affects_entity(spec.kind_id) {
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
                }
                MutationIntent::Create(CreateIntent::Relation(spec)) => {
                    include_affected_reference(
                        &mut visible_entities,
                        &spec.source,
                        state_view,
                        before_image_view,
                        access,
                    );
                    include_affected_reference(
                        &mut visible_entities,
                        &spec.target,
                        state_view,
                        before_image_view,
                        access,
                    );
                    if access.affects_relation(spec.kind_id) {
                        planned_relation_creates.push(PlannedRelationCreate::new(
                            spec.partition_id,
                            spec.kind_id,
                            spec.client_key.clone(),
                            spec.source.clone(),
                            spec.target.clone(),
                        ));
                    }
                }
                MutationIntent::Create(CreateIntent::RelationAspects(spec)) => {
                    include_affected_reference(
                        &mut visible_entities,
                        &spec.source,
                        state_view,
                        before_image_view,
                        access,
                    );
                    include_affected_reference(
                        &mut visible_entities,
                        &spec.target,
                        state_view,
                        before_image_view,
                        access,
                    );
                    if access.affects_relation(spec.kind_id) {
                        planned_relation_creates.push(PlannedRelationCreate::new(
                            spec.partition_id,
                            spec.kind_id,
                            spec.client_key.clone(),
                            spec.source.clone(),
                            spec.target.clone(),
                        ));
                    }
                }
                MutationIntent::Create(CreateIntent::BulkRelations(spec)) => {
                    if !work.try_charge(spec.endpoints.len().saturating_mul(2)) {
                        break;
                    }
                    for (source, target) in &spec.endpoints {
                        include_affected_reference(
                            &mut visible_entities,
                            source,
                            state_view,
                            before_image_view,
                            access,
                        );
                        include_affected_reference(
                            &mut visible_entities,
                            target,
                            state_view,
                            before_image_view,
                            access,
                        );
                    }
                    if access.affects_relation(spec.kind_id) {
                        if !work.try_charge(spec.client_keys.len()) {
                            break;
                        }
                        for ((source, target), client_key) in
                            spec.endpoints.iter().zip(spec.client_keys.iter())
                        {
                            planned_relation_creates.push(PlannedRelationCreate::new(
                                spec.partition_id,
                                spec.kind_id,
                                client_key.clone(),
                                source.clone(),
                                target.clone(),
                            ));
                        }
                    }
                }
                MutationIntent::Entity(EntityMutationIntent::UpdateFields(spec)) => {
                    include_affected_entity(
                        &mut visible_entities,
                        spec.entity_id,
                        state_view,
                        before_image_view,
                        access,
                    );
                }
                MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(spec)) => {
                    include_affected_entity(
                        &mut visible_entities,
                        spec.entity_id,
                        state_view,
                        before_image_view,
                        access,
                    );
                }
                MutationIntent::Entity(EntityMutationIntent::Revalidate(spec)) => {
                    include_affected_entity(
                        &mut visible_entities,
                        spec.entity_id,
                        state_view,
                        before_image_view,
                        access,
                    );
                }
                MutationIntent::Entity(EntityMutationIntent::Replace(spec)) => {
                    include_affected_entity(
                        &mut visible_entities,
                        spec.entity_id,
                        state_view,
                        before_image_view,
                        access,
                    );
                }
                MutationIntent::Entity(EntityMutationIntent::Delete(spec)) => {
                    if include_affected_entity(
                        &mut visible_entities,
                        spec.entity_id,
                        state_view,
                        before_image_view,
                        access,
                    ) {
                        planned_entity_deletes.push(spec.entity_id);
                    }
                }
                MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(spec)) => {
                    include_affected_relation(
                        &mut visible_entities,
                        &mut visible_relations,
                        spec.relation_id,
                        state_view,
                        before_image_view,
                        access,
                    );
                    include_affected_reference(
                        &mut visible_entities,
                        &spec.source,
                        state_view,
                        before_image_view,
                        access,
                    );
                    include_affected_reference(
                        &mut visible_entities,
                        &spec.target,
                        state_view,
                        before_image_view,
                        access,
                    );
                    if access.affects_relation(spec.kind_id) {
                        planned_relation_endpoint_updates.push(PlannedRelationEndpointUpdate::new(
                            spec.relation_id,
                            spec.kind_id,
                            spec.source.clone(),
                            spec.target.clone(),
                        ));
                    }
                }
                MutationIntent::Relation(RelationMutationIntent::ApplyAspectPatch(spec)) => {
                    include_affected_relation(
                        &mut visible_entities,
                        &mut visible_relations,
                        spec.relation_id,
                        state_view,
                        before_image_view,
                        access,
                    );
                }
                MutationIntent::Relation(RelationMutationIntent::Delete(spec)) => {
                    if include_affected_relation(
                        &mut visible_entities,
                        &mut visible_relations,
                        spec.relation_id,
                        state_view,
                        before_image_view,
                        access,
                    ) {
                        planned_relation_deletes.push(spec.relation_id);
                    }
                }
                MutationIntent::Materialization(intent) => match intent.record() {
                    crate::transactions::data::RecordRef::Entity(entity_id) => {
                        include_affected_entity(
                            &mut visible_entities,
                            entity_id,
                            state_view,
                            before_image_view,
                            access,
                        );
                    }
                    crate::transactions::data::RecordRef::Relation(relation_id) => {
                        include_affected_relation(
                            &mut visible_entities,
                            &mut visible_relations,
                            relation_id,
                            state_view,
                            before_image_view,
                            access,
                        );
                    }
                },
            }
        }
    }

    // Sparse relation overlays can omit endpoints from their touched entity
    // slots. Retarget and delete overlays can also replace or hide the old
    // endpoints. Seed both sides of the mutation boundary before adjacency
    // expansion, without enumerating either state root.
    let touched_relation_ids = visible_relations.iter().copied().collect::<Vec<_>>();
    for relation_id in touched_relation_ids {
        if !work.try_charge(1) {
            break;
        }
        if let Some(metadata) = state_view.relation_metadata(relation_id) {
            include_relation_metadata(&mut visible_entities, &mut touched_partitions, metadata);
        }
        if let Some(before_image) = before_image_view {
            if !work.try_charge(1) {
                break;
            }
            if let Some(metadata) = before_image.relation_metadata(relation_id) {
                include_relation_metadata(&mut visible_entities, &mut touched_partitions, metadata);
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
                include_relation_metadata(&mut visible_entities, &mut touched_partitions, metadata);
            }
        }
    }

    TouchedStructuralSet::new(
        direct_visible_entities
            .into_iter()
            .collect::<Vec<_>>()
            .into(),
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
