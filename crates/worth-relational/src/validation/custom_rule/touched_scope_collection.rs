use crate::authority::commit::semantic_plan::RelationalSemanticPlan;
use std::collections::BTreeSet;

use crate::transactions::data::{
    CreateIntent, EntityMutationIntent, MutationIntent, RelationMutationIntent,
};
use crate::validation::data::{
    PlannedEntityCreate, PlannedRelationCreate, PlannedRelationEndpointUpdate, TouchedStructuralSet,
};
use crate::validation::engine::state_view::InvariantStateView;

use super::affected_record_filter::{
    include_affected_entity, include_affected_reference, include_affected_relation,
};

mod claim_nested;
mod incidence;
mod metadata;
mod owned_graph;
use claim_nested::claim_intent_nested_allocations;

pub(crate) fn collect_touched_structural_set(
    state_view: &InvariantStateView<'_>,
    before_image_view: Option<&InvariantStateView<'_>>,
    semantic_plan: Option<RelationalSemanticPlan<'_>>,
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

    if let Some(plan) = semantic_plan {
        if let Some(selection) = plan.selection()
            && work.try_charge(1)
        {
            touched_partitions.insert(selection.entity().partition_id);
            include_affected_entity(
                &mut visible_entities,
                selection.entity(),
                state_view,
                before_image_view,
                access,
            );
        }
        if let Some(selection) = plan.graph_selection() {
            owned_graph::collect(
                selection,
                state_view,
                before_image_view,
                access,
                work,
                &mut visible_entities,
                &mut visible_relations,
                &mut touched_partitions,
                &mut planned_entity_deletes,
                &mut planned_relation_deletes,
            );
        }
        for intent in &plan.commands().merged_intents {
            if !work.try_charge(1) {
                break;
            }
            if !claim_intent_nested_allocations(intent, access, work) {
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
                            if !work.try_charge(0) {
                                break;
                            }
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
                        if !work.try_charge(0) {
                            break;
                        }
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
                            if !work.try_charge(0) {
                                break;
                            }
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
                MutationIntent::Relation(RelationMutationIntent::Revalidate(spec)) => {
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

    let direct_visible_entities = incidence::expand(
        &mut visible_entities,
        &mut visible_relations,
        &mut touched_partitions,
        state_view,
        before_image_view,
        access,
        work,
    );

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
