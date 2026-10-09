use crate::facade::transactions::{AspectTraceEvidence, MutationIntent};
use crate::publication::patch::data::{
    PublishedAuthoritativePatchOperation, PublishedAuthoritativePatchValue,
};
use crate::tests::support::*;
use worth_foundational::facade::{
    AspectKey, AspectValue, ContractValidatedAspectValueView, FieldKey,
};

#[test]
fn update_entity_fields_canonical_delta_uses_authoritative_patch_evidence() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let entity = create_entity(&runtime, "before");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("field-patch").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: crate::transactions::data::AspectFieldPatch::from_locator(
                    crate::transactions::data::planned_single_field_locator(
                        AspectKey::new("name").expect("valid test aspect key"),
                        FieldKey::new("name").expect("valid test field key"),
                    ),
                    AspectValue::String("after".into()),
                ),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let patch_record = &outcome.patch()[0];
    let current_read = runtime
        .read_truth()
        .read_snapshot(&outcome.snapshot)
        .unwrap();

    let trace = outcome
        .aspect_evaluation_traces()
        .iter()
        .find(|trace| trace.target == RecordRef::Entity(entity))
        .expect("entity field patch trace");
    let row = trace
        .binding_rows
        .iter()
        .find(|row| row.aspect_key == AspectKey::new("name").unwrap())
        .expect("name aspect row");

    assert!(row.changed);
    assert_eq!(
        trace.changed_aspects,
        ordered_aspect_keys([AspectKey::new("name").unwrap()])
    );
    assert_eq!(
        patch_record.authoritative_changed_aspects(),
        ordered_aspect_keys([AspectKey::new("name").unwrap()])
    );
    assert!(matches!(
        patch_record.authoritative_patch.full_grammar_operations(),
        [PublishedAuthoritativePatchOperation::WholeAspectSet {
            aspect_key,
            value: PublishedAuthoritativePatchValue::Scalar(value),
            ..
        }] if *aspect_key == AspectKey::new("name").unwrap()
            && *value == AspectValue::String("after".into())
    ));
    let authoritative_name = current_read
        .get_entity(entity)
        .unwrap()
        .authoritative_aspect_state
        .as_ref()
        .and_then(|state| state.get(&AspectKey::new("name").unwrap()))
        .expect("name aspect state");
    assert!(matches!(
        authoritative_name.view(),
        ContractValidatedAspectValueView::Scalar(value)
            if value == &AspectValue::String("after".into())
    ));
    let AspectTraceEvidence::AuthoritativePatch { patch, .. } = &row.evidence else {
        panic!("expected authoritative patch trace evidence");
    };
    assert_eq!(
        patch.scalar_set_for(&AspectKey::new("name").unwrap()),
        Some(&AspectValue::String("after".into()))
    );
}
