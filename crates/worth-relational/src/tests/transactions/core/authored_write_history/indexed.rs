//! Canonical equality-predicate writes, not just changed current candidates.
use super::*;
use crate::capabilities::AspectPlanSource;
use crate::indexes::data::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind};
use crate::storage::data::AuthoritativeFieldComparisonKey;
use worth_foundational::facade::{
    AspectValue, ContractValidationInput, PortableAspectContractBasis,
    PortableAspectPatchOperation, PortableRecordAspectPatch, StructAspectValue,
};

fn register(runtime: &RelationalRuntime, locator: AspectFieldLocator) -> DerivedIndexDefinition {
    runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "authored-predicate".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: locator,
            },
            branch_scoped: true,
        })
        .expect("actual field index registration succeeds")
}

fn key(definition: &DerivedIndexDefinition, value: &str) -> RelationalAuthoredWriteSelector {
    let DerivedIndexKind::EntityField { field_locator } = &definition.kind else {
        panic!("real field index");
    };
    RelationalAuthoredWriteSelector::EntityFieldIndex {
        definition: RelationalAuthoredEntityIndexDefinition {
            index_id: definition.index_id,
            name: definition.name.clone(),
            branch_scoped: definition.branch_scoped,
            locator: field_locator.clone(),
        },
        kind: crate::identity::data::KindId(1),
        value: AuthoritativeFieldComparisonKey::from_aspect_value(&AspectValue::String(
            value.into(),
        )),
    }
}

fn scan(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    source: &AdmittedRelationalBranchBasis,
    target: &AdmittedRelationalBranchBasis,
    key: RelationalAuthoredWriteSelector,
) -> (Vec<CommitId>, usize) {
    collect(
        runtime,
        execution,
        runtime
            .authored_write_history_cursor(execution, source, selected(runtime, target), &[key])
            .unwrap(),
    )
}

fn commit_patch(
    runtime: &RelationalRuntime,
    execution: ExecutionRequest<'_, '_>,
    entity: EntityId,
    operation: PortableAspectPatchOperation,
) -> CommitResult {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("indexed-patch").push(MutationIntent::Entity(
                EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                    entity_id: entity,
                    aspect_patch: PortableRecordAspectPatch::new([operation]),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    transaction.commit(&execution, runtime).unwrap()
}

#[test]
fn authored_index_membership_keeps_noops_phantoms_and_whole_aspects() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = AspectSchemaFixture {
        entity_aspects: vec![entity_summary_struct_aspect(
            aspect_key("summary"),
            field_key("summary"),
        )],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Planned,
        aspect_key("summary"),
        CanonicalFieldPath::single(field_key("status")),
    );
    let definition = register(&runtime, locator);
    let unsupported = register(
        &runtime,
        AspectFieldLocator::new(
            LocatorAuthority::Derived,
            aspect_key("summary"),
            CanonicalFieldPath::single(field_key("status")),
        ),
    );
    let entity = super::super::struct_field_patch_authority::create_entity_with_summary_fields(
        &execution, &runtime, "indexed", "title", "open", false, false,
    );
    let source = test_owner_main_basis(&runtime).unwrap();
    let contract = runtime
        .entity_aspect_plan(crate::identity::data::KindId(1))
        .unwrap()
        .contract_for(&aspect_key("summary"))
        .unwrap();
    let set = |field: &str, value: &str| PortableAspectPatchOperation::PatchFields {
        basis: PortableAspectContractBasis::from_contract(&contract),
        selected_fields: vec![field_key(field)],
        field_sets: vec![worth_foundational::facade::PortableAspectFieldSet::new(
            field_key(field),
            AspectValue::String(value.into()),
        )],
        field_clears: vec![],
    };
    let equal = commit_patch(&runtime, execution, entity, set("status", "open"));
    let away = commit_patch(&runtime, execution, entity, set("status", "closed"));
    let back = commit_patch(&runtime, execution, entity, set("status", "open"));
    let sibling = commit_patch(&runtime, execution, entity, set("title", "new-title"));
    let whole = commit_patch(
        &runtime,
        execution,
        entity,
        PortableAspectPatchOperation::SetWhole {
            basis: PortableAspectContractBasis::from_contract(&contract),
            value: ContractValidationInput::Struct(
                StructAspectValue::new([
                    (
                        field_key("title"),
                        AspectValue::String("whole-title".into()),
                    ),
                    (field_key("status"), AspectValue::String("open".into())),
                ])
                .unwrap(),
            ),
        },
    );
    let phantom = super::super::struct_field_patch_authority::create_entity_with_summary_fields(
        &execution,
        &runtime,
        "phantom",
        "other-title",
        "open",
        false,
        false,
    );
    let phantom_commit = test_owner_main_basis(&runtime)
        .unwrap()
        .observation()
        .commit_id()
        .unwrap();
    let retired = delete_entity(&execution, &runtime, phantom);
    let target = test_owner_main_basis(&runtime).unwrap();
    assert!(matches!(
        runtime.authored_write_history_cursor(
            execution,
            &source,
            selected(&runtime, &target),
            &[key(&unsupported, "open")],
        ),
        Err(RelationalAuthoredWriteDenial::UnsupportedIndexSelector)
    ));
    let expected = [
        equal.commit.commit_id,
        away.commit.commit_id,
        back.commit.commit_id,
        whole.commit.commit_id,
        phantom_commit,
        retired.commit.commit_id,
    ];
    assert_eq!(
        scan(
            &runtime,
            execution,
            &source,
            &target,
            key(&definition, "open")
        )
        .0,
        expected
    );
    let entries = runtime
        .history
        .canonical_envelope(retired.commit.commit_id)
        .unwrap();
    assert!(entries
        .authored_index_writes
        .verified_entries()
        .unwrap()
        .iter()
        .any(|entry| entry.entity == phantom
            && entry.entity.generation_value() == phantom.generation_value()
            && entry.old_kind == Some(crate::identity::data::KindId(1))
            && entry.new_kind.is_none()));
    for outcome in [&equal, &away, &back, &sibling, &whole, &retired] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}

#[test]
fn indexed_history_visits_exclude_other_values_and_sibling_ancestry() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let definition = register(&runtime, name_locator());
    let created = create_entity_outcome(&execution, &runtime, "match");
    let entity = changed_entities(&created)[0];
    let source = test_owner_main_basis(&runtime).unwrap();
    super::super::owner_service_phase3::fork_from_main(&execution, &runtime, "index-sibling");
    let one = update_entity(&execution, &runtime, entity, "away");
    let two = update_entity(&execution, &runtime, entity, "match");
    let target = test_owner_main_basis(&runtime).unwrap();
    let expected = scan(
        &runtime,
        execution,
        &source,
        &target,
        key(&definition, "match"),
    );
    assert_eq!(
        expected,
        (vec![one.commit.commit_id, two.commit.commit_id], 2)
    );
    for index in 0..32 {
        let other = create_entity_outcome(&execution, &runtime, &format!("different-{index}"));
        let sibling = update_entity_on_branch(
            &execution,
            &runtime,
            entity,
            "match",
            BranchId("index-sibling".into()),
        );
        release_test_commit_snapshot(&runtime, &other);
        release_test_commit_snapshot(&runtime, &sibling);
    }
    let target = test_owner_main_basis(&runtime).unwrap();
    assert_eq!(
        scan(
            &runtime,
            execution,
            &source,
            &target,
            key(&definition, "match")
        ),
        expected
    );
    assert!(scan(
        &runtime,
        execution,
        &source,
        &target,
        key(&definition, "never-present")
    )
    .0
    .is_empty());
    for outcome in [&created, &one, &two] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}

#[path = "indexed/recovery.rs"]
mod recovery;
