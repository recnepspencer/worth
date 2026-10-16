//! Empty population is an exact pinned-root fact, not a latest-head guess.
use super::*;
use crate::facade::transactions::{
    DeleteEntityIntent, EntityMutationIntent, MutationIntent, WorkerIntentBatch,
};

#[test]
fn pinned_population_keeps_nonempty_predecessor_and_admits_empty_successor() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime_with_index_field_aspects();
    let created = create_entity_outcome(execution, &runtime, "population");
    let entity = changed_entities(&created)[0];
    let locator = aspect_field_locator(aspect_key("name"), field_key("name"));
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name.population".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: locator.clone(),
        },
        branch_scoped: true,
    });
    let build = |commit| {
        let built = runtime.index_authority().build_for_commit(
            execution,
            DerivedIndexBuildRequest {
                source_commit_id: commit,
                branch_id: BranchId("main".into()),
                index_ids: vec![index.index_id],
            },
        );
        assert!(built.failed_indexes.is_empty());
    };
    build(created.commit.commit_id);
    let view = runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .unwrap();
    let prior = runtime
        .index_access()
        .prepare_entity_field_lookup(&view, index.index_id, KindId(1), &locator, execution)
        .unwrap();
    assert!(!prior
        .population_is_empty_with_admission(execution, || Ok::<_, ()>(()))
        .unwrap());

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("remove-only-population-member").push(MutationIntent::Entity(
                EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: entity }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let deleted = transaction.commit(execution, &runtime).unwrap();
    build(deleted.commit.commit_id);
    let view = runtime
        .read_truth()
        .project_snapshot(&deleted.snapshot)
        .unwrap();
    let current = runtime
        .index_access()
        .prepare_entity_field_lookup(&view, index.index_id, KindId(1), &locator, execution)
        .unwrap();
    assert!(current
        .population_is_empty_with_admission(execution, || Ok::<_, ()>(()))
        .unwrap());
    assert!(!prior
        .population_is_empty_with_admission(execution, || Ok::<_, ()>(()))
        .unwrap());
    assert!(matches!(
        current.population_is_empty_with_admission(execution, || Err("selection exhausted")),
        Err(Stop::Admission("selection exhausted")),
    ));
    let cancellation = worth_execution::CancellationSource::new();
    let stopped = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(1 << 26),
        cancellation.token(),
        None,
    );
    cancellation.cancel();
    let stopped = worth_execution::ExecutionRequest::serial(&stopped);
    let mut admission_called = false;
    let refusal = current.population_is_empty_with_admission(&stopped, || {
        admission_called = true;
        Err::<(), _>("selection exhausted")
    });
    assert!(
        matches!(refusal, Err(Stop::Lookup(cause)) if cause.kind() ==
        BoundedEntityFieldLookupDenialKind::Execution(
            crate::execution::RelationalExecutionDenialCause::Cancelled))
    );
    assert!(!admission_called);
    release_test_commit_snapshot(&runtime, &deleted);
    release_test_commit_snapshot(&runtime, &created);
}
