use super::*;

#[test]
fn branch_roots_with_one_schema_deduplicate_the_durable_carrier() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "schema-carrier-deduplication");
    let entity = changed_entities(&created)[0];
    create_branch_from_main(&runtime, "feature");
    update_entity_on_branch(
        execution,
        &runtime,
        entity,
        "schema-carrier-feature-root",
        BranchId("feature".to_owned()),
    );

    let checkpoint = runtime
        .durability_authority()
        .checkpoint(execution)
        .expect("production checkpoint succeeds");

    assert!(checkpoint.branch_roots.len() >= 2);
    assert_eq!(checkpoint.branch_root_schema_images.len(), 1);
}

#[test]
fn recovery_rejects_a_missing_branch_root_schema_carrier() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut plan = checkpoint_plan_with_one_root(execution, "missing-schema-carrier");
    plan.checkpoint
        .as_mut()
        .expect("checkpoint exists")
        .branch_root_schema_images
        .clear();

    assert_corrupt_schema_recovery(execution, plan, "missing schema carrier");
}

#[test]
fn recovery_rejects_a_tampered_branch_root_schema_carrier() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut plan = checkpoint_plan_with_one_root(execution, "tampered-schema-carrier");
    plan.checkpoint
        .as_mut()
        .and_then(|checkpoint| checkpoint.branch_root_schema_images.first_mut())
        .expect("checkpoint carries a schema carrier")
        .corrupt_schema_root_for_test();

    assert_corrupt_schema_recovery(execution, plan, "schema carrier digest mismatch");
}

#[test]
fn recovery_rejects_a_schema_carrier_swapped_between_exact_roots() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime =
        persisted_runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    create_entity_outcome(execution, &runtime, "schema-v1");
    create_branch_from_main(&runtime, "legacy");
    runtime.set_schema_registry_for_test(
        AspectSchemaFixture {
            schema_version_id: SchemaVersionId(2),
            ..AspectSchemaFixture::with_default_declared_aspects(
                CascadeDeletePolicy::CascadeDeleteRelations,
            )
        }
        .build_registry(),
    );
    let mut transaction = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime)
                .with_schema_transition(
                    schema_transition_for_subscriber_impact(
                        SchemaVersionId(2),
                        SchemaSubscriberImpact::ConsumableSurfaceChanged,
                    ),
                    Some(SchemaReconciliationPolicy::PreserveInformation),
                );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    transaction
        .push_batch(
            batch_create("schema-v2"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .commit(execution, &runtime)
        .expect("schema transition commits");
    runtime
        .durability_authority()
        .checkpoint(execution)
        .expect("production checkpoint succeeds");
    let mut plan = runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let checkpoint = plan.checkpoint.as_mut().expect("checkpoint exists");
    assert_eq!(checkpoint.branch_root_schema_images.len(), 2);
    let replacement = checkpoint.branch_roots[1].schema_carrier_digest;
    let root = &mut checkpoint.branch_roots[0];
    assert_ne!(root.schema_carrier_digest, replacement);
    root.schema_carrier_digest = replacement;
    root.root_image_digest = crate::durability::data::branch_root_image_digest(
        root.format_version,
        root.commit_id,
        root.partition_image_digest,
        root.schema_carrier_digest,
    );

    let mut recovered = RelationalRuntimeApi::builder()
        .schema_registry(
            AspectSchemaFixture {
                schema_version_id: SchemaVersionId(2),
                ..AspectSchemaFixture::with_default_declared_aspects(
                    CascadeDeletePolicy::CascadeDeleteRelations,
                )
            }
            .build_registry(),
        )
        .build();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("swapped root schema carrier cannot be readmitted");
    let error = crate::tests::support::expect_durability_failure(error);
    assert_eq!(
        error.class,
        RecoveryFailureClass::CorruptCheckpoint,
        "unexpected denial: {error:?}"
    );
    assert!(
        error.detail.contains("schema carrier linkage mismatch"),
        "unexpected denial: {error:?}"
    );
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

fn checkpoint_plan_with_one_root(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    label: &str,
) -> RecoveryPlan {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, label);
    runtime
        .durability_authority()
        .checkpoint(execution)
        .expect("production checkpoint succeeds");
    runtime
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap()
}

fn assert_corrupt_schema_recovery(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    plan: RecoveryPlan,
    expected_detail: &str,
) {
    assert_schema_recovery_denial(
        execution,
        plan,
        RecoveryFailureClass::CorruptCheckpoint,
        expected_detail,
    );
}

fn assert_schema_recovery_denial(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    plan: RecoveryPlan,
    expected_class: RecoveryFailureClass,
    expected_detail: &str,
) {
    let mut recovered = persisted_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("corrupt root schema carrier cannot be readmitted");
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, expected_class, "unexpected denial: {error:?}");
    assert!(
        error.detail.contains(expected_detail),
        "unexpected denial: {error:?}"
    );
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

#[path = "branch_root_schema_binding/exact_contracts.rs"]
mod exact_contracts;
