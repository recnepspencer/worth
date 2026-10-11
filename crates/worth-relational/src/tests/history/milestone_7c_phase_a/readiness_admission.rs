use crate::facade::history::BranchId;
use crate::facade::identity::KindId;
use crate::facade::merge::{
    AspectMergePolicyDeclaration, AspectMergePolicyKind, MergeExecutionPreparationError,
    MergeExecutionRequest, MergeIntent,
};
use crate::facade::runtime::RelationalRuntimeApi;
use crate::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationIntegrityDeclarations,
    RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::tests::support::{
    create_branch_from_main, create_entity, create_entity_outcome_on_branch, entity_field_aspect,
    persisted_runtime_with_test_schema, update_entity, update_entity_on_branch,
    CascadeDeletePolicy, CrossContextPolicy,
};
use worth_foundational::facade::AspectKey;

#[test]
fn prepare_merge_execution_admits_fully_ready_source_only_addition() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

    let request = MergeExecutionRequest {
        target_branch: BranchId("main".to_string()),
        source_branch: BranchId("feature".to_string()),
        merge_intent: MergeIntent::ReconcileIntoTarget,
    };

    let prepared = runtime
        .merge()
        .prepare_merge_execution(execution, request.clone())
        .expect("merge execution should prepare");
    let normalized_request = runtime
        .merge()
        .normalize_merge_request(request.clone())
        .expect("normalized request");

    assert_eq!(prepared.request(), &normalized_request);
    assert!(prepared.artifact().lowered_plan.fully_execution_ready);
    assert_eq!(prepared.artifact().lowered_plan.blocked_count, 0);
    assert_eq!(prepared.artifact().lowered_plan.rejected_count, 0);
    assert!(prepared
        .execution_ready_plan()
        .lowered_records
        .iter()
        .all(|record| matches!(
            record.record_decision,
            crate::merge::data::LoweredRecordDecision::Execute(_)
        )));
}

#[test]
fn prepare_merge_execution_rejects_blocked_merge_plans() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let shared = create_entity(execution, &runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    update_entity(execution, &runtime, shared, "shared-main");
    update_entity_on_branch(
        execution,
        &runtime,
        shared,
        "shared-feature",
        BranchId("feature".to_string()),
    );

    let error = runtime
        .merge()
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect_err("blocked merge plan must not prepare");

    match error {
        MergeExecutionPreparationError::NotExecutionReady(report) => {
            assert_eq!(report.blocked_count, 1);
            assert_eq!(report.rejected_count, 0);
            assert_eq!(report.denied_records.len(), 1);
            assert_eq!(
                report.denied_records[0].decision,
                crate::merge::data::LoweredRecordDecisionKind::Block
            );
        }
        other => panic!("expected readiness failure, got {other:?}"),
    }
}

#[test]
fn prepare_merge_execution_rejects_rejected_merge_plans() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_field_aspect(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                ),
            ])
            .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
                aspect_key: AspectKey::new("name").unwrap(),
                policy: AspectMergePolicyKind::FailOnConflict,
            }]),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
        .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build();
    let shared = create_entity(execution, &runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    update_entity(execution, &runtime, shared, "shared-main");
    update_entity_on_branch(
        execution,
        &runtime,
        shared,
        "shared-feature",
        BranchId("feature".to_string()),
    );

    let error = runtime
        .merge()
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect_err("rejected merge plan must not prepare");

    match error {
        MergeExecutionPreparationError::NotExecutionReady(report) => {
            assert_eq!(report.blocked_count, 0);
            assert_eq!(report.rejected_count, 1);
            assert_eq!(report.denied_records.len(), 1);
            assert_eq!(
                report.denied_records[0].decision,
                crate::merge::data::LoweredRecordDecisionKind::Reject
            );
        }
        other => panic!("expected readiness failure, got {other:?}"),
    }
}

#[test]
fn runtime_prepare_merge_execution_matches_merge_access_surface() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );
    let request = MergeExecutionRequest {
        target_branch: BranchId("main".to_string()),
        source_branch: BranchId("feature".to_string()),
        merge_intent: MergeIntent::ReconcileIntoTarget,
    };

    let via_runtime = runtime
        .prepare_merge_execution(execution, request.clone())
        .expect("runtime merge prepare");
    let via_access = runtime
        .merge()
        .prepare_merge_execution(execution, request.clone())
        .expect("merge access prepare");
    let normalized_request = runtime
        .merge()
        .normalize_merge_request(request)
        .expect("normalized request");

    assert_eq!(via_runtime.request(), &normalized_request);
    assert_eq!(via_runtime.artifact(), via_access.artifact());
}
