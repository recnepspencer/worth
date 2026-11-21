//! Declared aspect merge-policy registration, shared by its denial units.
mod aspect_merge_policy_denials;
mod merge_descriptive_touches;

use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::facade::identity::KindId;
use crate::facade::merge::{
    AspectMergePolicyKind, IdentityBasisDeclaration, IdentityBasisKind, IdentityBasisScope,
};
use crate::merge::data::AspectMergePolicyDeclaration;
use crate::schema::data::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationKindRegistration,
    RelationalSchemaRegistry, SchemaId, SchemaRegistryError, SchemaRegistryErrorClass,
    SchemaVersionId,
};
use crate::tests::support::{entity_field_aspect, entity_summary_struct_aspect, field_key};
use worth_foundational::facade::AspectKey;

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
                entity_field_aspect(name_key.clone(), field_key("name")),
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

// Fixture steps restored with this module's merge court (f9f15bf8c8).
fn runtime_with_aspect_field_merge_policy(
    aspect_key: AspectKey,
    field_key: worth_foundational::facade::FieldKey,
    merge_policy: AspectMergePolicyKind,
) -> crate::facade::runtime::RelationalRuntime {
    let value_aspect = entity_field_aspect(aspect_key.clone(), field_key);
    let registry = register_aspect_field_merge_policy(aspect_key, value_aspect, merge_policy)
        .expect("schema registry");
    crate::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .expect("valid native runtime configuration")
}

fn create_branch_from_main(runtime: &crate::facade::runtime::RelationalRuntime, branch_name: &str) {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    runtime
        .history_authority()
        .fork_branch_from(
            execution,
            crate::facade::history::BranchId(branch_name.to_string()),
            &crate::facade::history::BranchId("main".to_string()),
        )
        .unwrap();
}

fn create_entity_with_aspect_fields(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut crate::facade::runtime::RelationalRuntime,
    client_key: &str,
    fields: crate::transactions::data::AspectFieldPatch,
) -> crate::facade::identity::EntityId {
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
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_branch(
        runtime,
        crate::facade::history::BranchId("main".to_string()),
    );
    txn.push_batch(
        crate::facade::transactions::WorkerIntentBatch::new(format!("create-{client_key}")).push(
            crate::facade::transactions::MutationIntent::Create(
                crate::facade::transactions::CreateIntent::Entity(
                    crate::transactions::data::EntitySpec {
                        partition_id: crate::facade::identity::PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: crate::symbols::data::ClientKey::raw(client_key),
                        fields: crate::transactions::data::AspectFieldPatch::new(targets),
                    },
                ),
            ),
        ),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    crate::tests::support::changed_entities(&txn.commit(execution, runtime).unwrap())[0]
}

fn read_entity_aspect_field_display(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::facade::runtime::RelationalRuntime,
    branch: &crate::facade::history::BranchId,
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
        .read_version(execution, version_id)
        .expect("read request remains live")
        .get_entity(entity_id)
        .and_then(|entity| {
            crate::tests::support::read_entity_aspect_field(entity, aspect_key, field)
        })
        .expect("aspect field display value")
}

fn rebuild_branch_index(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::facade::runtime::RelationalRuntime,
    branch: &str,
    index: crate::facade::indexes::DerivedIndexId,
) {
    let identity = runtime
        .branch_identity(&crate::facade::history::BranchId(branch.to_string()))
        .unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    runtime
        .index_authority()
        .reconstruct_for_basis(
            execution,
            crate::facade::indexes::DerivedIndexBuildRequest {
                source_commit_id: basis.observation().commit_id().unwrap(),
                branch_id: crate::facade::history::BranchId(branch.to_string()),
                index_ids: vec![index],
            },
            &basis,
            crate::facade::indexes::DerivedIndexMaintenanceBudget {
                maximum_work_units: None,
                maximum_cold_record_slots: None,
                maximum_derived_rows: None,
            },
        )
        .expect("branch index rebuilt for its admitted basis");
}
