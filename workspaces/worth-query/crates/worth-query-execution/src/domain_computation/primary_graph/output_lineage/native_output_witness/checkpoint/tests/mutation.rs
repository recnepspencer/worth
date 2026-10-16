//! Native snapshot and committed field mutation fixtures for checkpoint witness tests.
use super::*;

pub(super) fn snapshot(runtime: &RelationalRuntime) -> SnapshotHandle {
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap()
}

pub(super) fn change_label(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut RelationalRuntime,
    entity: EntityId,
    locator: worth_foundational::facade::AspectFieldLocator,
) {
    let batch =
        WorkerIntentBatch::new("checkpoint-output-field-edit").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: AspectFieldPatch::from(BTreeMap::from([(
                    locator,
                    AspectValue::String(InternedString::Raw("changed-output".to_owned())),
                )])),
            }),
        ));
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("the real native edit must publish");
    };
    let committed = runtime.settle_performed_publication(performed).unwrap();
    release_test_commit_snapshot(runtime, &committed);
}
