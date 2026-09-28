use crate::facade::identity::EntityId;
use crate::facade::transactions::{AspectTraceEvidence, EntitySpec, MutationIntent};
use crate::publication::patch::data::{
    PublishedAuthoritativeFieldSet, PublishedAuthoritativePatchOperation,
};
use crate::tests::support::*;
use worth_foundational::facade::{
    AspectKey, AspectValue, ContractValidatedAspectValueView, FieldKey,
};
use worth_proof::TransitionOutcome;

#[test]
fn update_entity_fields_applies_struct_contract_field_patch() {
    let runtime = AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(
                crate::tests::support::aspect_key("name"),
                crate::tests::support::field_key("name"),
            ),
            entity_summary_struct_aspect(
                crate::tests::support::aspect_key("summary"),
                crate::tests::support::field_key("summary"),
            ),
        ],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let entity =
        create_entity_with_summary_fields(&runtime, "struct-patch", "before", "open", true, false);

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("summary-field-patch").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: crate::transactions::data::AspectFieldPatch::from_locator(
                    crate::transactions::data::planned_single_field_locator(
                        AspectKey::new("summary").expect("valid test aspect key"),
                        FieldKey::new("title").expect("valid test field key"),
                    ),
                    AspectValue::String("after".into()),
                ),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn.commit(&runtime).unwrap();
    let patch_record = &outcome.patch()[0];
    let current_read = runtime
        .read_truth()
        .read_snapshot(&outcome.snapshot)
        .unwrap();
    let record = current_read.get_entity(entity).unwrap();

    let authoritative_summary = record
        .authoritative_aspect_state
        .as_ref()
        .and_then(|state| state.get(&AspectKey::new("summary").unwrap()))
        .expect("summary aspect state");
    let ContractValidatedAspectValueView::Struct(authoritative_summary) =
        authoritative_summary.view()
    else {
        panic!("summary aspect state must remain struct shaped");
    };
    assert_eq!(
        authoritative_summary.get(&FieldKey::new("title").unwrap()),
        Some(&AspectValue::String("after".into()))
    );
    assert_eq!(
        authoritative_summary.get(&FieldKey::new("status").unwrap()),
        Some(&AspectValue::String("open".into()))
    );
    assert_eq!(
        patch_record.authoritative_changed_aspects(),
        ordered_aspect_keys([AspectKey::new("summary").unwrap()])
    );
    assert!(matches!(
        patch_record.authoritative_patch.full_grammar_operations(),
        [PublishedAuthoritativePatchOperation::FieldLevelPatch {
            aspect_key,
            field_sets,
            field_clears,
            ..
        }] if *aspect_key == AspectKey::new("summary").unwrap()
            && field_sets.len() == 1
            && field_sets[0].field == FieldKey::new("title").unwrap()
            && field_sets[0].value == AspectValue::String("after".into())
            && field_clears.is_empty()
    ));

    let trace = outcome
        .aspect_evaluation_traces()
        .iter()
        .find(|trace| trace.target == RecordRef::Entity(entity))
        .expect("summary field patch trace");
    let row = trace
        .binding_rows
        .iter()
        .find(|row| row.aspect_key == AspectKey::new("summary").unwrap())
        .expect("summary aspect row");

    assert!(row.changed);
    let AspectTraceEvidence::AuthoritativePatch { patch, .. } = &row.evidence else {
        panic!("expected authoritative patch trace evidence");
    };
    let summary = AspectKey::new("summary").unwrap();
    let field_sets = patch.field_sets_for(&summary).collect::<Vec<_>>();
    let field_clears = patch.field_clears_for(&summary).collect::<Vec<_>>();
    assert_eq!(
        field_sets,
        vec![&PublishedAuthoritativeFieldSet {
            field: FieldKey::new("title").expect("valid test field key"),
            value: AspectValue::String("after".into()),
        }]
    );
    assert!(field_clears.is_empty());

    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let selected = runtime
        .select_exact_commit(&basis.observation(), outcome.commit.commit_id)
        .into_outcome()
        .unwrap();
    let TransitionOutcome::Success(receipt) = runtime.mint_change_receipt(selected, None) else {
        panic!("a real field patch mints a change receipt");
    };
    assert_eq!(
        receipt.patch().authoritative_record_patches[0].authoritative_patch,
        patch_record.authoritative_patch
    );
}

pub(super) fn create_entity_with_summary_fields(
    runtime: &RelationalRuntime,
    client_key: &str,
    summary_title: &str,
    summary_status: &str,
    include_name: bool,
    include_scalar_title: bool,
) -> EntityId {
    let mut fields = std::collections::BTreeMap::from([
        (
            crate::transactions::data::planned_single_field_locator(
                AspectKey::new("summary").expect("valid summary aspect key"),
                FieldKey::new("title").expect("valid title field key"),
            ),
            AspectValue::String(summary_title.into()),
        ),
        (
            crate::transactions::data::planned_single_field_locator(
                AspectKey::new("summary").expect("valid summary aspect key"),
                FieldKey::new("status").expect("valid status field key"),
            ),
            AspectValue::String(summary_status.into()),
        ),
    ]);
    if include_name {
        fields.insert(
            crate::transactions::data::planned_single_field_locator(
                AspectKey::new("name").expect("valid name aspect key"),
                FieldKey::new("name").expect("valid name field key"),
            ),
            AspectValue::String(client_key.into()),
        );
    }
    if include_scalar_title {
        fields.insert(
            crate::transactions::data::planned_single_field_locator(
                AspectKey::new("title.scalar").expect("valid scalar title aspect key"),
                FieldKey::new("title").expect("valid title field key"),
            ),
            AspectValue::String("scalar-title".into()),
        );
    }
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    txn.push_batch(WorkerIntentBatch::new(format!("batch-{client_key}")).push(
        MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: crate::symbols::data::ClientKey::raw(client_key),
            fields: crate::transactions::data::AspectFieldPatch::new(fields),
        })),
    ))
    .expect("test staging stays within configured resource budgets");
    changed_entities(&txn.commit(runtime).unwrap())[0]
}
