use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn complexity_budget_schema_transition_classification_is_changed_atom_bounded() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = runtime_with_test_schema();
    let _ = create_entity_outcome(execution, &runtime, "anchor");

    runtime.set_schema_registry_for_test(
        AspectSchemaFixture {
            schema_version_id: SchemaVersionId(2),
            ..AspectSchemaFixture::with_default_declared_aspects(
                CascadeDeletePolicy::CascadeDeleteRelations,
            )
        }
        .build_registry(),
    );

    runtime.performance_access().reset_counters();
    let mut txn = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime)
                .with_schema_transition(
                    schema_transition_for_subscriber_impact(
                        SchemaVersionId(2),
                        crate::schema::data::SchemaSubscriberImpact::ConsumableSurfaceChanged,
                    ),
                    Some(crate::schema::data::SchemaReconciliationPolicy::PreserveInformation),
                );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    txn.push_batch(
        batch_create("b"),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(execution, &runtime).unwrap();
    let counters = runtime.performance_access().counters();

    assert_eq!(counters.schema_transition_atoms_inspected, 1);
    assert_eq!(counters.schema_changed_subtrees_inspected, 1);
    assert_eq!(counters.schema_unchanged_subtrees_reused_by_fingerprint, 0);
    assert_eq!(counters.schema_bridge_descriptors_built, 1);
    assert_eq!(counters.schema_transition_continue_visible_bridge_count, 1);
    assert_eq!(counters.schema_reconciliation_preserve_information_count, 1);
}

#[test]
fn complexity_budget_subscriber_resume_continuity_is_boundary_local() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = runtime_with_test_schema();
    let _ = create_entity_outcome(execution, &runtime, "anchor");

    runtime.set_schema_registry_for_test(
        AspectSchemaFixture {
            schema_version_id: SchemaVersionId(2),
            ..AspectSchemaFixture::with_default_declared_aspects(
                CascadeDeletePolicy::CascadeDeleteRelations,
            )
        }
        .build_registry(),
    );
    let mut txn = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime)
                .with_schema_transition(
                    schema_transition_for_subscriber_impact(
                        SchemaVersionId(2),
                        crate::schema::data::SchemaSubscriberImpact::ConsumableSurfaceChanged,
                    ),
                    Some(crate::schema::data::SchemaReconciliationPolicy::PreserveInformation),
                );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    txn.push_batch(
        batch_create("b"),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(execution, &runtime).unwrap();

    runtime.performance_access().reset_counters();
    let _ = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::from_head(10))
        .unwrap();
    let counters = runtime.performance_access().counters();

    assert_eq!(counters.subscriber_resume_evaluations, 1);
    assert_eq!(counters.subscriber_continue_visible_bridge_count, 1);
    assert_eq!(counters.schema_normalized_descriptor_compositions, 1);
}

#[test]
fn complexity_budget_milestone5_closeout_keeps_schema_cdc_and_recovery_boundary_local() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = durable_runtime_with_test_schema();
    let baseline = create_entity_outcome(execution, &runtime, "anchor");
    let baseline_checkpoint =
        checkpoint_for_schema_version(baseline.patch_position(), SchemaVersionId(1));

    runtime.set_schema_registry_for_test(
        AspectSchemaFixture {
            schema_version_id: SchemaVersionId(2),
            ..AspectSchemaFixture::with_default_declared_aspects(
                CascadeDeletePolicy::CascadeDeleteRelations,
            )
        }
        .build_registry(),
    );

    runtime.performance_access().reset_counters();
    let mut txn = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime)
                .with_schema_transition(
                    schema_transition_for_subscriber_impact(
                        SchemaVersionId(2),
                        crate::schema::data::SchemaSubscriberImpact::ConsumableSurfaceChanged,
                    ),
                    Some(crate::schema::data::SchemaReconciliationPolicy::PreserveInformation),
                );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    txn.push_batch(
        batch_create("after-boundary"),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let transitioned = txn.commit(execution, &runtime).unwrap();
    let schema_counters = runtime.performance_access().counters();

    assert_eq!(schema_counters.schema_transition_atoms_inspected, 1);
    assert_eq!(schema_counters.schema_changed_subtrees_inspected, 1);
    assert_eq!(schema_counters.schema_bridge_descriptors_built, 1);
    assert_eq!(
        schema_counters.schema_transition_continue_visible_bridge_count,
        1
    );
    assert_eq!(schema_counters.replay_digest_parity_checks, 0);
    assert_eq!(schema_counters.replay_deep_artifact_parity_checks, 0);

    runtime.performance_access().reset_counters();
    let _batch = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(
            baseline_checkpoint.clone(),
            32,
        ))
        .unwrap();
    let cdc_counters = runtime.performance_access().counters();

    assert_eq!(cdc_counters.schema_transition_atoms_inspected, 0);
    assert_eq!(cdc_counters.subscriber_resume_evaluations, 1);
    assert_eq!(cdc_counters.subscriber_continue_visible_bridge_count, 1);
    assert_eq!(cdc_counters.schema_normalized_descriptor_compositions, 1);
    assert_eq!(cdc_counters.replay_digest_parity_checks, 0);
    assert_eq!(cdc_counters.replay_deep_artifact_parity_checks, 0);

    runtime.performance_access().reset_counters();
    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let plan_counters = runtime.performance_access().counters();

    assert!(plan_counters.replay_digest_parity_checks >= 1);
    assert_eq!(plan_counters.replay_deep_artifact_parity_checks, 0);
    assert_eq!(
        plan.authority_continuity.verification_outcome,
        crate::durability::data::RecoveryVerificationOutcome::VerifiedAtLayer(
            crate::replay::data::ReplayVerificationLayer::DigestParity
        )
    );

    let mut recovered = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(
            AspectSchemaFixture {
                schema_version_id: SchemaVersionId(2),
                ..AspectSchemaFixture::with_default_declared_aspects(
                    CascadeDeletePolicy::CascadeDeleteRelations,
                )
            }
            .build_registry(),
        )
        .build_durable(TestDurableMedium::new().attachment());
    let _ = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let recovered_counters = recovered.performance_access().counters();

    assert!(recovered_counters.replay_digest_parity_checks >= 1);
    assert_eq!(recovered_counters.replay_deep_artifact_parity_checks, 0);
    assert!(recovered
        .replay()
        .canonical_commit_envelope(execution, transitioned.commit.commit_id)
        .unwrap()
        .map(|envelope| envelope.as_ref().clone())
        .is_some());
}
