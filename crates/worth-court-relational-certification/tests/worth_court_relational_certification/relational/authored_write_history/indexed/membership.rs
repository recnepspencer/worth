//! Equality-predicate writes include canonical no-ops, birth, and retirement.
use super::super::{admit_main, selected};
use super::scan;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    AspectFieldLocator, AspectValue, CanonicalFieldPath, ContractValidationInput, LocatorAuthority,
    PortableAspectContractBasis, PortableAspectFieldSet, PortableAspectPatchOperation,
    PortableRecordAspectPatch, StructAspectValue,
};
use worth_relational::facade::{
    history::{
        RelationalAuthoredEntityIndexDefinition, RelationalAuthoredWriteDenial,
        RelationalAuthoredWriteSelector,
    },
    identity::{EntityId, KindId, PartitionId},
    indexes::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind},
    runtime::RelationalRuntime,
    storage::AuthoritativeFieldComparisonKey,
    symbols::ClientKey,
    transactions::{CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch},
};
use worth_testkit_relational::{
    aspect_storage::{
        aspect_key, entity_summary_struct_aspect, field_key,
        publications::{
            begin_main_transaction, changed_entities, commit_entity_patch,
            release_test_commit_snapshot, string_aspect_field_patch,
        },
        AspectSchemaFixture,
    },
    ordinary_records::delete_entity,
    shared_relational_certification_test_execution_policy::serial_request,
};

fn create_with_summary(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
    title: &str,
) -> EntityId {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new(format!("batch-{key}")).push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw(key),
                    fields: string_aspect_field_patch([
                        (aspect_key("summary"), field_key("title"), title),
                        (aspect_key("summary"), field_key("status"), "open"),
                    ]),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(execution, runtime).unwrap();
    let entity = changed_entities(&committed)[0];
    release_test_commit_snapshot(runtime, &committed);
    entity
}

pub(super) fn register(
    runtime: &RelationalRuntime,
    locator: AspectFieldLocator,
) -> DerivedIndexDefinition {
    runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "authored-predicate".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: locator,
            },
            branch_scoped: true,
            unique_entity_kind: None,
        })
        .expect("actual authored-field index registration succeeds")
}

pub(super) fn key(
    definition: &DerivedIndexDefinition,
    value: &str,
) -> RelationalAuthoredWriteSelector {
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
        kind: KindId(1),
        value: AuthoritativeFieldComparisonKey::from_aspect_value(&AspectValue::String(
            value.into(),
        )),
    }
}

#[test]
fn authored_index_membership_keeps_noops_phantoms_and_whole_aspects() {
    let host = serial_request();
    let execution = ExecutionRequest::serial(&host);
    let field = entity_summary_struct_aspect(aspect_key("summary"), field_key("summary"));
    let contract = field.contract.clone();
    let runtime = AspectSchemaFixture {
        entity_aspects: vec![field],
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
    let entity = create_with_summary(&execution, &runtime, "indexed", "title");
    let source = admit_main(&runtime).unwrap();
    let commit_patch = |operation| {
        commit_entity_patch(
            &execution,
            &runtime,
            entity,
            PortableRecordAspectPatch::new([operation]),
        )
    };
    let set = |field: &str, value: &str| PortableAspectPatchOperation::PatchFields {
        basis: PortableAspectContractBasis::from_contract(&contract),
        selected_fields: vec![field_key(field)],
        field_sets: vec![PortableAspectFieldSet::new(
            field_key(field),
            AspectValue::String(value.into()),
        )],
        field_clears: vec![],
    };
    let equal = commit_patch(set("status", "open"));
    let away = commit_patch(set("status", "closed"));
    let back = commit_patch(set("status", "open"));
    let sibling = commit_patch(set("title", "new-title"));
    let whole = commit_patch(PortableAspectPatchOperation::SetWhole {
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
    });
    let phantom = create_with_summary(&execution, &runtime, "phantom", "other-title");
    let phantom_commit = admit_main(&runtime)
        .unwrap()
        .observation()
        .commit_id()
        .unwrap();
    let retired = delete_entity(&execution, &runtime, phantom);
    let target = admit_main(&runtime).unwrap();
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
    for outcome in [&equal, &away, &back, &sibling, &whole, &retired] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}
