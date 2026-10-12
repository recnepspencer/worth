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
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = AspectSchemaFixture {
        entity_aspects: vec![entity_summary_struct_aspect(
            aspect_key("summary"),
            field_key("summary"),
        )],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let entity = super::struct_field_patch_authority::create_entity_with_summary_fields(
        execution, &runtime, "mixed", "same", "open", false, false,
    );
    let before = snapshot_for_owner_branch(&runtime, &BranchId("main".to_owned()));
    let title = locator("summary", "title");
    let status = locator("summary", "status");
    let before_view = runtime.read_truth().project_snapshot(&before).unwrap();
    let title_before = before_view
        .entity_field_revision(execution, entity, &title)
        .expect("read request remains live")
        .unwrap();
    let status_before = before_view
        .entity_field_revision(execution, entity, &status)
        .expect("read request remains live")
        .unwrap();

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
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    let updated = txn.commit(execution, &runtime).unwrap();
    let after_view = runtime
        .read_truth()
        .project_snapshot(&updated.snapshot)
        .unwrap();
    assert_eq!(
        after_view
            .entity_field_revision(execution, entity, &title)
            .expect("read request remains live"),
        Some(title_before)
    );
    assert_ne!(
        after_view
            .entity_field_revision(execution, entity, &status)
            .expect("read request remains live"),
        Some(status_before)
    );
    runtime.snapshots().release_snapshot(&before).unwrap();
    release_test_commit_snapshot(&runtime, &updated);
}

#[test]
fn scalar_equal_value_is_stable_but_return_to_original_value_revises() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "alpha");
    let entity = changed_entities(&created)[0];
    let name = locator("name", "name");
    let revision = |outcome: &crate::facade::transactions::CommitResult| {
        runtime
            .read_truth()
            .project_snapshot(&outcome.snapshot)
            .unwrap()
            .entity_field_revision(execution, entity, &name)
            .expect("read request remains live")
            .unwrap()
    };
    let original = revision(&created);
    let changed = update_entity(execution, &runtime, entity, "beta");
    let changed_revision = revision(&changed);
    let returned = update_entity(execution, &runtime, entity, "alpha");
    let returned_revision = revision(&returned);
    let equal = update_entity(execution, &runtime, entity, "alpha");
    assert_ne!(original, changed_revision);
    assert_ne!(original, returned_revision);
    assert_ne!(changed_revision, returned_revision);
    assert_eq!(revision(&equal), returned_revision);
    for outcome in [&created, &changed, &returned, &equal] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}

#[test]
fn equal_scalar_patch_keeps_the_native_field_revision() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "equal-revision");
    let entity = changed_entities(&created)[0];
    let name = locator("name", "name");
    let before = runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .unwrap()
        .entity_field_revision(execution, entity, &name)
        .expect("read request remains live");
    assert!(before.is_some(), "creation must issue the native revision");
    let equal = update_entity(execution, &runtime, entity, "equal-revision");
    assert_eq!(
        runtime
            .read_truth()
            .project_snapshot(&equal.snapshot)
            .unwrap()
            .entity_field_revision(execution, entity, &name)
            .expect("read request remains live"),
        before,
        "equal canonical values must keep their native field revision"
    );
    for outcome in [&created, &equal] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}

#[test]
fn relation_equal_value_patch_preserves_native_field_revision() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let source = create_entity(execution, &runtime, "source");
    let target = create_entity(execution, &runtime, "target");
    let relation = create_relation(execution, &runtime, source, target, "original");
    let label = locator("label", "label");
    let before = snapshot_for_owner_branch(&runtime, &BranchId("main".to_owned()));
    let original = runtime
        .read_truth()
        .project_snapshot(&before)
        .unwrap()
        .relation_field_revision(execution, relation, &label)
        .expect("read request remains live")
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
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
        txn.commit(execution, &runtime).unwrap()
    };
    let equal = commit("original");
    let equal_revision = runtime
        .read_truth()
        .project_snapshot(&equal.snapshot)
        .unwrap()
        .relation_field_revision(execution, relation, &label)
        .expect("read request remains live")
        .unwrap();
    assert_eq!(equal_revision, original);
    let changed = commit("different");
    assert_ne!(
        runtime
            .read_truth()
            .project_snapshot(&changed.snapshot)
            .unwrap()
            .relation_field_revision(execution, relation, &label)
            .expect("read request remains live"),
        Some(original)
    );
    runtime.snapshots().release_snapshot(&before).unwrap();
    for outcome in [&equal, &changed] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}
