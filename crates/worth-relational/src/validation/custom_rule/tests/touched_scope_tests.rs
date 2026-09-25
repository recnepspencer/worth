use super::*;

#[test]
fn direct_entities_exclude_neighbors_admitted_for_structural_traversal() {
    let runtime = runtime_with_test_schema();
    let shared = create_entity(&runtime, "shared");
    let existing_neighbor = create_entity(&runtime, "existing-neighbor");
    create_relation(&runtime, shared, existing_neighbor, "existing");
    let new_neighbor = create_entity(&runtime, "new-neighbor");
    let intent = MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: crate::identity::data::PartitionId::main(),
        kind_id: KindId(2),
        client_key: ClientKey::raw("planned"),
        source: EntityReference::Existing(shared),
        target: EntityReference::Existing(new_neighbor),
        fields: crate::transactions::data::AspectFieldPatch::default(),
    }));
    let merged_plan = MergedCommitPlan {
        transaction_id: TransactionId(9_002),
        merged_intents: vec![intent],
    };
    let observation = InvariantObservation::committed(runtime.storage_access().current_edition());
    let access = crate::validation::data::CustomInvariantAccessContract {
        read_entity_kinds: vec![KindId(1)],
        read_relation_kinds: vec![KindId(2)],
        affected_entity_kinds: vec![KindId(1)],
        affected_relation_kinds: vec![KindId(2)],
        include_relation_endpoint_entity_touches: true,
    };
    let prepared_scope =
        prepared_scope_with_access(&runtime, &observation, Some(&merged_plan), &access);
    let planner = test_scope_planner(
        &runtime,
        &observation,
        runtime.current_version_id(),
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
    let runtime = runtime_with_test_schema();
    let old_source = create_entity(&runtime, "old-source");
    let old_target = create_entity(&runtime, "old-target");
    let new_target = create_entity(&runtime, "new-target");
    let relation_id = create_relation(&runtime, old_source, old_target, "edge");
    let intent = MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
        UpdateRelationEndpointsIntent {
            relation_id,
            kind_id: KindId(2),
            source: EntityReference::Existing(old_source),
            target: EntityReference::Existing(new_target),
        },
    ));
    let touched = proposed_relation_scope(
        &runtime,
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
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let relation_id = create_relation(&runtime, source, target, "edge");
    let intent = MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
        relation_id,
    }));
    let touched = proposed_relation_scope(
        &runtime,
        relation_id,
        intent,
        ProposedRelationState::Deleted,
    );

    assert_eq!(touched.direct_visible_entity_ids(), &[source, target]);
}

#[test]
fn touched_scope_tracks_planned_relation_endpoint_updates() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let old_target = create_entity(&runtime, "old-target");
    let new_target = create_entity(&runtime, "new-target");
    let relation_id = create_relation(&runtime, source, old_target, "edge");
    let intent = MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
        UpdateRelationEndpointsIntent {
            relation_id,
            kind_id: KindId(2),
            source: EntityReference::Existing(source),
            target: EntityReference::Existing(new_target),
        },
    ));
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(WorkerIntentBatch::new("rewire").push(intent.clone()))
        .expect("test staging stays within configured resource budgets");
    let merged_plan = MergedCommitPlan {
        transaction_id: txn.transaction_id,
        merged_intents: vec![intent],
    };
    let observation = InvariantObservation::committed(runtime.storage_access().current_edition());
    let prepared_scope = prepared_scope_with_access(
        &runtime,
        &observation,
        Some(&merged_plan),
        &test_access_contract(),
    );
    let planner = test_scope_planner(
        &runtime,
        &observation,
        runtime.current_version_id(),
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
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let old_target = create_entity(&runtime, "old-target");
    let relation_id = create_relation(&runtime, source, old_target, "edge");
    let created_target = CreatedEntityRef {
        partition_id: crate::identity::data::PartitionId(1),
        kind_id: KindId(1),
        client_key: ClientKey::raw("planned-target"),
    };
    let create_target = MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: created_target.partition_id,
        kind_id: created_target.kind_id,
        client_key: created_target.client_key.clone(),
        fields: crate::transactions::data::AspectFieldPatch::default(),
    }));
    let update_relation = MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
        UpdateRelationEndpointsIntent {
            relation_id,
            kind_id: KindId(2),
            source: EntityReference::Existing(source),
            target: EntityReference::Created(created_target.clone()),
        },
    ));
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("rewire-to-created")
            .push(create_target.clone())
            .push(update_relation.clone()),
    )
    .expect("test staging stays within configured resource budgets");
    let merged_plan = MergedCommitPlan {
        transaction_id: txn.transaction_id,
        merged_intents: vec![create_target, update_relation],
    };
    let observation = InvariantObservation::committed(runtime.storage_access().current_edition());
    let prepared_scope = prepared_scope_with_access(
        &runtime,
        &observation,
        Some(&merged_plan),
        &test_access_contract(),
    );
    let planner = test_scope_planner(
        &runtime,
        &observation,
        runtime.current_version_id(),
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
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let relation_id = create_relation(&runtime, source, target, "edge");
    let intent = MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
        relation_id,
    }));
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(WorkerIntentBatch::new("delete").push(intent.clone()))
        .expect("test staging stays within configured resource budgets");
    let merged_plan = MergedCommitPlan {
        transaction_id: txn.transaction_id,
        merged_intents: vec![intent],
    };
    let observation = InvariantObservation::committed(runtime.storage_access().current_edition());
    let prepared_scope = prepared_scope_with_access(
        &runtime,
        &observation,
        Some(&merged_plan),
        &test_access_contract(),
    );
    let planner = test_scope_planner(
        &runtime,
        &observation,
        runtime.current_version_id(),
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

#[test]
fn touched_scope_tracks_planned_entity_deletes() {
    let runtime = runtime_with_test_schema();
    let entity_id = create_entity(&runtime, "entity");
    let intent = MutationIntent::Entity(crate::facade::transactions::EntityMutationIntent::Delete(
        DeleteEntityIntent { entity_id },
    ));
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(WorkerIntentBatch::new("delete-entity").push(intent.clone()))
        .expect("test staging stays within configured resource budgets");
    let merged_plan = MergedCommitPlan {
        transaction_id: txn.transaction_id,
        merged_intents: vec![intent],
    };
    let observation = InvariantObservation::committed(runtime.storage_access().current_edition());
    let prepared_scope = prepared_scope_with_access(
        &runtime,
        &observation,
        Some(&merged_plan),
        &test_access_contract(),
    );
    let planner = test_scope_planner(
        &runtime,
        &observation,
        runtime.current_version_id(),
        &prepared_scope,
    );

    assert_eq!(planner.touched().planned_entity_deletes(), &[entity_id]);
    assert_eq!(planner.counts().planned_entity_delete_count(), 1);
    assert_eq!(
        planner
            .touched()
            .provenance_summary()
            .planned_entity_delete_count,
        1
    );
}

fn test_scope_planner<'a>(
    runtime: &'a crate::runtime::RelationalRuntime,
    observation: &'a InvariantObservation<'a>,
    version: crate::identity::data::VersionId,
    prepared: &PreparedCustomInvariantScope,
) -> CustomInvariantScopePlanner<'a> {
    let view = crate::validation::engine::InvariantRuntimeView::from_runtime(runtime);
    CustomInvariantScopePlanner::new_at_current_version(
        &view,
        observation,
        version,
        runtime.current_version_id(),
        prepared,
        super::CustomInvariantWorkMeter::new(std::num::NonZeroU64::new(4096).unwrap()),
        Arc::new(crate::validation::data::CustomInvariantAccessContract {
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
    runtime: &crate::runtime::RelationalRuntime,
    relation_id: crate::identity::data::RelationId,
    intent: MutationIntent,
    proposed_state: ProposedRelationState,
) -> crate::validation::data::TouchedStructuralSet {
    let committed = runtime.storage_access().current_edition();
    let current_version = runtime.current_version_id();
    let proposed_version = current_version.saturating_next();
    let partition_id = relation_id.partition_id;
    let mut proposed =
        crate::storage::overlay::WorkingState::from_touched_partitions_with_layout_and_sparse_slots(
            &committed,
            [partition_id],
            runtime.config.storage.adjacency_policy.clone(),
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
                .apply_extra_update(relation_slot, extra, proposed_version);
        }
        ProposedRelationState::Deleted => proposed
            .get_partition_mut(partition_id)
            .relation_arena
            .retire(relation_slot, proposed_version),
    }
    proposed.mark_relation_slot_touched(partition_id, relation_slot);
    let overlay = crate::storage::overlay::OverlayStateView::new(&committed, &proposed);
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
        &super::CustomInvariantWorkMeter::new(std::num::NonZeroU64::new(4096).unwrap()),
    );
    let touched = test_scope_planner(runtime, &observation, proposed_version, &prepared)
        .touched()
        .clone();
    touched
}

fn test_access_contract() -> crate::validation::data::CustomInvariantAccessContract {
    crate::validation::data::CustomInvariantAccessContract {
        read_entity_kinds: vec![KindId(1)],
        read_relation_kinds: vec![KindId(2)],
        affected_entity_kinds: vec![KindId(1)],
        affected_relation_kinds: vec![KindId(2)],
        include_relation_endpoint_entity_touches: true,
    }
}
