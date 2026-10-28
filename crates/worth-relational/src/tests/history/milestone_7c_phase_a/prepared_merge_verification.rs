use crate::facade::history::BranchId;
use crate::facade::identity::KindId;
use crate::facade::merge::{MergeExecutionError, MergeExecutionRequest, MergeIntent};
use crate::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationIntegrityDeclarations,
    RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::tests::support::{
    create_branch_from_main, create_entity, create_entity_outcome_on_branch,
    durable_runtime_with_test_schema, entity_field_aspect, CascadeDeletePolicy, CrossContextPolicy,
};

#[test]
fn verify_prepared_merge_execution_accepts_fresh_prepared_merge() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

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

    runtime
        .merge()
        .verify_prepared_merge_execution(execution, &prepared)
        .expect("fresh prepared merge should verify");
}

#[test]
fn verify_prepared_merge_execution_rejects_runtime_instance_mismatch() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

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
    let forked = runtime.fork().expect("settled runtime forks");

    match forked
        .merge()
        .verify_prepared_merge_execution(execution, &prepared)
    {
        Err(MergeExecutionError::RuntimeInstanceMismatch { .. }) => {}
        other => panic!("expected runtime instance mismatch, got {other:?}"),
    }
}

#[test]
fn verify_prepared_merge_execution_rejects_target_head_drift() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

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

    create_entity(execution, &runtime, "main-advance");

    match runtime
        .merge()
        .verify_prepared_merge_execution(execution, &prepared)
    {
        Err(MergeExecutionError::StaleBranchHead { branch, .. }) => {
            assert_eq!(branch, BranchId("main".to_string()));
        }
        other => panic!("expected target stale-head rejection, got {other:?}"),
    }
}

#[test]
fn verify_prepared_merge_execution_rejects_source_head_drift() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

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

    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-advance",
        BranchId("feature".to_string()),
    );

    match runtime
        .merge()
        .verify_prepared_merge_execution(execution, &prepared)
    {
        Err(MergeExecutionError::StaleBranchHead { branch, .. }) => {
            assert_eq!(branch, BranchId("feature".to_string()));
        }
        other => panic!("expected source stale-head rejection, got {other:?}"),
    }
}

#[test]
fn verify_prepared_merge_execution_rejects_schema_semantic_drift() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

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

    let drifted_registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(2),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_field_aspect(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                ),
                entity_field_aspect(
                    crate::tests::support::aspect_key("status"),
                    crate::tests::support::field_key("status"),
                ),
            ]),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(2),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
        .unwrap();
    runtime.set_schema_registry_for_test(drifted_registry);

    match runtime
        .merge()
        .verify_prepared_merge_execution(execution, &prepared)
    {
        Err(MergeExecutionError::SchemaSemanticDrift { .. }) => {}
        other => panic!("expected schema drift rejection, got {other:?}"),
    }
}

#[test]
fn verify_prepared_merge_execution_rejects_merge_base_drift() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

    let mut prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("prepared merge");
    prepared
        .authority_binding_mut_for_test()
        .merge_base_commit_id = crate::facade::history::CommitId(999_999);

    match runtime
        .merge()
        .verify_prepared_merge_execution(execution, &prepared)
    {
        Err(MergeExecutionError::MergeBaseDrift { .. }) => {}
        other => panic!("expected merge-base drift rejection, got {other:?}"),
    }
}
