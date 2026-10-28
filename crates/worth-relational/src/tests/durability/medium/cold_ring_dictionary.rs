//! Actual declared relation labels establish the ring's remote dictionary closure.
use super::*;
pub(super) fn populate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
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
    runtime: &crate::runtime::RelationalRuntime,
    source: EntityId,
    target: EntityId,
    label: &str,
) -> crate::symbols::data::Symbol {
    use crate::transactions::data::{EntityReference, RelationSpec};
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let input = worth_execution::ExecutionImmutableBytes::from_external_bytes(Arc::new(
        label.as_bytes().into(),
    ));
    let admitted = runtime.admit_field_text(execution, &basis, &input).unwrap();
    let fields = AspectFieldPatch::from_locator(
        crate::transactions::data::planned_single_field_locator(
            aspect_key("label"),
            field_key("label"),
        ),
        admitted.field_value_for(&basis).unwrap(),
    );
    let mut transaction = runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("paged-ring-edge").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: source.partition_id,
                    kind_id: KindId(2),
                    client_key: crate::symbols::data::ClientKey::raw(label),
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
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    let read = transaction.read_relation(execution, relation).unwrap();
    let record = read.base().unwrap();
    let value = crate::visibility::materialization::read_records::relation_query_locus_value(
        record,
        &aspect_field_locator(aspect_key("label"), field_key("label")),
    )
    .unwrap();
    let worth_foundational::facade::AspectValue::String(
        worth_foundational::facade::InternedString::Symbol(symbol),
    ) = value
    else {
        panic!("the actual declared edge must carry an interned field value");
    };
    let symbol = crate::symbols::data::Symbol(symbol.0);
    assert_eq!(
        runtime.services.symbols.interner_snapshot().resolve(symbol),
        Some(label)
    );
    symbol
}

#[test]
fn the_remote_ring_label_stays_cold_until_its_incoming_edge_is_read() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let cold = recover_shape_inner(execution, None, 4, true, true);
    let symbol = cold.remote_symbol.unwrap();
    assert!(
        cold.runtime
            .services
            .symbols
            .interner_snapshot()
            .resolve(symbol)
            .is_none(),
        "reopen must not admit the remote field text"
    );
    let mut transaction = test_owner_begin_transaction_for_main(&cold.runtime);
    transaction
        .read_entity(execution, cold.entities[0])
        .unwrap();
    assert!(
        cold.runtime
            .services
            .symbols
            .interner_snapshot()
            .resolve(symbol)
            .is_none(),
        "the anchor's dictionary closure excludes the remote label"
    );
    let member = cold
        .manifest
        .symbols
        .iter()
        .find(|page| page.first <= u64::from(symbol.0) && u64::from(symbol.0) <= page.last)
        .unwrap()
        .member;
    cold.source.failures.lock().unwrap().insert(
        super::super::cold_source::ColdRead::Member(member),
        Cause::MissingMember,
    );
    let (_, basis) = cold
        .runtime
        .observe_branch(&cold.runtime.main_branch_identity())
        .unwrap();
    let view = cold
        .runtime
        .read_truth()
        .project_observation(&basis.observation())
        .unwrap();
    let answer =
        view.bounded_incoming_relations_of_kind(execution, cold.entities[0], KindId(2), Some(32));
    assert!(
        matches!(
            answer,
            Err(crate::execution::RelationalExecutionDenialCause::StorageUnavailable(_))
        ),
        "incoming={answer:?}; symbol={symbol:?}; reads={:?}",
        cold.source.member_reads.lock().unwrap()
    );
}
