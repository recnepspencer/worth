mod aspect_merge_policy_denials;
mod deleted_convergence;
mod last_writer_wins_resolution;
mod merge_descriptive_touches;
mod monotonic_counter_resolution;
mod prepared_merge_denials;
mod topology_local_rewire;
mod topology_region;

use crate::diagnostics::data::RelationalDiagnosticValue;
use crate::facade::diagnostics::DiagnosticCode;
use crate::facade::history::BranchId;
use crate::facade::merge::{
    DeletionExecutionClass, IdentityBasisDeclaration, IdentityBasisKind, IdentityBasisScope,
    LoweredMergeBlockedReason, LoweredRecordDenialKind, MergeExecutionError, MergeExecutionRequest,
    MergeIntent, MergePolicyDecisionBoundary, MergeResolutionClass,
    MergeResolvedAspectValueStrategy, RelationConflictPropagation, TopologyExecutionClass,
    TopologyRegionConflictReason,
};
use crate::facade::runtime::RelationalRuntimeApi;
use crate::facade::transactions::{
    CreateIntent, EntityMutationIntent, MutationIntent, RecordRef, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};
use crate::merge::data::AspectMergePolicyDeclaration;
use crate::schema::data::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationKindRegistration, SchemaId,
    SchemaRegistryErrorClass, SchemaVersionId,
};
use crate::tests::support::{
    capture_aspect_truth_bundle, changed_entities, checkpoint_and_recover_with,
    create_branch_from_main, create_entity, create_relation_in_partition_on_branch, delete_entity,
    delete_entity_on_branch, delete_relation_on_branch, diagnostic_field,
    diagnostic_field_optional, entity_field_aspect, entity_i64_field_aspect,
    entity_summary_struct_aspect, field_key, persisted_runtime_with_test_schema,
    read_entity_aspect_field, unique_test_store_path, update_entity,
};
use worth_foundational::facade::AspectKey;

fn drifted_schema_registry() -> crate::facade::schema::RelationalSchemaRegistry {
    crate::facade::schema::RelationalSchemaRegistry::new()
        .register_entity_kind(crate::facade::schema::EntityKindRegistration {
            kind_id: crate::facade::identity::KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: crate::facade::schema::SchemaId("test".to_string()),
            schema_version_id: crate::facade::schema::SchemaVersionId(2),
            aspect_contract_declarations:
                crate::facade::schema::KindAspectContractDeclarations::new(vec![
                    crate::tests::support::entity_field_aspect(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                    ),
                    crate::tests::support::entity_field_aspect(
                        crate::tests::support::aspect_key("status"),
                        crate::tests::support::field_key("status"),
                    ),
                ]),
        })
        .and_then(|registry| {
            registry.register_relation_kind(crate::facade::schema::RelationKindRegistration {
                kind_id: crate::facade::identity::KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: crate::facade::schema::SchemaId("test".to_string()),
                schema_version_id: crate::facade::schema::SchemaVersionId(2),
                cross_context_policy: crate::tests::support::CrossContextPolicy::AllowExplicit,
                cascade_delete_policy:
                    crate::tests::support::CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations:
                    crate::facade::schema::KindAspectContractDeclarations::default(),
                relation_integrity: crate::facade::schema::RelationIntegrityDeclarations::default(),
            })
        })
        .expect("drifted schema registry")
}

fn topology_identity_registry() -> crate::facade::schema::RelationalSchemaRegistry {
    let label_key = worth_foundational::facade::AspectKey::new("label").unwrap();
    crate::facade::schema::RelationalSchemaRegistry::new()
        .register_entity_kind(crate::facade::schema::EntityKindRegistration {
            kind_id: crate::facade::identity::KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: crate::facade::schema::SchemaId("test".to_string()),
            schema_version_id: crate::facade::schema::SchemaVersionId(1),
            aspect_contract_declarations:
                crate::facade::schema::KindAspectContractDeclarations::default(),
        })
        .and_then(|registry| {
            registry.register_relation_kind(crate::facade::schema::RelationKindRegistration {
                kind_id: crate::facade::identity::KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: crate::facade::schema::SchemaId("test".to_string()),
                schema_version_id: crate::facade::schema::SchemaVersionId(1),
                cross_context_policy: crate::tests::support::CrossContextPolicy::AllowExplicit,
                cascade_delete_policy:
                    crate::tests::support::CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations:
                    crate::facade::schema::KindAspectContractDeclarations::new(vec![
                        crate::tests::support::relation_field_aspect(
                            crate::tests::support::aspect_key("label"),
                            crate::tests::support::field_key("label"),
                        ),
                        crate::tests::support::relation_source_aspect(),
                        crate::tests::support::relation_target_aspect(),
                    ])
                    .with_identity_declarations(vec![
                        crate::facade::merge::IdentityBasisDeclaration {
                            scope: crate::facade::merge::IdentityBasisScope::AspectKey(
                                label_key.clone(),
                            ),
                            basis: crate::facade::merge::IdentityBasisKind::DeclaredKeySet(
                                std::sync::Arc::from([label_key]),
                            ),
                        },
                    ]),
                relation_integrity: crate::facade::schema::RelationIntegrityDeclarations::default(),
            })
        })
        .expect("topology identity registry")
}

fn persisted_runtime_with_topology_identity_registry(
    root_path: std::path::PathBuf,
) -> crate::facade::runtime::RelationalRuntime {
    crate::facade::runtime::RelationalRuntimeApi::builder()
        .profile(crate::tests::support::RelationalRuntimeProfile::CertificationCore)
        .schema_registry(topology_identity_registry())
        .durability_mode(crate::tests::support::DurabilityMode::PersistedSegmentedLocalFs)
        .durable_store_layout(crate::tests::support::DurableStoreLayout {
            root_path,
            segment_commit_capacity: 2,
        })
        .build()
}
use crate::{
    config::data::{CascadeDeletePolicy, CrossContextPolicy},
    facade::identity::KindId,
    facade::merge::AspectMergePolicyKind,
    schema::data::{RelationalSchemaRegistry, SchemaRegistryError},
};

fn runtime_with_aspect_field_merge_policy(
    aspect_key: AspectKey,
    field_key: worth_foundational::facade::FieldKey,
    merge_policy: AspectMergePolicyKind,
) -> crate::facade::runtime::RelationalRuntime {
    let value_aspect = match merge_policy {
        AspectMergePolicyKind::MonotonicCounter => {
            entity_i64_field_aspect(aspect_key.clone(), field_key.clone())
        }
        _ => entity_field_aspect(aspect_key.clone(), field_key.clone()),
    };
    let registry = register_aspect_field_merge_policy(aspect_key, value_aspect, merge_policy)
        .expect("schema registry");
    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
}

fn register_aspect_field_merge_policy(
    aspect_key: AspectKey,
    value_aspect: crate::schema::data::DeclaredAspectContractBinding,
    merge_policy: AspectMergePolicyKind,
) -> Result<RelationalSchemaRegistry, SchemaRegistryError> {
    let name_key = AspectKey::new("name").unwrap();
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_field_aspect(name_key.clone(), crate::tests::support::field_key("name")),
                value_aspect,
            ])
            .with_identity_declarations(vec![IdentityBasisDeclaration {
                scope: IdentityBasisScope::AspectKey(name_key.clone()),
                basis: IdentityBasisKind::DeclaredKeySet(vec![name_key].into()),
            }])
            .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
                aspect_key,
                policy: merge_policy,
            }]),
        })
        .and_then(|registry: RelationalSchemaRegistry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: crate::schema::data::RelationIntegrityDeclarations::default(),
            })
        })
}

fn i64_counter_value(value: i64) -> worth_foundational::facade::AspectValue {
    worth_foundational::facade::AspectValue::Int64(value)
}

fn string_aspect_field_patch_for_target(
    aspect_key: AspectKey,
    field_key: worth_foundational::facade::FieldKey,
    value: &str,
) -> crate::transactions::data::AspectFieldPatch {
    crate::transactions::data::AspectFieldPatch::from(std::collections::BTreeMap::from([(
        crate::transactions::data::planned_single_field_locator(aspect_key, field_key),
        crate::tests::support::string_aspect_value(value),
    )]))
}

fn create_entity_with_aspect_fields(
    runtime: &mut crate::facade::runtime::RelationalRuntime,
    client_key: &str,
    fields: crate::transactions::data::AspectFieldPatch,
) -> crate::facade::identity::EntityId {
    create_entity_with_aspect_fields_on_branch(
        runtime,
        client_key,
        fields,
        BranchId("main".to_string()),
    )
}

fn create_entity_with_aspect_fields_on_branch(
    runtime: &mut crate::facade::runtime::RelationalRuntime,
    client_key: &str,
    fields: crate::transactions::data::AspectFieldPatch,
    branch_id: BranchId,
) -> crate::facade::identity::EntityId {
    let mut txn =
        crate::tests::support::test_owner_begin_transaction_for_branch(runtime, branch_id);
    txn.push_batch(WorkerIntentBatch::new(format!("create-{client_key}")).push(
        MutationIntent::Create(CreateIntent::Entity(
            crate::transactions::data::EntitySpec {
                partition_id: crate::facade::identity::PartitionId::main(),
                kind_id: KindId(1),
                client_key: crate::symbols::data::ClientKey::raw(client_key),
                fields: aspect_fields_with_identity_name(client_key, fields),
            },
        )),
    ))
    .expect("test staging stays within configured resource budgets");
    changed_entities(&txn.commit(runtime).unwrap())[0]
}

fn update_entity_aspect_fields_on_branch(
    runtime: &mut crate::facade::runtime::RelationalRuntime,
    entity_id: crate::facade::identity::EntityId,
    fields: crate::transactions::data::AspectFieldPatch,
    branch_id: BranchId,
) {
    let stable_name = read_entity_aspect_field_display(
        runtime,
        &branch_id,
        entity_id,
        AspectKey::new("name").unwrap(),
        field_key("name"),
    )
    .to_string();
    let mut txn =
        crate::tests::support::test_owner_begin_transaction_for_branch(runtime, branch_id);
    txn.push_batch(
        WorkerIntentBatch::new("update-aspect-fields").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id,
                fields: aspect_fields_with_identity_name(&stable_name, fields),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(runtime).unwrap();
}

fn aspect_fields_with_identity_name(
    client_key: &str,
    fields: crate::transactions::data::AspectFieldPatch,
) -> crate::transactions::data::AspectFieldPatch {
    let mut targets = fields
        .iter()
        .map(|(target, value)| (target.clone(), value.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    targets
        .entry(crate::transactions::data::planned_single_field_locator(
            AspectKey::new("name").expect("valid identity aspect key"),
            worth_foundational::facade::FieldKey::new("name").expect("valid identity field key"),
        ))
        .or_insert_with(|| crate::tests::support::string_aspect_value(client_key));
    crate::transactions::data::AspectFieldPatch::new(targets)
}

fn read_entity_aspect_field_display(
    runtime: &crate::facade::runtime::RelationalRuntime,
    branch: &BranchId,
    entity_id: crate::facade::identity::EntityId,
    aspect_key: AspectKey,
    field: worth_foundational::facade::FieldKey,
) -> String {
    let version_id = runtime
        .history()
        .branch_head(branch)
        .expect("branch head")
        .version_id;
    runtime
        .read_truth()
        .read_version(version_id)
        .get_entity(entity_id)
        .and_then(|entity| read_entity_aspect_field(entity, aspect_key, field))
        .expect("aspect field display value")
}
