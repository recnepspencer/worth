//! Original field-text admission and publications with real retained snapshots.
use super::current_field_meanings::{aspect_key, field_key};
use std::sync::Arc;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::{
    identity::*, mvcc::RelationalTransactionIntent, runtime::RelationalRuntime,
    snapshots::SnapshotHandle, symbols::Symbol, transactions::*,
};
use worth_testkit_relational::aspect_storage::publications::changed_entities;
pub(super) fn admit_population(request: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime) {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    for id in 0..=600 {
        let text = format!("paged-view-value-{id:04}");
        let input = ExecutionImmutableBytes::from_external_bytes(Arc::new(text.as_bytes().into()));
        runtime.admit_field_text(request, &basis, &input).unwrap();
    }
}
pub(super) enum TextMutation {
    Create,
    Update(EntityId),
}
pub(super) fn publish_text(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    mutation: TextMutation,
    text: &str,
) -> (EntityId, SnapshotHandle, Symbol) {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let input = ExecutionImmutableBytes::from_external_bytes(Arc::new(text.as_bytes().into()));
    let admitted = runtime.admit_field_text(request, &basis, &input).unwrap();
    let value = admitted.field_value_for(&basis).unwrap();
    let AspectValue::String(InternedString::Symbol(symbol)) = &value else {
        panic!("admitted text has its actual original symbol")
    };
    let symbol = Symbol(symbol.0);
    let fields = AspectFieldPatch::from_locator(
        planned_single_field_locator(aspect_key("name"), field_key("name")),
        value,
    );
    let intent = match mutation {
        TextMutation::Create => MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: "symbol-view-record".into(),
            fields,
        })),
        TextMutation::Update(entity_id) => MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent { entity_id, fields }),
        ),
    };
    let mut txn = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    txn.push_batch(
        WorkerIntentBatch::new("symbol-view").push(intent),
        request.allocation_policy(),
    )
    .unwrap();
    let result = txn.commit(request, runtime).unwrap();
    (
        changed_entities(&result)[0],
        result.snapshot.clone(),
        symbol,
    )
}
