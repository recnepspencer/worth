use crate::facade::history::BranchId;
use crate::facade::merge::{
    AspectMergePolicyDeclaration, AspectMergePolicyKind, IdentityBasisDeclaration,
    IdentityBasisKind, IdentityBasisScope, MergeIntent,
};
use crate::facade::runtime::RelationalRuntimeApi;
use crate::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationKindRegistration,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::tests::support::{
    checkpoint_and_recover_with, create_branch_from_main, create_entity, entity_field_aspect,
    persisted_runtime_with_test_schema, update_entity, update_entity_on_branch,
};
use worth_foundational::facade::AspectKey;

fn inspect_recovered_merge_planning_artifact(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> crate::facade::merge::MergePlanningArtifactCore {
    let runtime = persisted_runtime_with_test_schema();
    let shared = create_entity(&runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    update_entity(&runtime, shared, "shared-main");
    update_entity_on_branch(
        &runtime,
        shared,
        "shared-feature",
        BranchId("feature".to_string()),
    );

    let artifact = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            crate::merge::data::MergePlanningRequest::new(
                BranchId("main".to_string()),
                BranchId("feature".to_string()),
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("live merge planning artifact");

    let (_recovery, recovered) =
        checkpoint_and_recover_with(execution, &runtime, persisted_runtime_with_test_schema);
    let recovered_artifact = recovered
        .merge()
        .inspect_planning_scope(
            execution,
            crate::merge::data::MergePlanningRequest::new(
                BranchId("main".to_string()),
                BranchId("feature".to_string()),
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("recovered merge planning artifact");

    assert_eq!(recovered_artifact, artifact);
    artifact
}

#[test]
fn merge_planning_artifact_certification_is_stable_across_recovery() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let artifact = inspect_recovered_merge_planning_artifact(execution);

    assert_eq!(artifact.digest_basis.schema, artifact.schema_snapshot);
    assert_eq!(
        artifact.digest_basis.decision_log,
        artifact.decision_log_digest_basis
    );
    assert_eq!(
        artifact.digest_basis.lowered_plan.records.len(),
        artifact.lowered_plan.records.len()
    );
    assert_eq!(
        artifact.decision_log_digest_basis.canonical_records.len(),
        artifact.decision_log.decisions.len()
    );
    assert!(!artifact.schema_snapshot.touched_kinds.is_empty());
    assert!(artifact.lowered_plan.record_count > 0);
    assert!(artifact.summary.request_summary.contains("main"));
}

#[test]
fn merge_planning_schema_snapshot_changes_when_schema_semantics_change() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    fn runtime_with_registry(
        merge_policy: AspectMergePolicyKind,
    ) -> crate::facade::runtime::RelationalRuntime {
        let name_key = AspectKey::new("name").unwrap();
        let registry = RelationalSchemaRegistry::new()
            .register_entity_kind(EntityKindRegistration {
                kind_id: crate::facade::identity::KindId(1),
                kind_name: "test.entity".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                    entity_field_aspect(name_key.clone(), crate::tests::support::field_key("name")),
                ])
                .with_identity_declarations(vec![IdentityBasisDeclaration {
                    scope: IdentityBasisScope::AspectKey(name_key.clone()),
                    basis: IdentityBasisKind::DeclaredKeySet(vec![name_key.clone()].into()),
                }])
                .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
                    aspect_key: name_key,
                    policy: merge_policy,
                }]),
            })
            .and_then(|registry| {
                registry.register_relation_kind(RelationKindRegistration {
                    kind_id: crate::facade::identity::KindId(2),
                    kind_name: "test.relation".to_string(),
                    schema_id: SchemaId("test".to_string()),
                    schema_version_id: SchemaVersionId(1),
                    cross_context_policy: crate::config::data::CrossContextPolicy::AllowExplicit,
                    cascade_delete_policy:
                        crate::config::data::CascadeDeletePolicy::CascadeDeleteRelations,
                    aspect_contract_declarations: KindAspectContractDeclarations::default(),
                    relation_integrity: crate::schema::data::RelationIntegrityDeclarations::default(
                    ),
                })
            })
            .unwrap();
        RelationalRuntimeApi::builder()
            .schema_registry(registry)
            .build()
    }

    fn snapshot_digest(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &mut crate::facade::runtime::RelationalRuntime,
    ) -> String {
        let shared = create_entity(runtime, "shared");
        create_branch_from_main(runtime, "feature");
        update_entity(runtime, shared, "shared-main");
        update_entity_on_branch(
            runtime,
            shared,
            "shared-feature",
            BranchId("feature".to_string()),
        );
        let artifact = runtime
            .merge()
            .inspect_planning_scope(
                execution,
                crate::merge::data::MergePlanningRequest::new(
                    BranchId("main".to_string()),
                    BranchId("feature".to_string()),
                    MergeIntent::ReconcileIntoTarget,
                ),
            )
            .expect("merge planning artifact");
        crate::merge::data::schema_snapshot_digest(&artifact.schema_snapshot)
    }

    let mut prefer_richer_runtime = runtime_with_registry(AspectMergePolicyKind::PreferRicher);
    let mut fail_on_conflict_runtime = runtime_with_registry(AspectMergePolicyKind::FailOnConflict);
    let prefer_richer_digest = snapshot_digest(execution, &mut prefer_richer_runtime);
    let fail_on_conflict_digest = snapshot_digest(execution, &mut fail_on_conflict_runtime);

    assert_ne!(prefer_richer_digest, fail_on_conflict_digest);
}
