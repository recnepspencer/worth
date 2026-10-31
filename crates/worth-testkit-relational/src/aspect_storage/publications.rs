//! Ordinary owner-admitted Native publications over the shared aspect schema.
use worth_foundational::facade::*;
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId},
    mvcc::{BranchBoundRelationalTransaction, RelationalTransactionIntent},
    runtime::RelationalRuntime,
    symbols::ClientKey,
    transactions::*,
};

pub fn begin_main_transaction(runtime: &RelationalRuntime) -> BranchBoundRelationalTransaction {
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap()
}
pub fn changed_entities(commit: &CommitResult) -> Vec<EntityId> {
    commit
        .changed_records
        .iter()
        .filter_map(|record| match record {
            RecordRef::Entity(entity) => Some(*entity),
            RecordRef::Relation(_) => None,
        })
        .collect()
}
pub fn release_test_commit_snapshot(runtime: &RelationalRuntime, commit: &CommitResult) {
    runtime
        .snapshots()
        .release_snapshot(&commit.snapshot)
        .unwrap();
}
pub fn commit_entity_create(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    aspect_patch: PortableRecordAspectPatch,
) -> CommitResult {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("native-entity-create").push(MutationIntent::Create(
                CreateIntent::EntityAspects(EntityAspectCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("native-entity"),
                    aspect_patch,
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.commit(execution, runtime).unwrap()
}
pub fn commit_entity_patch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity_id: EntityId,
    aspect_patch: PortableRecordAspectPatch,
) -> CommitResult {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("native-entity-patch").push(MutationIntent::Entity(
                EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                    entity_id,
                    aspect_patch,
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.commit(execution, runtime).unwrap()
}
pub fn whole_set(contract: &AspectContract, value: AspectValue) -> PortableRecordAspectPatch {
    PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(contract),
        value: ContractValidationInput::Scalar(value),
    }])
}
pub fn whole_clear(contract: &AspectContract) -> PortableRecordAspectPatch {
    PortableRecordAspectPatch::new([PortableAspectPatchOperation::ClearWhole {
        basis: PortableAspectContractBasis::from_contract(contract),
    }])
}
pub fn text(value: &str) -> AspectValue {
    AspectValue::String(value.into())
}
pub fn string_aspect_value(value: &str) -> AspectValue {
    text(value)
}
pub fn string_aspect_field_patch<'a>(
    fields: impl IntoIterator<Item = (AspectKey, FieldKey, &'a str)>,
) -> AspectFieldPatch {
    fields
        .into_iter()
        .map(|(aspect, field, value)| (planned_single_field_locator(aspect, field), text(value)))
        .collect::<std::collections::BTreeMap<_, _>>()
        .into()
}
