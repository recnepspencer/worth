//! Merge preparation admits only fully execution-ready plans and reports each denied readiness.
use super::branch_history_publications::{
    create_entity, create_entity_outcome_on_branch, fork, update_entity, update_entity_on_branch,
};
use super::merge_execution_schema::entity_field_aspect;
use super::merge_proof_journey::merge_request;
use worth_foundational::facade::AspectKey;
use worth_relational::facade::{
    config::{CascadeDeletePolicy, CrossContextPolicy},
    history::BranchId,
    identity::KindId,
    merge::{
        AspectMergePolicyDeclaration, AspectMergePolicyKind, LoweredRecordDecision,
        LoweredRecordDecisionKind, MergeExecutionPreparationError,
    },
    runtime::RelationalRuntimeApi,
    schema::{
        EntityKindRegistration, KindAspectContractDeclarations, RelationIntegrityDeclarations,
        RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
    },
};
use worth_testkit_relational::{
    aspect_storage::definition::{aspect_key, field_key},
    durable_medium::TestDurableMedium,
    durable_publication::durable_runtime,
};

#[test]
fn prepare_merge_execution_admits_fully_ready_source_only_addition() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "root");
    fork(execution, &runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

    let request = merge_request();

    let prepared = runtime
        .merge()
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(request.clone())
                .unwrap(),
        )
        .expect("merge execution should prepare");
    let normalized_request = runtime
        .merge()
        .normalize_merge_request(runtime.bind_merge_execution_request(request).unwrap())
        .expect("normalized request");

    assert_eq!(prepared.request(), &normalized_request);
    assert!(prepared.artifact().lowered_plan.fully_execution_ready);
    assert_eq!(prepared.artifact().lowered_plan.blocked_count, 0);
    assert_eq!(prepared.artifact().lowered_plan.rejected_count, 0);
    assert!(prepared
        .artifact()
        .lowered_plan
        .records
        .iter()
        .all(|record| matches!(record.record_decision, LoweredRecordDecision::Execute(_))));
}

#[test]
fn prepare_merge_execution_rejects_blocked_merge_plans() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let shared = create_entity(execution, &runtime, "shared");
    fork(execution, &runtime, "feature");
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
            runtime
                .bind_merge_execution_request(merge_request())
                .unwrap(),
        )
        .expect_err("blocked merge plan must not prepare");

    match error {
        MergeExecutionPreparationError::NotExecutionReady(report) => {
            assert_eq!(report.blocked_count, 1);
            assert_eq!(report.rejected_count, 0);
            assert_eq!(report.denied_records.len(), 1);
            assert_eq!(
                report.denied_records[0].decision,
                LoweredRecordDecisionKind::Block
            );
        }
        other => panic!("expected readiness failure, got {other:?}"),
    }
}

#[test]
fn prepare_merge_execution_rejects_rejected_merge_plans() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_field_aspect(aspect_key("name"), field_key("name")),
            ])
            .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
                aspect_key: AspectKey::new("name").unwrap(),
                policy: AspectMergePolicyKind::FailOnConflict,
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
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
        .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .expect("valid native runtime configuration");
    let shared = create_entity(execution, &runtime, "shared");
    fork(execution, &runtime, "feature");
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
            runtime
                .bind_merge_execution_request(merge_request())
                .unwrap(),
        )
        .expect_err("rejected merge plan must not prepare");

    match error {
        MergeExecutionPreparationError::NotExecutionReady(report) => {
            assert_eq!(report.blocked_count, 0);
            assert_eq!(report.rejected_count, 1);
            assert_eq!(report.denied_records.len(), 1);
            assert_eq!(
                report.denied_records[0].decision,
                LoweredRecordDecisionKind::Reject
            );
        }
        other => panic!("expected readiness failure, got {other:?}"),
    }
}

#[test]
fn runtime_prepare_merge_execution_matches_merge_access_surface() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "root");
    fork(execution, &runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );
    let request = merge_request();

    let via_runtime = runtime
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(request.clone())
                .unwrap(),
        )
        .expect("runtime merge prepare");
    let via_access = runtime
        .merge()
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(request.clone())
                .unwrap(),
        )
        .expect("merge access prepare");
    let normalized_request = runtime
        .merge()
        .normalize_merge_request(runtime.bind_merge_execution_request(request).unwrap())
        .expect("normalized request");

    assert_eq!(via_runtime.request(), &normalized_request);
    assert_eq!(via_runtime.artifact(), via_access.artifact());
}
