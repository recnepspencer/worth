use super::*;

pub(super) fn prepare_update(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
    entity: EntityId,
    locator: AspectFieldLocator,
) -> PreparedRelationalCommitCandidate {
    let identity = runtime.branch_identity(branch).unwrap();
    let basis = runtime.admit_branch_basis(&identity).unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("two-native-hints").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: AspectFieldPatch::from(BTreeMap::from([(
                        locator,
                        AspectValue::String(InternedString::Raw("closed".to_owned())),
                    )])),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap()
}
