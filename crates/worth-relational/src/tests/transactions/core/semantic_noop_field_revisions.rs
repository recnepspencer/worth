use crate::capabilities::AspectPlanSource;
use crate::facade::history::BranchId;
use crate::facade::identity::KindId;
use crate::facade::mvcc::WorkerIntentBatch;
use crate::facade::transactions::{
    ApplyRelationAspectPatchIntent, EntityMutationIntent, MutationIntent, RelationMutationIntent,
    UpdateEntityFieldsIntent,
};
use crate::tests::support::*;
use worth_foundational::facade::{
    AspectFieldLocator, AspectValue, CanonicalFieldPath, ContractValidationInput, LocatorAuthority,
    PortableAspectContractBasis, PortableAspectPatchOperation, PortableRecordAspectPatch,
};

fn locator(aspect: &str, field: &str) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        aspect_key(aspect),
        CanonicalFieldPath::single(field_key(field)),
    )
}

#[test]
fn mixed_struct_patch_revises_only_the_field_with_a_new_value() {
    let runtime = AspectSchemaFixture {
        entity_aspects: vec![entity_summary_struct_aspect(
            aspect_key("summary"),
            field_key("summary"),
        )],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let entity = super::struct_field_patch_authority::create_entity_with_summary_fields(
        &runtime, "mixed", "same", "open", false, false,
    );
    let before = snapshot_for_owner_branch(&runtime, &BranchId("main".to_owned()));
    let title = locator("summary", "title");
    let status = locator("summary", "status");
    let before_view = runtime.read_truth().project_snapshot(&before).unwrap();
    let title_before = before_view.entity_field_revision(entity, &title).unwrap();
    let status_before = before_view.entity_field_revision(entity, &status).unwrap();

    let mut txn = test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("mixed-struct-update").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: string_aspect_field_patch([
                    (aspect_key("summary"), field_key("title"), "same"),
                    (aspect_key("summary"), field_key("status"), "closed"),
                ]),
            }),
        )),
    )
    .unwrap();
    let updated = txn.commit(&runtime).unwrap();
    let after_view = runtime
        .read_truth()
        .project_snapshot(&updated.snapshot)
        .unwrap();
    assert_eq!(
        after_view.entity_field_revision(entity, &title),
        Some(title_before)
    );
    assert_ne!(
        after_view.entity_field_revision(entity, &status),
        Some(status_before)
    );
    runtime.snapshots().release_snapshot(&before).unwrap();
    release_test_commit_snapshot(&runtime, &updated);
}

#[test]
fn scalar_equal_value_is_stable_but_return_to_original_value_revises() {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "alpha");
    let entity = changed_entities(&created)[0];
    let name = locator("name", "name");
    let revision = |outcome: &crate::facade::transactions::CommitResult| {
        runtime
            .read_truth()
            .project_snapshot(&outcome.snapshot)
            .unwrap()
            .entity_field_revision(entity, &name)
            .unwrap()
    };
    let original = revision(&created);
    let changed = update_entity(&runtime, entity, "beta");
    let changed_revision = revision(&changed);
    let returned = update_entity(&runtime, entity, "alpha");
    let returned_revision = revision(&returned);
    let equal = update_entity(&runtime, entity, "alpha");
    assert_ne!(original, changed_revision);
    assert_ne!(original, returned_revision);
    assert_ne!(changed_revision, returned_revision);
    assert_eq!(revision(&equal), returned_revision);
    for outcome in [&created, &changed, &returned, &equal] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}

#[test]
fn relation_equal_value_patch_preserves_native_field_revision() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let relation = create_relation(&runtime, source, target, "original");
    let label = locator("label", "label");
    let before = snapshot_for_owner_branch(&runtime, &BranchId("main".to_owned()));
    let original = runtime
        .read_truth()
        .project_snapshot(&before)
        .unwrap()
        .relation_field_revision(relation, &label)
        .unwrap();
    let contract = runtime
        .relation_aspect_plan(KindId(2))
        .unwrap()
        .executable_bindings
        .iter()
        .find(|binding| binding.aspect_key() == &aspect_key("label"))
        .unwrap()
        .contract
        .clone();
    let patch = |value: &str| {
        PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
            basis: PortableAspectContractBasis::from_contract(&contract),
            value: ContractValidationInput::Scalar(AspectValue::String(value.into())),
        }])
    };
    let commit = |value: &str| {
        let mut txn = test_owner_begin_transaction_for_main(&runtime);
        txn.push_batch(
            WorkerIntentBatch::new("relation-label").push(MutationIntent::Relation(
                RelationMutationIntent::ApplyAspectPatch(ApplyRelationAspectPatchIntent {
                    relation_id: relation,
                    aspect_patch: patch(value),
                }),
            )),
        )
        .unwrap();
        txn.commit(&runtime).unwrap()
    };
    let equal = commit("original");
    let equal_revision = runtime
        .read_truth()
        .project_snapshot(&equal.snapshot)
        .unwrap()
        .relation_field_revision(relation, &label)
        .unwrap();
    assert_eq!(equal_revision, original);
    let changed = commit("different");
    assert_ne!(
        runtime
            .read_truth()
            .project_snapshot(&changed.snapshot)
            .unwrap()
            .relation_field_revision(relation, &label),
        Some(original)
    );
    runtime.snapshots().release_snapshot(&before).unwrap();
    for outcome in [&equal, &changed] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}
