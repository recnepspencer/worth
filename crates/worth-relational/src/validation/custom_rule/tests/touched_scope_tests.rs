use super::*;

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
    let prepared_scope = prepared_scope(&runtime, &observation, Some(&merged_plan));
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
    let prepared_scope = prepared_scope(&runtime, &observation, Some(&merged_plan));
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
    let prepared_scope = prepared_scope(&runtime, &observation, Some(&merged_plan));
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
    let prepared_scope = prepared_scope(&runtime, &observation, Some(&merged_plan));
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
        }),
    )
}
