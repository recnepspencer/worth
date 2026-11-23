use super::*;
use crate::facade::merge::MergeExecutionError;
use crate::tests::support::{
    create_branch_from_main, delete_entity, delete_entity_on_branch, durable_runtime_for_medium,
    OwnedTestDurableMedium,
};

fn drifted_schema_registry() -> crate::facade::schema::RelationalSchemaRegistry {
    crate::facade::schema::RelationalSchemaRegistry::new()
        .register_entity_kind(crate::facade::schema::EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(2),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_field_aspect(
                    crate::tests::support::aspect_key("name"),
                    field_key("name"),
                ),
                entity_field_aspect(
                    crate::tests::support::aspect_key("status"),
                    field_key("status"),
                ),
            ]),
        })
        .and_then(|registry| {
            registry.register_relation_kind(crate::facade::schema::RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(2),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: crate::facade::schema::RelationIntegrityDeclarations::default(),
            })
        })
        .expect("drifted schema registry")
}

#[test]
fn deleted_on_both_sides_prepared_merge_rejects_schema_semantic_drift() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let mut runtime = durable_runtime_for_medium(&medium);
    let entity = create_entity(execution, &runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    delete_entity(execution, &runtime, entity);
    delete_entity_on_branch(execution, &runtime, entity, BranchId("feature".to_string()));

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("prepared merge");
    runtime.set_schema_registry_for_test(drifted_schema_registry());

    match runtime.execute_prepared_merge(execution, prepared) {
        Err(MergeExecutionError::SchemaSemanticDrift { .. }) => {}
        other => panic!("expected schema semantic drift rejection, got {other:?}"),
    }
}
