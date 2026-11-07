use crate::facade::history::BranchId;
use crate::facade::indexes::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
};
use crate::storage::data::AuthoritativeFieldComparisonKey;
use crate::tests::support::*;

#[test]
fn derived_index_build_materializes_latest_visible_entity_field_values() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let alpha = create_entity(execution, &runtime, "alpha");
    let create_outcome = update_entity(execution, &runtime, alpha, "gamma");
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "entity.name".to_string(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
            branch_scoped: true,
        })
        .expect("index fixture has identity capacity");

    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: create_outcome.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        },
    );
    let index_access = runtime.index_access();
    let generation = index_access
        .read_latest_generation(execution, index.index_id, &BranchId("main".to_string()))
        .unwrap()
        .expect("latest generation");

    assert!(build.failed_indexes.is_empty());
    match &generation.entries {
        crate::indexes::data::DerivedIndexEntries::EntityField(entries) => {
            assert!(!entries.contains_key(&field_comparison_key("alpha")));
            assert_eq!(
                entries
                    .get(&field_comparison_key("gamma"))
                    .cloned()
                    .unwrap_or_default(),
                vec![alpha].into()
            );
        }
        other => panic!("expected entity field entries, got {other:?}"),
    }
}

#[test]
fn derived_index_build_materializes_declared_struct_field_through_field_projection_scope() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(aspect_key("name"), field_key("name")),
            entity_summary_struct_aspect(aspect_key("summary"), field_key("summary")),
        ],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("batch-alpha").push(MutationIntent::Create(CreateIntent::Entity(
            crate::transactions::data::EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: crate::symbols::data::ClientKey::raw("alpha"),
                fields: AspectFieldPatch::new(std::collections::BTreeMap::from([
                    (
                        crate::transactions::data::planned_single_field_locator(
                            aspect_key("name"),
                            field_key("name"),
                        ),
                        string_aspect_value("alpha"),
                    ),
                    (
                        crate::transactions::data::planned_single_field_locator(
                            aspect_key("summary"),
                            field_key("title"),
                        ),
                        string_aspect_value("projected-title"),
                    ),
                    (
                        crate::transactions::data::planned_single_field_locator(
                            aspect_key("summary"),
                            field_key("status"),
                        ),
                        string_aspect_value("hidden-status"),
                    ),
                ])),
            },
        ))),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn
        .commit(execution, &runtime)
        .expect("entity create succeeds");
    let alpha = changed_entities(&outcome)[0];
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "entity.summary.title".to_string(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("summary"), field_key("title")),
            },
            branch_scoped: true,
        })
        .expect("index fixture has identity capacity");

    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: outcome.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        },
    );
    let index_access = runtime.index_access();
    let generation = index_access
        .read_latest_generation(execution, index.index_id, &BranchId("main".to_string()))
        .unwrap()
        .expect("latest generation");

    assert!(build.failed_indexes.is_empty());
    match &generation.entries {
        crate::indexes::data::DerivedIndexEntries::EntityField(entries) => {
            assert_eq!(
                entries
                    .get(&field_comparison_key("projected-title"))
                    .cloned()
                    .unwrap_or_default(),
                vec![alpha].into()
            );
            assert!(!entries.contains_key(&field_comparison_key("hidden-status")));
        }
        other => panic!("expected entity field entries, got {other:?}"),
    }
}

fn field_comparison_key(value: &str) -> AuthoritativeFieldComparisonKey {
    AuthoritativeFieldComparisonKey::from_aspect_value(&string_aspect_value(value))
}
