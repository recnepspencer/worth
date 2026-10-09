use super::*;

#[test]
fn savepoint_rollback_restores_overlay_footprint_and_cached_plan() {
    let runtime = runtime_with_test_schema();
    let retained = create_entity(&runtime, "retained");
    let rolled_back = create_entity(&runtime, "rolled-back");
    let (_, mut transaction) = begin_on(&runtime, "main");
    transaction
        .read_entity(retained)
        .expect("retained read projects");
    let retained_batch = update_batch(retained, "retained-write");
    let retained_intent = retained_batch.intents[0].clone();
    let retained_mutation = match &retained_intent {
        MutationIntent::Entity(intent) => intent.clone(),
        other => panic!("expected retained entity mutation, got {other:?}"),
    };
    transaction
        .push_batch(retained_batch, AllocationPolicy::SystemAllocation)
        .expect("test staging stays within configured resource budgets");
    let savepoint = transaction.create_savepoint().unwrap();
    let expected_footprint = transaction.footprint().clone();

    transaction
        .read_entity(rolled_back)
        .expect("rolled-back read projects");
    transaction
        .push_batch(
            update_batch(rolled_back, "discarded-write"),
            AllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let discarded_create = created_entity("discarded-create");
    transaction
        .push_batch(
            batch_create("discarded-create"),
            AllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    assert!(transaction
        .read_created_entity(&discarded_create)
        .unwrap()
        .is_some());
    assert_eq!(
        transaction
            .merged_plan(&runtime, AllocationPolicy::SystemAllocation,)
            .expect("pre-rollback plan includes staged work")
            .merged_intents
            .len(),
        3
    );

    transaction
        .rollback_to_savepoint(savepoint)
        .expect("savepoint rollback succeeds");
    assert_eq!(transaction.footprint(), &expected_footprint);
    assert!(transaction
        .read_entity(rolled_back)
        .expect("post-rollback read projects")
        .staged_mutations()
        .is_empty());
    assert!(transaction
        .read_created_entity(&discarded_create)
        .unwrap()
        .is_none());
    assert_eq!(
        transaction
            .read_entity(retained)
            .expect("retained mutation projects")
            .staged_mutations(),
        &[retained_mutation]
    );
    assert_eq!(
        transaction
            .merged_plan(&runtime, AllocationPolicy::SystemAllocation,)
            .expect("post-rollback plan is rebuilt from retained batches")
            .merged_intents,
        vec![retained_intent]
    );
}
