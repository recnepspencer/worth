use super::*;

#[test]
fn touched_scope_tracks_planned_entity_deletes() {
    let host_request = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let id = PartitionId::main();
    let (partition, entities, _) = relation_partition_for_test(id, 1, &[]);
    let partitions = BTreeMap::from([(id, partition)]);
    let entity_id = entities[0];
    let intent = MutationIntent::Entity(
        crate::transactions::data::intents::mutation_intent::EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id },
        ),
    );
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
