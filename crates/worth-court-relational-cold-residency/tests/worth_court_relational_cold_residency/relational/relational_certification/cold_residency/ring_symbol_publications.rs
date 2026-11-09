//! Actual declared relation labels establish the ring's remote dictionary closure.
use super::current_publications::changed_relations;
use std::sync::Arc;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::RelationalRuntime,
    transactions::*,
};
use worth_testkit_relational::aspect_storage::{
    aspect_key, field_key,
    publications::{begin_main_transaction, release_test_commit_snapshot},
};

pub(super) fn populate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    for index in 0..300 {
        let text = format!("unrelated-ring-dictionary-text-{index}");
        let input = worth_execution::ExecutionImmutableBytes::from_external_bytes(Arc::new(
            text.as_bytes().into(),
        ));
        runtime.admit_field_text(execution, &basis, &input).unwrap();
    }
}
pub(super) fn publish_edge(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: EntityId,
    target: EntityId,
    label: &str,
) -> worth_relational::facade::symbols::Symbol {
    use worth_relational::facade::transactions::{EntityReference, RelationSpec};
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let input = worth_execution::ExecutionImmutableBytes::from_external_bytes(Arc::new(
        label.as_bytes().into(),
    ));
    let admitted = runtime.admit_field_text(execution, &basis, &input).unwrap();
    let fields = AspectFieldPatch::from_locator(
        worth_relational::facade::transactions::planned_single_field_locator(
            aspect_key("label"),
            field_key("label"),
        ),
        admitted.field_value_for(&basis).unwrap(),
    );
    let mut transaction = runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("paged-ring-edge").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: source.partition_id,
                    kind_id: KindId(2),
                    client_key: worth_relational::facade::symbols::ClientKey::raw(label),
                    source: EntityReference::Existing(source),
                    target: EntityReference::Existing(target),
                    fields,
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let outcome = transaction.commit(execution, runtime).unwrap();
    let relation = changed_relations(&outcome)[0];
    release_test_commit_snapshot(runtime, &outcome);
    let mut transaction = begin_main_transaction(runtime);
    let read = transaction.read_relation(execution, relation).unwrap();
    let record = read.base().unwrap();
    let value = super::relation_field_values::label(record).unwrap();
    let worth_foundational::facade::AspectValue::String(
        worth_foundational::facade::InternedString::Symbol(symbol),
    ) = value
    else {
        panic!("the actual declared edge must carry an interned field value");
    };
    worth_relational::facade::symbols::Symbol(symbol.0)
}
