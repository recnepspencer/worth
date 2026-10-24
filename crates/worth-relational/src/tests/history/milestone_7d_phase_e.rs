use crate::facade::history::BranchId;
use crate::facade::merge::{
    IdentityBasisDeclaration, IdentityBasisKind, IdentityBasisScope, LoweredMergeBlockedReason,
    MergeExecutionRequest, MergeIntent, MergeResolutionClass, RelationalMergeInspectionAdmission,
    RelationalMergeInspectionArtifact, TopologyExecutionClass,
};
use crate::facade::runtime::RelationalRuntimeApi;
use crate::schema::data::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationKindRegistration, SchemaId,
    SchemaVersionId,
};
use crate::tests::support::{
    create_branch_from_main, create_entity, create_relation,
    create_relation_in_partition_on_branch, delete_entity_on_branch, delete_relation_on_branch,
    durable_runtime_with_test_schema, relation_field_aspect, relation_source_aspect,
    relation_target_aspect, update_entity, CascadeDeletePolicy, CrossContextPolicy, KindId,
    PartitionId, RelationalRuntime, RelationalSchemaRegistry,
};
use worth_foundational::facade::AspectKey;

#[test]
fn inspection_input_round_trips_deleted_vs_modified_without_host_projection() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    update_entity(execution, &runtime, entity, "main-modified");
    delete_entity_on_branch(execution, &runtime, entity, BranchId("feature".to_string()));

    let request = MergeExecutionRequest {
        target_branch: BranchId("main".to_string()),
        source_branch: BranchId("feature".to_string()),
        merge_intent: MergeIntent::ReconcileIntoTarget,
    };
    let planning = runtime
        .merge()
        .inspect_planning_scope(execution, request.clone().into())
        .expect("planning artifact");
    let input = planning.inspection_input();
    let from_input = RelationalMergeInspectionArtifact::from_input(input.clone());
    let from_access = runtime
        .merge()
        .inspect_execution_surface(execution, request.clone().into())
        .expect("execution surface");
    let normalized_request = runtime
        .merge()
        .normalize_merge_request(request)
        .expect("normalized request");

    assert_eq!(input.request(), &normalized_request);
    assert_eq!(input.lowered_plan(), &planning.lowered_plan);
    assert_eq!(from_input, from_access);
    assert_eq!(from_input.request(), &normalized_request);
    assert_eq!(from_input.rows().len(), planning.lowered_plan.records.len());

    let denied = from_input
        .rows()
        .iter()
        .find(|row| row.blocked_reason() == Some(LoweredMergeBlockedReason::DeletedVsModified))
        .expect("deleted-vs-modified row");
    assert_eq!(
        denied.admission(),
        RelationalMergeInspectionAdmission::ExecutionDenied
    );
    assert_eq!(
        denied.blocked_reason(),
        Some(LoweredMergeBlockedReason::DeletedVsModified)
    );
}

#[test]
fn execution_surface_preserves_topology_region_conflict_authority_rows() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime =
        runtime_with_topology_identity_registry(crate::tests::support::TestDurableMedium::new());
    let source = create_entity(execution, &runtime, "source");
    let target_a = create_entity(execution, &runtime, "target-a");
    let target_b = create_entity(execution, &runtime, "target-b");
    let target_c = create_entity(execution, &runtime, "target-c");
    let target_d = create_entity(execution, &runtime, "target-d");
    let relation_a = create_relation(execution, &runtime, source, target_a, "edge-a");
    let relation_b = create_relation(execution, &runtime, source, target_b, "edge-b");
    create_branch_from_main(&runtime, "feature");
    delete_relation_on_branch(
        execution,
        &runtime,
        relation_a,
        BranchId("feature".to_string()),
    );
    delete_relation_on_branch(
        execution,
        &runtime,
        relation_b,
        BranchId("feature".to_string()),
    );
    create_relation_in_partition_on_branch(
        execution,
        &runtime,
        source,
        target_c,
        "edge-a",
        "edge-a",
        PartitionId::main(),
        BranchId("feature".to_string()),
    );
    create_relation_in_partition_on_branch(
        execution,
        &runtime,
        source,
        target_d,
        "edge-b",
        "edge-b",
        PartitionId::main(),
        BranchId("feature".to_string()),
    );

    let artifact = runtime
        .merge()
        .inspect_execution_surface(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            }
            .into(),
        )
        .expect("execution surface");

    let topology_rows = artifact
        .rows()
        .iter()
        .filter(|row| {
            row.blocked_reason() == Some(LoweredMergeBlockedReason::TopologyRegionConflict)
                && matches!(
                    row.resolution_class(),
                    MergeResolutionClass::Topology(TopologyExecutionClass::TopologyRegionConflict)
                )
        })
        .collect::<Vec<_>>();
    assert_eq!(topology_rows.len(), 2);
    assert!(topology_rows
        .iter()
        .all(|row| { row.admission() == RelationalMergeInspectionAdmission::ExecutionDenied }));
}

fn runtime_with_topology_identity_registry(
    medium: std::sync::Arc<crate::tests::support::TestDurableMedium>,
) -> RelationalRuntime {
    let label_key = AspectKey::new("label").unwrap();
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                    relation_field_aspect(
                        label_key.clone(),
                        crate::tests::support::field_key("label"),
                    ),
                    relation_source_aspect(),
                    relation_target_aspect(),
                ])
                .with_identity_declarations(vec![IdentityBasisDeclaration {
                    scope: IdentityBasisScope::AspectKey(label_key.clone()),
                    basis: IdentityBasisKind::DeclaredKeySet(std::sync::Arc::from([label_key])),
                }]),
                relation_integrity: crate::schema::data::RelationIntegrityDeclarations::default(),
            })
        })
        .expect("topology identity registry");

    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build_durable(medium.attachment())
}
