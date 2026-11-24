use super::*;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::storage::overlay::{
    CurrentPartitionAccess, OverlayStateView, PartitionState, WorkingState,
};
use crate::validation::data::descriptor::CustomInvariantAccessContract;
use std::collections::BTreeMap;

#[test]
fn direct_entities_exclude_neighbors_admitted_for_structural_traversal() {
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let (partitions, entities, _) = scope_partitions(3);
    let (shared, existing_neighbor, new_neighbor) = (entities[0], entities[1], entities[2]);
    let intent = MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: crate::identity::data::PartitionId::main(),
        kind_id: KindId(2),
        client_key: ClientKey::raw("planned"),
        source: EntityReference::Existing(shared),
        target: EntityReference::Existing(new_neighbor),
        fields: crate::transactions::data::aspect_field_patch::AspectFieldPatch::default(),
    }));
    let merged_plan = MergedCommitPlan {
        transaction_id: TransactionId(9_002),
        merged_intents: vec![intent],
    };
    let observation = InvariantObservation::committed_branch(&partitions);
    let access = CustomInvariantAccessContract {
        read_entity_kinds: vec![KindId(1)],
        read_relation_kinds: vec![KindId(2)],
        affected_entity_kinds: vec![KindId(1)],
        affected_relation_kinds: vec![KindId(2)],
        include_relation_endpoint_entity_touches: true,
    };
    let version = VersionId(1);
    let prepared_scope = scope_with_access(version, &observation, Some(&merged_plan), &access);
    let instrumentation = RuntimeInstrumentation::default();
    let planner = test_scope_planner(
        &instrumentation,
        *execution,
        &observation,
        version,
        &prepared_scope,
    );

    assert_eq!(
        planner.touched().direct_visible_entity_ids(),
        &[shared, new_neighbor]
    );
    assert!(planner
        .touched()
        .visible_entity_ids()
        .contains(&existing_neighbor));
}

#[test]
fn proposed_retarget_direct_scope_keeps_old_and_new_endpoints() {
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (partitions, entities, relation_id) = scope_partitions(3);
    let (old_source, old_target, new_target) = (entities[0], entities[1], entities[2]);
    let intent = MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
        UpdateRelationEndpointsIntent {
            relation_id,
            kind_id: KindId(2),
            source: EntityReference::Existing(old_source),
            target: EntityReference::Existing(new_target),
        },
    ));
    let touched = proposed_relation_scope(
        &partitions,
        *execution,
        relation_id,
        intent,
        ProposedRelationState::Retarget {
            source: old_source,
            target: new_target,
        },
    );

    assert_eq!(
        touched.direct_visible_entity_ids(),
        &[old_source, old_target, new_target]
    );
}

#[test]
fn proposed_delete_direct_scope_keeps_both_old_endpoints() {
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (partitions, entities, relation_id) = scope_partitions(2);
    let (source, target) = (entities[0], entities[1]);
    let intent = MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
        relation_id,
    }));
    let touched = proposed_relation_scope(
        &partitions,
        *execution,
        relation_id,
        intent,
        ProposedRelationState::Deleted,
    );

    assert_eq!(touched.direct_visible_entity_ids(), &[source, target]);
}

#[test]
fn touched_scope_tracks_planned_relation_endpoint_updates() {
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (partitions, entities, relation_id) = scope_partitions(3);
    let (source, new_target) = (entities[0], entities[2]);
    let intent = MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
        UpdateRelationEndpointsIntent {
            relation_id,
            kind_id: KindId(2),
            source: EntityReference::Existing(source),
            target: EntityReference::Existing(new_target),
        },
    ));
    let merged_plan = MergedCommitPlan {
        transaction_id: TransactionId(9_004),
        merged_intents: vec![intent],
    };
    let observation = InvariantObservation::committed_branch(&partitions);
    let prepared_scope = scope_with_access(
        VersionId(1),
        &observation,
        Some(&merged_plan),
        &test_access_contract(),
    );
    let instrumentation = RuntimeInstrumentation::default();
    let planner = test_scope_planner(
        &instrumentation,
        *execution,
        &observation,
        VersionId(1),
        &prepared_scope,
    );

    let updates = planner.touched().planned_relation_endpoint_updates();
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].relation_id(), relation_id);
    assert_eq!(updates[0].kind_id(), KindId(2));
    assert_eq!(updates[0].source(), &EntityReference::Existing(source));
    assert_eq!(updates[0].target(), &EntityReference::Existing(new_target));
    assert_eq!(planner.counts().planned_relation_endpoint_update_count(), 1);
    assert_eq!(
        planner
            .touched()
            .provenance_summary()
            .planned_relation_endpoint_update_count,
        1
    );
}

#[test]
fn touched_scope_tracks_planned_relation_endpoint_updates_to_created_entities() {
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (partitions, entities, relation_id) = scope_partitions(2);
    let source = entities[0];
    let created_target = CreatedEntityRef {
        partition_id: crate::identity::data::PartitionId(1),
        kind_id: KindId(1),
        client_key: ClientKey::raw("planned-target"),
    };
    let create_target = MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: created_target.partition_id,
        kind_id: created_target.kind_id,
        client_key: created_target.client_key.clone(),
        fields: crate::transactions::data::aspect_field_patch::AspectFieldPatch::default(),
    }));
    let update_relation = MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
        UpdateRelationEndpointsIntent {
            relation_id,
            kind_id: KindId(2),
            source: EntityReference::Existing(source),
            target: EntityReference::Created(created_target.clone()),
        },
    ));
    let merged_plan = MergedCommitPlan {
        transaction_id: TransactionId(9_004),
        merged_intents: vec![create_target, update_relation],
    };
    let observation = InvariantObservation::committed_branch(&partitions);
    let prepared_scope = scope_with_access(
        VersionId(1),
        &observation,
        Some(&merged_plan),
        &test_access_contract(),
    );
    let instrumentation = RuntimeInstrumentation::default();
    let planner = test_scope_planner(
        &instrumentation,
        *execution,
        &observation,
        VersionId(1),
        &prepared_scope,
    );

    let updates = planner.touched().planned_relation_endpoint_updates();
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].relation_id(), relation_id);
    assert_eq!(updates[0].kind_id(), KindId(2));
    assert_eq!(updates[0].source(), &EntityReference::Existing(source));
    assert_eq!(
        updates[0].target(),
        &EntityReference::Created(created_target)
    );
}

#[test]
fn touched_scope_tracks_planned_relation_deletes() {
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (partitions, _, relation_id) = scope_partitions(2);
    let intent = MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
        relation_id,
    }));
    let merged_plan = MergedCommitPlan {
        transaction_id: TransactionId(9_004),
        merged_intents: vec![intent],
    };
    let observation = InvariantObservation::committed_branch(&partitions);
    let prepared_scope = scope_with_access(
        VersionId(1),
        &observation,
        Some(&merged_plan),
        &test_access_contract(),
    );
    let instrumentation = RuntimeInstrumentation::default();
    let planner = test_scope_planner(
        &instrumentation,
        *execution,
        &observation,
        VersionId(1),
        &prepared_scope,
    );

    assert_eq!(planner.touched().planned_relation_deletes(), &[relation_id]);
    assert_eq!(planner.counts().planned_relation_delete_count(), 1);
    assert_eq!(
        planner
            .touched()
            .provenance_summary()
            .planned_relation_delete_count,
        1
    );
}

fn test_scope_planner<'a>(
    instrumentation: &'a RuntimeInstrumentation,
    execution: worth_execution::ExecutionRequest<'a, 'a>,
    observation: &'a InvariantObservation<'a>,
    version: crate::identity::data::VersionId,
    prepared: &PreparedCustomInvariantScope,
) -> CustomInvariantScopePlanner<'a> {
    let view = empty_invariant_view(instrumentation);
    CustomInvariantScopePlanner::new_at_current_version(
        &view,
        execution,
        observation,
        version,
        VersionId(1),
        prepared,
        super::CustomInvariantWorkMeter::new(Some(std::num::NonZeroU64::new(4096).unwrap())),
        Arc::new(CustomInvariantAccessContract {
            read_entity_kinds: vec![KindId(1)],
            read_relation_kinds: vec![KindId(2)],
            affected_entity_kinds: vec![KindId(1)],
            affected_relation_kinds: vec![KindId(2)],
            include_relation_endpoint_entity_touches: true,
        }),
    )
}

enum ProposedRelationState {
    Retarget {
        source: crate::identity::data::EntityId,
        target: crate::identity::data::EntityId,
    },
    Deleted,
}

fn proposed_relation_scope(
    committed: &dyn CurrentPartitionAccess,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    relation_id: crate::identity::data::RelationId,
    intent: MutationIntent,
    proposed_state: ProposedRelationState,
) -> crate::validation::data::custom_rule::touched_scope::TouchedStructuralSet {
    let current_version = VersionId(1);
    let proposed_version = current_version.saturating_next();
    let partition_id = relation_id.partition_id;
    let mut proposed = WorkingState::from_touched_partitions_with_layout_and_sparse_slots(
        committed,
        [partition_id],
        crate::config::data::RelationalRuntimeConfig::default()
            .storage
            .adjacency_policy,
        crate::storage::overlay::PartitionCloneMode::Full,
        crate::storage::overlay::EntityWorkingSetLayout::CanonicalSoA,
        None,
        None,
    );
    let relation_slot = relation_id.slot_index();
    match proposed_state {
        ProposedRelationState::Retarget { source, target } => {
            let partition = proposed.get_partition_mut(partition_id);
            let mut extra = partition
                .relation_arena
                .get(&relation_id)
                .expect("proposed fixture retains the relation")
                .extra()
                .clone();
            extra.endpoints = Some(crate::storage::substrate::RelationEndpoints { source, target });
            partition
                .relation_arena
                .apply_extra_update(relation_slot, extra, proposed_version)
                .expect("fixture slot is materialized");
        }
        ProposedRelationState::Deleted => proposed
            .get_partition_mut(partition_id)
            .relation_arena
            .retire(relation_slot, proposed_version)
            .expect("fixture slot is materialized"),
    }
    proposed.mark_relation_slot_touched(partition_id, relation_slot);
    let overlay = OverlayStateView::new(committed, &proposed);
    let observation =
        InvariantObservation::speculative_with_proposal(overlay, current_version, None);
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(9_003),
        merged_intents: vec![intent],
    };
    let access = test_access_contract();
    let prepared = PreparedCustomInvariantScope::capture(
        &observation,
        proposed_version,
        Some(&plan),
        &access,
        &super::CustomInvariantWorkMeter::new(Some(std::num::NonZeroU64::new(4096).unwrap())),
    );

    let instrumentation = RuntimeInstrumentation::default();
    test_scope_planner(
        &instrumentation,
        execution,
        &observation,
        proposed_version,
        &prepared,
    )
    .touched()
    .clone()
}

fn test_access_contract() -> CustomInvariantAccessContract {
    CustomInvariantAccessContract {
        read_entity_kinds: vec![KindId(1)],
        read_relation_kinds: vec![KindId(2)],
        affected_entity_kinds: vec![KindId(1)],
        affected_relation_kinds: vec![KindId(2)],
        include_relation_endpoint_entity_touches: true,
    }
}

fn scope_partitions(
    entity_count: usize,
) -> (
    BTreeMap<PartitionId, PartitionState>,
    Vec<crate::identity::data::EntityId>,
    crate::identity::data::RelationId,
) {
    let id = PartitionId::main();
    let (partition, entities, relations) =
        relation_partition(id, entity_count, &[(0, 1, KindId(2))]);
    (BTreeMap::from([(id, partition)]), entities, relations[0])
}

mod entity_deletes;
