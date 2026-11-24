use super::*;

#[test]
fn touched_scope_tracks_planned_entity_deletes() {
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let entity_id = create_entity(execution, &runtime, "entity");
    let intent = MutationIntent::Entity(
        crate::transactions::data::intents::mutation_intent::EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id },
        ),
    );
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("delete-entity").push(intent.clone()),
        AllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let merged_plan = MergedCommitPlan {
        transaction_id: txn.transaction_id,
        merged_intents: vec![intent],
    };
    let observation = InvariantObservation::committed(runtime.storage_access().current_edition());
    let prepared_scope = scope_with_access(
        runtime.current_version_id(),
        &observation,
        Some(&merged_plan),
        &test_access_contract(),
    );
    let planner = test_scope_planner(
        &runtime,
        *execution,
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
