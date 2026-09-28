use crate::facade::history::BranchId;
use crate::facade::indexes::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
};
use crate::storage::data::AuthoritativeFieldComparisonKey;
use crate::tests::support::*;
use crate::validation::data::{InvariantCatalog, InvariantRegistration, InvariantRule};

#[test]
fn unique_entity_aspect_field_index_refresh_rewrites_name_membership_after_entity_update() {
    let runtime = runtime_with_declared_aspect_schema_and_invariants(InvariantCatalog {
        registrations: vec![InvariantRegistration::mutation_sensitive_blocking(
            InvariantRule::unique_entity_aspect_field(aspect_key("name"), field_key("name")),
        )],
    });
    let alpha = create_entity(&runtime, "alpha");
    let beta = create_entity(&runtime, "beta");

    runtime
        .index_authority()
        .rebuild_unique_entity_aspect_field_indexes()
        .expect("main-branch unique index rebuild admits its exact basis");
    update_entity(&runtime, beta, "gamma");

    let index_access = runtime.index_access();
    let name_field = aspect_field_locator(aspect_key("name"), field_key("name"));
    let entries = index_access
        .entity_unique_field_entries(&name_field)
        .expect("name entries");

    assert_eq!(
        entries
            .get(&field_comparison_key("alpha"))
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>(),
        vec![alpha]
    );
    assert!(!entries.contains_key(&field_comparison_key("beta")));
    assert_eq!(
        entries
            .get(&field_comparison_key("gamma"))
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>(),
        vec![beta]
    );
}

#[test]
fn unique_entity_aspect_field_index_keeps_same_field_key_separate_by_aspect_locator() {
    let name_field = field_key("name");
    let legal_name = aspect_key("legal.name");
    let display_name = aspect_key("display.name");
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(
            AspectSchemaFixture {
                entity_aspects: vec![
                    entity_field_aspect(legal_name.clone(), name_field.clone()),
                    entity_field_aspect(display_name.clone(), name_field.clone()),
                    lifecycle_aspect(),
                ],
                relation_aspects: vec![
                    relation_field_aspect(aspect_key("label"), field_key("label")),
                    lifecycle_aspect(),
                    relation_source_aspect(),
                    relation_target_aspect(),
                ],
                ..AspectSchemaFixture::default()
            }
            .build_registry(),
        )
        .invariant_catalog(InvariantCatalog {
            registrations: vec![
                InvariantRegistration::mutation_sensitive_blocking(
                    InvariantRule::unique_entity_aspect_field(
                        legal_name.clone(),
                        name_field.clone(),
                    ),
                ),
                InvariantRegistration::mutation_sensitive_blocking(
                    InvariantRule::unique_entity_aspect_field(
                        display_name.clone(),
                        name_field.clone(),
                    ),
                ),
            ],
        })
        .build();

    let alpha = create_entity_with_aspect_fields(
        &runtime,
        "alpha",
        aspect_field_patch_from_values([
            (
                legal_name.clone(),
                name_field.clone(),
                string_aspect_value("alpha-legal"),
            ),
            (
                display_name.clone(),
                name_field.clone(),
                string_aspect_value("alpha-display"),
            ),
        ]),
    );
    let beta = create_entity_with_aspect_fields(
        &runtime,
        "beta",
        aspect_field_patch_from_values([
            (
                legal_name.clone(),
                name_field.clone(),
                string_aspect_value("beta-legal"),
            ),
            (
                display_name.clone(),
                name_field.clone(),
                string_aspect_value("beta-display"),
            ),
        ]),
    );

    runtime
        .index_authority()
        .rebuild_unique_entity_aspect_field_indexes()
        .expect("main-branch unique index rebuild admits its exact basis");
    let index_access = runtime.index_access();
    let legal_entries = index_access
        .entity_unique_field_entries(&aspect_field_locator(legal_name, name_field.clone()))
        .expect("legal name entries");
    let display_entries = index_access
        .entity_unique_field_entries(&aspect_field_locator(display_name, name_field))
        .expect("display name entries");

    assert_eq!(
        legal_entries
            .get(&field_comparison_key("alpha-legal"))
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>(),
        vec![alpha]
    );
    assert!(!legal_entries.contains_key(&field_comparison_key("alpha-display")));
    assert_eq!(
        display_entries
            .get(&field_comparison_key("beta-display"))
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>(),
        vec![beta]
    );
    assert!(!display_entries.contains_key(&field_comparison_key("beta-legal")));
}

#[test]
fn derived_index_build_materializes_latest_visible_entity_field_values() {
    let runtime = runtime_with_test_schema();
    let alpha = create_entity(&runtime, "alpha");
    let create_outcome = update_entity(&runtime, alpha, "gamma");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: true,
    });

    let build = runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: create_outcome.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        });
    let index_access = runtime.index_access();
    let generation = index_access
        .latest_generation(index.index_id, &BranchId("main".to_string()))
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
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn.commit(&runtime).expect("entity create succeeds");
    let alpha = changed_entities(&outcome)[0];
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.summary.title".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("summary"), field_key("title")),
        },
        branch_scoped: true,
    });

    let build = runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: outcome.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        });
    let index_access = runtime.index_access();
    let generation = index_access
        .latest_generation(index.index_id, &BranchId("main".to_string()))
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

fn create_entity_with_aspect_fields(
    runtime: &RelationalRuntime,
    client_key: &str,
    fields: AspectFieldPatch,
) -> crate::facade::identity::EntityId {
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    txn.push_batch(WorkerIntentBatch::new(format!("batch-{client_key}")).push(
        MutationIntent::Create(CreateIntent::Entity(
            crate::transactions::data::EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: crate::symbols::data::ClientKey::raw(client_key),
                fields,
            },
        )),
    ))
    .expect("test staging stays within configured resource budgets");
    changed_entities(&txn.commit(runtime).expect("entity create succeeds"))[0]
}
