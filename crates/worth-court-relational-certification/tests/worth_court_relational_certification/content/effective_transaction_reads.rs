//! A carried read projection equals one cold replay over the same staged input.
use super::historical_kind::fixture::World;
use crate::leased_preparation::ordinary_context::support;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{mvcc::RelationalTransactionIntent, transactions::*};

#[test]
fn appended_projection_and_rollback_equal_one_replay_of_retained_intents() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let mut world = World::new();
    world.create(&execution, 7, "before");
    let id = world.entities[0];
    let (_, basis) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    let begin = || {
        world
            .runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap()
    };
    let mut carried = begin();
    let mut cold = begin();
    for value in ["one", "two", "three"] {
        let batch = WorkerIntentBatch::new(value).push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: id,
                fields: support::string_field_patch(
                    support::aspect_key("name"),
                    support::field_key("name"),
                    value,
                ),
            }),
        ));
        carried
            .push_batch(batch.clone(), execution.allocation_policy())
            .unwrap();
        cold.push_batch(batch, execution.allocation_policy())
            .unwrap();
        let read = carried.read_entity(&execution, id).unwrap();
        assert_eq!(
            carried.read_entity(&execution, id).unwrap(),
            read,
            "unchanged read retains both state and authoring history"
        );
    }
    let before_rollback = carried.read_entity(&execution, id).unwrap();
    assert_eq!(before_rollback, cold.read_entity(&execution, id).unwrap());
    let savepoint = carried.create_savepoint().unwrap();
    carried
        .push_batch(
            WorkerIntentBatch::new("delete").push(MutationIntent::Entity(
                EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: id }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    assert!(carried.read_entity(&execution, id).unwrap().is_deleted());
    carried.rollback_to_savepoint(savepoint).unwrap();
    assert_eq!(
        carried.read_entity(&execution, id).unwrap(),
        before_rollback
    );
    assert_eq!(
        cold.read_entity(&execution, id).unwrap(),
        before_rollback,
        "the prior read's shared history is immutable"
    );
}
